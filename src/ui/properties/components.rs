use super::FontCache;
use super::common;
use super::editor::PropertiesUI;
use super::preview::PreviewContent;
use crate::assets::AssetStore;
use crate::model::{ComponentDef, ComponentType};
use crate::state::PreviewState;
use crate::ui::context_menu::ContextMenu;
use eframe::egui;

impl PropertiesUI for ComponentDef {
    /// Renders font selection and duration for HUD components.
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        fonts: &FontCache,
        assets: &AssetStore,
        _state: &PreviewState,
    ) -> bool {
        let mut changed = false;

        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 190.0).max(0.0) / 2.0);
                ui.label("Font:");

                let id = ui.make_persistent_id("hud_font_selector");
                let button_res =
                    ui.add(egui::Button::new(&self.font).min_size(egui::vec2(140.0, 18.0)));

                if button_res.clicked() {
                    ContextMenu::open(ui, id, button_res.rect.left_bottom());
                }

                if let Some(menu) = ContextMenu::get(ui, id) {
                    ContextMenu::show(ui, menu, button_res.clicked(), |ui| {
                        let h = (fonts.hud_font_names.len() as f32 * 42.0).min(1000.0);
                        ui.set_min_height(h);
                        ui.set_width(200.0);
                        changed |=
                            common::draw_hud_font_selectors(ui, &mut self.font, fonts, assets);
                    });
                }
            });

            use crate::model::ComponentType::*;
            if matches!(self.type_, StatTotals | Coordinates) {
                ui.add_space(4.0);
                changed |= ui.checkbox(&mut self.vertical, "Vertical Layout").changed();
            }

            if matches!(self.type_, Message | AnnounceLevelTitle) {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 140.0).max(0.0) / 2.0);
                    ui.label("Duration:");

                    let mut tenths = (self.duration * 10.0).round() as i32;

                    let res = ui.add(
                        egui::DragValue::new(&mut tenths)
                            .speed(1)
                            .range(0..=600)
                            .custom_formatter(|n, _| format!("{:.1}s", n / 10.0)),
                    );

                    if res.changed() {
                        self.duration = tenths as f32 / 10.0;
                        changed = true;
                    }
                });
            }
        });
        changed
    }

    /// Renders a sample of the component's output in the preview panel.
    fn get_preview_content(
        &self,
        ui: &egui::Ui,
        fonts: &FontCache,
        state: &PreviewState,
    ) -> Option<PreviewContent> {
        let stem = fonts.get_hud_stem(&self.font);
        let text_val = match self.type_ {
            ComponentType::Time => {
                let total_seconds = ui.input(|i| i.time) as u64;
                format!(":{:02}", total_seconds % 60)
            }
            ComponentType::LevelTitle => "MAP01: ENTRYWAY".to_string(),
            ComponentType::FpsCounter => format!("{:.0}", state.editor.display_fps),
            ComponentType::Coordinates => "X: ### Y: ###".to_string(),
            ComponentType::StatTotals => "K:0/0".to_string(),
            ComponentType::Message => "You got the Shotgun!".to_string(),
            _ => format!("[{:?}]", self.type_),
        };
        Some(PreviewContent::Text {
            text: text_val,
            stem,
            is_number_font: false,
        })
    }
}

/// Helper to render the subtype selection list for engine components.
pub(super) fn draw_component_options(ui: &mut egui::Ui, type_: &mut ComponentType) -> bool {
    use crate::model::ComponentType::*;
    let mut changed = false;
    let items = [
        ("Time", Time),
        ("Level Title", LevelTitle),
        ("Announce Level Title", AnnounceLevelTitle),
        ("Message", Message),
        ("Coordinates", Coordinates),
        ("FPS Counter", FpsCounter),
        ("Stat Totals", StatTotals),
    ];

    for (label, target) in items {
        if common::custom_menu_item(ui, label, *type_ == target) {
            *type_ = target;
            changed = true;
        }
    }
    changed
}
