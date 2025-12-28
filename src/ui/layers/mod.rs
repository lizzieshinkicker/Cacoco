use crate::assets::AssetStore;
use crate::document::{self, LayerAction};
use crate::model::{
    AnimationDef, CanvasDef, ComponentDef, ComponentType, Element, ElementWrapper, FaceDef,
    GraphicDef, NumberDef, NumberType, SBarDefFile, TextHelperDef,
};
use crate::state::PreviewState;
use crate::ui::font_wizard::FontWizardState;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

mod browser;
pub(crate) mod colors;
mod layouts;
pub mod thumbnails;
pub(crate) mod tree;

const TAB_STATE_KEY: &str = "cacoco_layers_tab_state";
const THUMB_ZOOM_KEY: &str = "cacoco_layers_thumb_zoom";
const SHOW_FONTS_KEY: &str = "cacoco_show_fonts_state";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BrowserTab {
    Layouts,
    Graphics,
    Fonts,
    IWAD,
    Library,
}

pub fn draw_layers_panel(
    ui: &mut egui::Ui,
    file: &mut Option<SBarDefFile>,
    selection: &mut HashSet<Vec<usize>>,
    selection_pivot: &mut Option<Vec<usize>>,
    assets: &mut AssetStore,
    current_bar_idx: &mut usize,
    state: &mut PreviewState,
    wizard_state: &mut Option<FontWizardState>,
    confirmation_modal: &mut Option<crate::app::ConfirmationRequest>,
) -> (Vec<LayerAction>, bool) {
    let mut changed = false;
    let split_id = ui.make_persistent_id("layers_panel_split");
    let split_fraction = ui
        .ctx()
        .data(|d| d.get_temp::<f32>(split_id).unwrap_or(0.35));

    let mut current_tab: Option<BrowserTab> = ui.data(|d| {
        d.get_temp(egui::Id::new(TAB_STATE_KEY))
            .unwrap_or(Some(BrowserTab::Layouts))
    });

    let mut zoom = ui.data(|d| d.get_temp(egui::Id::new(THUMB_ZOOM_KEY)).unwrap_or(1.0f32));
    let mut show_fonts = ui.data(|d| d.get_temp(egui::Id::new(SHOW_FONTS_KEY)).unwrap_or(true));

    let available_height = ui.available_height();
    let min_h = 100.0;
    let is_collapsed = current_tab.is_none();

    let header_h = 32.0;
    let top_height = if is_collapsed {
        header_h
    } else {
        (available_height * split_fraction).clamp(min_h, available_height - min_h)
    };

    let top_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), top_height),
    );

    ui.allocate_rect(top_rect, egui::Sense::hover());

    let mut top_ui = ui.new_child(egui::UiBuilder::new().max_rect(top_rect));
    top_ui.set_clip_rect(top_rect);

    let mut actions = Vec::new();

    top_ui.add_space(4.0);

    top_ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 1.0;
        let total_w = ui.available_width();
        let btn_w = (total_w - 4.0) / 5.0;
        let btn_h = 24.0;

        let mut draw_tab = |ui: &mut egui::Ui, target: BrowserTab, label: &str| {
            let is_active = current_tab == Some(target);
            let res = ui.add_sized([btn_w, btn_h], |ui: &mut egui::Ui| {
                shared::compact_header_button(ui, label, is_active)
            });

            if res.clicked() {
                current_tab = if is_active { None } else { Some(target) };
            }
        };

        draw_tab(ui, BrowserTab::Layouts, "Layouts");
        draw_tab(ui, BrowserTab::Graphics, "Graphics");
        draw_tab(ui, BrowserTab::Fonts, "Fonts");
        draw_tab(ui, BrowserTab::IWAD, "IWAD");
        draw_tab(ui, BrowserTab::Library, "Library");
    });

    if let Some(active_tab) = current_tab {
        top_ui.add_space(8.0);
        let header_bottom = top_rect.min.y + 36.0;
        let has_footer = !matches!(active_tab, BrowserTab::Fonts | BrowserTab::Layouts);
        let footer_h = if has_footer { 32.0 } else { 0.0 };

        let footer_top = (top_rect.max.y - footer_h).max(header_bottom);

        let scroll_rect = egui::Rect::from_min_max(
            egui::pos2(top_rect.min.x, header_bottom),
            egui::pos2(top_rect.max.x, footer_top),
        );

        top_ui.scope_builder(egui::UiBuilder::new().max_rect(scroll_rect), |ui| {
            egui::ScrollArea::vertical()
                .id_salt("browser_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| match active_tab {
                    BrowserTab::Layouts => {
                        if let Some(f) = file {
                            ui.vertical(|ui| {
                                changed |= layouts::draw_layouts_browser(
                                    ui,
                                    f,
                                    selection,
                                    current_bar_idx,
                                    &mut actions,
                                    confirmation_modal,
                                );
                            });
                        } else {
                            shared::draw_no_file_placeholder(ui);
                        }
                    }
                    BrowserTab::Fonts => {
                        if let Some(f) = file {
                            changed |= browser::draw_fonts_content(ui, f, assets);
                        } else {
                            shared::draw_no_file_placeholder(ui);
                        }
                    }
                    BrowserTab::Graphics => {
                        changed |= browser::draw_filtered_browser(
                            ui,
                            assets,
                            file,
                            zoom,
                            true,
                            wizard_state,
                            confirmation_modal,
                            show_fonts,
                        );
                    }
                    BrowserTab::IWAD => {
                        changed |= browser::draw_filtered_browser(
                            ui,
                            assets,
                            file,
                            zoom,
                            false,
                            wizard_state,
                            confirmation_modal,
                            show_fonts,
                        );
                    }
                    BrowserTab::Library => {
                        changed |= browser::draw_library_browser(ui, assets, file, zoom);
                    }
                });
        });

        if has_footer && footer_top < top_rect.max.y {
            let footer_rect = egui::Rect::from_min_max(
                egui::pos2(top_rect.min.x, footer_top),
                egui::pos2(top_rect.max.x, top_rect.max.y),
            );

            top_ui.scope_builder(egui::UiBuilder::new().max_rect(footer_rect), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.add_space(3.0);
                        ui.label(egui::RichText::new("Zoom").weak().size(12.0));
                    });

                    ui.add_sized(
                        [80.0, 20.0],
                        egui::Slider::new(&mut zoom, 0.5..=3.0).show_value(false),
                    );

                    if active_tab == BrowserTab::Graphics || active_tab == BrowserTab::IWAD {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(4.0);
                            if active_tab == BrowserTab::Graphics {
                                ui.menu_button("Import...", |ui| {
                                    if ui.button("Files").clicked() {
                                        let count =
                                            crate::io::import_images_dialog(ui.ctx(), assets);
                                        if count > 0 {
                                            state.push_message(format!("Imported {count} images."));
                                            changed = true;
                                        }
                                        ui.close();
                                    }
                                    if ui.button("Folder").clicked() {
                                        let count =
                                            crate::io::import_folder_dialog(ui.ctx(), assets);
                                        if count > 0 {
                                            state.push_message(format!(
                                                "Imported {count} images from folder."
                                            ));
                                            changed = true;
                                        }
                                        ui.close();
                                    }
                                });
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);
                            } else if active_tab == BrowserTab::IWAD {
                                ui.add_space(87.0);
                            }
                            ui.checkbox(&mut show_fonts, "Fonts");
                        });
                    }
                });
            });
        }
    }

    ui.data_mut(|d| {
        d.insert_temp(egui::Id::new(TAB_STATE_KEY), current_tab);
        d.insert_temp(egui::Id::new(THUMB_ZOOM_KEY), zoom);
        d.insert_temp(egui::Id::new(SHOW_FONTS_KEY), show_fonts);
    });

    if is_collapsed {
        ui.add_space(2.0);
    } else {
        ui.add_space(4.0);
    }

    let remaining_h = ui.available_height();
    let bottom_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), remaining_h),
    );
    ui.allocate_rect(bottom_rect, egui::Sense::hover());
    let mut bottom_ui = ui.new_child(egui::UiBuilder::new().max_rect(bottom_rect));
    bottom_ui.set_clip_rect(bottom_rect);

    if current_tab.is_none() {
        bottom_ui.add_space(-1.0);
    }

    if let Some(f) = file {
        bottom_ui.horizontal(|ui| {
            ui.heading("Layers");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                ui.menu_button("New Layer...", |ui| {
                    let mut new_element = None;
                    let default_hud_font = f.data.hud_fonts.first().map(|font| font.name.clone());
                    let default_num_font =
                        f.data.number_fonts.first().map(|font| font.name.clone());
                    let has_hud = default_hud_font.is_some();
                    let has_num = default_num_font.is_some();

                    if ui.button("Canvas Group").clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Canvas(CanvasDef::default()),
                            ..Default::default()
                        });
                    }

                    let txt_btn = ui.add_enabled(has_hud, egui::Button::new("Text String"));
                    if !has_hud {
                        txt_btn.on_hover_text("Add a HUD Font in the 'Fonts' tab first!");
                    } else if txt_btn.clicked() {
                        let mut el = ElementWrapper {
                            data: Element::Canvas(CanvasDef::default()),
                            _cacoco_text: Some(TextHelperDef {
                                text: "NEW TEXT".to_string(),
                                font: default_hud_font.clone().unwrap(),
                                spacing: 0,
                            }),
                            ..Default::default()
                        };
                        let fonts = crate::ui::properties::font_cache::FontCache::new(f);
                        crate::ui::properties::text_helper::rebake_text(&mut el, assets, &fonts);
                        new_element = Some(el);
                    }

                    ui.separator();
                    if ui.button("Graphic").clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Graphic(GraphicDef {
                                patch: "HICACOCO".to_string(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                    }
                    if ui.button("Animation").clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Animation(AnimationDef::default()),
                            ..Default::default()
                        });
                    }
                    if ui.button("Doomguy").clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Face(FaceDef::default()),
                            ..Default::default()
                        });
                    }
                    if ui.button("Face Background").clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::FaceBackground(FaceDef::default()),
                            ..Default::default()
                        });
                    }

                    ui.separator();
                    let num_btn = ui.add_enabled(has_num, egui::Button::new("Number"));
                    if !has_num {
                        num_btn.on_hover_text("Add a Number Font in the 'Fonts' tab first!");
                    } else if num_btn.clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Number(NumberDef {
                                font: default_num_font.clone().unwrap(),
                                type_: NumberType::Health,
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                    }

                    let sel_ammo_btn = ui.add_enabled(has_num, egui::Button::new("Selected Ammo"));
                    if sel_ammo_btn.clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Number(NumberDef {
                                font: default_num_font.clone().unwrap(),
                                type_: NumberType::AmmoSelected,
                                common: crate::model::CommonAttrs {
                                    conditions: vec![crate::model::ConditionDef {
                                        condition:
                                            crate::model::ConditionType::SelectedWeaponHasAmmo,
                                        param: 0,
                                        param2: 0,
                                    }],
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                    }

                    let pct_btn = ui.add_enabled(has_num, egui::Button::new("Percent"));
                    if !has_num {
                        pct_btn.on_hover_text("Add a Number Font in the 'Fonts' tab first!");
                    } else if pct_btn.clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Percent(NumberDef {
                                font: default_num_font.clone().unwrap(),
                                type_: NumberType::Health,
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                    }

                    let sel_pct_btn = ui.add_enabled(has_num, egui::Button::new("Selected Ammo %"));
                    if sel_pct_btn.clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Percent(NumberDef {
                                font: default_num_font.unwrap(),
                                type_: NumberType::AmmoSelected,
                                common: crate::model::CommonAttrs {
                                    conditions: vec![crate::model::ConditionDef {
                                        condition:
                                            crate::model::ConditionType::SelectedWeaponHasAmmo,
                                        param: 0,
                                        param2: 0,
                                    }],
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                    }

                    ui.separator();
                    let comp_btn = ui.add_enabled(has_hud, egui::Button::new("Component"));
                    if !has_hud {
                        comp_btn.on_hover_text("Add a HUD Font in the 'Fonts' tab first!");
                    } else if comp_btn.clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Component(ComponentDef {
                                font: default_hud_font.unwrap(),
                                type_: ComponentType::Time,
                                ..Default::default()
                            }),
                            ..Default::default()
                        });
                    }

                    if let Some(element) = new_element {
                        let (parent_path, insert_idx) =
                            document::determine_insertion_point(selection, *current_bar_idx);
                        actions.push(LayerAction::UndoSnapshot);
                        actions.push(LayerAction::Add {
                            parent_path,
                            insert_idx,
                            element,
                        });
                        ui.close();
                    }
                });
            });
        });

        bottom_ui.separator();

        if !f.data.status_bars.is_empty() {
            if *current_bar_idx >= f.data.status_bars.len() {
                *current_bar_idx = 0;
            }
            egui::ScrollArea::vertical()
                .id_salt("layers_scroll")
                .auto_shrink([false, false])
                .show(&mut bottom_ui, |ui| {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(2, 0))
                        .show(ui, |ui| {
                            tree::draw_layer_tree_root(
                                ui,
                                f,
                                *current_bar_idx,
                                selection,
                                selection_pivot,
                                assets,
                                state,
                                &mut actions,
                                confirmation_modal,
                            );
                        });
                    ui.add_space(2.0);
                });
        }
    } else {
        shared::draw_no_file_placeholder(&mut bottom_ui);
    }

    (actions, changed)
}
