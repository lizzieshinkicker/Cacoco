use crate::assets::AssetStore;
use crate::model::{ComponentType, Element, ElementWrapper, NumberType, SBarDefFile};
use crate::state::PreviewState;
use crate::ui::shared;
use eframe::egui;

pub const THUMB_SIZE: f32 = 36.0;
const ROUNDING: f32 = 4.0;
const BG_COLOR: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);
const INNER_MARGIN: f32 = 2.0;

pub fn draw_thumb_bg(ui: &mut egui::Ui, rect: egui::Rect) {
    ui.painter().rect_filled(rect, ROUNDING, BG_COLOR);
}

pub fn draw_thumbnail(
    ui: &mut egui::Ui,
    element: &ElementWrapper,
    assets: &AssetStore,
    file: &SBarDefFile,
    state: &PreviewState,
    is_visible: bool,
    _is_selected: bool,
) -> egui::Response {
    let time = ui.input(|i| i.time);

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(THUMB_SIZE, THUMB_SIZE), egui::Sense::click());
    draw_thumb_bg(ui, rect);

    match &element.data {
        Element::Component(c) => {
            if c.type_ == ComponentType::Time {
                ui.ctx().request_repaint();
                draw_live_time_thumbnail(ui, rect, time, c, file, assets, !is_visible);
            } else {
                draw_live_component_thumbnail(ui, rect, c, file, assets, !is_visible, state);
            }
        }
        Element::Number(n) => {
            ui.ctx().request_repaint();
            draw_live_number_thumbnail(ui, rect, n, false, state, file, assets, !is_visible);
        }
        Element::Percent(p) => {
            ui.ctx().request_repaint();
            draw_live_number_thumbnail(ui, rect, p, true, state, file, assets, !is_visible);
        }
        _ => {
            let texture = get_preview_texture(element, assets, file, state, false);
            draw_static_texture_content(ui, rect, texture, None, !is_visible);
        }
    }

    response
}

pub fn draw_thumbnail_widget(
    ui: &mut egui::Ui,
    texture: Option<&egui::TextureHandle>,
    fallback_icon: Option<&str>,
    is_dimmed: bool,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(THUMB_SIZE, THUMB_SIZE), egui::Sense::click());
    draw_thumb_bg(ui, rect);
    draw_static_texture_content(ui, rect, texture, fallback_icon, is_dimmed);
    response
}

fn draw_static_texture_content(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    texture: Option<&egui::TextureHandle>,
    fallback_icon: Option<&str>,
    is_dimmed: bool,
) {
    let tint = if is_dimmed {
        egui::Color32::from_white_alpha(64)
    } else {
        egui::Color32::WHITE
    };

    if let Some(tex) = texture {
        shared::draw_scaled_image(ui, rect.shrink(INNER_MARGIN), tex, tint, 4.0);
    } else if let Some(icon) = fallback_icon {
        let color = if is_dimmed {
            egui::Color32::from_gray(100)
        } else {
            egui::Color32::from_gray(160)
        };
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(18.0),
            color,
        );
    }
}

fn draw_live_number_thumbnail(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    number_def: &crate::model::NumberDef,
    is_percent: bool,
    state: &PreviewState,
    file: &SBarDefFile,
    assets: &AssetStore,
    is_dimmed: bool,
) {
    let tint = if is_dimmed {
        egui::Color32::from_white_alpha(64)
    } else {
        egui::Color32::WHITE
    };

    let Some(stem) = find_number_stem(file, &number_def.font) else {
        draw_font_error(ui, rect);
        return;
    };

    let val = match number_def.type_ {
        NumberType::Health => state.player.health,
        NumberType::Armor => state.player.armor,
        NumberType::AmmoSelected => state.get_ammo(state.get_selected_ammo_type()),
        NumberType::Ammo => state.get_ammo(number_def.param),
        NumberType::MaxAmmo => state.get_max_ammo(number_def.param),
        _ => 0,
    };
    let text = if is_percent {
        format!("{}%", val)
    } else {
        format!("{}", val)
    };

    draw_live_patches(ui, rect, &text, stem, assets, tint, true);
}

fn draw_live_time_thumbnail(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    time: f64,
    component: &crate::model::ComponentDef,
    file: &SBarDefFile,
    assets: &AssetStore,
    is_dimmed: bool,
) {
    let tint = if is_dimmed {
        egui::Color32::from_white_alpha(64)
    } else {
        egui::Color32::WHITE
    };

    let Some(stem) = find_hud_stem(file, &component.font) else {
        draw_font_error(ui, rect);
        return;
    };

    let seconds = (time as u64) % 60;
    let text = format!(":{:02}", seconds);
    draw_live_patches(ui, rect, &text, stem, assets, tint, false);
}

fn draw_live_component_thumbnail(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    component: &crate::model::ComponentDef,
    file: &SBarDefFile,
    assets: &AssetStore,
    is_dimmed: bool,
    state: &PreviewState,
) {
    let tint = if is_dimmed {
        egui::Color32::from_white_alpha(64)
    } else {
        egui::Color32::WHITE
    };

    let Some(stem) = find_hud_stem(file, &component.font) else {
        draw_font_error(ui, rect);
        return;
    };

    let text_buf;
    let text = match component.type_ {
        ComponentType::StatTotals => "K:0",
        ComponentType::LevelTitle => "ENT",
        ComponentType::Coordinates => "XYZ",
        ComponentType::FpsCounter => {
            ui.ctx().request_repaint();
            text_buf = format!("{:.0}", state.display_fps);
            &text_buf
        }
        _ => "TXT",
    };

    draw_live_patches(ui, rect, text, stem, assets, tint, false);
}

