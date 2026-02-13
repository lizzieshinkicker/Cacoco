use crate::document::actions::DocumentAction;
use crate::models::finale::FinaleDefFile;
use crate::ui::properties::editor::{LumpUI, PropertyContext, ViewportContext};
use eframe::egui;
use std::collections::HashSet;

impl LumpUI for FinaleDefFile {
    fn draw_properties(&mut self, _ui: &mut egui::Ui, _ctx: &PropertyContext) -> bool {
        false
    }

    fn header_info(&self, _sel: &HashSet<Vec<usize>>) -> (String, String, egui::Color32) {
        (
            "Finale Definition".into(),
            "Art screens and cast calls.".into(),
            egui::Color32::from_rgb(60, 40, 60),
        )
    }

    fn render_viewport(&self, ui: &mut egui::Ui, ctx: &mut ViewportContext) -> Vec<DocumentAction> {
        crate::ui::viewport::render_id24_background(
            ui,
            &self.data.background,
            ctx.assets,
            ctx.proj,
        );
        Vec::new()
    }
}
