use crate::{
    models::sbardef::{MinimapDef},
    ui::properties::editor::PropertiesUI,
};
use eframe::egui;

impl PropertiesUI for MinimapDef {
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        _fonts: &super::FontCache,
        _assets: &crate::assets::AssetStore,
        _state: &crate::state::PreviewState,
    ) -> bool {
        let mut changed = false;

        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
                ui.label("Width:");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.width)
                            .range(0..=320),
                    )
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
                ui.label("Height:");
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.height)
                            .range(0..=200),
                    )
                    .changed();
            });
        });

        changed
    }
}
