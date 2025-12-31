use crate::assets::{AssetId, AssetStore};
use crate::state::PreviewState;
use crate::ui::shared;
use eframe::egui;

const BTN_SIZE: f32 = 52.0;
const ROUNDING: f32 = 4.0;
const INNER_MARGIN: f32 = 5.0;
const GRID_SPACING: f32 = 11.0;

/// The total width required to fit 4 columns of buttons and their spacing.
const TOTAL_GRID_WIDTH: f32 = (4.0 * BTN_SIZE) + (3.0 * GRID_SPACING);

/// Internal identifier for inventory and powerup items within the UI panels.
#[derive(Copy, Clone, PartialEq)]
enum ItemId {
    BlueCard,
    YellowCard,
    RedCard,
    BlueSkull,
    YellowSkull,
    RedSkull,
    Berserk,
    Invisibility,
    Map,
    Radsuit,
    Liteamp,
    Invuln,
    Chainsaw,
    Pistol,
    Shotgun,
    SuperShotgun,
    Chaingun,
    RocketLauncher,
    PlasmaGun,
    BFG,
}

/// Draws the "Held Items" grid.
///
/// Arranges the Doom items and powerups in a 4-column grid that mirrors
/// the logical grouping of the classic status bar.
pub fn draw_gamestate_panel(ui: &mut egui::Ui, state: &mut PreviewState, assets: &AssetStore) {
    ui.vertical(|ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 4.0);

        ui.vertical_centered(|ui| {
            ui.allocate_ui(egui::vec2(TOTAL_GRID_WIDTH, 0.0), |ui| {
                egui::Grid::new("items_top_grid")
                    .spacing(egui::vec2(GRID_SPACING, GRID_SPACING))
                    .show(ui, |ui| {
                        item_btn(
                            ui,
                            assets,
                            state,
                            "BKEYA0",
                            None,
                            ItemId::BlueCard,
                            "Blue Card",
                        );
                        item_btn(
                            ui,
                            assets,
                            state,
                            "BSKUB0",
                            None,
                            ItemId::BlueSkull,
                            "Blue Skull",
                        );
                        item_btn(
                            ui,
                            assets,
                            state,
                            "PSTRA0",
                            Some("_BADGE_BERSERK"),
                            ItemId::Berserk,
                            "Berserk",
                        );
                        item_btn(
                            ui,
                            assets,
                            state,
                            "PINSA0",
                            Some("_BADGE_BLURSPHERE"),
                            ItemId::Invisibility,
                            "Invisibility",
                        );
                        ui.end_row();

                        item_btn(
                            ui,
                            assets,
                            state,
                            "YKEYA0",
                            None,
                            ItemId::YellowCard,
                            "Yel. Card",
                        );
                        item_btn(
                            ui,
                            assets,
                            state,
                            "YSKUB0",
                            None,
                            ItemId::YellowSkull,
                            "Yel. Skull",
                        );
                        item_btn(
                            ui,
                            assets,
                            state,
                            "PMAPA0",
                            Some("_BADGE_ALLMAP"),
                            ItemId::Map,
                            "Area Map",
                        );
                        item_btn(
                            ui,
                            assets,
                            state,
                            "SUITA0",
                            Some("_BADGE_RADSUIT"),
                            ItemId::Radsuit,
                            "Radsuit",
                        );
                        ui.end_row();

                        item_btn(
                            ui,
                            assets,
                            state,
                            "RKEYA0",
                            None,
                            ItemId::RedCard,
                            "Red Card",
                        );
                        item_btn(
                            ui,
                            assets,
                            state,
                            "RSKUB0",
                            None,
                            ItemId::RedSkull,
                            "Red Skull",
                        );
                        item_btn(
                            ui,
                            assets,
                            state,
                            "PVISA0",
                            Some("_BADGE_LITEAMP"),
                            ItemId::Liteamp,
                            "Liteamp",
                        );
                        item_btn(
                            ui,
                            assets,
                            state,
                            "PINVA0",
                            Some("_BADGE_INVULN"),
                            ItemId::Invuln,
                            "Invuln.",
                        );
                        ui.end_row();
                    });
            });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.vertical_centered(|ui| {
            ui.allocate_ui(egui::vec2(TOTAL_GRID_WIDTH, 0.0), |ui| {
                egui::Grid::new("weapons_bottom_grid")
                    .spacing(egui::vec2(GRID_SPACING, GRID_SPACING))
                    .show(ui, |ui| {
                        weapon_complex_btn(
                            ui,
                            assets,
                            state,
                            "SAWGA0",
                            1,
                            ItemId::Chainsaw,
                            "Chainsaw",
                        );
                        weapon_complex_btn(
                            ui,
                            assets,
                            state,
                            "STGNUM2",
                            2,
                            ItemId::Pistol,
                            "Pistol",
                        );
                        weapon_complex_btn(
                            ui,
                            assets,
                            state,
                            "STGNUM3",
                            3,
                            ItemId::Shotgun,
                            "Shotgun",
                        );
                        weapon_complex_btn(
                            ui,
                            assets,
                            state,
                            "STGNUM4",
                            4,
                            ItemId::Chaingun,
                            "Chaingun",
                        );
                        ui.end_row();

                        weapon_complex_btn(
                            ui,
                            assets,
                            state,
                            "SHT2A0",
                            3,
                            ItemId::SuperShotgun,
                            "S.Shotgun",
                        );
                        weapon_complex_btn(
                            ui,
                            assets,
                            state,
                            "STGNUM5",
                            5,
                            ItemId::RocketLauncher,
                            "Rocket",
                        );
                        weapon_complex_btn(
                            ui,
                            assets,
                            state,
                            "STGNUM6",
                            6,
                            ItemId::PlasmaGun,
                            "Plasma",
                        );
                        weapon_complex_btn(ui, assets, state, "STGNUM7", 7, ItemId::BFG, "BFG9000");
                        ui.end_row();
                    });
            });
        });
    });
}

