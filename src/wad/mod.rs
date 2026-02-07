//! The primary module for interacting with Doom WAD files and legacy metadata.
//!
//! This module orchestrates the loading of IWAD resources and the generation
//! of compatible WAD structures for export, including legacy texture tables.

pub mod legacy;
pub mod umapinfo;
pub mod util;

use crate::assets::AssetStore;
use crate::render::palette::DoomPalette;
use crate::render::patch;
use eframe::egui;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};

pub use legacy::{build_merged_pnames, build_merged_texture1, serialize_pnames};
pub use umapinfo::generate_simple_umapinfo;
pub use util::{is_graphic_lump, parse_lump_name};

/// Scans a WAD file and populates the AssetStore with its contents.
///
/// If the WAD is an IWAD, this function also captures PNAMES and TEXTUREx
/// tables to be used as a template for later exports.
pub fn load_wad_into_store(
    _ctx: &egui::Context,
    file: &mut fs::File,
    assets: &mut AssetStore,
) -> anyhow::Result<()> {
    let mut header = [0u8; 12];
    file.read_exact(&mut header).ok();

    let sig = &header[0..4];
    let is_iwad = sig == b"IWAD";
    if sig != b"IWAD" && sig != b"PWAD" {
        return Ok(());
    }

    let num_lumps = i32::from_le_bytes(header[4..8].try_into()?) as usize;
    let dir_offset = i32::from_le_bytes(header[8..12].try_into()?) as u64;

    file.seek(SeekFrom::Start(dir_offset))?;
    let mut dir_buffer = vec![0u8; num_lumps * 16];
    file.read_exact(&mut dir_buffer)?;

    let mut palette = DoomPalette::default();

    for i in 0..num_lumps {
        let entry = &dir_buffer[i * 16..(i + 1) * 16];
        let name = parse_lump_name(&entry[8..16]);
        let size = i32::from_le_bytes(entry[4..8].try_into()?) as usize;
        let file_pos = i32::from_le_bytes(entry[0..4].try_into()?) as u64;

        if name == "PLAYPAL" {
            file.seek(SeekFrom::Start(file_pos))?;
            let mut pal_bytes = vec![0u8; 768];
            file.read_exact(&mut pal_bytes)?;
            palette = DoomPalette::from_raw(&pal_bytes);
        }

        if is_iwad {
            match name.as_str() {
                "PNAMES" => {
                    file.seek(SeekFrom::Start(file_pos))?;
                    let mut data = vec![0u8; size];
                    file.read_exact(&mut data)?;
                    if data.len() >= 4 {
                        let count = i32::from_le_bytes(data[0..4].try_into()?) as usize;
                        assets.base_pnames.clear();
                        for j in 0..count {
                            let start = 4 + (j * 8);
                            if start + 8 <= data.len() {
                                assets
                                    .base_pnames
                                    .push(parse_lump_name(&data[start..start + 8]));
                            }
                        }
                    }
                }
                "TEXTURE1" => {
                    file.seek(SeekFrom::Start(file_pos))?;
                    assets.base_texture1 = vec![0u8; size];
                    file.read_exact(&mut assets.base_texture1)?;
                }
                "TEXTURE2" => {
                    file.seek(SeekFrom::Start(file_pos))?;
                    assets.base_texture2 = vec![0u8; size];
                    file.read_exact(&mut assets.base_texture2)?;
                }
                _ => {}
            }
        }
    }

    for i in 0..num_lumps {
        let entry = &dir_buffer[i * 16..(i + 1) * 16];
        let name = parse_lump_name(&entry[8..16]);
        let size = i32::from_le_bytes(entry[4..8].try_into()?) as usize;
        let file_pos = i32::from_le_bytes(entry[0..4].try_into()?) as u64;
        if size == 0 {
            continue;
        }
        if is_graphic_lump(&name) {
            let mut lump_data = vec![0u8; size];
            file.seek(SeekFrom::Start(file_pos))?;
            file.read_exact(&mut lump_data)?;
            if let Some((width, height, left, top, pixels)) =
                patch::decode_doom_patch(&lump_data, &palette)
            {
                assets.load_rgba_with_offset(_ctx, &name, width, height, left, top, &pixels);
            } else if size == 4096 {
                if let Some((w, h, pixels)) = patch::decode_doom_flat(&lump_data, &palette) {
                    assets.load_rgba(_ctx, &name, w, h, &pixels);
                }
            } else {
                assets.load_reference_image(_ctx, &name, &lump_data);
            }
        }
    }
    Ok(())
}

