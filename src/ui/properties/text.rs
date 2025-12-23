use crate::assets::AssetStore;
use crate::model::{ComponentDef, ComponentType, ElementWrapper, NumberDef, NumberType, Element};
use crate::state::PreviewState;
use crate::ui::layers::thumbnails;
use eframe::egui;
use super::editor::PropertiesUI;
use super::preview::PreviewContent;
use super::FontCache;

const HEADER_MENU_KEY: &str = "cacoco_prop_header_menu_id";

impl PropertiesUI for NumberDef {
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        fonts: &FontCache,
        assets: &AssetStore,
        _state: &PreviewState,
    ) {
        ui.horizontal(|ui| {
            ui.label("Font:");
            egui::ComboBox::from_id_salt("num_font_selector")
                .selected_text(self.font.clone())
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    let h = (fonts.number_font_names.len() as f32 * 42.0).min(250.0);
                    ui.set_min_height(h);
                    for (i, name) in fonts.number_font_names.iter().enumerate() {
                        let stem = fonts.get_number_stem(name);
                        draw_font_selection_row(
                            ui,
                            &mut self.font,
                            name,
                            stem.as_ref(),
                            assets,
                            true,
                            i,
                        );
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Param:");
            ui.add(egui::DragValue::new(&mut self.param));
        });

        ui.horizontal(|ui| {
            ui.label("Max Len:");
            ui.add(egui::DragValue::new(&mut self.maxlength));
        });
    }

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
            NumberType::AmmoSelected => state.get_ammo(state.get_selected_ammo_type()),
            NumberType::Ammo => state.get_ammo(self.param),
            NumberType::MaxAmmo => state.get_max_ammo(self.param),
            NumberType::AmmoWeapon => state
                .get_weapon_ammo_type(self.param)
                .map_or(0, |idx| state.get_ammo(idx)),
            NumberType::MaxAmmoWeapon => state
                .get_weapon_ammo_type(self.param)
                .map_or(0, |idx| state.get_max_ammo(idx)),
        };
        Some(PreviewContent::Text {
            text: format!("{}", val),
            stem,
            is_number_font: true,
        })
    }
}

impl PropertiesUI for ComponentDef {
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        fonts: &FontCache,
        assets: &AssetStore,
        _state: &PreviewState,
    ) {
        ui.horizontal(|ui| {
            ui.label("Font:");
            egui::ComboBox::from_id_salt("hud_font_selector")
                .selected_text(self.font.clone())
                .width(ui.available_width())
                .height(600.0)
                .show_ui(ui, |ui| {
                    for (i, name) in fonts.hud_font_names.iter().enumerate() {
                        let stem = fonts.get_hud_stem(name);
                        draw_font_selection_row(
                            ui,
                            &mut self.font,
                            name,
                            stem.as_ref(),
                            assets,
                            false,
                            i,
                        );
                    }
                });
        });
    }

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
            ComponentType::FpsCounter => format!("{:.0}", state.display_fps),
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

pub fn draw_interactive_header(
    ui: &mut egui::Ui,
    element: &mut ElementWrapper,
    helper_text: &str,
    frame_color: egui::Color32,
) {
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
                        Element::Component(c) => component_type_name(c.type_),
                        Element::Graphic(_) => "Graphic".to_string(),
                        Element::Face(_) => "Doomguy".to_string(),
                        Element::FaceBackground(_) => "Face Background".to_string(),
                        Element::Animation(_) => "Animation".to_string(),
                        Element::Canvas(_) => "Canvas Group".to_string(),
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

    let is_interactive = matches!(element.data, Element::Number(_) | Element::Percent(_) | Element::Component(_));

    if is_interactive {
        let interact = ui.interact(response.rect, response.id, egui::Sense::click());
        if interact.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            ui.painter()
                .rect_filled(response.rect, 4.0, egui::Color32::from_white_alpha(20));
        }

        let header_id = response.id;
        let is_open =
            ui.data(|d| d.get_temp::<egui::Id>(egui::Id::new(HEADER_MENU_KEY))) == Some(header_id);
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
                    let frame = egui::Frame::popup(ui.style());
                    frame.show(ui, |ui| {
                        ui.set_min_width(response.rect.width().max(100.0));
                        ui.set_max_width(200.0);
                        let mut close = false;
                        match &mut element.data {
                            Element::Number(n) => close = draw_number_options(ui, &mut n.type_),
                            Element::Percent(p) => close = draw_number_options(ui, &mut p.type_),
                            Element::Component(c) => close = draw_component_options(ui, &mut c.type_),
                            _ => {}
                        }
                        if close {
                            ui.data_mut(|d| d.remove::<egui::Id>(egui::Id::new(HEADER_MENU_KEY)));
                        }
                    });
                })
                .response;
            if ui.input(|i| i.pointer.any_click()) && !area_response.hovered() && !response.hovered()
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
                    }
                    egui::DragAndDrop::clear_payload(ui.ctx());
                }
            }
        }
    }
}

