use crate::assets::AssetStore;
use crate::model::SBarDefFile;
use crate::state::PreviewState;
use crate::ui::layers::colors;
use eframe::egui;
use std::collections::HashSet;

pub mod common;
mod conditions;
mod descriptions;
pub mod editor;
pub(crate) mod font_cache;
mod graphics;
mod lookups;
pub(crate) mod preview;
mod text;
pub mod text_helper;

use crate::ui::shared;
use editor::PropertiesUI;
use font_cache::FontCache;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PropertyTab {
    Properties,
    Conditions,
}

const PROP_TAB_KEY: &str = "cacoco_prop_tab_state";

pub fn draw_properties_panel(
    ui: &mut egui::Ui,
    file: &mut Option<SBarDefFile>,
    selection: &HashSet<Vec<usize>>,
    assets: &AssetStore,
    state: &PreviewState,
) -> bool {
    let mut changed = false;

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
                            crate::model::Element::Number(n) => {
                                text::number_type_name(n.type_).to_string()
                            }
                            crate::model::Element::Percent(p) => {
                                text::number_type_name(p.type_).to_string()
                            }
                            crate::model::Element::Component(c) => format!("{:?}", c.type_),
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
                header_desc = "The root configuration for a HUD layout. Defines height and background behavior.".to_string();
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
        header_desc = "No project file loaded. Open or create a new SBARDEF to begin!".to_string();
    }

    let mut preview_content = None;
    if let Some(f) = file {
        if let Some(path) = selection.iter().next() {
            if path.len() > 1 {
                let font_cache = FontCache::new(f);
                if let Some(el) = f.get_element(path) {
                    preview_content = el.get_preview_content(ui, &font_cache, state);
                }
            }
        }
    }

    if let Some(content) = preview_content {
        preview::draw_preview_panel(ui, assets, content);
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
                current_tab == PropertyTab::Properties,
            )
            .clicked()
            {
                current_tab = PropertyTab::Properties;
            }
            if shared::section_header_button(
                &mut uis[1],
                "Conditions",
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
            ui.add_space(0.0);

            if let Some(f) = file {
                let font_cache = FontCache::new(f);

                if let Some(path) = selection.iter().next() {
                    if path.len() > 1 {
                        if let Some(el) = f.get_element_mut(path) {
                            match current_tab {
                                PropertyTab::Properties => {
                                    if el._cacoco_text.is_none() {
                                        ui.horizontal(|ui| {
                                            ui.label("Name:");
                                            let mut name =
                                                el._cacoco_name.clone().unwrap_or_default();
                                            let edit = egui::TextEdit::singleline(&mut name)
                                                .desired_width(ui.available_width() - 50.0);

                                            if ui.add(edit).changed() {
                                                el._cacoco_name =
                                                    if name.is_empty() { None } else { Some(name) };
                                                changed = true;
                                            }
                                        });
                                        ui.add_space(4.0);
                                    }
                                    changed |= common::draw_transform_editor(ui, el);
                                    ui.add_space(4.0);
                                    if el._cacoco_text.is_some() || el.has_specific_fields() {
                                        ui.group(|ui| {
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
                                        });
                                    }
                                }
                                PropertyTab::Conditions => {
                                    changed |=
                                        conditions::draw_conditions_editor(ui, el, assets, state);
                                }
                            }
                        }
                    } else if is_layout {
                        if let Some(bar) = f.data.status_bars.get_mut(path[0]) {
                            changed |= common::draw_root_statusbar_fields(ui, bar);
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
