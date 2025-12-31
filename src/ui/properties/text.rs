use super::FontCache;
use super::common;
use super::editor::PropertiesUI;
use super::lookups;
use super::preview::PreviewContent;
use crate::assets::AssetStore;
use crate::model::{Element, ElementWrapper, NumberDef, NumberType, StringDef};
use crate::state::PreviewState;
use crate::ui::context_menu::ContextMenu;
use eframe::egui;

const HEADER_MENU_KEY: &str = "cacoco_prop_header_menu_id";

impl PropertiesUI for NumberDef {
    /// Renders the font and source parameter editor for numeric elements.
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

                let id = ui.make_persistent_id("num_font_selector");
                let button_res =
                    ui.add(egui::Button::new(&self.font).min_size(egui::vec2(140.0, 18.0)));

                if button_res.clicked() {
                    ContextMenu::open(ui, id, button_res.rect.left_bottom());
                }

                if let Some(menu) = ContextMenu::get(ui, id) {
                    ContextMenu::show(ui, menu, button_res.clicked(), |ui| {
                        let h = (fonts.number_font_names.len() as f32 * 42.0).min(400.0);
                        ui.set_min_height(h);
                        ui.set_width(200.0);
                        changed |=
                            common::draw_number_font_selectors(ui, &mut self.font, fonts, assets);
                    });
                }
            });

            match self.type_ {
                NumberType::Ammo | NumberType::MaxAmmo => {
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 200.0).max(0.0) / 2.0);
                        ui.label("Ammo Type:");
                        changed |= common::draw_lookup_param_dd(
                            ui,
                            "num_param_ammo",
                            &mut self.param,
                            lookups::AMMO_TYPES,
                            assets,
                        );
                    });
                }
                NumberType::AmmoWeapon | NumberType::MaxAmmoWeapon => {
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 220.0).max(0.0) / 2.0);
                        ui.label("Weapon Source:");
                        changed |= common::draw_lookup_param_dd(
                            ui,
                            "num_param_weapon",
                            &mut self.param,
                            lookups::WEAPONS,
                            assets,
                        );
                    });
                }
                NumberType::PowerupDuration => {
                    ui.horizontal(|ui| {
                        ui.add_space((ui.available_width() - 200.0).max(0.0) / 2.0);
                        ui.label("Powerup:");
                        changed |= common::draw_lookup_param_dd(
                            ui,
                            "num_param_powerup",
                            &mut self.param,
                            lookups::POWERUPS,
                            assets,
                        );
                    });
                }
                _ => {}
            }

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 130.0).max(0.0) / 2.0);
                ui.label("Max Length:");
                changed |= ui
                    .add_sized(
                        [40.0, 18.0],
                        egui::DragValue::new(&mut self.maxlength).range(0..=9),
                    )
                    .changed();
            });
        });

        changed
    }

    /// Pulls current stats from the simulated player to populate the preview string.
    fn get_preview_content(
        &self,
        _ui: &egui::Ui,
        fonts: &FontCache,
        state: &PreviewState,
    ) -> Option<PreviewContent> {
        let stem = fonts.get_number_stem(&self.font);
        let val = match self.type_ {
            NumberType::Health => state.player.health,
            NumberType::Armor => state.player.armor,
            NumberType::Frags => 0,
            NumberType::AmmoSelected => {
                let slot = state.selected_weapon_slot;
                let idx = state.inventory.get_selected_ammo_type(slot);
                state.inventory.get_ammo(idx)
            }
            NumberType::Ammo => state.inventory.get_ammo(self.param),
            NumberType::MaxAmmo => state.inventory.get_max_ammo(self.param),
            NumberType::AmmoWeapon => state
                .inventory
                .get_weapon_ammo_type(self.param)
                .map_or(0, |idx| state.inventory.get_ammo(idx)),
            NumberType::MaxAmmoWeapon => state
                .inventory
                .get_weapon_ammo_type(self.param)
                .map_or(0, |idx| state.inventory.get_max_ammo(idx)),
            NumberType::Kills => state.player.kills,
            NumberType::Items => state.player.items,
            NumberType::Secrets => state.player.secrets,
            NumberType::MaxKills => state.player.max_kills,
            NumberType::MaxItems => state.player.max_items,
            NumberType::MaxSecrets => state.player.max_secrets,
            NumberType::KillsPercent => {
                state.get_stat_percent(state.player.kills, state.player.max_kills)
            }
            NumberType::ItemsPercent => {
                state.get_stat_percent(state.player.items, state.player.max_items)
            }
            NumberType::SecretsPercent => {
                state.get_stat_percent(state.player.secrets, state.player.max_secrets)
            }
            NumberType::PowerupDuration => state
                .player
                .powerup_durations
                .get(&self.param)
                .cloned()
                .unwrap_or(0.0) as i32,
        };
        Some(PreviewContent::Text {
            text: format!("{}", val),
            stem,
            is_number_font: true,
        })
    }
}

