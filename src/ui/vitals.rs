use crate::assets::AssetStore;
use crate::state::PreviewState;
use eframe::egui;

pub fn draw_vitals_panel(ui: &mut egui::Ui, state: &mut PreviewState, assets: &AssetStore) {
    let total_width = ui.available_width();

    let sep_width = 20.0;
    let num_cols = 4.0;
    let num_seps = 3.0;

    let col_width = ((total_width - (sep_width * num_seps)) / num_cols).max(150.0);

    let old_h = state.player.health;
    let old_a = state.player.armor;
    let old_bul = state.inventory.ammo_bullets;
    let old_shl = state.inventory.ammo_shells;
    let old_rkt = state.inventory.ammo_rockets;
    let old_cel = state.inventory.ammo_cells;
    let old_pack = state.inventory.has_backpack;

    ui.vertical_centered(|ui| {
        ui.add_space(6.0);

        ui.allocate_ui(egui::vec2(total_width, 120.0), |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                draw_dash_column(ui, col_width, "Game Context", |ui| {
                    egui::Grid::new("ctx_grid")
                        .spacing(egui::vec2(4.0, 4.0))
                        .show(ui, |ui| {
                            ui.label("Mode:");
                            egui::ComboBox::from_id_salt("mode_dd")
                                .selected_text(match state.world.session_type {
                                    0 => "Single Player",
                                    1 => "Cooperative",
                                    2 => "Deathmatch",
                                    _ => "Unknown",
                                })
                                .width(110.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut state.world.session_type, 0, "Single Player");
                                    ui.selectable_value(&mut state.world.session_type, 1, "Cooperative");
                                    ui.selectable_value(&mut state.world.session_type, 2, "Deathmatch");
                                });
                            ui.end_row();

                            ui.label("Map:");
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut state.world.episode)
                                        .prefix("E")
                                        .range(1..=9),
                                );

                                ui.add_space(6.0);

                                ui.add(
                                    egui::DragValue::new(&mut state.world.level)
                                        .prefix("M")
                                        .range(1..=32),
                                );
                            });
                            ui.end_row();

                            ui.label("Version:");
                            ui.add(egui::DragValue::new(&mut state.world.game_version));
                            ui.end_row();
                        });
                });

                draw_sep(ui, sep_width);

                draw_dash_column(ui, col_width, "Vitals", |ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing.x = 8.0;
                        ui.set_row_height(100.0);

                        egui::Grid::new("vit_grid")
                            .spacing(egui::vec2(4.0, 4.0))
                            .show(ui, |ui| {
                                ui.label("Health:");
                                ui.add(egui::DragValue::new(&mut state.player.health).range(0..=200));
                                ui.end_row();

                                ui.label("Armor:");
                                ui.add(egui::DragValue::new(&mut state.player.armor).range(0..=200));
                                ui.end_row();
                            });

                        ui.vertical(|ui| {
                            ui.set_width(42.0);
                            ui.vertical_centered(|ui| {
                                let is_blue = state.player.armor_max == 200;
                                let patch = if is_blue { "ARM2A0" } else { "ARM1A0" };

                                if draw_icon_button(ui, assets, patch, true, "Armor").clicked() {
                                    if is_blue {
                                        state.player.armor_max = 100;
                                        state.push_message("Picked up the armor.");
                                    } else {
                                        state.player.armor_max = 200;
                                        state.push_message("Picked up the Megaarmor!");
                                    }
                                }

                                let label = if is_blue { "Blue" } else { "Green" };
                                ui.label(egui::RichText::new(label).weak().size(10.0));
                            });
                        });
                    });
                });

                draw_sep(ui, sep_width);

                draw_dash_column(ui, col_width, "Ammo", |ui| {
                    ui.horizontal(|ui| {
                        ui.style_mut().spacing.item_spacing.x = 8.0;

                        ui.vertical(|ui| {
                            let m_bul = state.get_max_ammo(0);
                            let m_shl = state.get_max_ammo(1);
                            let m_rkt = state.get_max_ammo(3);
                            let m_cel = state.get_max_ammo(2);

                            egui::Grid::new("amm_grid")
                                .spacing(egui::vec2(4.0, 1.0))
                                .show(ui, |ui| {
                                    ui.label("Bullets:");
                                    ui.add(egui::DragValue::new(&mut state.inventory.ammo_bullets).range(0..=m_bul));
                                    ui.end_row();
                                    ui.label("Shells:");
                                    ui.add(egui::DragValue::new(&mut state.inventory.ammo_shells).range(0..=m_shl));
                                    ui.end_row();
                                    ui.label("Rockets:");
                                    ui.add(egui::DragValue::new(&mut state.inventory.ammo_rockets).range(0..=m_rkt));
                                    ui.end_row();
                                    ui.label("Cells:");
                                    ui.add(egui::DragValue::new(&mut state.inventory.ammo_cells).range(0..=m_cel));
                                    ui.end_row();
                                });
                        });

                        ui.vertical(|ui| {
                            ui.set_width(42.0);
                            ui.vertical_centered(|ui| {
                                if draw_icon_button(ui, assets, "BPAKA0", state.inventory.has_backpack, "Pack").clicked() {
                                    state.inventory.has_backpack = !state.inventory.has_backpack;
                                }
                                ui.label(egui::RichText::new("Backpack").weak().size(10.0));
                            });
                        });
                    });
                });

                draw_sep(ui, sep_width);

                draw_dash_column(ui, col_width, "Engine State", |ui| {
                    ui.checkbox(&mut state.engine.widescreen_mode, "Widescreen Mode");

                    ui.horizontal(|ui| {
                        ui.label("HUD Style:");
                        ui.add_space(8.0);

                        let hud_text = if state.engine.hud_mode == 0 { "Standard" } else { "Compact" };
                        if ui.button(hud_text).clicked() {
                            state.engine.hud_mode = 1 - state.engine.hud_mode;
                        }
                    });

                    ui.add_space(2.0);

                    ui.label("Automap:");
                    ui.horizontal(|ui| {
                        if ui.toggle_value(&mut state.engine.automap_active, "Active").changed() {
                            if !state.engine.automap_active {
                                state.engine.automap_overlay = false;
                            }
                        }

                        if state.engine.automap_active {
                            ui.add_space(8.0);
                            ui.toggle_value(&mut state.engine.automap_overlay, "Overlay");
                        }
                    });
                });
            } );
        });
    });

    if state.player.health > old_h {
        state.push_message("Picked up a health bonus.");
    } else if state.player.health < old_h && state.player.health > 0 {
        state.pain_timer = 1.0;
    } else if state.player.health == 0 && old_h > 0 {
        state.push_message("Doomguy was killed by a cruel SBARDEF editor.");
    }

    if state.player.armor > old_a { state.push_message("Picked up an armor bonus."); }

    if state.inventory.ammo_bullets > old_bul { state.push_message("Picked up a clip."); }
    if state.inventory.ammo_shells > old_shl { state.push_message("Picked up 4 shotgun shells."); }
    if state.inventory.ammo_rockets > old_rkt { state.push_message("Picked up a rocket."); }
    if state.inventory.ammo_cells > old_cel { state.push_message("Picked up an energy cell."); }

    if state.inventory.has_backpack && !old_pack { state.push_message("Picked up a backpack full of ammo!"); }
}

