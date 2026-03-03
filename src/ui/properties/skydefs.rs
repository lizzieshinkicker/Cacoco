use super::editor::{LayerContext, LumpUI, PropertyContext, TickContext, ViewportContext};
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
        ui.add_space((ui.available_width() - 185.0).max(0.0) / 2.0);
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
            ui.add_space((ui.available_width() - 110.0).max(0.0) / 2.0);
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
    draw_row("Scroll X:     ", sx, -1024.0..=1024.0, "", 1.0);
    draw_row("Scroll Y:     ", sy, -1024.0..=1024.0, "", 1.0);

    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 110.0).max(0.0) / 2.0);
        changed |= shared::drag_percentage(ui, "Scale X %: ", scx);
    });
    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 110.0).max(0.0) / 2.0);
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

    let slot_id = egui::Id::new("FIRE_PICKER_ID");
    let stored_slot: isize = ui.ctx().data(|d| d.get_temp(slot_id).unwrap_or(-1));
    let mut active_slot = if stored_slot >= 0 {
        Some(stored_slot as usize)
    } else {
        None
    };

    if active_slot.is_some() {
        ui.ctx().request_repaint();
    }

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
                            ui.add_space((ui.available_width() - 130.0).max(0.0) / 2.0);
                            ui.label("Update Delay:");
                            let mut tics = (fire.updatetime * 35.0).round() as i32;
                            if ui
                                .add(
                                    egui::DragValue::new(&mut tics)
                                        .range(1..=35)
                                        .suffix(" tics"),
                                )
                                .changed()
                            {
                                fire.updatetime = tics as f32 / 35.0;
                                changed = true;
                            }
                        });

                        ui.label(
                            egui::RichText::new(format!("({:.5}s)", fire.updatetime))
                                .weak()
                                .size(10.0),
                        );

                        ui.add_space(12.0);
                        ui.label("Decay Palette Ramp");
                        ui.label(
                            egui::RichText::new("Cooling (left) to Ignition (right)")
                                .weak()
                                .size(10.0),
                        );
                        ui.add_space(4.0);

                        let ramp_h = 32.0;
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2((ui.available_width() - 20.0).max(0.0), ramp_h),
                            egui::Sense::click(),
                        );

                        if !fire.palette.is_empty() {
                            let step_w = rect.width() / fire.palette.len() as f32;
                            if response.clicked() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    let idx = ((pos.x - rect.min.x) / step_w).floor() as usize;
                                    let idx = idx.min(fire.palette.len() - 1);

                                    active_slot = Some(idx);
                                    ui.ctx().data_mut(|d| d.insert_temp(slot_id, idx as isize));
                                    ui.ctx().request_repaint();
                                }
                            }

                            for (i, &idx) in fire.palette.iter().enumerate() {
                                let color = _assets.palette.get(idx as u8);
                                let step_rect = egui::Rect::from_min_size(
                                    rect.min + egui::vec2(i as f32 * step_w, 0.0),
                                    egui::vec2(step_w, ramp_h),
                                );

                                let is_editing = active_slot == Some(i);
                                ui.painter().rect_filled(step_rect, 0.0, color);
                                if is_editing {
                                    ui.painter().rect_stroke(
                                        step_rect,
                                        0.0,
                                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                                        egui::StrokeKind::Inside,
                                    );
                                }
                            }
                        }

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.add_space((ui.available_width() - 155.0).max(0.0) / 2.0);
                            if ui.button("+ Add").clicked() {
                                fire.palette.push(0);
                                changed = true;
                            }
                            if ui.button("- Remove").clicked() && fire.palette.len() > 1 {
                                fire.palette.pop();
                                changed = true;
                            }
                            if ui.button("Reset").clicked() {
                                fire.palette = vec![
                                    0, 47, 191, 187, 235, 234, 232, 167, 166, 165, 223, 221, 220,
                                    219, 217, 216, 215, 214, 213, 164, 163, 162, 161, 160, 231,
                                    230, 229, 228, 227, 226, 225, 224,
                                ];
                                fire.updatetime = 0.05715;
                                changed = true;
                                ui.ctx().data_mut(|d| d.insert_temp(slot_id, -1isize));
                            }
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
            ui.heading("Global Flat Mappings");
            ui.label(
                egui::RichText::new("Redirect any floor/ceiling flat to this Sky.")
                    .weak()
                    .size(10.0),
            );

            ui.add_space(4.0);
            if ui.button("+ Add Mapping").clicked() {
                let list = file.data.flatmapping.get_or_insert(Vec::new());
                list.push(crate::models::skydefs::FlatMap {
                    flat: "F_SKY1".to_string(),
                    sky: s.name.clone(),
                });
                changed = true;
            }

            if let Some(mappings) = &mut file.data.flatmapping {
                let mut to_remove = None;
                ui.add_space(4.0);
                let row_width = ui.available_width();
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    for (idx, map) in mappings.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add_space((row_width - 265.0).max(0.0) / 2.0);

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

        if let Some(slot_idx) = active_slot {
            if let Some(fire) = &mut s.fire {
                if slot_idx < fire.palette.len() {
                    let mut is_open = true;
                    egui::Window::new("Select Fire Color")
                        .id(egui::Id::new("CACOCO_GLOBAL_FIRE_WINDOW"))
                        .resizable(false)
                        .collapsible(false)
                        .open(&mut is_open)
                        .show(ui.ctx(), |ui| {
                            let current = fire.palette[slot_idx] as u8;
                            if let Some(new_color) = super::palette_picker::draw_palette_grid(
                                ui,
                                &_assets.palette,
                                current,
                            ) {
                                fire.palette[slot_idx] = new_color as i32;
                                changed = true;
                            }
                        });

                    if !is_open {
                        ui.ctx().data_mut(|d| d.insert_temp(slot_id, -1isize));
                    }
                }
            }
        }
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

    fn tick(&self, ctx: &mut TickContext) {
        for sky in &self.data.skies {
            if sky.sky_type == SkyType::Fire {
                if let Some(fire_def) = &sky.fire {
                    let sky_id = ctx.assets.resolve_sky_id(&sky.name);

                    let sim = ctx.state.viewer.fire_sims.entry(sky_id).or_insert_with(|| {
                        let (w, h) = if let Some(tex) = ctx.assets.textures.get(&sky_id) {
                            (tex.size()[0] as u32, tex.size()[1] as u32)
                        } else {
                            (256, 128)
                        };
                        crate::render::fire::FireSimulation::new(w, h, ctx.time)
                    });

                    if ctx.time - sim.last_step_time >= fire_def.updatetime as f64 {
                        sim.step();
                        sim.last_step_time = ctx.time;
                        let rgba = sim.generate_rgba(&fire_def.palette, &ctx.assets.palette);
                        let dynamic_key = format!("_FIRE_ANIM_{}", sky.name);
                        ctx.assets
                            .load_rgba(ctx.ctx, &dynamic_key, sim.width, sim.height, &rgba);
                        ctx.ctx.request_repaint();
                    }
                }
            }
        }
    }

    fn draw_layer_list(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut LayerContext,
    ) -> (Vec<DocumentAction>, bool) {
        let mut actions = Vec::new();
        if shared::heading_action_button(ui, "Skies", Some("Add Sky"), false).clicked() {
            actions.push(DocumentAction::UndoSnapshot);
            actions.push(DocumentAction::Sky(
                crate::document::actions::SkyAction::Add,
            ));
        }

        egui::ScrollArea::vertical()
            .id_salt("sky_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                crate::ui::layers::sky::draw_sky_layers_list(
                    ui,
                    self,
                    ctx.selection,
                    ctx.current_item_idx,
                    ctx.assets,
                    &mut actions,
                    ctx.confirmation_modal,
                );
            });
        (actions, false)
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
