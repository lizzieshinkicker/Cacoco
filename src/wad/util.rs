//! Low-level utility functions for parsing Doom WAD data structures.

use once_cell::sync::Lazy;
use std::collections::HashSet;

/// A collection of standard Doom lump prefixes used to identify graphical data
/// such as Patches, Flats, and Sprites during the WAD scanning phase.
pub static GRAPHIC_PREFIXES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let prefixes = [
        "ST", "WI", "M_", "BRDR", "DGT", "NUM", "PRCN", "MINUS", "PUNG", "SAWG", "PISG", "SHTG",
        "SHT2", "CHGG", "MISG", "PLSG", "BFGG", "BKEY", "YKEY", "RKEY", "BSKU", "YSKU", "RSKU",
        "PINV", "PSTR", "PINS", "SUIT", "PMAP", "PVIS", "ARM", "MEDI", "BPAK", "AMMO", "SHEL",
        "CELL", "ROCK", "INTER", "FINALE", "TITLE", "PAT", "GRN", "SKY", "RSKY", "F_SKY",
    ];
    prefixes.into_iter().collect()
});

/// Converts a fixed-length null-terminated or space-padded byte slice
/// from a WAD directory into a clean, uppercase Rust String.
pub fn parse_lump_name(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[0..len])
        .to_string()
        .to_uppercase()
}

/// Returns true if the given lump name begins with a known graphical prefix.
pub fn is_graphic_lump(name: &str) -> bool {
    GRAPHIC_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// Sniffs the dimensions (Width, Height) of an image lump by inspecting its header.
/// Supports both standard PNG files and Doom's internal Patch format.
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
    (256, 128) // Default fallback for sky-sized textures
}
