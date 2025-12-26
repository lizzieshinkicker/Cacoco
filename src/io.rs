use crate::assets::AssetStore;
use crate::model::SBarDefFile;
use crate::wad;
use eframe::egui;
use rfd::FileDialog;
use std::env;
use std::fs;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub struct LoadedProject {
    pub file: SBarDefFile,
    pub assets: AssetStore,
}

pub fn open_project_dialog() -> Option<String> {
    if let Some(path) = FileDialog::new()
        .add_filter("SBARDEF Projects", &["pk3", "zip", "json", "txt"])
        .set_title("Open SBARDEF Project")
        .pick_file()
    {
        return Some(path.to_string_lossy().into_owned());
    }
    None
}

pub fn load_project_from_path(ctx: &egui::Context, path_str: &str) -> Option<LoadedProject> {
    let path = PathBuf::from(path_str);
    if !path.exists() {
        eprintln!("File not found: {}", path_str);
        return None;
    }

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "pk3" || ext == "zip" {
        load_pk3(ctx, &path)
    } else {
        load_text_file(&path)
    }
}

fn load_text_file(path: &PathBuf) -> Option<LoadedProject> {
    match fs::read_to_string(path) {
        Ok(json_content) => match serde_json::from_str::<SBarDefFile>(&json_content) {
            Ok(parsed_file) => Some(LoadedProject {
                file: parsed_file,
                assets: AssetStore::default(),
            }),
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            None
        }
    }
}

fn load_pk3(ctx: &egui::Context, path: &PathBuf) -> Option<LoadedProject> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let mut sbardef_content = String::new();
    let mut found_def = false;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name();
        if name.eq_ignore_ascii_case("SBARDEF")
            || name.eq_ignore_ascii_case("SBARDEF.txt")
            || name.eq_ignore_ascii_case("SBARDEF.json")
        {
            if let Ok(_) = file.read_to_string(&mut sbardef_content) {
                found_def = true;
            }
            break;
        }
    }

    if !found_def {
        eprintln!("Error: No SBARDEF file found in PK3.");
        return None;
    }

    let parsed_file = match serde_json::from_str::<SBarDefFile>(&sbardef_content) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("PK3 Parse Error: {}", e);
            return None;
        }
    };

    let mut assets = AssetStore::default();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_string();

        if name.to_lowercase().starts_with("graphics/") && !name.ends_with('/') {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                assets.load_image(ctx, &name, &buffer);
            }
        }
    }

    Some(LoadedProject {
        file: parsed_file,
        assets,
    })
}

fn build_pk3<W: Write + Seek>(
    writer: W,
    file: &SBarDefFile,
    assets: &AssetStore,
) -> anyhow::Result<()> {
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    if let Ok(json) = serde_json::to_string_pretty(file) {
        zip.start_file("SBARDEF", options)?;
        zip.write_all(json.as_bytes())?;
    }

    for (name, bytes) in &assets.raw_files {
        zip.start_file(name, options)?;
        zip.write_all(bytes)?;
    }

    zip.finish()?;
    Ok(())
}

pub fn save_json_dialog(file: &SBarDefFile, opened_path: Option<String>) -> Option<String> {
    let mut dialog = FileDialog::new()
        .add_filter("SBARDEF JSON", &["json", "txt", "JSON", "TXT"])
        .set_title("Export SBARDEF JSON");

    if let Some(p_str) = &opened_path {
        let p = Path::new(p_str);
        if let Some(parent) = p.parent() {
            dialog = dialog.set_directory(parent);
        }
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("SBARDEF");
        dialog = dialog.set_file_name(format!("{}.json", stem));
    } else {
        dialog = dialog.set_file_name("SBARDEF.json");
    }

    if let Some(path) = dialog.save_file() {
        if let Ok(json) = serde_json::to_string_pretty(file) {
            if fs::write(&path, json).is_ok() {
                return Some(path.to_string_lossy().into_owned());
            }
        }
    }
    None
}

