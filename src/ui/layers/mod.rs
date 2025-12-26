use crate::assets::AssetStore;
use crate::document::{self, LayerAction};
use crate::model::{
    AnimationDef, CanvasDef, ComponentDef, ComponentType, Element, ElementWrapper, FaceDef,
    GraphicDef, NumberDef, NumberType, SBarDefFile, StatusBarLayout, TextHelperDef,
};
use crate::state::PreviewState;
use crate::ui::font_wizard::FontWizardState;
use crate::ui::properties::common;
use eframe::egui;
use std::collections::HashSet;
use crate::ui::shared;

mod browser;
pub(crate) mod colors;
pub mod thumbnails;
mod tree;

const TAB_STATE_KEY: &str = "cacoco_layers_tab_state";
const THUMB_ZOOM_KEY: &str = "cacoco_layers_thumb_zoom";
const SHOW_FONTS_KEY: &str = "cacoco_show_fonts_state";

#[derive(Clone, Copy, PartialEq, Eq)]
enum BrowserTab {
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
    let mut split_fraction = ui
        .ctx()
        .data(|d| d.get_temp::<f32>(split_id).unwrap_or(0.30));

    let available_height = ui.available_height();
    let min_h = 100.0;
    let top_height = (available_height * split_fraction).clamp(min_h, available_height - min_h);

