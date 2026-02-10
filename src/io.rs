use crate::assets::AssetStore;
use crate::models::sbardef::ExportTarget;
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

/// Container for a successfully loaded project and its assets.
pub struct LoadedProject {
    pub lumps: Vec<crate::models::ProjectData>,
    pub assets: AssetStore,
}

/// Opens the system file dialog to pick a SBARDEF project file.
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

/// Entry point for loading project data from any supported file format.
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
        Ok(json_content) => match serde_json::from_str::<crate::models::ProjectData>(&json_content)
        {
            Ok(mut parsed_file) => {
                parsed_file.set_target(parsed_file.determine_target());
                parsed_file.normalize_for_target();

                Some(LoadedProject {
                    lumps: vec![parsed_file],
                    assets: AssetStore::default(),
                })
            }
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
    let mut lumps = Vec::new();
    let valid_lumps = ["SBARDEF", "SKYDEFS", "INTERLEVEL", "FINALE", "UMAPINFO"];

    for i in 0..archive.len() {
        let mut f = archive.by_index(i).unwrap();
        let name = f.name().to_string();
        let stem = Path::new(&name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if valid_lumps.iter().any(|&l| l.eq_ignore_ascii_case(stem)) {
            let mut content = String::new();
            if f.read_to_string(&mut content).is_ok() {
                if let Ok(mut parsed) = serde_json::from_str::<crate::models::ProjectData>(&content)
                {
                    parsed.set_target(parsed.determine_target());
                    parsed.normalize_for_target();
                    lumps.push(parsed);
                } else if stem.eq_ignore_ascii_case("UMAPINFO") {
                    lumps.push(crate::models::ProjectData::UmapInfo(
                        crate::models::umapinfo::UmapInfoFile::from_umapinfo_text(&content),
                    ));
                }
            }
        }
    }

    if lumps.is_empty() {
        return None;
    }

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

    Some(LoadedProject { lumps, assets })
}

/// Internal helper to compress project data into a PK3 structure.
fn build_pk3<W: Write + Seek>(
    writer: W,
    lumps: &[crate::models::ProjectData],
    assets: &AssetStore,
) -> anyhow::Result<()> {
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let has_explicit_umapinfo = lumps.iter().any(|l| l.standard_lump_name() == "UMAPINFO");
    let has_skydefs = lumps.iter().any(|l| l.standard_lump_name() == "SKYDEFS");

    for lump in lumps {
        zip.start_file(lump.standard_lump_name(), options)?;
        zip.write_all(lump.to_sanitized_json(assets).as_bytes())?;
    }

    if has_skydefs {
        if !has_explicit_umapinfo {
            let umapinfo_text = wad::generate_simple_umapinfo(lumps);
            if !umapinfo_text.is_empty() {
                zip.start_file("UMAPINFO", options)?;
                zip.write_all(umapinfo_text.as_bytes())?;
            }
        }
        let merged_pnames = wad::build_merged_pnames(assets);
        let merged_texture1 = wad::build_merged_texture1(&merged_pnames, assets);
        let pnames_data = wad::serialize_pnames(&merged_pnames);
        zip.start_file("PNAMES", options)?;
        zip.write_all(&pnames_data)?;
        zip.start_file("TEXTURE1", options)?;
        zip.write_all(&merged_texture1)?;

        for (id, bytes) in &assets.raw_files {
            let name = assets
                .names
                .get(id)
                .cloned()
                .unwrap_or_else(|| format!("{}", id));
            let mut stem = AssetStore::stem(&name);
            stem.truncate(8);

            zip.start_file(stem, options)?;
            zip.write_all(bytes)?;
        }
    } else {
        for (id, bytes) in &assets.raw_files {
            let original_name = assets
                .names
                .get(id)
                .cloned()
                .unwrap_or_else(|| format!("{}.png", id));
            if original_name.contains('/') || original_name.contains('\\') {
                zip.start_file(&original_name, options)?;
                zip.write_all(bytes)?;
            } else {
                let stem = AssetStore::stem(&original_name);
                let ext = Path::new(&original_name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png");
                zip.start_file(format!("textures/{}.{}", stem, ext), options)?;
                zip.write_all(bytes)?;
                zip.start_file(format!("graphics/{}.{}", stem, ext), options)?;
                zip.write_all(bytes)?;
            }
        }
    }

    zip.finish()?;
    Ok(())
}

pub fn save_json_dialog(json_content: &str, opened_path: Option<String>) -> Option<String> {
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
        if fs::write(&path, json_content).is_ok() {
            return Some(path.to_string_lossy().into_owned());
        }
    }
    None
}

pub fn save_pk3_dialog(
    lumps: &[crate::models::ProjectData],
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
            .unwrap_or("project.pk3")
            .to_string();
        let lower = final_name.to_lowercase();
        if !lower.ends_with(".pk3") && !lower.ends_with(".zip") {
            final_name.push_str(".pk3");
        }
        dialog = dialog.set_file_name(&final_name);
    } else {
        dialog = dialog.set_file_name("project.pk3");
    }

    if let Some(path) = dialog.save_file() {
        match fs::File::create(&path) {
            Ok(fs_file) => {
                if let Err(e) = build_pk3(fs_file, lumps, assets) {
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

pub fn save_wad_dialog(
    lumps: &[crate::models::ProjectData],
    assets: &AssetStore,
    opened_path: Option<String>,
) -> Option<String> {
    let mut dialog = FileDialog::new()
        .add_filter("Doom WAD", &["wad", "WAD"])
        .set_title("Export as WAD (KEX Compatible)");

    if let Some(p_str) = opened_path {
        let p = Path::new(&p_str);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("project");
        dialog = dialog.set_file_name(format!("{}.wad", stem));
    } else {
        dialog = dialog.set_file_name("project.wad");
    }

    if let Some(path) = dialog.save_file() {
        if let Ok(mut f) = fs::File::create(&path) {
            if wad::write_wad_to_file(&mut f, lumps, assets).is_ok() {
                return Some(path.to_string_lossy().into_owned());
            }
        }
    }
    None
}

pub fn save_pk3_silent(
    lumps: &[crate::models::ProjectData],
    assets: &AssetStore,
    path_str: &str,
) -> anyhow::Result<()> {
    let path = Path::new(path_str);
    let fs_file = fs::File::create(path)?;
    build_pk3(fs_file, lumps, assets)?;
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
            true
        }
    } else {
        false
    }
}

/// Launches the game with the current project data.
///
/// If targeting 'Basic', it produces a temporary .WAD for KEX compatibility.
/// Otherwise, it produces a temporary .PK3.
pub fn launch_game(
    assets: &AssetStore,
    source_port: &str,
    iwad: &str,
    target: ExportTarget,
    lumps: &[crate::models::ProjectData],
) {
    let mut temp_path = env::temp_dir();
    let extension = if target == ExportTarget::Basic {
        "wad"
    } else {
        "zip"
    };
    temp_path.push(format!("cacotest.{}", extension));
    let temp_path_str = temp_path.to_string_lossy().into_owned();

    match fs::File::create(&temp_path) {
        Ok(mut fs_file) => {
            if extension == "wad" {
                let _ = wad::write_wad_to_file(&mut fs_file, lumps, assets);
            } else {
                let _ = build_pk3(fs_file, lumps, assets);
            }
        }
        Err(e) => {
            eprintln!("Failed to create temp file: {}", e);
            return;
        }
    };

    let program;
    let mut args = Vec::new();

    if Path::new(source_port).is_file() {
        program = source_port.to_string();
    } else {
        let mut words = shlex::split(source_port).unwrap_or_default();
        if words.is_empty() {
            return;
        }
        program = words.remove(0);
        args = words;
    }

    let _ = Command::new(&program)
        .args(args)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::AssetId;
    use crate::models::sbardef::SBarDefFile;
    use std::io::Cursor;

    #[test]
    fn test_pk3_structure_preservation() {
        let file = SBarDefFile {
            type_: "statusbar".to_string(),
            version: "1.0.0".to_string(),
            target: ExportTarget::Basic,
            data: Default::default(),
        };
        let project_data = crate::models::ProjectData::StatusBar(file);
        let lumps = vec![project_data];

        let mut assets = AssetStore::default();
        let dummy_bytes = vec![0u8; 10];

        let id_wad = AssetId::new("STBAR");
        assets.raw_files.insert(id_wad, dummy_bytes.clone());
        assets.names.insert(id_wad, "STBAR".to_string());

        let id_path = AssetId::new("graphics/patch.png");
        assets.raw_files.insert(id_path, dummy_bytes.clone());
        assets
            .names
            .insert(id_path, "graphics/patch.png".to_string());

        let mut buffer = Cursor::new(Vec::new());
        build_pk3(&mut buffer, &lumps, &assets).expect("Failed to build PK3");

        let mut zip = zip::ZipArchive::new(buffer).expect("Failed to open built ZIP");

        assert!(zip.by_name("SBARDEF").is_ok(), "SBARDEF missing from root");

        assert!(
            zip.by_name("graphics/STBAR.png").is_ok(),
            "Loose lump failed to move to graphics/ or gain extension"
        );

        assert!(
            zip.by_name("graphics/patch.png").is_ok(),
            "Explicit path failed to preserve correctly"
        );
    }
}

#[cfg(test)]
mod sniffer_tests {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct LumpSniffer {
        #[serde(rename = "type")]
        lump_type: String,
    }

    #[test]
    fn test_lump_sniffing() {
        let sbardef_json = r#"{ "type": "statusbar", "version": "1.2.0", "data": {} }"#;
        let finale_json = r#"{ "type": "finale", "version": "1.0.0", "music": "D_VICTO" }"#;

        let sniff_sbar: LumpSniffer = serde_json::from_str(sbardef_json).unwrap();
        let sniff_fin: LumpSniffer = serde_json::from_str(finale_json).unwrap();

        assert_eq!(sniff_sbar.lump_type, "statusbar");
        assert_eq!(sniff_fin.lump_type, "finale");

        println!(
            "Sniffer successfully identified: {} and {}",
            sniff_sbar.lump_type, sniff_fin.lump_type
        );
    }
}