/// Draws the "Game Context" panel (Health, Ammo counts, World level, Engine settings).
pub fn draw_context_panel(ui: &mut egui::Ui, state: &mut PreviewState, assets: &AssetStore) {
    ui.vertical_centered(|ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 6.0);

        let map_format_id = ui.make_persistent_id("use_doom2_map_format");
        let mut is_doom2 = ui.data(|d| d.get_temp::<bool>(map_format_id).unwrap_or(true));

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 170.0).max(0.0) / 2.0);
            ui.spacing_mut().item_spacing.x = 12.0;

            ui.vertical(|ui| {
                egui::Grid::new("sb_vit_grid")
                    .spacing(egui::vec2(4.0, 4.0))
                    .show(ui, |ui| {
                        ui.label("Health:");
                        ui.add(egui::DragValue::new(&mut state.player.health).range(0..=200));
                        ui.end_row();

                        ui.label("Armor:");
                        ui.add(egui::DragValue::new(&mut state.player.armor).range(0..=200));
                        ui.end_row();
                    });
            });

            ui.vertical(|ui| {
                ui.set_width(42.0);
                let is_blue = state.player.armor_max == 200;
                let patch = if is_blue { "ARM2A0" } else { "ARM1A0" };
                let is_active = state.player.armor > 0;

                let tooltip = if is_blue {
                    "Blue Armor (Megaarmor)"
                } else {
                    "Green Armor"
                };

                let response =
                    draw_icon_button(ui, assets, patch, is_active, "Armor").on_hover_text(tooltip);

                if response.clicked() {
                    state.player.armor_max = if is_blue { 100 } else { 200 };
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 180.0).max(0.0) / 2.0);
            ui.spacing_mut().item_spacing.x = 12.0;

            ui.vertical(|ui| {
                let m_bul = state.inventory.get_max_ammo(0);
                let m_shl = state.inventory.get_max_ammo(1);
                let m_rkt = state.inventory.get_max_ammo(3);
                let m_cel = state.inventory.get_max_ammo(2);

                egui::Grid::new("sb_amm_grid")
                    .spacing(egui::vec2(4.0, 1.0))
                    .show(ui, |ui| {
                        ui.label("Bullets:");
                        ui.add(
                            egui::DragValue::new(&mut state.inventory.ammo_bullets)
                                .range(0..=m_bul),
                        );
                        ui.end_row();
                        ui.label("Shells:");
                        ui.add(
                            egui::DragValue::new(&mut state.inventory.ammo_shells).range(0..=m_shl),
                        );
                        ui.end_row();
                        ui.label("Rockets:");
                        ui.add(
                            egui::DragValue::new(&mut state.inventory.ammo_rockets)
                                .range(0..=m_rkt),
                        );
                        ui.end_row();
                        ui.label("Cells:");
                        ui.add(
                            egui::DragValue::new(&mut state.inventory.ammo_cells).range(0..=m_cel),
                        );
                        ui.end_row();
                    });
            });

            ui.vertical(|ui| {
                ui.set_width(42.0);
                let _ =
                    draw_icon_button(ui, assets, "BPAKA0", state.inventory.has_backpack, "Pack")
                        .clicked()
                        .then(|| state.inventory.has_backpack = !state.inventory.has_backpack);
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 170.0).max(0.0) / 2.0);
            egui::Grid::new("sb_stats_grid")
                .num_columns(2)
                .spacing(egui::vec2(12.0, 6.0))
                .show(ui, |ui| {
                    let draw_stat_row =
                        |ui: &mut egui::Ui, label: &str, cur: &mut i32, max: &mut i32| {
                            ui.label(label);
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(cur).range(0..=999).speed(0.5));
                                ui.label("/");
                                ui.add(egui::DragValue::new(max).range(0..=999).speed(0.5));
                            });
                            ui.end_row();
                        };
                    draw_stat_row(
                        ui,
                        "Kills:",
                        &mut state.player.kills,
                        &mut state.player.max_kills,
                    );
                    draw_stat_row(
                        ui,
                        "Items:",
                        &mut state.player.items,
                        &mut state.player.max_items,
                    );
                    draw_stat_row(
                        ui,
                        "Secrets:",
                        &mut state.player.secrets,
                        &mut state.player.max_secrets,
                    );
                });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 200.0).max(0.0) / 2.0);
            egui::Grid::new("sb_ctx_grid")
                .num_columns(2)
                .spacing(egui::vec2(12.0, 4.0))
                .show(ui, |ui| {
                    ui.label("Mode:");
                    egui::ComboBox::from_id_salt("sb_mode_dd")
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

                    ui.label("Feature Set:");
                    egui::ComboBox::from_id_salt("sb_ver_dd")
                        .selected_text(format!("{:?}", state.world.game_version))
                        .width(110.0)
                        .show_ui(ui, |ui| {
                            use crate::model::FeatureLevel::*;
                            ui.selectable_value(&mut state.world.game_version, Doom19, "Doom 1.9");
                            ui.selectable_value(
                                &mut state.world.game_version,
                                LimitRemoving,
                                "Limit Removing",
                            );
                            ui.selectable_value(&mut state.world.game_version, Boom, "Boom 2.02");
                            ui.selectable_value(
                                &mut state.world.game_version,
                                Complevel9,
                                "Comp Lvl 9",
                            );
                            ui.selectable_value(&mut state.world.game_version, MBF, "MBF");
                            ui.selectable_value(&mut state.world.game_version, MBF21, "MBF21");
                            ui.selectable_value(&mut state.world.game_version, ID24, "ID24");
                        });
                    ui.end_row();

                    ui.label("Map:");
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        if is_doom2 {
                            ui.add(
                                egui::DragValue::new(&mut state.world.level)
                                    .range(1..=999)
                                    .prefix("MAP"),
                            );
                        } else {
                            ui.add(
                                egui::DragValue::new(&mut state.world.episode)
                                    .prefix("E")
                                    .range(1..=9),
                            );
                            ui.add(
                                egui::DragValue::new(&mut state.world.level)
                                    .prefix("M")
                                    .range(1..=32),
                            );
                        }
                        if ui.button("🔄").clicked() {
                            is_doom2 = !is_doom2;
                        }
                    });
                    ui.end_row();
                });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        ui.checkbox(&mut state.engine.widescreen_mode, "Widescreen Mode");

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
            ui.label("Automap:");
            if ui
                .toggle_value(&mut state.engine.automap_active, "Active")
                .changed()
            {
                if !state.engine.automap_active {
                    state.engine.automap_overlay = false;
                }
            }
            ui.add_enabled(state.engine.automap_active, |ui: &mut egui::Ui| {
                ui.toggle_value(&mut state.engine.automap_overlay, "Overlay")
            });
        });

        ui.data_mut(|d| d.insert_temp(map_format_id, is_doom2));
    });
}

