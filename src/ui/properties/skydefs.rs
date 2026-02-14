use super::editor::{LumpUI, PropertyContext, ViewportContext};
use crate::assets::AssetStore;
use crate::document::DocumentAction;
use crate::models::skydefs::{SkyDefsFile, SkyType};
use crate::state::PreviewState;
use crate::ui::context_menu::ContextMenu;
use crate::ui::properties::common;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

/// Helper to draw the standard sky parameters (texture, mid, scroll, scale).
/// Returns true if any value was modified.
fn draw_sky_params_fields(
    ui: &mut egui::Ui,
    name: &mut String,
    mid: &mut f32,
    sx: &mut f32,
    sy: &mut f32,
    scx: &mut f32,
    scy: &mut f32,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 210.0).max(0.0) / 2.0);
        ui.label("Texture:");
        let mut buf = name.clone();
        if ui
            .add_sized([130.0, 18.0], egui::TextEdit::singleline(&mut buf))
            .changed()
        {
            *name = AssetStore::stem(&buf);
            changed = true;
        }
    });

    ui.add_space(4.0);

    let mut draw_row = |label: &str,
                        val: &mut f32,
                        range: std::ops::RangeInclusive<f32>,
                        suffix: &str,
                        speed: f32| {
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
            ui.label(label);
            changed |= ui
                .add(
                    egui::DragValue::new(val)
                        .range(range)
                        .suffix(suffix)
                        .speed(speed),
                )
                .changed();
        });
    };

    draw_row("Mid Texel: ", mid, 0.0..=2048.0, "", 1.0);
    draw_row("Scroll X:  ", sx, -1024.0..=1024.0, "", 1.0);
    draw_row("Scroll Y:  ", sy, -1024.0..=1024.0, "", 1.0);

    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
        changed |= shared::drag_percentage(ui, "Scale X %: ", scx);
    });
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
        changed |= shared::drag_percentage(ui, "Scale Y %: ", scy);
    });

    changed
}

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
                    SkyType::WithForeground => "With Foreground",
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
                            if s.fire.is_none() {
                                s.fire = Some(Default::default());
                            }
                            changed = true;
                            ContextMenu::close(ui);
                        }
                        if common::custom_menu_item(
                            ui,
                            "With Foreground",
                            s.sky_type == SkyType::WithForeground,
                        ) {
                            s.sky_type = SkyType::WithForeground;
                            if s.foregroundtex.is_none() {
                                s.foregroundtex = Some(Default::default());
                            }
                            changed = true;
                            ContextMenu::close(ui);
                        }
                    });
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.heading("Primary Background");
            changed |= draw_sky_params_fields(
                ui,
                &mut s.name,
                &mut s.mid,
                &mut s.scrollx,
                &mut s.scrolly,
                &mut s.scalex,
                &mut s.scaley,
            );

            match s.sky_type {
                SkyType::Fire => {
                    if let Some(fire) = &mut s.fire {
                        ui.add_space(12.0);
                        ui.heading("Fire Simulation");

                        ui.horizontal(|ui| {
                            ui.label("Update Delay:");

                            let mut tics = (fire.updatetime * 35.0).round() as i32;
                            let res = ui.add(
                                egui::DragValue::new(&mut tics)
                                    .range(1..=35)
                                    .suffix(" tics"),
                            );

                            if res.changed() {
                                fire.updatetime = tics as f32 / 35.0;
                                changed = true;
                            }

                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(format!("({:.5}s)", fire.updatetime))
                                    .weak()
                                    .size(10.0),
                            );
                        });

                        ui.add_space(8.0);
                        ui.label("Decay Palette Ramp:");
                        ui.label(
                            egui::RichText::new("Cooling (left) to Ignition (right)")
                                .weak()
                                .size(10.0),
                        );

                        let ramp_h = 32.0;
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), ramp_h),
                            egui::Sense::hover(),
                        );

                        if !fire.palette.is_empty() {
                            let step_w = rect.width() / fire.palette.len() as f32;
                            for (i, &idx) in fire.palette.iter().enumerate() {
                                let color = _assets.palette.get(idx as u8);
                                let step_rect = egui::Rect::from_min_size(
                                    rect.min + egui::vec2(i as f32 * step_w, 0.0),
                                    egui::vec2(step_w, ramp_h),
                                );
                                ui.painter().rect_filled(step_rect, 0.0, color);
                            }
                        }

                        ui.horizontal(|ui| {
                            if ui.button("+ Add Step").clicked() {
                                fire.palette.push(0);
                                changed = true;
                            }
                            if ui.button("- Remove").clicked() && fire.palette.len() > 1 {
                                fire.palette.pop();
                                changed = true;
                            }
                            if ui.button("↺ Reset Ramp").clicked() {
                                fire.palette = vec![
                                    0, 47, 191, 187, 235, 234, 232, 167, 166, 165, 223, 221, 220,
                                    219, 217, 216, 215, 214, 213, 164, 163, 162, 161, 160, 231,
                                    230, 229, 228, 227, 226, 225, 224,
                                ];
                                fire.updatetime = 0.05715;
                                changed = true;
                            }
                        });

                        ui.add_space(4.0);
                        ui.label("Raw Indices:");
                        egui::ScrollArea::horizontal().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for val in fire.palette.iter_mut() {
                                    changed |=
                                        ui.add(egui::DragValue::new(val).range(0..=255)).changed();
                                }
                            });
                        });
                    }
                }
                SkyType::WithForeground => {
                    if let Some(fore) = &mut s.foregroundtex {
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(12.0);
                        ui.heading("Foreground Layer");
                        ui.label(
                            egui::RichText::new("Palette index 0 is used for transparency.")
                                .weak()
                                .size(10.0),
                        );
                        changed |= draw_sky_params_fields(
                            ui,
                            &mut fore.name,
                            &mut fore.mid,
                            &mut fore.scrollx,
                            &mut fore.scrolly,
                            &mut fore.scalex,
                            &mut fore.scaley,
                        );
                    }
                }
                _ => {}
            }

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
                egui::RichText::new("Redirect any floor/ceiling flat to this Sky.")
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

