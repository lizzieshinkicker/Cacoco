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

/// A collection of exact names for known non-graphic lumps (metadata, maps, tables).
pub static NON_GRAPHIC_EXACT: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let names = [
        "PLAYPAL", "COLORMAP", "TINTTAB", "PNAMES", "TEXTURE1", "TEXTURE2", "SWANTBLS", "ANIMATED",
        "SWITCHES", "THINGS", "LINEDEFS", "SIDEDEFS", "VERTEXES", "SEGS", "SSECTORS", "NODES",
        "SECTORS", "REJECT", "BLOCKMAP", "UMAPINFO", "ZMAPINFO", "MAPINFO", "EMAPINFO", "SNDINFO",
        "MUSINFO", "LOCKDEFS", "DEHACKED", "COMPLVL", "CCARDS", "DECLARE", "ENDOOM", "REDMAP",
        "RED2MAP", "BLUEMAP", "GREENMAP", "YELLOMAP", "BLACKMAP", "TRANSRM2", "TRANSRED", "IN_E1",
        "IN_E2", "IN_E3",
    ];
    names.into_iter().collect()
});

/// A collection of prefixes for known non-graphic lumps (sounds, music, demos).
pub static NON_GRAPHIC_PREFIXES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let prefixes = ["DS", "D_", "DP"];
    prefixes.into_iter().collect()
});

/// Returns true if the given lump is definitively known to not be graphical.
pub fn is_known_non_graphic_lump(name: &str) -> bool {
    if NON_GRAPHIC_EXACT.contains(name) {
        return true;
    }
    if NON_GRAPHIC_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
    {
        return true;
    }
    if name.starts_with("DEMO") {
        return true;
    }
    if name.ends_with("MAP")
        && name.starts_with('R')
        && name.len() > 3
        && name.chars().nth(1).unwrap().is_ascii_digit()
    {
        return true;
    }
    false
}

/// Converts a fixed-length null-terminated or space-padded byte slice
/// from a WAD directory into a clean, uppercase Rust String.
pub fn parse_lump_name(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[0..len])
        .trim()
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
    (256, 128)
}
