use crate::models::sbardef::{Element, ElementWrapper};
use eframe::egui::Color32;

// --- System & UI Colors ---
pub const PANEL_BG: Color32 = Color32::from_rgb(30, 30, 30);
pub const VIEWPORT_BG: Color32 = Color32::from_rgb(15, 15, 15);
pub const DESTRUCTIVE: Color32 = Color32::from_rgb(140, 50, 50);

// --- Lump Type Headers ---
pub const LUMP_SBARDEF: Color32 = Color32::from_rgb(150, 70, 70);
pub const LUMP_SKYDEFS: Color32 = Color32::from_rgb(70, 110, 150);
pub const LUMP_INTERLEVEL: Color32 = Color32::from_rgb(120, 80, 150);
pub const LUMP_FINALE: Color32 = Color32::from_rgb(160, 100, 60);
pub const LUMP_UMAPINFO: Color32 = Color32::from_rgb(80, 140, 100);

// --- Interlevel Hierarchy ---
pub const HEADER_LAYER: Color32 = Color32::from_rgb(90, 110, 130);
pub const HEADER_ANIM: Color32 = Color32::from_rgb(90, 130, 110);
pub const HEADER_FRAME: Color32 = Color32::from_rgb(140, 130, 80);

// --- SBARDEF Tree Semantic Colors (Legacy Migration) ---
pub fn get_layer_color(element: &ElementWrapper) -> Option<Color32> {
    match &element.data {
        _ if element._cacoco_text.is_some() => Some(Color32::from_rgb(200, 100, 200)),
        Element::List(_) => Some(Color32::from_rgb(220, 140, 60)),
        Element::Native(_) => Some(Color32::from_rgb(80, 180, 130)),
        Element::Graphic(_) => Some(Color32::from_rgb(90, 150, 200)),
        Element::Animation(_) => Some(Color32::from_rgb(150, 90, 200)),
        Element::Face(_) => Some(Color32::from_rgb(70, 160, 70)),
        Element::Number(_) | Element::Percent(_) => Some(Color32::from_rgb(140, 180, 60)),
        Element::String(_) => Some(Color32::from_rgb(220, 180, 50)),
        Element::Component(_) => Some(Color32::from_rgb(50, 160, 160)),
        Element::Minimap(_) => Some(Color32::from_rgb(90, 120, 200)),
        _ => None,
    }
}

/// Mutes a solid palette color for use as a subtle header background.
pub fn as_header_bg(color: Color32) -> Color32 {
    color.linear_multiply(0.05)
}
