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
        "CELL", "ROCK", "INTER", "FINALE", "TITLE", "PAT", "GRN",
    ];
    prefixes.into_iter().collect()
});

/// Scans a Doom WAD file and loads all compatible graphical lumps into the store.
pub fn load_wad_into_store(
    ctx: &egui::Context,
    file: &mut fs::File,
    assets: &mut AssetStore,
) -> anyhow::Result<()> {
    let mut header = [0u8; 12];
    file.read_exact(&mut header).ok();

    let sig = &header[0..4];
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
        if name == "PLAYPAL" {
            let file_pos = i32::from_le_bytes(entry[0..4].try_into()?) as u64;
            file.seek(SeekFrom::Start(file_pos))?;
            let mut pal_bytes = vec![0u8; 768];
            file.read_exact(&mut pal_bytes)?;
            palette = DoomPalette::from_raw(&pal_bytes);
            break;
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
                assets.load_rgba_with_offset(ctx, &name, width, height, left, top, &pixels);
            } else if size == 4096 {
                if let Some((w, h, pixels)) = patch::decode_doom_flat(&lump_data, &palette) {
                    assets.load_rgba(ctx, &name, w, h, &pixels);
                }
            } else {
                assets.load_reference_image(ctx, &name, &lump_data);
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

pub fn write_wad_to_file<W: Write + Seek>(
    writer: &mut W,
    sbardef_json: &[u8],
    assets: &AssetStore,
) -> anyhow::Result<()> {
    writer.write_all(b"PWAD")?;

    let num_lumps = 1 + assets.raw_files.len();
    writer.write_all(&(num_lumps as i32).to_le_bytes())?;

    let directory_offset_marker = writer.stream_position()?;
    writer.write_all(&0i32.to_le_bytes())?;

    struct LumpRecord {
        pos: u32,
        size: u32,
        name: String,
    }
    let mut records = Vec::new();

    let pos = writer.stream_position()? as u32;
    writer.write_all(sbardef_json)?;
    records.push(LumpRecord {
        pos,
        size: sbardef_json.len() as u32,
        name: "SBARDEF".to_string(),
    });

    for (id, bytes) in &assets.raw_files {
        let original_name = assets
            .names
            .get(id)
            .cloned()
            .unwrap_or_else(|| format!("{}", id));
        let mut stem = AssetStore::stem(&original_name);
        stem.truncate(8);

        let pos = writer.stream_position()? as u32;
        writer.write_all(bytes)?;
        records.push(LumpRecord {
            pos,
            size: bytes.len() as u32,
            name: stem,
        });
    }

    let directory_pos = writer.stream_position()? as u32;
    for rec in records {
        writer.write_all(&rec.pos.to_le_bytes())?;
        writer.write_all(&rec.size.to_le_bytes())?;

        let mut name_bytes = [0u8; 8];
        let b = rec.name.as_bytes();
        let len = b.len().min(8);
        name_bytes[..len].copy_from_slice(&b[..len]);
        writer.write_all(&name_bytes)?;
    }

    writer.seek(SeekFrom::Start(directory_offset_marker))?;
    writer.write_all(&(directory_pos as i32).to_le_bytes())?;

    Ok(())
}