fn item_btn(
    ui: &mut egui::Ui,
    assets: &AssetStore,
    state: &mut PreviewState,
    patch: &str,
    badge: Option<&str>,
    item_id: ItemId,
    label: &str,
) {
    let pwr_id = match item_id {
        ItemId::Invuln => Some(0),
        ItemId::Berserk => Some(1),
        ItemId::Invisibility => Some(2),
        ItemId::Radsuit => Some(3),
        ItemId::Map => Some(4),
        ItemId::Liteamp => Some(5),
        _ => None,
    };

    let is_owned = if let Some(id) = pwr_id {
        state
            .player
            .powerup_durations
            .get(&id)
            .map_or(false, |v| *v > 0.0)
    } else {
        match item_id {
            ItemId::BlueCard => state.inventory.has_blue_card,
            ItemId::YellowCard => state.inventory.has_yellow_card,
            ItemId::RedCard => state.inventory.has_red_card,
            ItemId::BlueSkull => state.inventory.has_blue_skull,
            ItemId::YellowSkull => state.inventory.has_yellow_skull,
            ItemId::RedSkull => state.inventory.has_red_skull,
            _ => false,
        }
    };

    ui.vertical_centered(|ui| {
        let key = badge.or(Some(patch)).unwrap();
        let (_rect, response) = draw_asset_button(ui, assets, key, false, is_owned);

        if response.clicked() {
            let new_val = !is_owned;
            if let Some(id) = pwr_id {
                let dur = if new_val {
                    match id {
                        0 => 30.0,
                        1 => 1.0,
                        2 => 60.0,
                        3 => 60.0,
                        4 => 1.0,
                        5 => 120.0,
                        _ => 30.0,
                    }
                } else {
                    0.0
                };
                state.player.powerup_durations.insert(id, dur);
            } else {
                match item_id {
                    ItemId::BlueCard => state.inventory.has_blue_card = new_val,
                    ItemId::YellowCard => state.inventory.has_yellow_card = new_val,
                    ItemId::RedCard => state.inventory.has_red_card = new_val,
                    ItemId::BlueSkull => state.inventory.has_blue_skull = new_val,
                    ItemId::YellowSkull => state.inventory.has_yellow_skull = new_val,
                    ItemId::RedSkull => state.inventory.has_red_skull = new_val,
                    _ => {}
                }
            }
        }

        let mut display_label = label.to_string();
        if let Some(id) = pwr_id {
            let dur = state
                .player
                .powerup_durations
                .get(&id)
                .cloned()
                .unwrap_or(0.0);
            if dur > 0.0 && id != 1 && id != 4 {
                let secs = dur as i32;
                display_label = format!("{}:{:02}", secs / 60, secs % 60);
            }
        }

        ui.label(egui::RichText::new(display_label).size(11.0));
    });
}

