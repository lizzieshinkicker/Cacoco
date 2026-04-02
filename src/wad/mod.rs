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
pub use util::{is_graphic_lump, is_known_non_graphic_lump, parse_lump_name};

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
    base_iwad: Option<&str>,
) -> anyhow::Result<crate::io::LoadedProject> {
    let mut file = fs::File::open(path)?;
    let mut assets = AssetStore::default();

    if let Some(iwad_path) = base_iwad {
        if let Ok(mut iwad_file) = fs::File::open(iwad_path) {
            let _ = load_wad_into_store(ctx, &mut iwad_file, &mut assets, true);
        }
    }

    load_wad_into_store(ctx, &mut file, &mut assets, false)?;

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

        if size == 0 {
            continue;
        }

        let mut lump_data = vec![0u8; size];
        file.seek(std::io::SeekFrom::Start(file_pos))?;
        file.read_exact(&mut lump_data)?;

        let managed_names = ["SBARDEF", "SKYDEFS", "INTERLEVEL", "FINALE", "UMAPINFO"];
        let is_known_name = managed_names.iter().any(|&m| m.eq_ignore_ascii_case(&name));

        let looks_like_json = lump_data
            .iter()
            .find(|b| !b.is_ascii_whitespace())
            .map_or(false, |&b| b == b'{');

        let mut claimed_by_cacoco = false;

        if is_known_name || looks_like_json {
            println!(">>> ATTEMPTING TO PARSE LUMP: {}", name);
            if let Some(parsed) = ProjectData::parse_lump(&name, &lump_data) {
                println!("    SUCCESSFULLY CLAIMED: {}", name);
                if let ProjectData::Interlevel(mut new_ilvl) = parsed {
                    if let Some(ProjectData::Interlevel(existing)) = lumps
                        .iter_mut()
                        .find(|l| matches!(l, ProjectData::Interlevel(_)))
                    {
                        existing.screens.append(&mut new_ilvl.screens);
                    } else {
                        lumps.push(ProjectData::Interlevel(new_ilvl));
                    }
                } else {
                    lumps.push(parsed);
                }
                claimed_by_cacoco = true;
            } else {
                println!("    FAILED TO PARSE AS ID24: {}", name);
            }
        }

        if !claimed_by_cacoco {
            passthrough_lumps.push(RawLump {
                name: name.clone(),
                data: lump_data,
            });
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
    ctx: &eframe::egui::Context,
    file: &mut fs::File,
    assets: &mut AssetStore,
    strict: bool,
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

        let mut lump_data = vec![0u8; size];
        file.seek(std::io::SeekFrom::Start(file_pos))?;
        file.read_exact(&mut lump_data)?;

        let is_json = lump_data.iter().find(|b: &&u8| !b.is_ascii_whitespace()) == Some(&b'{');
        let is_non_graphic = is_known_non_graphic_lump(&name);
        let should_try_load = (!strict || is_graphic_lump(&name)) && !is_json && !is_non_graphic;

        if should_try_load {
            if let Some((width, height, left, top, pixels)) =
                patch::decode_doom_patch(&lump_data, &assets.palette)
            {
                if width == 0 && height == 0 {
                    let id = crate::assets::AssetId::new(&name);
                    assets.load_rgba(ctx, &name, 1, 1, &[0, 0, 0, 0]);
                    assets.offsets.insert(id, (32767, 32767));
                } else if width <= 2048 && height <= 2048 {
                    assets.load_rgba_with_offset(ctx, &name, width, height, left, top, &pixels);
                }
            } else if size == 4096 {
                if let Some((w, h, pixels)) = patch::decode_doom_flat(&lump_data, &assets.palette) {
                    assets.load_rgba(ctx, &name, w, h, &pixels);
                }
            } else {
                if !strict || lump_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                    assets.load_reference_image(ctx, &name, &lump_data);
                }
            }
        }
    }
    Ok(())
}

/// Performs a highly targeted search of the IWAD for specific lump names requested by the editor.
pub fn deep_search_iwad(
    ctx: &eframe::egui::Context,
    iwad_path: &str,
    targets: &[String],
    assets: &mut AssetStore,
) -> std::collections::HashSet<String> {
    let mut found = std::collections::HashSet::new();
    if targets.is_empty() {
        return found;
    }

    let mut file = match fs::File::open(iwad_path) {
        Ok(f) => f,
        Err(_) => return found,
    };

    let mut header = [0u8; 12];
    if file.read_exact(&mut header).is_err() {
        return found;
    }

    let num_lumps = i32::from_le_bytes(header[4..8].try_into().unwrap_or([0; 4])) as usize;
    let dir_offset = i32::from_le_bytes(header[8..12].try_into().unwrap_or([0; 4])) as u64;

    if file.seek(std::io::SeekFrom::Start(dir_offset)).is_err() {
        return found;
    }
    let mut dir_buffer = vec![0u8; num_lumps * 16];
    if file.read_exact(&mut dir_buffer).is_err() {
        return found;
    }

    let target_set: std::collections::HashSet<&str> = targets.iter().map(|s| s.as_str()).collect();

    for i in 0..num_lumps {
        let entry = &dir_buffer[i * 16..(i + 1) * 16];
        let name = parse_lump_name(&entry[8..16]);
        let size = i32::from_le_bytes(entry[4..8].try_into().unwrap_or([0; 4])) as usize;
        let file_pos = i32::from_le_bytes(entry[0..4].try_into().unwrap_or([0; 4])) as u64;

        if size > 0 && target_set.contains(name.as_str()) {
            if is_known_non_graphic_lump(&name) {
                continue;
            }

            let mut lump_data = vec![0u8; size];
            if file.seek(std::io::SeekFrom::Start(file_pos)).is_ok()
                && file.read_exact(&mut lump_data).is_ok()
            {
                if let Some((width, height, left, top, pixels)) =
                    patch::decode_doom_patch(&lump_data, &assets.palette)
                {
                    if width == 0 && height == 0 {
                        let id = crate::assets::AssetId::new(&name);
                        assets.load_rgba(ctx, &name, 1, 1, &[0, 0, 0, 0]);
                        assets.offsets.insert(id, (32767, 32767));
                        found.insert(name.clone());
                    } else if width <= 2048 && height <= 2048 {
                        assets.load_rgba_with_offset(ctx, &name, width, height, left, top, &pixels);
                        found.insert(name.clone());
                    }
                } else if size == 4096 {
                    if let Some((w, h, pixels)) =
                        patch::decode_doom_flat(&lump_data, &assets.palette)
                    {
                        assets.load_rgba(ctx, &name, w, h, &pixels);
                        found.insert(name.clone());
                    }
                } else {
                    assets.load_reference_image(ctx, &name, &lump_data);
                    found.insert(name.clone());
                }
            }
        }
    }
    found
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
        for (name, content) in l.get_export_entries(assets) {
            managed_map.insert(name, content);
        }
    }

    for raw in passthrough {
        let name_upper = raw.name.to_uppercase();
        let pos = writer.stream_position()? as u32;
        let mut size = raw.data.len() as u32;

        if let Some(managed) = managed_map.remove(&name_upper) {
            let new_data = managed;
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
        let new_data = managed;
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