fn draw_dash_column<F>(ui: &mut egui::Ui, width: f32, title: &str, add_contents: F)
where
    F: FnOnce(&mut egui::Ui),
{
    ui.allocate_ui(egui::vec2(width, 110.0), |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.heading(title);
            ui.separator();
            add_contents(ui);
        });
    });
}

fn draw_sep(ui: &mut egui::Ui, width: f32) {
    ui.allocate_ui(egui::vec2(width, 110.0), |ui| {
        ui.centered_and_justified(|ui| {
            ui.add(egui::Separator::default().vertical());
        });
    });
}

fn draw_icon_button(
    ui: &mut egui::Ui,
    assets: &AssetStore,
    patch: &str,
    is_active: bool,
    fallback_text: &str,
) -> egui::Response {
    let size = 42.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

    let bg_color = if is_active {
        egui::Color32::from_gray(60)
    } else {
        egui::Color32::from_gray(30)
    };

    let stroke = if is_active {
        ui.visuals().widgets.hovered.bg_stroke
    } else {
        egui::Stroke::new(1.0, egui::Color32::from_gray(50))
    };

    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
    } else {
        ui.painter().rect_filled(rect, 4.0, bg_color);
    }
    ui.painter()
        .rect_stroke(rect, 4.0, stroke, egui::StrokeKind::Middle);

    let tint = if is_active {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_gray(100)
    };

    if let Some(tex) = assets.textures.get(patch) {
        let content_size = size - 8.0;
        let tex_size = tex.size_vec2();
        if tex_size.x > 0.0 && tex_size.y > 0.0 {
            let scale = (content_size / tex_size.x)
                .min(content_size / tex_size.y)
                .min(2.0);
            let final_size = tex_size * scale;
            let draw_rect = egui::Rect::from_center_size(rect.center(), final_size);
            ui.painter().image(
                tex.id(),
                draw_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
        }
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            fallback_text,
            egui::FontId::proportional(12.0),
            tint,
        );
    }

    response
}