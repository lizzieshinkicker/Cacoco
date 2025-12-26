use crate::assets::AssetStore;
use crate::render::palette::DoomPalette;
use crate::render::patch;
use eframe::egui;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Seek, SeekFrom};

static GRAPHIC_PREFIXES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let prefixes = [
        "ST", "WI", "M_", "BRDR", "DGT", "NUM", "PRCN", "MINUS", "PUNG", "SAWG", "PISG", "SHTG",
        "SHT2", "CHGG", "MISG", "PLSG", "BFGG", "BKEY", "YKEY", "RKEY", "BSKU", "YSKU", "RSKU",
        "PINV", "PSTR", "PINS", "SUIT", "PMAP", "PVIS", "ARM", "MEDI", "BPAK", "AMMO", "SHEL",
        "CELL", "ROCK", "INTER", "FINALE", "TITLE", "PAT", "GRN",
    ];
    prefixes.into_iter().collect()
});
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
    let mut found_pal = false;

    for i in 0..num_lumps {
        let entry = &dir_buffer[i * 16..(i + 1) * 16];
        let name = parse_lump_name(&entry[8..16]);
        if name == "PLAYPAL" {
            let file_pos = i32::from_le_bytes(entry[0..4].try_into()?) as u64;
            file.seek(SeekFrom::Start(file_pos))?;
            let mut pal_bytes = vec![0u8; 768];
            file.read_exact(&mut pal_bytes)?;
            palette = DoomPalette::from_raw(&pal_bytes);
            found_pal = true;
            break;
        }
    }

    if found_pal {
        println!("WAD: Found PLAYPAL.");
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
