use crate::assets::AssetStore;
use crate::render::palette::DoomPalette;
use crate::render::patch;
use eframe::egui;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};

/// List of common Doom lump name prefixes that indicate graphical data.
static GRAPHIC_PREFIXES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let prefixes = [
        "ST", "WI", "M_", "BRDR", "DGT", "NUM", "PRCN", "MINUS", "PUNG", "SAWG", "PISG", "SHTG",
        "SHT2", "CHGG", "MISG", "PLSG", "BFGG", "BKEY", "YKEY", "RKEY", "BSKU", "YSKU", "RSKU",
        "PINV", "PSTR", "PINS", "SUIT", "PMAP", "PVIS", "ARM", "MEDI", "BPAK", "AMMO", "SHEL",
        "CELL", "ROCK", "INTER", "FINALE", "TITLE", "PAT", "GRN", "SKY", "RSKY", "F_SKY",
    ];
    prefixes.into_iter().collect()
});

/// Scans a Doom WAD file and loads all compatible graphical lumps into the store.
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

fn parse_lump_name(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[0..len])
        .to_string()
        .to_uppercase()
}

fn is_graphic_lump(name: &str) -> bool {
    GRAPHIC_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// Helper to sniff dimensions from PNG or Doom Patch bytes without full decoding.
pub fn get_image_dimensions(bytes: &[u8]) -> (u16, u16) {
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) && bytes.len() > 24 {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        return (w as u16, h as u16);
    }
    if bytes.len() >= 4 {
        let w = u16::from_le_bytes([bytes[0], bytes[1]]);
        let h = u16::from_le_bytes([bytes[2], bytes[3]]);
        return (w, h);
    }
    (256, 128)
}

/// Low-level builder for a PNAMES lump from a list of strings.
pub(crate) fn serialize_pnames(names: &[String]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(names.len() as i32).to_le_bytes());
    for name in names {
        let mut name8 = [0u8; 8];
        let upper = name.to_uppercase();
        let b = upper.as_bytes();
        let len = b.len().min(8);
        name8[..len].copy_from_slice(&b[..len]);
        buf.extend_from_slice(&name8);
    }
    buf
}

/// Low-level builder for a standalone TEXTURE1 lump (used as fallback).
fn serialize_texture1(names: &[String], assets: &AssetStore) -> Vec<u8> {
    let mut buf = Vec::new();
    let num_tex = names.len() as i32;
    buf.extend_from_slice(&num_tex.to_le_bytes());

    let mut offset = 4 + (num_tex * 4);
    let mut offsets = Vec::new();
    let mut data_block = Vec::new();

    for (idx, name) in names.iter().enumerate() {
        offsets.push(offset);

        let mut tex_entry = Vec::new();
        let mut name8 = [0u8; 8];
        let upper = name.to_uppercase();
        let b = upper.as_bytes();
        let len = b.len().min(8);
        name8[..len].copy_from_slice(&b[..len]);

        let id = crate::assets::AssetId::new(name);
        let (w, h) = assets
            .raw_files
            .get(&id)
            .map(|b| get_image_dimensions(b))
            .unwrap_or((256, 128));

        tex_entry.extend_from_slice(&name8);
        tex_entry.extend_from_slice(&0i32.to_le_bytes());
        tex_entry.extend_from_slice(&w.to_le_bytes());
        tex_entry.extend_from_slice(&h.to_le_bytes());
        tex_entry.extend_from_slice(&0i32.to_le_bytes());
        tex_entry.extend_from_slice(&1u16.to_le_bytes());

        tex_entry.extend_from_slice(&0i16.to_le_bytes());
        tex_entry.extend_from_slice(&0i16.to_le_bytes());
        tex_entry.extend_from_slice(&(idx as u16).to_le_bytes());
        tex_entry.extend_from_slice(&1u16.to_le_bytes());
        tex_entry.extend_from_slice(&1u16.to_le_bytes());

        offset += tex_entry.len() as i32;
        data_block.extend(tex_entry);
    }

    for off in offsets {
        buf.extend_from_slice(&off.to_le_bytes());
    }
    buf.extend(data_block);
    buf
}