pub fn save_pk3_dialog(
    file: &SBarDefFile,
    assets: &AssetStore,
    opened_path: Option<String>,
) -> Option<String> {
    let mut dialog = FileDialog::new()
        .add_filter("Doom Package", &["pk3", "zip", "PK3", "ZIP"])
        .set_title("Save PK3");

    if let Some(p_str) = &opened_path {
        let p = Path::new(p_str);
        if let Some(parent) = p.parent() {
            dialog = dialog.set_directory(parent);
        }
        let mut final_name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("status_bar.pk3")
            .to_string();
        let lower = final_name.to_lowercase();
        if !lower.ends_with(".pk3") && !lower.ends_with(".zip") {
            final_name.push_str(".pk3");
        }
        dialog = dialog.set_file_name(&final_name);
    } else {
        dialog = dialog.set_file_name("status_bar.pk3");
    }

    if let Some(path) = dialog.save_file() {
        match fs::File::create(&path) {
            Ok(fs_file) => {
                if let Err(e) = build_pk3(fs_file, file, assets) {
                    eprintln!("Failed to build PK3: {}", e);
                } else {
                    return Some(path.to_string_lossy().into_owned());
                }
            }
            Err(e) => eprintln!("Failed to create file at {:?}: {}", path, e),
        }
    }
    None
}

pub fn save_pk3_silent(
    file: &SBarDefFile,
    assets: &AssetStore,
    path_str: &str,
) -> anyhow::Result<()> {
    let path = Path::new(path_str);
    let fs_file = fs::File::create(path)?;
    build_pk3(fs_file, file, assets)?;
    Ok(())
}

pub fn load_iwad_dialog(ctx: &egui::Context, assets: &mut AssetStore) -> Option<String> {
    if let Some(path) = FileDialog::new()
        .add_filter("Doom WAD", &["wad", "WAD"])
        .set_title("Select Base WAD (e.g., DOOM2.WAD)")
        .pick_file()
    {
        if let Ok(mut file) = fs::File::open(&path) {
            if wad::load_wad_into_store(ctx, &mut file, assets).is_ok() {
                return path.to_str().map(|s| s.to_string());
            }
        }
    }
    None
}

pub fn load_wad_from_path(ctx: &egui::Context, path_str: &str, assets: &mut AssetStore) -> bool {
    let path = Path::new(path_str);
    if let Ok(mut file) = fs::File::open(path) {
        if let Err(e) = wad::load_wad_into_store(ctx, &mut file, assets) {
            eprintln!("Failed to auto-load WAD at {:?}: {}", path, e);
            false
        } else {
            println!("Auto-loaded Base WAD: {:?}", path);
            true
        }
    } else {
        false
    }
}

pub fn launch_game(file: &SBarDefFile, assets: &AssetStore, source_port: &str, iwad: &str) {
    let mut temp_path = env::temp_dir();
    temp_path.push("cacotest.pk3");
    let temp_path_str = temp_path.to_string_lossy().into_owned();

    match fs::File::create(&temp_path) {
        Ok(fs_file) => {
            if let Err(e) = build_pk3(fs_file, file, assets) {
                eprintln!("Failed to build temporary PK3: {}", e);
                return;
            }
        }
        Err(e) => {
            eprintln!("Failed to create temp file: {}", e);
            return;
        }
    };

    let _ = Command::new(source_port)
        .arg("-iwad")
        .arg(iwad)
        .arg("-file")
        .arg(&temp_path_str)
        .arg("-skill")
        .arg("4")
        .arg("-warp")
        .arg("1")
        .spawn();
}

pub fn import_images_dialog(ctx: &egui::Context, assets: &mut AssetStore) -> usize {
    if let Some(paths) = FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "PNG", "JPG", "JPEG"])
        .set_title("Import Graphics")
        .pick_files()
    {
        return load_images_from_paths(ctx, assets, paths);
    }
    0
}

pub fn import_folder_dialog(ctx: &egui::Context, assets: &mut AssetStore) -> usize {
    if let Some(path) = FileDialog::new()
        .set_title("Import Folder Recursively")
        .pick_folder()
    {
        let mut paths = Vec::new();
        visit_dirs_for_images(&path, &mut paths);
        return load_images_from_paths(ctx, assets, paths);
    }
    0
}

fn load_images_from_paths(
    ctx: &egui::Context,
    assets: &mut AssetStore,
    paths: Vec<PathBuf>,
) -> usize {
    let mut count = 0;
    for path in paths {
        if let Ok(bytes) = fs::read(&path) {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            assets.load_image(ctx, name, &bytes);
            count += 1;
        }
    }
    count
}

fn visit_dirs_for_images(dir: &Path, paths: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs_for_images(&path, paths);
            } else {
                let ext = path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_lowercase();
                if ext == "png" || ext == "jpg" || ext == "jpeg" {
                    paths.push(path);
                }
            }
        }
    }
}