/// Writes a collection of ID24 project lumps and associated assets into a new PWAD.
///
/// If the project contains a SKYDEFS lump, this function automatically generates
/// legacy PNAMES, TEXTURE1, and UMAPINFO lumps to ensure cross-port compatibility.
pub fn write_wad_to_file<W: Write + Seek>(
    writer: &mut W,
    lumps: &[crate::models::ProjectData],
    assets: &AssetStore,
) -> anyhow::Result<()> {
    let has_skydefs = lumps.iter().any(|l| l.standard_lump_name() == "SKYDEFS");

    let merged_pnames = build_merged_pnames(assets);
    let merged_texture1 = build_merged_texture1(&merged_pnames, assets);

    let umapinfo_text = generate_simple_umapinfo(lumps);

    writer.write_all(b"PWAD")?;

    let mut num_lumps = lumps.len() + assets.raw_files.len() + 2;
    if has_skydefs {
        num_lumps += 3;
    }

    writer.write_all(&(num_lumps as i32).to_le_bytes())?;
    let dir_marker = writer.stream_position()?;
    writer.write_all(&0i32.to_le_bytes())?;

    struct Record {
        pos: u32,
        size: u32,
        name: String,
    }
    let mut records = Vec::new();

    for lump in lumps {
        let content = lump.to_sanitized_json(assets);
        let pos = writer.stream_position()? as u32;
        writer.write_all(content.as_bytes())?;
        records.push(Record {
            pos,
            size: content.len() as u32,
            name: lump.standard_lump_name().to_string(),
        });
    }

    if has_skydefs {
        let pos = writer.stream_position()? as u32;
        writer.write_all(umapinfo_text.as_bytes())?;
        records.push(Record {
            pos,
            size: umapinfo_text.len() as u32,
            name: "UMAPINFO".into(),
        });

        let p_data = serialize_pnames(&merged_pnames);
        let pos = writer.stream_position()? as u32;
        writer.write_all(&p_data)?;
        records.push(Record {
            pos,
            size: p_data.len() as u32,
            name: "PNAMES".into(),
        });

        let pos = writer.stream_position()? as u32;
        writer.write_all(&merged_texture1)?;
        records.push(Record {
            pos,
            size: merged_texture1.len() as u32,
            name: "TEXTURE1".into(),
        });
    }

    records.push(Record {
        pos: writer.stream_position()? as u32,
        size: 0,
        name: "P_START".into(),
    });
    for (id, bytes) in &assets.raw_files {
        let name = assets
            .names
            .get(id)
            .map(|n| AssetStore::stem(n))
            .unwrap_or_else(|| "UNKN".into());
        let pos = writer.stream_position()? as u32;
        writer.write_all(bytes)?;
        records.push(Record {
            pos,
            size: bytes.len() as u32,
            name,
        });
    }
    records.push(Record {
        pos: writer.stream_position()? as u32,
        size: 0,
        name: "P_END".into(),
    });

    let directory_pos = writer.stream_position()? as u32;
    for rec in records {
        writer.write_all(&rec.pos.to_le_bytes())?;
        writer.write_all(&rec.size.to_le_bytes())?;
        let mut name8 = [0u8; 8];
        let b = rec.name.as_bytes();
        let len = b.len().min(8);
        name8[..len].copy_from_slice(&b[..len]);
        writer.write_all(&name8)?;
    }
    writer.seek(SeekFrom::Start(dir_marker))?;
    writer.write_all(&(directory_pos as i32).to_le_bytes())?;
    Ok(())
}