impl PropertiesUI for StringDef {
    /// Renders the specialized editor for SBARDEF String elements.
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
                ui.add_space((ui.available_width() - 210.0).max(0.0) / 2.0);
                ui.label("String Type:");
                let mut current_type = self.type_ as i32;
                if common::draw_lookup_param_dd(
                    ui,
                    "string_type_selector",
                    &mut current_type,
                    lookups::STRING_TYPES,
                    assets,
                ) {
                    self.type_ = current_type as u8;
                    changed = true;
                }
            });

            if self.type_ == 0 {
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 220.0).max(0.0) / 2.0);
                    ui.label("Data:");
                    let mut buf = self.data.clone().unwrap_or_default();
                    if ui
                        .add_sized([140.0, 18.0], egui::TextEdit::singleline(&mut buf))
                        .changed()
                    {
                        self.data = Some(buf);
                        changed = true;
                    }
                });
            }

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 190.0).max(0.0) / 2.0);
                ui.label("Font:");

                let id = ui.make_persistent_id("string_font_selector");
                let button_res =
                    ui.add(egui::Button::new(&self.font).min_size(egui::vec2(140.0, 18.0)));

                if button_res.clicked() {
                    ContextMenu::open(ui, id, button_res.rect.left_bottom());
                }

                if let Some(menu) = ContextMenu::get(ui, id) {
                    ContextMenu::show(ui, menu, button_res.clicked(), |ui| {
                        let h = (fonts.hud_font_names.len() as f32 * 42.0).min(400.0);
                        ui.set_min_height(h);
                        ui.set_width(200.0);
                        changed |=
                            common::draw_hud_font_selectors(ui, &mut self.font, fonts, assets);
                    });
                }
            });
        });
        changed
    }

    /// Generates a preview string based on the current world and level data.
    fn get_preview_content(
        &self,
        _ui: &egui::Ui,
        fonts: &FontCache,
        _state: &PreviewState,
    ) -> Option<PreviewContent> {
        let stem = fonts.get_hud_stem(&self.font);
        let text = match self.type_ {
            0 => self
                .data
                .as_deref()
                .unwrap_or("Having Fun with Cacoco!")
                .to_string(),
            1 => "Entryway".to_string(),
            2 => "MAP01".to_string(),
            3 => "Sandy Petersen".to_string(),
            _ => String::new(),
        };
        Some(PreviewContent::Text {
            text,
            stem,
            is_number_font: false,
        })
    }
}