fn number_type_name(t: NumberType) -> &'static str {
    match t {
        NumberType::Health => "Health",
        NumberType::Armor => "Armor",
        NumberType::Frags => "Frags",
        NumberType::Ammo => "Ammo (by Type)",
        NumberType::AmmoSelected => "Selected Ammo",
        NumberType::MaxAmmo => "Max Ammo (by Type)",
        NumberType::AmmoWeapon => "Ammo (by Weapon)",
        NumberType::MaxAmmoWeapon => "Max Ammo (by Weapon)",
    }
}

fn component_type_name(t: ComponentType) -> String {
    format!("{:?}", t)
}

fn custom_menu_item(ui: &mut egui::Ui, text: &str, selected: bool) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width().max(100.0), 24.0), egui::Sense::click());
    if response.hovered() || selected {
        ui.painter().rect_filled(
            rect,
            4.0,
            if selected {
                ui.visuals().selection.bg_fill
            } else {
                egui::Color32::from_gray(60)
            },
        );
    }
    ui.painter().text(
        rect.left_center() + egui::vec2(8.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(14.0),
        if selected {
            ui.visuals().selection.stroke.color
        } else {
            egui::Color32::from_gray(240)
        },
    );
    response.clicked()
}

fn draw_number_options(ui: &mut egui::Ui, type_: &mut NumberType) -> bool {
    let mut changed = false;
    if custom_menu_item(ui, "Health", *type_ == NumberType::Health) {
        *type_ = NumberType::Health;
        changed = true;
    }
    if custom_menu_item(ui, "Armor", *type_ == NumberType::Armor) {
        *type_ = NumberType::Armor;
        changed = true;
    }
    if custom_menu_item(ui, "Frags", *type_ == NumberType::Frags) {
        *type_ = NumberType::Frags;
        changed = true;
    }
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
    if custom_menu_item(ui, "Ammo (by Type)", *type_ == NumberType::Ammo) {
        *type_ = NumberType::Ammo;
        changed = true;
    }
    if custom_menu_item(ui, "Selected Ammo", *type_ == NumberType::AmmoSelected) {
        *type_ = NumberType::AmmoSelected;
        changed = true;
    }
    if custom_menu_item(ui, "Max Ammo (by Type)", *type_ == NumberType::MaxAmmo) {
        *type_ = NumberType::MaxAmmo;
        changed = true;
    }
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
    if custom_menu_item(ui, "Ammo (by Weapon)", *type_ == NumberType::AmmoWeapon) {
        *type_ = NumberType::AmmoWeapon;
        changed = true;
    }
    if custom_menu_item(ui, "Max Ammo (by Weapon)", *type_ == NumberType::MaxAmmoWeapon) {
        *type_ = NumberType::MaxAmmoWeapon;
        changed = true;
    }
    changed
}

fn draw_component_options(ui: &mut egui::Ui, type_: &mut ComponentType) -> bool {
    use crate::model::ComponentType::*;
    let mut changed = false;
    if custom_menu_item(ui, "Time", *type_ == Time) {
        *type_ = Time;
        changed = true;
    }
    if custom_menu_item(ui, "Level Title", *type_ == LevelTitle) {
        *type_ = LevelTitle;
        changed = true;
    }
    if custom_menu_item(ui, "Announce Level Title", *type_ == AnnounceLevelTitle) {
        *type_ = AnnounceLevelTitle;
        changed = true;
    }
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);
    if custom_menu_item(ui, "Message", *type_ == Message) {
        *type_ = Message;
        changed = true;
    }
    if custom_menu_item(ui, "Coordinates", *type_ == Coordinates) {
        *type_ = Coordinates;
        changed = true;
    }
    if custom_menu_item(ui, "FPS Counter", *type_ == FpsCounter) {
        *type_ = FpsCounter;
        changed = true;
    }
    if custom_menu_item(ui, "Stat Totals", *type_ == StatTotals) {
        *type_ = StatTotals;
        changed = true;
    }
    changed
}

pub fn draw_font_selection_row(
    ui: &mut egui::Ui,
    current_val: &mut String,
    target_name: &str,
    stem: Option<&String>,
    assets: &AssetStore,
    is_number_font: bool,
    index: usize,
) {
    let preview_char = if is_number_font {
        std::char::from_digit((index % 10) as u32, 10).unwrap_or('0')
    } else {
        (b'A' + (index % 26) as u8) as char
    };
    let patch_name = stem.map(|s| assets.resolve_patch_name(s, preview_char, is_number_font));
    let texture = patch_name.and_then(|n| assets.textures.get(&n));
    let response = thumbnails::ListRow::new(target_name)
        .subtitle(format!("({})", stem.unwrap_or(&"???".to_string())))
        .texture(texture)
        .fallback("?")
        .selected(*current_val == target_name)
        .show(ui);
    if response.clicked() {
        *current_val = target_name.to_string();
        ui.close();
    }
}