pub fn build_merged_pnames(assets: &AssetStore) -> Vec<String> {
    let mut merged = assets.base_pnames.clone();
    for (id, name) in &assets.names {
        if assets.raw_files.contains_key(id) {
            let stem = AssetStore::stem(name);
            if !merged.iter().any(|n| n.eq_ignore_ascii_case(&stem)) {
                merged.push(stem);
            }
        }
    }
    merged
}

pub fn build_merged_texture1(new_pnames: &[String], assets: &AssetStore) -> Vec<u8> {
    if assets.base_texture1.is_empty() {
        let mut to_add = Vec::new();
        for (id, name) in &assets.names {
            if assets.raw_files.contains_key(id) {
                to_add.push(AssetStore::stem(name));
            }
        }
        return serialize_texture1(&to_add, assets);
    }

    let data = &assets.base_texture1;
    let old_num_tex = i32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;

    let mut existing_names = Vec::new();
    for i in 0..old_num_tex {
        let off_pos = 4 + (i * 4);
        let offset = i32::from_le_bytes(data[off_pos..off_pos + 4].try_into().unwrap()) as usize;
        if offset + 8 <= data.len() {
            existing_names.push(parse_lump_name(&data[offset..offset + 8]));
        } else {
            existing_names.push("".to_string());
        }
    }

    let mut to_append = Vec::new();
    let mut to_overwrite = HashSet::new();

    for (id, name) in &assets.names {
        if assets.raw_files.contains_key(id) {
            let stem = AssetStore::stem(name);
            if let Some(pos) = existing_names
                .iter()
                .position(|n| n.eq_ignore_ascii_case(&stem))
            {
                to_overwrite.insert(pos);
            } else {
                to_append.push(stem);
            }
        }
    }

    let new_num_tex = old_num_tex + to_append.len();
    let mut new_lump = Vec::new();
    new_lump.extend_from_slice(&(new_num_tex as i32).to_le_bytes());

    let mut running_offset = 4 + (new_num_tex * 4);
    let mut new_offsets = Vec::new();
    let mut new_data_payload = Vec::new();

    for i in 0..new_num_tex {
        new_offsets.push(running_offset as i32);
        let mut entry = Vec::new();

        if i < old_num_tex {
            let old_off_pos = 4 + (i * 4);
            let old_off =
                i32::from_le_bytes(data[old_off_pos..old_off_pos + 4].try_into().unwrap()) as usize;
            let next_off = if i + 1 < old_num_tex {
                i32::from_le_bytes(data[old_off_pos + 4..old_off_pos + 8].try_into().unwrap())
                    as usize
            } else {
                data.len()
            };

            if to_overwrite.contains(&i) {
                let name = &existing_names[i];
                let mut name8 = [0u8; 8];
                let b = name.to_uppercase();
                let b_bytes = b.as_bytes();
                name8[..b_bytes.len().min(8)].copy_from_slice(&b_bytes[..b_bytes.len().min(8)]);

                let id = crate::assets::AssetId::new(name);
                let (w, h) = assets
                    .raw_files
                    .get(&id)
                    .map(|b| get_image_dimensions(b))
                    .unwrap_or((256, 128));
                let p_idx = new_pnames
                    .iter()
                    .position(|n| n.eq_ignore_ascii_case(name))
                    .unwrap_or(0) as u16;

                entry.extend_from_slice(&name8);
                entry.extend_from_slice(&0i32.to_le_bytes()); // Masked
                entry.extend_from_slice(&w.to_le_bytes());
                entry.extend_from_slice(&h.to_le_bytes());
                entry.extend_from_slice(&0i32.to_le_bytes()); // Obsolete
                entry.extend_from_slice(&1u16.to_le_bytes()); // Count
                entry.extend_from_slice(&[0, 0, 0, 0]); // x, y (0, 0)
                entry.extend_from_slice(&p_idx.to_le_bytes());
                entry.extend_from_slice(&[1, 0, 1, 0]); // step, cmap (1, 1)
            } else {
                entry.extend_from_slice(&data[old_off..next_off]);
            }
        } else {
            let name = &to_append[i - old_num_tex];
            let mut name8 = [0u8; 8];
            let b = name.to_uppercase();
            let b_bytes = b.as_bytes();
            name8[..b_bytes.len().min(8)].copy_from_slice(&b_bytes[..b_bytes.len().min(8)]);

            let id = crate::assets::AssetId::new(name);
            let (w, h) = assets
                .raw_files
                .get(&id)
                .map(|b| get_image_dimensions(b))
                .unwrap_or((256, 128));
            let p_idx = new_pnames
                .iter()
                .position(|n| n.eq_ignore_ascii_case(name))
                .unwrap_or(0) as u16;

            entry.extend_from_slice(&name8);
            entry.extend_from_slice(&0i32.to_le_bytes());
            entry.extend_from_slice(&w.to_le_bytes());
            entry.extend_from_slice(&h.to_le_bytes());
            entry.extend_from_slice(&0i32.to_le_bytes());
            entry.extend_from_slice(&1u16.to_le_bytes());
            entry.extend_from_slice(&[0, 0, 0, 0]);
            entry.extend_from_slice(&p_idx.to_le_bytes());
            entry.extend_from_slice(&[1, 0, 1, 0]);
        }

        running_offset += entry.len();
        new_data_payload.extend(entry);
    }

    for off in new_offsets {
        new_lump.extend_from_slice(&off.to_le_bytes());
    }
    new_lump.extend(new_data_payload);
    new_lump
}