fn weapon_complex_btn(
    ui: &mut egui::Ui,
    assets: &AssetStore,
    state: &mut PreviewState,
    patch: &str,
    slot: u8,
    item_id: ItemId,
    label: &str,
) {
    let owned = match item_id {
        ItemId::Chainsaw => state.inventory.has_chainsaw,
        ItemId::Pistol => state.inventory.has_pistol,
        ItemId::Shotgun => state.inventory.has_shotgun,
        ItemId::SuperShotgun => state.inventory.has_super_shotgun,
        ItemId::Chaingun => state.inventory.has_chaingun,
        ItemId::RocketLauncher => state.inventory.has_rocket_launcher,
        ItemId::PlasmaGun => state.inventory.has_plasma_gun,
        ItemId::BFG => state.inventory.has_bfg,
        _ => false,
    };

    let is_selected_slot = state.selected_weapon_slot == slot;
    let is_ssg_variant = item_id == ItemId::SuperShotgun;
    let is_truly_selected = if slot == 3 {
        is_selected_slot && state.use_super_shotgun == is_ssg_variant
    } else {
        is_selected_slot
    };

    let patch_to_use = if is_truly_selected && patch.starts_with("STGNUM") {
        format!("STYSNUM{}", slot)
    } else {
        patch.to_string()
    };

    ui.vertical_centered(|ui| {
        let (_rect, response) =
            draw_asset_button(ui, assets, &patch_to_use, is_truly_selected, owned);

        if response.clicked() {
            let new_owned = !owned;
            match item_id {
                ItemId::Chainsaw => state.inventory.has_chainsaw = new_owned,
                ItemId::Pistol => state.inventory.has_pistol = new_owned,
                ItemId::Shotgun => state.inventory.has_shotgun = new_owned,
                ItemId::SuperShotgun => state.inventory.has_super_shotgun = new_owned,
                ItemId::Chaingun => state.inventory.has_chaingun = new_owned,
                ItemId::RocketLauncher => state.inventory.has_rocket_launcher = new_owned,
                ItemId::PlasmaGun => state.inventory.has_plasma_gun = new_owned,
                ItemId::BFG => state.inventory.has_bfg = new_owned,
                _ => {}
            }
        }

        if response.secondary_clicked() {
            state.selected_weapon_slot = if is_truly_selected { 0 } else { slot };
            if slot == 3 {
                state.use_super_shotgun = is_ssg_variant;
            }
        }

        ui.label(egui::RichText::new(label).size(11.0));
    });
}

