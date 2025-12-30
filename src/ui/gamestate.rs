use crate::assets::AssetStore;
use crate::state::PreviewState;
use crate::ui::shared;
use eframe::egui;

const BTN_SIZE: f32 = 52.0;
const ROUNDING: f32 = 4.0;
const INNER_MARGIN: f32 = 5.0;
const GRID_SPACING: f32 = 11.0;

const TOTAL_GRID_WIDTH: f32 = (4.0 * BTN_SIZE) + (3.0 * GRID_SPACING);

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
        let key = badge.or(Some(patch));
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
            draw_asset_button(ui, assets, Some(&patch_to_use), is_truly_selected, owned);

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
            if !new_owned && is_truly_selected {
                state.selected_weapon_slot = 0;
            }
        }

        if response.secondary_clicked() {
            if is_truly_selected {
                state.selected_weapon_slot = 0;
            } else {
                state.selected_weapon_slot = slot;
                match item_id {
                    ItemId::Chainsaw => state.inventory.has_chainsaw = true,
                    ItemId::Pistol => state.inventory.has_pistol = true,
                    ItemId::Shotgun => state.inventory.has_shotgun = true,
                    ItemId::SuperShotgun => state.inventory.has_super_shotgun = true,
                    ItemId::Chaingun => state.inventory.has_chaingun = true,
                    ItemId::RocketLauncher => state.inventory.has_rocket_launcher = true,
                    ItemId::PlasmaGun => state.inventory.has_plasma_gun = true,
                    ItemId::BFG => state.inventory.has_bfg = true,
                    _ => {}
                }
                if slot == 3 {
                    state.use_super_shotgun = is_ssg_variant;
                }
            }
        }

        ui.label(egui::RichText::new(label).size(11.0));
    });
}

fn draw_asset_button(
    ui: &mut egui::Ui,
    assets: &AssetStore,
    patch_key: Option<&str>,
    is_selected: bool,
    is_owned: bool,
) -> (egui::Rect, egui::Response) {
    let size = egui::vec2(BTN_SIZE, BTN_SIZE);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    let (bg_color, stroke) = if is_owned {
        (
            egui::Color32::from_gray(40),
            egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
        )
    } else {
        (
            egui::Color32::from_gray(25),
            egui::Stroke::new(1.0, egui::Color32::from_gray(50)),
        )
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
        (bg_color, stroke)
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

    if let Some(tex) = patch_key.and_then(|k| assets.textures.get(k)) {
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