    let top_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), top_height),
    );

    ui.allocate_rect(top_rect, egui::Sense::hover());

    let mut top_ui = ui.new_child(egui::UiBuilder::new().max_rect(top_rect));
    top_ui.set_clip_rect(top_rect);

    let mut current_tab = top_ui.data(|d| {
        d.get_temp(egui::Id::new(TAB_STATE_KEY))
            .unwrap_or(BrowserTab::Graphics)
    });
    let mut zoom = top_ui.data(|d| d.get_temp(egui::Id::new(THUMB_ZOOM_KEY)).unwrap_or(1.0f32));
    let mut show_fonts = top_ui.data(|d| d.get_temp(egui::Id::new(SHOW_FONTS_KEY)).unwrap_or(true));

    top_ui.add_space(3.0);

    top_ui.horizontal(|ui| {
        ui.add_space(4.0);
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
        ui.selectable_value(&mut current_tab, BrowserTab::Graphics, " Graphics ");
        ui.add(egui::Separator::default().vertical().spacing(10.0));
        ui.selectable_value(&mut current_tab, BrowserTab::Fonts, " Fonts ");
        ui.add(egui::Separator::default().vertical().spacing(10.0));
        ui.selectable_value(&mut current_tab, BrowserTab::IWAD, " IWAD ");
        ui.add(egui::Separator::default().vertical().spacing(10.0));
        ui.selectable_value(&mut current_tab, BrowserTab::Library, " Library ");
    });

    top_ui.add_space(1.0);
    top_ui.separator();

    let header_bottom = top_ui.cursor().min.y;
    let footer_h = if current_tab != BrowserTab::Fonts {
        32.0
    } else {
        0.0
    };

    let scroll_rect = egui::Rect::from_min_max(
        egui::pos2(top_rect.min.x, header_bottom),
        egui::pos2(top_rect.max.x, top_rect.max.y - footer_h),
    );

    top_ui.scope_builder(egui::UiBuilder::new().max_rect(scroll_rect), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("browser_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(4.0);
                match current_tab {
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
                }
            });
    });

    if current_tab != BrowserTab::Fonts {
        let footer_rect = egui::Rect::from_min_max(
            egui::pos2(top_rect.min.x, top_rect.max.y - footer_h),
            egui::pos2(top_rect.max.x, top_rect.max.y),
        );

        top_ui.scope_builder(egui::UiBuilder::new().max_rect(footer_rect), |ui| {
            ui.separator();
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

                if current_tab == BrowserTab::Graphics || current_tab == BrowserTab::IWAD {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);

                        if current_tab == BrowserTab::Graphics {
                            ui.menu_button("Import...", |ui| {
                                if ui.button("Files").clicked() {
                                    let count = crate::io::import_images_dialog(ui.ctx(), assets);
                                    if count > 0 {
                                        state.push_message(format!("Imported {} images.", count));
                                        changed = true;
                                    }
                                    ui.close();
                                }
                                if ui.button("Folder").clicked() {
                                    let count = crate::io::import_folder_dialog(ui.ctx(), assets);
                                    if count > 0 {
                                        state.push_message(format!(
                                            "Imported {} images from folder.",
                                            count
                                        ));
                                        changed = true;
                                    }
                                    ui.close();
                                }
                            });
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);
                        } else if current_tab == BrowserTab::IWAD {
                            ui.add_space(87.0);
                        }

                        ui.checkbox(&mut show_fonts, "Fonts");
                    });
                }
            });
        });
    }

    top_ui.data_mut(|d| {
        d.insert_temp(egui::Id::new(TAB_STATE_KEY), current_tab);
        d.insert_temp(egui::Id::new(THUMB_ZOOM_KEY), zoom);
        d.insert_temp(egui::Id::new(SHOW_FONTS_KEY), show_fonts);
    });

    let (response, painter) =
        ui.allocate_painter(egui::vec2(ui.available_width(), 8.0), egui::Sense::drag());

    let color = if response.hovered() || response.dragged() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke.color
    };

    painter.line_segment(
        [
            egui::pos2(response.rect.left(), response.rect.center().y),
            egui::pos2(response.rect.right(), response.rect.center().y),
        ],
        egui::Stroke::new(1.0, color),
    );

    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }
    if response.dragged() {
        split_fraction += response.drag_delta().y / available_height;
        ui.ctx()
            .data_mut(|d| d.insert_temp(split_id, split_fraction));
    }

    let remaining_h = ui.available_height();
    let bottom_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), remaining_h),
    );
    ui.allocate_rect(bottom_rect, egui::Sense::hover());
    let mut bottom_ui = ui.new_child(egui::UiBuilder::new().max_rect(bottom_rect));
    bottom_ui.set_clip_rect(bottom_rect);

    let mut actions = Vec::new();

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

                    let pct_btn = ui.add_enabled(has_num, egui::Button::new("Percent"));
                    if !has_num {
                        pct_btn.on_hover_text("Add a Number Font in the 'Fonts' tab first!");
                    } else if pct_btn.clicked() {
                        new_element = Some(ElementWrapper {
                            data: Element::Percent(NumberDef {
                                font: default_num_font.unwrap(),
                                type_: NumberType::Health,
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

        if f.data.status_bars.is_empty() {
            bottom_ui.label("No status bars defined.");
            if bottom_ui.button("Add Status Bar").clicked() {
                f.data.status_bars.push(StatusBarLayout::default());
                changed = true;
            }
        } else {
            bottom_ui.horizontal(|ui| {
                ui.add_space(1.0);
                let combo_width = ui.available_width() - 64.0;

                egui::ComboBox::from_id_salt("statusbar_selector")
                    .width(combo_width)
                    .selected_text(format!("Status Bar #{}", *current_bar_idx))
                    .show_ui(ui, |ui| {
                        for i in 0..f.data.status_bars.len() {
                            ui.selectable_value(current_bar_idx, i, format!("Status Bar #{}", i));
                        }
                    });

                if ui
                    .button(" + ")
                    .on_hover_text("Add New Status Bar")
                    .clicked()
                {
                    f.data.status_bars.push(StatusBarLayout::default());
                    *current_bar_idx = f.data.status_bars.len() - 1;
                    changed = true;
                }

                let can_delete = f.data.status_bars.len() > 1;
                if ui
                    .add_enabled(can_delete, egui::Button::new(" 🗑 "))
                    .on_hover_text("Delete Current Status Bar")
                    .clicked()
                {
                    let bar = &f.data.status_bars[*current_bar_idx];
                    if bar.children.is_empty() {
                        f.data.status_bars.remove(*current_bar_idx);
                        if *current_bar_idx >= f.data.status_bars.len() {
                            *current_bar_idx = f.data.status_bars.len().saturating_sub(1);
                        }
                        changed = true;
                    } else {
                        *confirmation_modal = Some(
                            crate::app::ConfirmationRequest::DeleteStatusBar(*current_bar_idx),
                        );
                    }
                }
            });

            if *current_bar_idx >= f.data.status_bars.len() {
                *current_bar_idx = 0;
            }

            {
                let bar = &mut f.data.status_bars[*current_bar_idx];
                bottom_ui.add_space(4.0);
                egui::CollapsingHeader::new("Layout Settings")
                    .default_open(false)
                    .show(&mut bottom_ui, |ui| {
                        changed |= common::draw_root_statusbar_fields(ui, bar);
                    });
            }

            bottom_ui.add_space(4.0);
            bottom_ui.separator();

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