/// Renders the interactive property panel header.
///
/// For interactive types (Number, Component, String), clicking the header
/// opens a popup to change the specific subtype.
pub fn draw_interactive_header(
    ui: &mut egui::Ui,
    element: &mut ElementWrapper,
    helper_text: &str,
    frame_color: egui::Color32,
) -> bool {
    let mut changed = false;
    let frame = egui::Frame::new()
        .inner_margin(8.0)
        .corner_radius(4.0)
        .fill(frame_color)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)));

    let response = frame
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let title = if element._cacoco_text.is_some() {
                    "Text String".to_string()
                } else {
                    match &element.data {
                        Element::Number(n) => number_type_name(n.type_).to_string(),
                        Element::Percent(p) => number_type_name(p.type_).to_string(),
                        Element::Component(c) => format!("{:?}", c.type_),
                        Element::String(s) => {
                            format!("String: {}", lookups::STRING_TYPES[s.type_ as usize].name)
                        }
                        Element::Graphic(_) => "Graphic".to_string(),
                        Element::Face(_) => "Doomguy".to_string(),
                        Element::FaceBackground(_) => "Face Background".to_string(),
                        Element::Animation(_) => "Animation".to_string(),
                        Element::Canvas(_) => "Canvas Group".to_string(),
                        Element::List(_) => "List Container".to_string(),
                        Element::Carousel(_) => "Carousel".to_string(),
                    }
                };

                ui.add_sized(
                    [ui.available_width(), 0.0],
                    egui::Label::new(egui::RichText::new(title).size(16.0).strong()),
                );
                ui.add(egui::Separator::default().spacing(8.0));
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(helper_text).weak().italics());
                });
            });
        })
        .response;

    let is_interactive = matches!(
        element.data,
        Element::Number(_) | Element::Percent(_) | Element::Component(_) | Element::String(_)
    );

    if is_interactive {
        let interact = ui.interact(response.rect, response.id, egui::Sense::click());
        if interact.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            ui.painter()
                .rect_filled(response.rect, 4.0, egui::Color32::from_white_alpha(20));
        }

        let header_id = response.id;
        let is_open =
            ui.data(|d| d.get_temp::<egui::Id>(egui::Id::new(HEADER_MENU_KEY)) == Some(header_id));

        if interact.clicked() {
            if is_open {
                ui.data_mut(|d| d.remove::<egui::Id>(egui::Id::new(HEADER_MENU_KEY)));
            } else {
                ui.data_mut(|d| d.insert_temp(egui::Id::new(HEADER_MENU_KEY), header_id));
            }
        }

        if is_open {
            let area_response = egui::Area::new(header_id.with("manual_popup"))
                .order(egui::Order::Foreground)
                .fixed_pos(response.rect.left_bottom())
                .show(ui.ctx(), |ui| {
                    let frame = egui::Frame::popup(ui.style())
                        .inner_margin(4.0)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(60)));

                    frame.show(ui, |ui| {
                        ui.set_min_width(response.rect.width().max(100.0));
                        ui.set_max_width(200.0);
                        let mut close = false;

                        match &mut element.data {
                            Element::Number(n) => {
                                if draw_number_options(ui, &mut n.type_, &mut n.param) {
                                    close = true;
                                    changed = true;
                                }
                            }
                            Element::Percent(p) => {
                                if draw_number_options(ui, &mut p.type_, &mut p.param) {
                                    close = true;
                                    changed = true;
                                }
                            }
                            Element::Component(c) => {
                                if super::components::draw_component_options(ui, &mut c.type_) {
                                    close = true;
                                    changed = true;
                                }
                            }
                            Element::String(s) => {
                                for item in lookups::STRING_TYPES {
                                    if common::custom_menu_item(
                                        ui,
                                        item.name,
                                        s.type_ == item.id as u8,
                                    ) {
                                        s.type_ = item.id as u8;
                                        close = true;
                                        changed = true;
                                    }
                                }
                            }
                            _ => {}
                        }
                        if close {
                            ui.data_mut(|d| d.remove::<egui::Id>(egui::Id::new(HEADER_MENU_KEY)));
                        }
                    });
                })
                .response;

            if ui.input(|i| i.pointer.any_click())
                && !area_response.hovered()
                && !response.hovered()
            {
                ui.data_mut(|d| d.remove::<egui::Id>(egui::Id::new(HEADER_MENU_KEY)));
            }
        }
    }

    if let Element::Graphic(g) = &mut element.data {
        if let Some(asset_keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
            if ui.rect_contains_pointer(response.rect) {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Copy);
                ui.painter()
                    .rect_filled(response.rect, 4.0, ui.visuals().widgets.active.bg_fill);
                ui.painter().rect_stroke(
                    response.rect,
                    4.0,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    egui::StrokeKind::Inside,
                );

                ui.painter().text(
                    response.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Replace Graphic",
                    egui::FontId::proportional(16.0),
                    egui::Color32::WHITE,
                );

                if ui.input(|i| i.pointer.any_released()) {
                    if let Some(key) = asset_keys.get(0) {
                        g.patch = key.clone();
                        changed = true;
                    }
                    egui::DragAndDrop::clear_payload(ui.ctx());
                }
            }
        }
    }
    changed
}