pub fn write_wad_to_file<W: Write + Seek>(
    writer: &mut W,
    lumps: &[crate::models::ProjectData],
    assets: &AssetStore,
) -> anyhow::Result<()> {
    let has_skydefs = lumps.iter().any(|l| l.standard_lump_name() == "SKYDEFS");

    let merged_pnames = build_merged_pnames(assets);
    let merged_texture1 = build_merged_texture1(&merged_pnames, assets);
    let umapinfo_text = generate_simple_umapinfo(assets);

    writer.write_all(b"PWAD")?;

    let mut num_lumps = lumps.len() + assets.raw_files.len() + 2;
    if has_skydefs {
        num_lumps += 3;
    }

    writer.write_all(&(num_lumps as i32).to_le_bytes())?;
    let directory_offset_marker = writer.stream_position()?;
    writer.write_all(&0i32.to_le_bytes())?;

    struct Record {
        pos: u32,
        size: u32,
        name: String,
    }
    let mut records = Vec::new();

    for lump in lumps {
        let json_content = lump.to_sanitized_json(assets);
        let pos = writer.stream_position()? as u32;
        writer.write_all(json_content.as_bytes())?;
        records.push(Record {
            pos,
            size: json_content.len() as u32,
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
    writer.seek(SeekFrom::Start(directory_offset_marker))?;
    writer.write_all(&(directory_pos as i32).to_le_bytes())?;
    Ok(())
}

pub fn generate_simple_umapinfo(assets: &AssetStore) -> String {
    let mut output = String::new();

    let mut bespoke_names = Vec::new();
    for (id, name) in &assets.names {
        if assets.raw_files.contains_key(id) {
            bespoke_names.push(AssetStore::stem(name));
        }
    }
    bespoke_names.sort();

    if let Some(first_sky) = bespoke_names.first() {
        output.push_str("map MAP01\n{\n");
        output.push_str(&format!("   skytexture = \"{}\"\n", first_sky));
        output.push_str("}\n");
    }

    output
}