pub fn draw_live_patches(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    text: &str,
    stem: &str,
    assets: &AssetStore,
    tint: egui::Color32,
    is_number_font: bool,
) {
    let mut textures = Vec::new();
    for char in text.chars() {
        let patch_name = assets.resolve_patch_name(stem, char, is_number_font);
        if let Some(tex) = assets.textures.get(&patch_name) {
            textures.push(tex);
        }
    }

    if textures.is_empty() {
        return;
    }

    let total_width: f32 = textures.iter().map(|t| t.size_vec2().x).sum();
    let max_height: f32 = textures
        .iter()
        .map(|t| t.size_vec2().y)
        .reduce(f32::max)
        .unwrap_or(8.0);
    if total_width == 0.0 || max_height == 0.0 {
        return;
    }

    let content_size = THUMB_SIZE - (INNER_MARGIN * 2.0);
    let scale = (content_size / total_width)
        .min(content_size / max_height)
        .min(4.0);

    let mut current_x = rect.center().x - (total_width * scale / 2.0);
    let center_y = rect.center().y;

    for tex in textures {
        let scaled_size = tex.size_vec2() * scale;
        let y_pos = center_y - (scaled_size.y / 2.0);
        let draw_rect = egui::Rect::from_min_size(egui::pos2(current_x, y_pos), scaled_size);

        ui.painter().image(
            tex.id(),
            draw_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
        current_x += scaled_size.x;
    }
}

pub fn get_preview_texture<'a>(
    element: &'a ElementWrapper,
    assets: &'a AssetStore,
    file: &'a SBarDefFile,
    state: &PreviewState,
    ouch: bool,
) -> Option<&'a egui::TextureHandle> {
    let patch_name = match &element.data {
        Element::Graphic(g) => Some(g.patch.clone()),
        Element::Animation(a) => a.frames.first().map(|f| f.lump.clone()),
        Element::Face(_) => Some(state.get_face_sprite(ouch, 1)),
        Element::FaceBackground(_) => Some("STFB0".to_string()),
        Element::Number(n) => file
            .data
            .number_fonts
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(&n.font))
            .map(|f| format!("{}0", f.stem.to_uppercase())),
        _ => None,
    };

    if let Some(name) = patch_name {
        let key = name.to_uppercase();
        if let Some(tex) = assets.textures.get(&key) {
            return Some(tex);
        }
    }
    None
}

pub struct ListRow<'a> {
    pub title: String,
    pub subtitle: Option<String>,
    pub texture: Option<&'a egui::TextureHandle>,
    pub fallback_icon: Option<&'a str>,
    pub selected: bool,
    pub active: bool,
    pub dimmed: bool,
}

impl<'a> ListRow<'a> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            texture: None,
            fallback_icon: None,
            selected: false,
            active: false,
            dimmed: false,
        }
    }

    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }

    pub fn texture(mut self, t: Option<&'a egui::TextureHandle>) -> Self {
        self.texture = t;
        self
    }

    pub fn fallback(mut self, s: &'a str) -> Self {
        self.fallback_icon = Some(s);
        self
    }

    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }

    pub fn active(mut self, a: bool) -> Self {
        self.active = a;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let height = 42.0;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height),
            egui::Sense::click_and_drag(),
        );

        let mut bg = if self.active {
            egui::Color32::from_rgba_unmultiplied(0, 255, 255, 10)
        } else {
            egui::Color32::TRANSPARENT
        };

        if response.hovered() {
            bg = ui.visuals().widgets.hovered.bg_fill;
        }

        let stroke = if self.selected {
            ui.visuals().selection.stroke
        } else {
            egui::Stroke::NONE
        };
        ui.painter()
            .rect(rect, 4.0, bg, stroke, egui::StrokeKind::Outside);

        let center_y = rect.center().y;
        let thumb_rect = egui::Rect::from_center_size(
            egui::pos2(rect.min.x + 22.0, center_y),
            egui::vec2(THUMB_SIZE, THUMB_SIZE),
        );

        let mut thumb_ui = ui.new_child(egui::UiBuilder::new().max_rect(thumb_rect));
        draw_thumbnail_widget(&mut thumb_ui, self.texture, self.fallback_icon, self.dimmed);

        let title_pos_x = rect.min.x + 44.0;
        let text_color = if self.selected {
            ui.visuals().selection.stroke.color
        } else {
            ui.visuals().text_color()
        };

        if let Some(sub) = self.subtitle {
            ui.painter().text(
                egui::pos2(title_pos_x, center_y - 7.0),
                egui::Align2::LEFT_CENTER,
                self.title,
                egui::FontId::proportional(14.0),
                text_color,
            );
            ui.painter().text(
                egui::pos2(title_pos_x, center_y + 8.0),
                egui::Align2::LEFT_CENTER,
                sub,
                egui::FontId::proportional(11.0),
                ui.visuals().weak_text_color(),
            );
        } else {
            ui.painter().text(
                egui::pos2(title_pos_x, center_y),
                egui::Align2::LEFT_CENTER,
                self.title,
                egui::FontId::proportional(14.0),
                text_color,
            );
        }

        response
    }
}

fn find_hud_stem<'a>(file: &'a SBarDefFile, name: &str) -> Option<&'a str> {
    file.data
        .hud_fonts
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case(name))
        .map(|f| f.stem.as_str())
}

fn find_number_stem<'a>(file: &'a SBarDefFile, name: &str) -> Option<&'a str> {
    file.data
        .number_fonts
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case(name))
        .map(|f| f.stem.as_str())
}

fn draw_font_error(ui: &egui::Ui, rect: egui::Rect) {
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "?",
        egui::FontId::proportional(16.0),
        egui::Color32::RED,
    );
}
