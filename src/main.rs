#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod model;
mod config;
mod app;
mod assets;
mod io;
mod ui;
mod state;
mod wad;
mod render;
mod cheats;
mod conditions;
mod library;
mod document;
mod constants;
mod history;
mod hotkeys;

use app::CacocoApp;
use eframe::{egui, NativeOptions};
use rand::seq::IndexedRandom;

const TITLES: &[&str] = &[
    "it's id24 maker!!",
    "It's Japanese for \"Baby Caco\"!",
    "Canvases, Conditionals, and Components",
    "SBARDEF INTERLEVEL is an anagram for AFTERLIVES BLENDER.",
    "I feel silly saying SBARDEF out loud. SBARDEF. SBARDEF...",
    "I wouldn't leave if I were you. JSON is much worse.",
    "*Cacodemon noise*",
];

fn main() -> eframe::Result<()> {
    let icon_data = load_icon();

    let mut rng = rand::rng();
    let title_flavor = TITLES.choose(&mut rng).unwrap_or(&"Cacoco").to_string();

    let native_options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([1280.0, 720.0])
            .with_title("Cacoco")
            .with_icon(icon_data),
        ..Default::default()
    };

    eframe::run_native(
        "Cacoco",
        native_options,
        Box::new(move |cc| {
            cc.egui_ctx.data_mut(|d| d.insert_temp(egui::Id::new("random_title"), title_flavor));
            Ok(Box::new(CacocoApp::new(cc)))
        }),
    )
}

fn load_icon() -> eframe::egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon_bytes = include_bytes!("../icon.png");
        let image = image::load_from_memory(icon_bytes)
            .expect("Failed to load icon data")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}