fn draw_icon_button(
    ui: &mut egui::Ui,
    assets: &AssetStore,
    patch: &str,
    is_active: bool,
    fallback_text: &str,
) -> egui::Response {
    let size = 40.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());

    let (bg_color, stroke) = if is_active {
        (
            egui::Color32::from_gray(60),
            ui.visuals().widgets.hovered.bg_stroke,
        )
    } else {
        (
            egui::Color32::from_gray(30),
            egui::Stroke::new(1.0, egui::Color32::from_gray(50)),
        )
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

    let id = AssetId::new(patch);
    if let Some(tex) = assets.textures.get(&id) {
        shared::draw_scaled_image(ui, rect.shrink(4.0), tex, tint, 4.0);
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

fn draw_asset_button(
    ui: &mut egui::Ui,
    assets: &AssetStore,
    patch_key: &str,
    is_selected: bool,
    is_owned: bool,
) -> (egui::Rect, egui::Response) {
    let size = egui::vec2(BTN_SIZE, BTN_SIZE);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    let bg_color = if is_owned {
        egui::Color32::from_gray(40)
    } else {
        egui::Color32::from_gray(25)
    };
    let (final_bg, final_stroke) = if is_selected {
        (
            ui.visuals().selection.bg_fill,
            ui.visuals().selection.stroke,
        )
    } else if response.hovered() {
        (
            ui.visuals().widgets.hovered.bg_fill,
            ui.visuals().widgets.hovered.bg_stroke,
        )
    } else {
        (
            bg_color,
            egui::Stroke::new(1.0, egui::Color32::from_gray(50)),
        )
    };

    ui.painter().rect(
        rect,
        ROUNDING,
        final_bg,
        final_stroke,
        egui::StrokeKind::Middle,
    );
    let tint = if is_owned {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_gray(100)
    };

    let id = AssetId::new(patch_key);
    if let Some(tex) = assets.textures.get(&id) {
        shared::draw_scaled_image(ui, rect.shrink(INNER_MARGIN), tex, tint, 4.0);
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "?",
            egui::FontId::proportional(14.0),
            tint,
        );
    }
    (rect, response)
}
