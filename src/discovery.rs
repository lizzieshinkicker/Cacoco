use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use winreg::RegKey;
#[cfg(windows)]
use winreg::enums::*;

const IWAD_NAMES: &[&str] = &[
    "DOOM2.WAD",
    "DOOM.WAD",
    "TNT.WAD",
    "PLUTONIA.WAD",
    "DOOM1.WAD",
    "FREEDOOM2.WAD",
    "FREEDOOM1.WAD",
];

/// Attempts to find a valid Doom IWAD automatically.
pub fn find_iwad() -> Option<PathBuf> {
    // Check Environment Variables (The most standard way)
    if let Some(path) = check_env_vars() {
        return Some(path);
    }

    // Check Platform Specifics (Registry or hardcoded paths)
    if let Some(path) = check_platform_defaults() {
        return Some(path);
    }

    None
}

fn check_env_vars() -> Option<PathBuf> {
    // Check DOOMWADDIR (Single directory)
    if let Ok(val) = env::var("DOOMWADDIR") {
        if let Some(found) = scan_dir_for_iwad(Path::new(&val)) {
            return Some(found);
        }
    }

    // Check DOOMWADPATH (List of directories)
    if let Ok(val) = env::var("DOOMWADPATH") {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for path_str in val.split(sep) {
            if let Some(found) = scan_dir_for_iwad(Path::new(path_str)) {
                return Some(found);
            }
        }
    }

    None
}

fn check_platform_defaults() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        // Steam App IDs: Doom II, Ultimate Doom, Final Doom, Master Levels, BFG, Eternal
        let steam_apps = ["2300", "2280", "2290", "9160", "208200", "782330"];
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        for app_id in steam_apps {
            let key_path = format!(
                r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Steam App {}",
                app_id
            );
            if let Ok(sub_key) = hklm.open_subkey(&key_path) {
                if let Ok(install_path) = sub_key.get_value::<String, _>("InstallLocation") {
                    if let Some(found) = scan_dir_recursive(Path::new(&install_path), 2) {
                        return Some(found);
                    }
                }
            }
        }

        // GOG IDs (Woof style)
        let gog_apps = [
            "1435848814",
            "1135892318",
            "1435848742",
            "1435827232",
            "2015545325",
        ];
        let software_key = if cfg!(target_pointer_width = "64") {
            "Software\\WOW6432Node"
        } else {
            "Software"
        };

        for gog_id in gog_apps {
            let key_path = format!(r"{}\GOG.com\Games\{}", software_key, gog_id);
            if let Ok(sub_key) = hklm.open_subkey(&key_path) {
                if let Ok(install_path) = sub_key.get_value::<String, _>("path") {
                    if let Some(found) = scan_dir_recursive(Path::new(&install_path), 2) {
                        return Some(found);
                    }
                }
            }
        }
    }

    #[cfg(unix)]
    {
        let mut paths = vec![
            PathBuf::from("/usr/share/games/doom"),
            PathBuf::from("/usr/local/share/games/doom"),
            PathBuf::from("/opt/doom"),
        ];

        if let Ok(home) = env::var("HOME") {
            let h = PathBuf::from(home);
            paths.push(h.join(".local/share/games/doom"));
            paths.push(h.join(".steam/root/steamapps/common"));
            paths.push(h.join(".local/share/Steam/steamapps/common"));
        }

        for path in paths {
            if let Some(found) = scan_dir_recursive(&path, 3) {
                return Some(found);
            }
        }
    }

    None
}

/// Looks for a known IWAD filename in a single directory.
fn scan_dir_for_iwad(dir: &Path) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }

    for name in IWAD_NAMES {
        let mut candidate = dir.to_path_buf();
        candidate.push(name);

        // On Unix systems, filenames are case-sensitive.
        // If the exact match fails, we manually scan the directory for a case-insensitive match.
        if !candidate.exists() {
            if let Ok(entries) = dir.read_dir() {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.eq_ignore_ascii_case(name) {
                            candidate = entry.path();
                            break;
                        }
                    }
                }
            }
        }

        if is_valid_iwad(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Recursively scans subdirectories up to a certain depth (e.g. searching for 'base' folders).
fn scan_dir_recursive(dir: &Path, depth: usize) -> Option<PathBuf> {
    if let Some(found) = scan_dir_for_iwad(dir) {
        return Some(found);
    }
    if depth == 0 {
        return None;
    }

    if let Ok(entries) = dir.read_dir() {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = scan_dir_recursive(&path, depth - 1) {
                    return Some(found);
                }
            }
        }
    }
    None
}

/// Verifies that a file starts with the "IWAD" magic bytes.
fn is_valid_iwad(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    if let Ok(mut f) = File::open(path) {
        let mut header = [0u8; 4];
        if f.read_exact(&mut header).is_ok() {
            return &header == b"IWAD";
        }
    }
    false
}