/// Helper to get the human-readable label for a numeric statistic type.
pub fn number_type_name(t: NumberType) -> &'static str {
    match t {
        NumberType::Health => "Health",
        NumberType::Armor => "Armor",
        NumberType::Frags => "Frags",
        NumberType::Ammo => "Ammo (by Type)",
        NumberType::AmmoSelected => "Selected Ammo",
        NumberType::MaxAmmo => "Max Ammo (by Type)",
        NumberType::AmmoWeapon => "Ammo (by Weapon)",
        NumberType::MaxAmmoWeapon => "Max Ammo (by Weapon)",
        NumberType::Kills => "Kills",
        NumberType::Items => "Items",
        NumberType::Secrets => "Secrets",
        NumberType::KillsPercent => "Kills %",
        NumberType::ItemsPercent => "Items %",
        NumberType::SecretsPercent => "Secrets %",
        NumberType::MaxKills => "Max Kills",
        NumberType::MaxItems => "Max Items",
        NumberType::MaxSecrets => "Max Secrets",
        NumberType::PowerupDuration => "Powerup Time",
    }
}

fn draw_number_options(ui: &mut egui::Ui, type_: &mut NumberType, param: &mut i32) -> bool {
    let mut changed = false;
    let items = [
        ("Health", NumberType::Health),
        ("Armor", NumberType::Armor),
        ("Frags", NumberType::Frags),
        ("Kills", NumberType::Kills),
        ("Items", NumberType::Items),
        ("Secrets", NumberType::Secrets),
    ];

    for (label, target) in items {
        if common::custom_menu_item(ui, label, *type_ == target) {
            *type_ = target;
            changed = true;
        }
    }

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    if common::custom_menu_item(ui, "Ammo (by Type)", *type_ == NumberType::Ammo) {
        *type_ = NumberType::Ammo;
        if !lookups::AMMO_TYPES.iter().any(|i| i.id == *param) {
            *param = 0;
        }
        changed = true;
    }
    if common::custom_menu_item(ui, "Max Ammo (by Type)", *type_ == NumberType::MaxAmmo) {
        *type_ = NumberType::MaxAmmo;
        if !lookups::AMMO_TYPES.iter().any(|i| i.id == *param) {
            *param = 0;
        }
        changed = true;
    }
    if common::custom_menu_item(ui, "Powerup Time", *type_ == NumberType::PowerupDuration) {
        *type_ = NumberType::PowerupDuration;
        if !lookups::POWERUPS.iter().any(|i| i.id == *param) {
            *param = 0;
        }
        changed = true;
    }

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    if common::custom_menu_item(ui, "Ammo (by Weapon)", *type_ == NumberType::AmmoWeapon) {
        *type_ = NumberType::AmmoWeapon;
        if !lookups::WEAPONS.iter().any(|i| i.id == *param) {
            *param = 101;
        }
        changed = true;
    }
    if common::custom_menu_item(
        ui,
        "Max Ammo (by Weapon)",
        *type_ == NumberType::MaxAmmoWeapon,
    ) {
        *type_ = NumberType::MaxAmmoWeapon;
        if !lookups::WEAPONS.iter().any(|i| i.id == *param) {
            *param = 101;
        }
        changed = true;
    }
    changed
}