impl LumpUI for SkyDefsFile {
    fn draw_properties(&mut self, ui: &mut egui::Ui, ctx: &PropertyContext) -> bool {
        if let Some(path) = ctx.selection.iter().next() {
            return draw_skydefs_editor(ui, self, path, ctx.assets, ctx.state);
        }
        false
    }

    fn header_info(&self, selection: &HashSet<Vec<usize>>) -> (String, String, egui::Color32) {
        if let Some(path) = selection.iter().next() {
            if let Some(s) = self.data.skies.get(path[0]) {
                return (
                    format!("Sky: {}", s.name),
                    "Configuration for custom ID24 cylindrical sky panoramas.".to_string(),
                    egui::Color32::from_rgb(40, 40, 60),
                );
            }
        }
        (
            "Sky Definitions".into(),
            "Select a sky from the list to edit its parameters.".into(),
            egui::Color32::TRANSPARENT,
        )
    }

    fn render_viewport(
        &self,
        _ui: &mut egui::Ui,
        ctx: &mut ViewportContext,
    ) -> Vec<DocumentAction> {
        let sky_idx = ctx
            .current_item_idx
            .min(self.data.skies.len().saturating_sub(1));
        if let Some(sky) = self.data.skies.get(sky_idx) {
            crate::render::sky::draw_sky_view(
                _ui.painter(),
                sky,
                ctx.assets,
                ctx.state,
                ctx.proj,
                _ui.input(|i| i.time),
            );
        } else {
            _ui.painter()
                .rect_filled(ctx.proj.screen_rect, 0.0, egui::Color32::BLACK);
        }
        Vec::new()
    }
}
