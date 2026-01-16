use crate::assets::AssetStore;
use crate::models::sbardef::ListDef;
use crate::state::PreviewState;
use eframe::egui;

use super::editor::PropertiesUI;
use super::font_cache::FontCache;

impl PropertiesUI for ListDef {
    /// Renders control for the List element's layout behavior.
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        _: &FontCache,
        _: &AssetStore,
        _: &PreviewState,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 100.0).max(0.0) / 2.0);
            changed |= ui.checkbox(&mut self.horizontal, "Horizontal").changed();
        });

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 100.0).max(0.0) / 2.0);
            ui.label("Spacing:");
            changed |= ui.add(egui::DragValue::new(&mut self.spacing)).changed();
        });

        changed
    }
}
