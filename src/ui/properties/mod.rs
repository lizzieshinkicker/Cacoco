use std::collections::HashSet;
use eframe::egui;
use crate::model::SBarDefFile;
use crate::assets::AssetStore;
use crate::state::PreviewState;
use crate::ui::layers::colors;

pub mod common;
mod graphics;
mod text;
pub(crate) mod preview;
mod lookups;
pub(crate) mod font_cache;
mod conditions;
pub mod editor;
pub mod text_helper;
mod descriptions;

use font_cache::FontCache;
use editor::PropertiesUI;

pub fn draw_properties_panel(
    ui: &mut egui::Ui,
    file: &mut Option<SBarDefFile>,
    selection: &HashSet<Vec<usize>>,
    assets: &AssetStore,
    state: &PreviewState,
) {
    ui.heading("Properties");
    ui.separator();

    let file_ref = match file {
        Some(f) => f,
        None => { ui.label("No file loaded."); return; }
    };

    if selection.len() > 1 {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new("Multiple Objects Selected").strong().size(16.0));
            ui.label(format!("{} objects", selection.len()));
        });
        return;
    }

    let path = match selection.iter().next() {
        Some(p) => p,
        None => { ui.label("Select a layer to edit."); return; }
    };

    let font_cache = FontCache::new(file_ref);

    if let Some(element) = file_ref.get_element_mut(path) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(content) = element.get_preview_content(ui, &font_cache, state) {
                preview::draw_preview_panel(ui, assets, content);
                ui.add_space(4.0);
            }

            let helper_text = descriptions::get_helper_text(element);
            let frame_color = colors::get_layer_color(element)
                .map(|c| c.linear_multiply(0.05))
                .unwrap_or_else(|| ui.visuals().widgets.noninteractive.bg_fill);

            text::draw_interactive_header(ui, element, helper_text, frame_color);
            ui.separator();

            if element._cacoco_text.is_none() {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    let mut name = element._cacoco_name.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut name).changed() {
                        element._cacoco_name = if name.is_empty() { None } else { Some(name) };
                    }
                });
                ui.add_space(4.0);
            }

            common::draw_transform_editor(ui, element);
            ui.add_space(4.0);

            let has_content = element._cacoco_text.is_some() || element.has_specific_fields();
            if has_content {
                ui.group(|ui| {
                    if element._cacoco_text.is_some() {
                        text_helper::draw_text_helper_editor(ui, element, &font_cache, assets);
                    } else {
                        element.draw_specific_fields(ui, &font_cache, assets, state);
                    }
                });
                ui.add_space(4.0);
            }

            conditions::draw_conditions_editor(ui, element, assets, state);
        });
    } else {
        let bar_idx = path[0];
        if let Some(bar) = file_ref.data.status_bars.get_mut(bar_idx) {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(format!("Status Bar #{} Layout", bar_idx));
                ui.separator();
                common::draw_root_statusbar_fields(ui, bar);
            });
        }
    }
}