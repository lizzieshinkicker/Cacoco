use crate::assets::AssetStore;
use crate::model::{Element, ElementWrapper, ExportTarget, SBarDefFile};
use crate::state::PreviewState;
use crate::ui::layers::colors;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

mod animation;
pub mod common;
pub mod compatibility;
mod components;
mod conditions;
mod descriptions;
pub mod editor;
mod face;
pub(crate) mod font_cache;
mod graphics;
mod list;
mod lookups;
pub(crate) mod preview;
mod text;
pub mod text_helper;

use editor::PropertiesUI;
use font_cache::FontCache;
use preview::PreviewContent;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PropertyTab {
    Properties,
    Conditions,
}

const PROP_TAB_KEY: &str = "cacoco_prop_tab_state";

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
        }
    }
}

/// Renders the entire properties sidebar panel.
pub fn draw_properties_panel(
    ui: &mut egui::Ui,
    file: &mut Option<SBarDefFile>,
    selection: &HashSet<Vec<usize>>,
    assets: &AssetStore,
    state: &PreviewState,
) -> bool {
    let mut changed = false;
    let target = file.as_ref().map_or(ExportTarget::Basic, |f| f.target);

    ui.data_mut(|d| d.insert_temp(egui::Id::new("cacoco_current_target"), target));

    ui.add_space(5.0);

    let mut current_tab = ui.data(|d| {
        d.get_temp(egui::Id::new(PROP_TAB_KEY))
            .unwrap_or(PropertyTab::Properties)
    });

    let mut header_title = "Properties".to_string();
    let mut header_desc = "Select a layer or layout to view properties.".to_string();
    let mut header_color = ui.visuals().widgets.noninteractive.bg_fill;
    let mut show_tabs = false;
    let mut is_layout = false;

    if let Some(f) = file {
        if selection.len() == 1 {
            let path = selection.iter().next().unwrap();
            if path.len() > 1 {
                show_tabs = true;
                if let Some(el) = f.get_element(path) {
                    header_title = if el._cacoco_text.is_some() {
                        "Text String".to_string()
                    } else {
                        match &el.data {
                            Element::Number(n) => text::number_type_name(n.type_).to_string(),
                            Element::Percent(p) => text::number_type_name(p.type_).to_string(),
                            Element::Component(c) => format!("{:?}", c.type_),
                            Element::List(_) => "List Container".to_string(),
                            Element::String(_) => "Dynamic String".to_string(),
                            _ => el.display_name(),
                        }
                    };
                    header_desc = descriptions::get_helper_text(el).to_string();
                    header_color = colors::get_layer_color(el)
                        .map(|c| c.linear_multiply(0.05))
                        .unwrap_or(header_color);
                }
            } else {
                header_title = format!("Layout #{}", path[0]);
                header_desc = "Root configuration for a HUD layout.".to_string();
                is_layout = true;
            }
        } else if selection.len() > 1 {
            header_title = "Bulk Selection".to_string();
            header_desc = format!(
                "{} objects selected. Multi-editing is coming soon!",
                selection.len()
            );
        }
    } else {
        header_title = "Cacoco Editor".to_string();
        header_desc = "No project file loaded. Open or create a new SBARDEF.".to_string();
    }

    let mut preview_content = None;
    let mut is_incompatible = false;
    if let Some(f) = file {
        if let Some(path) = selection.iter().next() {
            if path.len() > 1 {
                let font_cache = FontCache::new(f);
                if let Some(el) = f.get_element(path) {
                    preview_content = el.get_preview_content(ui, &font_cache, state);
                    is_incompatible = !compatibility::is_compatible(el, f.target);
                }
            }
        }
    }

    if let Some(content) = preview_content {
        preview::draw_preview_panel(ui, assets, content);
        ui.add_space(4.0);
    }

    if is_incompatible {
        let warn_frame = egui::Frame::default()
            .inner_margin(6.0)
            .fill(egui::Color32::from_rgb(80, 30, 30))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(200, 100, 100),
            ))
            .corner_radius(4.0);

        warn_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("⚠");
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Target Incompatibility").strong());
                    ui.label(egui::RichText::new("This element uses Extended features not supported by the Basic target.").size(10.0));
                });
            });
        });
        ui.add_space(4.0);
    }

    if let Some(f) = file {
        if let Some(path) = selection.iter().next() {
            if path.len() > 1 {
                if let Some(el) = f.get_element_mut(path) {
                    changed |= text::draw_interactive_header(ui, el, &header_desc, header_color);
                }
            } else {
                draw_static_header(ui, &header_title, &header_desc, header_color);
            }
        } else {
            draw_static_header(ui, &header_title, &header_desc, header_color);
        }
    } else {
        draw_static_header(ui, &header_title, &header_desc, header_color);
    }
    ui.add_space(4.0);

    if show_tabs {
        ui.add_space(2.0);
        ui.columns(2, |uis| {
            if shared::section_header_button(
                &mut uis[0],
                "Properties",
                None,
                current_tab == PropertyTab::Properties,
            )
            .clicked()
            {
                current_tab = PropertyTab::Properties;
            }
            if shared::section_header_button(
                &mut uis[1],
                "Conditions",
                None,
                current_tab == PropertyTab::Conditions,
            )
            .clicked()
            {
                current_tab = PropertyTab::Conditions;
            }
        });
        ui.add_space(3.0);
        ui.separator();
    }

    egui::ScrollArea::vertical()
        .id_salt("prop_content_scroll")
        .show(ui, |ui| {
            ui.add_space(4.0);

            if let Some(f) = file {
                let font_cache = FontCache::new(f);

                if let Some(path) = selection.iter().next() {
                    if path.len() > 1 {
                        if let Some(el) = f.get_element_mut(path) {
                            match current_tab {
                                PropertyTab::Properties => {
                                    ui.vertical_centered(|ui| {
                                        if el._cacoco_text.is_none() {
                                            ui.horizontal(|ui| {
                                                ui.add_space(
                                                    (ui.available_width() - 210.0).max(0.0) / 2.0,
                                                );
                                                ui.label("Name:");
                                                let mut name =
                                                    el._cacoco_name.clone().unwrap_or_default();
                                                let edit = egui::TextEdit::singleline(&mut name)
                                                    .desired_width(150.0);

                                                if ui.add(edit).changed() {
                                                    el._cacoco_name = if name.is_empty() {
                                                        None
                                                    } else {
                                                        Some(name)
                                                    };
                                                    changed = true;
                                                }
                                            });
                                            ui.add_space(4.0);
                                        }
                                        changed |= common::draw_transform_editor(ui, el, target);
                                        ui.add_space(4.0);
                                        if el._cacoco_text.is_some() || el.has_specific_fields() {
                                            if el._cacoco_text.is_some() {
                                                changed |= text_helper::draw_text_helper_editor(
                                                    ui,
                                                    el,
                                                    &font_cache,
                                                    assets,
                                                );
                                            } else {
                                                changed |= el.draw_specific_fields(
                                                    ui,
                                                    &font_cache,
                                                    assets,
                                                    state,
                                                );
                                            }
                                        }
                                    });
                                }
                                PropertyTab::Conditions => {
                                    changed |=
                                        conditions::draw_conditions_editor(ui, el, assets, state);
                                }
                            }
                        }
                    } else if is_layout {
                        let bar_idx = path[0];

                        if let Some(bar) = f.data.status_bars.get_mut(bar_idx) {
                            if let Some(reason) = &bar._cacoco_system_locked {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(20.0);
                                    ui.label(egui::RichText::new("Managed Slot.").color(egui::Color32::from_rgb(200, 100, 100)).strong());
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new(reason).weak());
                                    ui.add_space(8.0);

                                    if bar_idx == 0 {
                                        ui.horizontal(|ui| {
                                            ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
                                            ui.label("Bar Height:");
                                            changed |= ui.add(egui::DragValue::new(&mut bar.height).range(0..=200)).changed();
                                        });
                                        ui.add_space(8.0);
                                        ui.label(egui::RichText::new("You can adjust the height, but this slot must remain Non-Fullscreen for KEX compatibility.").italics().weak());
                                    } else {
                                        ui.label(egui::RichText::new("This specific slot must remain blank and fullscreen for KEX demo compatibility.").italics().weak());
                                    }
                                });
                            } else {
                                changed |= common::draw_root_statusbar_fields(ui, bar);
                            }
                        }
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("No Selection").weak());
                    });
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new("Create a new project to edit properties.").weak(),
                    );
                });
            }
        });

    ui.data_mut(|d| d.insert_temp(egui::Id::new(PROP_TAB_KEY), current_tab));
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
