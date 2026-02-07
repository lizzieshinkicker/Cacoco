//! Handling of legacy binary tables (PNAMES and TEXTURE1/2) required for
//! backward compatibility in many Doom engines.

use super::util::{get_image_dimensions, parse_lump_name};
use crate::assets::{AssetId, AssetStore};
use std::collections::HashSet;

/// Merges current project assets into the existing PNAMES list from the loaded IWAD.
/// Returns a complete list of patch names, ensuring no duplicates are added.
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

/// Serializes a list of names into a binary PNAMES lump.
pub fn serialize_pnames(names: &[String]) -> Vec<u8> {
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

/// Reconstructs the TEXTURE1 table by merging original IWAD definitions with
/// new definitions from the project. Existing textures are overwritten if
/// a replacement image is found in the project.
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
            if to_overwrite.contains(&i) {
                let name = &existing_names[i];
                let mut name8 = [0u8; 8];
                let upper = name.to_uppercase();
                let b_bytes = upper.as_bytes();
                name8[..b_bytes.len().min(8)].copy_from_slice(&b_bytes[..b_bytes.len().min(8)]);

                let id = AssetId::new(name);
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
            } else {
                let old_off_pos = 4 + (i * 4);
                let old_off =
                    i32::from_le_bytes(data[old_off_pos..old_off_pos + 4].try_into().unwrap())
                        as usize;
                let next_off = if i + 1 < old_num_tex {
                    i32::from_le_bytes(data[old_off_pos + 4..old_off_pos + 8].try_into().unwrap())
                        as usize
                } else {
                    data.len()
                };
                entry.extend_from_slice(&data[old_off..next_off]);
            }
        } else {
            let name = &to_append[i - old_num_tex];
            let mut name8 = [0u8; 8];
            let upper = name.to_uppercase();
            let b_bytes = upper.as_bytes();
            name8[..b_bytes.len().min(8)].copy_from_slice(&b_bytes[..b_bytes.len().min(8)]);

            let id = AssetId::new(name);
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

/// Creates a minimal, standalone TEXTURE1 lump for use when no base IWAD data is available.
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
        name8[..b.len().min(8)].copy_from_slice(&b[..b.len().min(8)]);

        let id = AssetId::new(name);
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
