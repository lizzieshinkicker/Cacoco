use crate::models::sbardef::{Element, ElementWrapper};
use eframe::egui::Color32;

pub fn get_layer_color(element: &ElementWrapper) -> Option<Color32> {
    if element._cacoco_text.is_some() {
        return Some(Color32::from_rgb(255, 100, 255));
    }

    match &element.data {
        Element::Canvas(_) | Element::Carousel(_) => None,
        Element::List(_) => Some(Color32::from_rgb(255, 165, 0)),
        Element::Native(_) => Some(Color32::from_rgb(84, 255, 159)),
        Element::Graphic(_) => Some(Color32::from_rgb(100, 180, 255)),
        Element::Animation(_) => Some(Color32::from_rgb(180, 100, 255)),
        Element::Face(_) => Some(Color32::from_rgb(50, 205, 50)),
        Element::FaceBackground(_) => Some(Color32::from_rgb(160, 82, 45)),
        Element::Number(_) | Element::Percent(_) => Some(Color32::from_rgb(154, 205, 50)),
        Element::String(_) => Some(Color32::from_rgb(255, 215, 0)),
        Element::Component(_) => Some(Color32::from_rgb(0, 200, 200)),
    }
}
