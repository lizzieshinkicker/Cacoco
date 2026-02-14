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
use std::fs;
use std::io::{Read, Seek, Write};

use crate::models::ProjectData;
pub use legacy::{build_merged_pnames, build_merged_texture1, serialize_pnames};
pub use umapinfo::generate_simple_umapinfo;
pub use util::{is_graphic_lump, parse_lump_name};

/// Represents a lump Cacoco doesn't interpret, but preserves.
#[derive(Clone)]
pub struct RawLump {
    pub name: String,
    pub data: Vec<u8>,
}

/// Scans a WAD for both assets and ID24 project lumps.
pub fn load_wad_project(
    ctx: &eframe::egui::Context,
    path: &std::path::PathBuf,
) -> anyhow::Result<crate::io::LoadedProject> {
    let mut file = fs::File::open(path)?;
    let mut assets = AssetStore::default();

    load_wad_into_store(ctx, &mut file, &mut assets)?;

    file.seek(std::io::SeekFrom::Start(0))?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;
    let num_lumps = i32::from_le_bytes(header[4..8].try_into()?) as usize;
    let dir_offset = i32::from_le_bytes(header[8..12].try_into()?) as u64;

    file.seek(std::io::SeekFrom::Start(dir_offset))?;
    let mut dir_buffer = vec![0u8; num_lumps * 16];
    file.read_exact(&mut dir_buffer)?;

    let mut lumps = Vec::new();
    let mut passthrough_lumps = Vec::new();
    let _managed_names = ["SBARDEF", "SKYDEFS", "INTERLEVEL", "FINALE", "UMAPINFO"];

    for i in 0..num_lumps {
        let entry = &dir_buffer[i * 16..(i + 1) * 16];
        let name = parse_lump_name(&entry[8..16]);
        let size = i32::from_le_bytes(entry[4..8].try_into()?) as usize;
        let file_pos = i32::from_le_bytes(entry[0..4].try_into()?) as u64;

        let mut lump_data = vec![0u8; size];
        if size > 0 {
            let current_pos = file.stream_position()?;
            file.seek(std::io::SeekFrom::Start(file_pos))?;
            file.read_exact(&mut lump_data)?;
            file.seek(std::io::SeekFrom::Start(current_pos))?;
        }

        passthrough_lumps.push(RawLump {
            name: name.clone(),
            data: lump_data.clone(),
        });

        let managed_names = ["SBARDEF", "SKYDEFS", "INTERLEVEL", "FINALE", "UMAPINFO"];

        if managed_names.iter().any(|&m| m.eq_ignore_ascii_case(&name)) {
            if let Some(parsed) = ProjectData::parse_lump(&name, &lump_data) {
                lumps.push(parsed);
            }
        }
    }

    if lumps.is_empty() {
        lumps.push(ProjectData::StatusBar(
            crate::models::sbardef::SBarDefFile::new_empty(),
        ));
    }

    Ok(crate::io::LoadedProject {
        lumps,
        assets,
        passthrough_lumps,
    })
}

/// Scans a WAD file and populates the AssetStore with its contents.
///
/// If the WAD is an IWAD, this function also captures PNAMES and TEXTUREx
/// tables to be used as a template for later exports.
pub fn load_wad_into_store(
    _ctx: &eframe::egui::Context,
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

    file.seek(std::io::SeekFrom::Start(dir_offset))?;
    let mut dir_buffer = vec![0u8; num_lumps * 16];
    file.read_exact(&mut dir_buffer)?;

    for i in 0..num_lumps {
        let entry = &dir_buffer[i * 16..(i + 1) * 16];
        let name = parse_lump_name(&entry[8..16]);
        let size = i32::from_le_bytes(entry[4..8].try_into()?) as usize;
        let file_pos = i32::from_le_bytes(entry[0..4].try_into()?) as u64;

        if name == "PLAYPAL" {
            file.seek(std::io::SeekFrom::Start(file_pos))?;
            let mut pal_bytes = vec![0u8; 768];
            if file.read_exact(&mut pal_bytes).is_ok() {
                assets.palette = DoomPalette::from_raw(&pal_bytes);
            }
        }

        if is_iwad {
            match name.as_str() {
                "PNAMES" => {
                    file.seek(std::io::SeekFrom::Start(file_pos))?;
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
                    file.seek(std::io::SeekFrom::Start(file_pos))?;
                    assets.base_texture1 = vec![0u8; size];
                    file.read_exact(&mut assets.base_texture1)?;
                }
                "TEXTURE2" => {
                    file.seek(std::io::SeekFrom::Start(file_pos))?;
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
            file.seek(std::io::SeekFrom::Start(file_pos))?;
            file.read_exact(&mut lump_data)?;

            if let Some((width, height, left, top, pixels)) =
                patch::decode_doom_patch(&lump_data, &assets.palette)
            {
                assets.load_rgba_with_offset(_ctx, &name, width, height, left, top, &pixels);
            } else if size == 4096 {
                if let Some((w, h, pixels)) = patch::decode_doom_flat(&lump_data, &assets.palette) {
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
    lumps: &[ProjectData],
    assets: &AssetStore,
    passthrough: &[RawLump],
) -> anyhow::Result<()> {
    writer.write_all(b"PWAD")?;
    writer.write_all(&0i32.to_le_bytes())?;
    writer.write_all(&0i32.to_le_bytes())?;

    let mut records = Vec::new();

    let mut managed_map = std::collections::HashMap::new();
    for l in lumps {
        managed_map.insert(l.standard_lump_name().to_string(), l);
    }

    for raw in passthrough {
        let name_upper = raw.name.to_uppercase();
        let pos = writer.stream_position()? as u32;
        let mut size = raw.data.len() as u32;

        if let Some(managed) = managed_map.remove(&name_upper) {
            let new_data = managed.to_sanitized_json(assets);
            writer.write_all(new_data.as_bytes())?;
            size = new_data.len() as u32;
        } else {
            writer.write_all(&raw.data)?;
        }

        records.push(Record {
            pos,
            size,
            name: name_upper,
        });
    }

    for (name, managed) in managed_map {
        let pos = writer.stream_position()? as u32;
        let new_data = managed.to_sanitized_json(assets);
        writer.write_all(new_data.as_bytes())?;
        records.push(Record {
            pos,
            size: new_data.len() as u32,
            name,
        });
    }

    let num_lumps = records.len() as i32;
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

    writer.seek(std::io::SeekFrom::Start(4))?;
    writer.write_all(&num_lumps.to_le_bytes())?;
    writer.write_all(&directory_pos.to_le_bytes())?;
    Ok(())
}

struct Record {
    pos: u32,
    size: u32,
    name: String,
}
