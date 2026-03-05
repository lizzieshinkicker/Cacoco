use super::editor::PropertiesUI;
use super::font_cache::FontCache;
use super::preview::PreviewContent;
use crate::assets::AssetStore;
use crate::models::sbardef::MinimapBackground;
use crate::models::sbardef::MinimapDef;
use crate::state::PreviewState;
use eframe::egui;

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
                ui.add_space((ui.available_width() - 130.0).max(0.0) / 2.0);
                ui.add_sized([70.0, 18.0], egui::Label::new("Width:"));
                changed |= ui
                    .add_sized(
                        [50.0, 18.0],
                        egui::DragValue::new(&mut self.width).range(0..=2048),
                    )
                    .changed();
            });

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 130.0).max(0.0) / 2.0);
                ui.add_sized([70.0, 18.0], egui::Label::new("Height:"));
                changed |= ui
                    .add_sized(
                        [50.0, 18.0],
                        egui::DragValue::new(&mut self.height).range(0..=2048),
                    )
                    .changed();
            });

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 130.0).max(0.0) / 2.0);
                ui.add_sized([70.0, 18.0], egui::Label::new("Scale:"));

                let mut pct = (self.scale * 100.0).round() as i32;
                let res = ui.add_sized(
                    [50.0, 18.0],
                    egui::DragValue::new(&mut pct)
                        .speed(1)
                        .range(1..=2000)
                        .custom_formatter(|n, _| format!("{}%", n)),
                );
                if res.changed() {
                    self.scale = pct as f32 / 100.0;
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 160.0).max(0.0) / 2.0);
                ui.add_sized([70.0, 18.0], egui::Label::new("Background:"));

                let id = ui.make_persistent_id("minimap_bg_selector");
                let bg_text = match self.background {
                    MinimapBackground::Off => "Off",
                    MinimapBackground::Dark => "Dark",
                    MinimapBackground::Black => "Solid Black",
                };

                let button_res =
                    ui.add(egui::Button::new(bg_text).min_size(egui::vec2(80.0, 18.0)));

                if button_res.clicked() {
                    crate::ui::context_menu::ContextMenu::open(
                        ui,
                        id,
                        button_res.rect.left_bottom(),
                    );
                }

                if let Some(menu) = crate::ui::context_menu::ContextMenu::get(ui, id) {
                    crate::ui::context_menu::ContextMenu::show(
                        ui,
                        menu,
                        button_res.clicked(),
                        |ui| {
                            ui.set_width(120.0);

                            let current_id = self.background as i32;

                            for item in crate::ui::properties::lookups::MINIMAP_BACKGROUNDS {
                                let is_selected = current_id == item.id;
                                if crate::ui::properties::common::custom_menu_item(
                                    ui,
                                    item.name,
                                    is_selected,
                                ) {
                                    self.background = MinimapBackground::from_i32(item.id);
                                    changed = true;
                                    crate::ui::context_menu::ContextMenu::close(ui);
                                }
                            }
                        },
                    );
                }
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
