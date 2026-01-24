use crate::assets::AssetStore;
use crate::models::skydefs::{SkyDefsFile, SkyType};
use crate::state::PreviewState;
use crate::ui::context_menu::ContextMenu;
use crate::ui::properties::common;
use eframe::egui;

/// Renders the specialized editor for SKYDEFS lumps.
pub fn draw_skydefs_editor(
    ui: &mut egui::Ui,
    file: &mut SkyDefsFile,
    selection_path: &[usize],
    _assets: &AssetStore,
    _state: &PreviewState,
) -> bool {
    let mut changed = false;

    if let Some(s) = file.data.skies.get_mut(selection_path[0]) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 200.0).max(0.0) / 2.0);
                ui.label("Sky Type:");
                let type_id = ui.make_persistent_id("sky_type_dd");
                let type_label = match s.sky_type {
                    SkyType::Normal => "Normal",
                    SkyType::Fire => "Fire",
                    SkyType::WithForeground => "Foreground Layered",
                };

                let btn = ui.add(egui::Button::new(type_label).min_size(egui::vec2(120.0, 0.0)));
                if btn.clicked() {
                    ContextMenu::open(ui, type_id, btn.rect.left_bottom());
                }

                if let Some(menu) = ContextMenu::get(ui, type_id) {
                    ContextMenu::show(ui, menu, btn.clicked(), |ui| {
                        if common::custom_menu_item(ui, "Normal", s.sky_type == SkyType::Normal) {
                            s.sky_type = SkyType::Normal;
                            changed = true;
                            ContextMenu::close(ui);
                        }
                        if common::custom_menu_item(ui, "Fire", s.sky_type == SkyType::Fire) {
                            s.sky_type = SkyType::Fire;
                            s.fire = Some(Default::default());
                            changed = true;
                            ContextMenu::close(ui);
                        }
                        if common::custom_menu_item(
                            ui,
                            "With Foreground",
                            s.sky_type == SkyType::WithForeground,
                        ) {
                            s.sky_type = SkyType::WithForeground;
                            s.foregroundtex = Some(Default::default());
                            changed = true;
                            ContextMenu::close(ui);
                        }
                    });
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 210.0).max(0.0) / 2.0);
                ui.label("Texture:");
                let mut buf = s.name.clone();
                if ui
                    .add_sized([130.0, 18.0], egui::TextEdit::singleline(&mut buf))
                    .changed()
                {
                    s.name = AssetStore::stem(&buf);
                    changed = true;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 120.0).max(0.0) / 2.0);
                ui.label("Mid Texel:");
                let mut mid_val = s.mid as i32;
                if ui
                    .add(egui::DragValue::new(&mut mid_val).range(0..=2048))
                    .changed()
                {
                    s.mid = mid_val as f32;
                    changed = true;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 120.0).max(0.0) / 2.0);
                ui.label("Scroll X:     ");
                let mut sx = s.scrollx as i32;
                if ui
                    .add(egui::DragValue::new(&mut sx).range(-1024..=1024))
                    .changed()
                {
                    s.scrollx = sx as f32;
                    changed = true;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 120.0).max(0.0) / 2.0);
                ui.label("Scroll Y:     ");
                let mut sy = s.scrolly as i32;
                if ui
                    .add(egui::DragValue::new(&mut sy).range(-1024..=1024))
                    .changed()
                {
                    s.scrolly = sy as f32;
                    changed = true;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 120.0).max(0.0) / 2.0);
                ui.label("Scale X %:");
                let mut scx = (s.scalex * 100.0) as i32;
                if ui
                    .add(egui::DragValue::new(&mut scx).range(1..=1000).suffix("%"))
                    .changed()
                {
                    s.scalex = scx as f32 / 100.0;
                    changed = true;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 120.0).max(0.0) / 2.0);
                ui.label("Scale Y %:");
                let mut scy = (s.scaley * 100.0) as i32;
                if ui
                    .add(egui::DragValue::new(&mut scy).range(1..=1000).suffix("%"))
                    .changed()
                {
                    s.scaley = scy as f32 / 100.0;
                    changed = true;
                }
            });

            ui.add_space(12.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.heading("Global Flat Mappings");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(2.0);

                    if ui.button("+ Add Mapping").clicked() {
                        let list = file.data.flatmapping.get_or_insert(Vec::new());
                        list.push(crate::models::skydefs::FlatMap {
                            flat: "F_SKY1".to_string(),
                            sky: s.name.clone(),
                        });
                        changed = true;
                    }
                });
            });
            ui.label(
                egui::RichText::new("Redirect a floor/ceiling flat to this Sky definition.")
                    .weak()
                    .size(10.0),
            );

            if let Some(mappings) = &mut file.data.flatmapping {
                let mut to_remove = None;

                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;

                    for (idx, map) in mappings.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add_space(8.0);

                            if ui.add_sized([20.0, 20.0], egui::Button::new("X")).clicked() {
                                to_remove = Some(idx);
                            }

                            ui.label("Flat:");
                            if ui
                                .add_sized([75.0, 18.0], egui::TextEdit::singleline(&mut map.flat))
                                .changed()
                            {
                                map.flat = map.flat.to_uppercase();
                                changed = true;
                            }

                            ui.label("to Sky:");
                            if ui
                                .add_sized([75.0, 18.0], egui::TextEdit::singleline(&mut map.sky))
                                .changed()
                            {
                                map.sky = map.sky.to_uppercase();
                                changed = true;
                            }

                            ui.add_space(8.0);
                        });
                    }
                });

                if let Some(idx) = to_remove {
                    mappings.remove(idx);
                    changed = true;
                }

                if mappings.is_empty() {
                    file.data.flatmapping = None;
                    changed = true;
                }
            }
        });
    }

    changed
}
