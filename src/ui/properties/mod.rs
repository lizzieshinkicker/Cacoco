use crate::assets::AssetStore;
use crate::models::ProjectData;
use crate::models::sbardef::{Element, ElementWrapper, ExportTarget};
use crate::state::PreviewState;
use crate::ui::layers::colors;
use eframe::egui;
use std::collections::HashSet;

mod animation;
pub mod common;
pub mod compatibility;
mod components;
pub mod conditions;
mod descriptions;
pub mod editor;
mod face;
mod finale;
pub(crate) mod font_cache;
mod graphics;
mod interlevel;
mod list;
mod lookups;
pub mod palette_picker;
pub(crate) mod preview;
mod sbardef;
mod skydefs;
mod text;
pub mod text_helper;
mod umapinfo;
mod minimap;

use editor::PropertiesUI;
use font_cache::FontCache;
use preview::PreviewContent;

impl PropertiesUI for ElementWrapper {
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        fonts: &FontCache,
        assets: &AssetStore,
        state: &PreviewState,
    ) -> bool {
        match &mut self.data {
            Element::Canvas(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::List(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Graphic(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Animation(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Face(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::FaceBackground(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Number(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Percent(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::String(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Component(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Carousel(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Native(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Minimap(e) => e.draw_specific_fields(ui, fonts, assets, state),
        }
    }

    fn get_preview_content(
        &self,
        ui: &egui::Ui,
        fonts: &FontCache,
        state: &PreviewState,
    ) -> Option<PreviewContent> {
        match &self.data {
            Element::Canvas(e) => e.get_preview_content(ui, fonts, state),
            Element::List(e) => e.get_preview_content(ui, fonts, state),
            Element::Graphic(e) => e.get_preview_content(ui, fonts, state),
            Element::Animation(e) => e.get_preview_content(ui, fonts, state),
            Element::Face(e) => e.get_preview_content(ui, fonts, state),
            Element::FaceBackground(_) => Some(PreviewContent::Image("STFB0".to_string())),
            Element::Number(e) => e.get_preview_content(ui, fonts, state),
            Element::Percent(e) => {
                let mut content = e.get_preview_content(ui, fonts, state)?;
                if let PreviewContent::Text { text, .. } = &mut content {
                    *text = format!("{}%", text);
                }
                Some(content)
            }
            Element::String(e) => e.get_preview_content(ui, fonts, state),
            Element::Component(e) => e.get_preview_content(ui, fonts, state),
            Element::Carousel(e) => e.get_preview_content(ui, fonts, state),
            Element::Native(e) => e.get_preview_content(ui, fonts, state),
            Element::Minimap(e) => None,
        }
    }

    fn has_specific_fields(&self) -> bool {
        match &self.data {
            Element::Canvas(e) => e.has_specific_fields(),
            Element::List(e) => e.has_specific_fields(),
            Element::Graphic(e) => e.has_specific_fields() || e.crop.is_some(),
            Element::Animation(e) => e.has_specific_fields(),
            Element::Face(e) => e.has_specific_fields() || e.crop.is_some(),
            Element::FaceBackground(e) => e.has_specific_fields() || e.crop.is_some(),
            Element::Number(e) => e.has_specific_fields(),
            Element::Percent(e) => e.has_specific_fields(),
            Element::String(e) => e.has_specific_fields(),
            Element::Component(e) => e.has_specific_fields(),
            Element::Carousel(e) => e.has_specific_fields(),
            Element::Native(e) => e.has_specific_fields(),
            Element::Minimap(_) => true,
        }
    }
}

/// Renders the entire properties sidebar panel.
pub fn draw_properties_panel(
    ui: &mut egui::Ui,
    file: &mut Option<ProjectData>,
    selection: &HashSet<Vec<usize>>,
    assets: &AssetStore,
    state: &PreviewState,
) -> bool {
    let mut changed = false;
    let target = file.as_ref().map_or(ExportTarget::Basic, |f| f.target());
    ui.data_mut(|d| d.insert_temp(egui::Id::new("cacoco_current_target"), target));

    let prop_ctx = editor::PropertyContext {
        selection,
        assets,
        state,
        target,
    };
    ui.add_space(5.0);

    if let Some(f) = file {
        let (title, desc, color) = f.get_header(selection);
        if let Some(content) = f.get_preview(ui, &prop_ctx) {
            preview::draw_preview_panel(ui, assets, content);
            ui.add_space(4.0);
        }

        match f {
            ProjectData::StatusBar(sbar) => {
                if let Some(path) = selection.iter().next() {
                    if path.len() > 1 {
                        if let Some(el) = sbar.get_element_mut(path) {
                            changed |= text::draw_interactive_header(ui, el, &desc, color);
                        }
                    } else {
                        draw_static_header(ui, &title, &desc, color);
                    }
                } else {
                    draw_static_header(ui, &title, &desc, color);
                }
            }
            _ => {
                draw_static_header(ui, &title, &desc, color);
            }
        }

        ui.add_space(4.0);
        egui::ScrollArea::vertical()
            .id_salt("prop_scroll")
            .show(ui, |ui| {
                changed |= f.draw_properties(ui, &prop_ctx);
            });
    } else {
        draw_static_header(
            ui,
            "Cacoco",
            "No project loaded.",
            egui::Color32::TRANSPARENT,
        );
    }

    changed
}

fn draw_static_header(ui: &mut egui::Ui, title: &str, desc: &str, color: egui::Color32) {
    let frame = egui::Frame::NONE
        .inner_margin(8.0)
        .corner_radius(4.0)
        .fill(color)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)));

    frame.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.add_sized(
                [ui.available_width(), 0.0],
                egui::Label::new(egui::RichText::new(title).size(16.0).strong()),
            );
            ui.add(egui::Separator::default().spacing(8.0));
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(desc).weak().italics());
            });
        });
    });
}
