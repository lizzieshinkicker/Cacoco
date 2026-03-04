use crate::assets::AssetStore;
use crate::models::sbardef::MinimapDef;
use crate::state::PreviewState;
use eframe::egui;
use super::editor::PropertiesUI;
use super::font_cache::FontCache;
use super::preview::PreviewContent;

impl PropertiesUI for MinimapDef {
    /// Renders the specialized editor for the Minimap element.
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        _fonts: &FontCache,
        _assets: &AssetStore,
        _state: &PreviewState,
    ) -> bool {
        let mut changed = false;

        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 160.0).max(0.0) / 2.0);
                ui.label("Size:");
                changed |= ui.add(egui::DragValue::new(&mut self.width).prefix("W: ")).changed();
                changed |= ui.add(egui::DragValue::new(&mut self.height).prefix("H: ")).changed();
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 140.0).max(0.0) / 2.0);
                ui.label("Scale:");
                changed |= ui.add(egui::DragValue::new(&mut self.scale).speed(0.1).range(0.1..=10.0)).changed();
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 140.0).max(0.0) / 2.0);
                ui.label("BG Type:");
                changed |= ui.add(egui::DragValue::new(&mut self.background).range(0..=2)).changed();
            });
        });

        changed
    }

    fn get_preview_content(
        &self,
        _ui: &egui::Ui,
        _fonts: &FontCache,
        _state: &PreviewState,
    ) -> Option<PreviewContent> {
        Some(PreviewContent::Text {
            text: "MINIMAP".to_string(),
            stem: None,
            is_number_font: false,
        })
    }

    fn has_specific_fields(&self) -> bool {
        true
    }
}