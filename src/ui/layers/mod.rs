use crate::assets::AssetStore;
use crate::document::actions::DocumentAction;
use crate::models::ProjectData;
use crate::state::PreviewState;
use crate::ui::font_wizard::FontWizardState;
use crate::ui::messages::{self, EditorEvent};
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

mod browser;
pub(crate) mod colors;
mod layouts;
pub(crate) mod sky;
pub mod thumbnails;
pub(crate) mod tree;
pub(crate) mod umapinfo;

const TAB_STATE_KEY: &str = "cacoco_layers_tab_state";
const LAST_TAB_STATE_KEY: &str = "cacoco_layers_last_tab_state";
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

/// Draws the entire right-side panel, including the asset browser and layer hierarchy.
pub fn draw_layers_panel(
    ui: &mut egui::Ui,
    file: &mut Option<ProjectData>,
    selection: &mut HashSet<Vec<usize>>,
    selection_pivot: &mut Option<Vec<usize>>,
    assets: &mut AssetStore,
    current_bar_idx: &mut usize,
    state: &mut PreviewState,
    wizard_state: &mut Option<FontWizardState>,
    confirmation_modal: &mut Option<crate::app::ConfirmationRequest>,
) -> (Vec<DocumentAction>, bool) {
    let mut changed = false;
    let split_id = ui.make_persistent_id("layers_panel_split");

    let split_fraction = ui
        .ctx()
        .data(|d| d.get_temp::<f32>(split_id).unwrap_or(0.35));

    let mut current_tab: Option<BrowserTab> = ui.data(|d| {
        d.get_temp(egui::Id::new(TAB_STATE_KEY))
            .unwrap_or(Some(BrowserTab::Layouts))
    });

    let mut last_tab: BrowserTab = ui.data(|d| {
        d.get_temp(egui::Id::new(LAST_TAB_STATE_KEY))
            .unwrap_or(BrowserTab::Layouts)
    });

    if let Some(active) = current_tab {
        last_tab = active;
    }

    let mut zoom = ui.data(|d| d.get_temp(egui::Id::new(THUMB_ZOOM_KEY)).unwrap_or(1.0f32));
    let mut show_fonts = ui.data(|d| d.get_temp(egui::Id::new(SHOW_FONTS_KEY)).unwrap_or(true));

    let available_height = ui.available_height();
    let min_h = 100.0;
    let header_h = 32.0;
    let is_collapsed = current_tab.is_none();

    let target_h = if is_collapsed {
        header_h
    } else {
        (available_height * split_fraction).clamp(min_h, available_height - min_h)
    };

    let anim_top_h = ui.ctx().animate_value_with_time(
        ui.make_persistent_id("browser_panel_anim"),
        target_h,
        0.1,
    );

    let top_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), anim_top_h),
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

        let mut draw_tab = |ui: &mut egui::Ui, target: BrowserTab, label: &str| {
            let is_active = current_tab == Some(target);
            let res = ui.add_sized([btn_w, 24.0], |ui: &mut egui::Ui| {
                shared::compact_header_button(ui, label, None, is_active)
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

    if anim_top_h > header_h + 1.0 {
        top_ui.add_space(8.0);
        let header_bottom = top_rect.min.y + 36.0;
        let active_tab = current_tab.unwrap_or(last_tab);
        let has_footer = !matches!(active_tab, BrowserTab::Fonts | BrowserTab::Layouts);
        let footer_h = if has_footer { 28.0 } else { 0.0 };
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
                        if let Some(ProjectData::StatusBar(sbar)) = file {
                            changed |= layouts::draw_layouts_browser(
                                ui,
                                sbar,
                                selection,
                                current_bar_idx,
                                &mut actions,
                                confirmation_modal,
                            );
                        } else {
                            shared::draw_no_file_placeholder(ui);
                        }
                    }
                    BrowserTab::Fonts => {
                        if let Some(ProjectData::StatusBar(sbar)) = file {
                            changed |= browser::draw_fonts_content(ui, sbar, assets);
                        } else {
                            shared::draw_no_file_placeholder(ui);
                        }
                    }
                    BrowserTab::Graphics => {
                        let mut sbar_opt = file.as_mut().and_then(|f| f.as_sbar_mut().cloned());
                        changed |= browser::draw_filtered_browser(
                            ui,
                            assets,
                            state,
                            &mut sbar_opt,
                            zoom,
                            true,
                            wizard_state,
                            confirmation_modal,
                            show_fonts,
                        );
                        if let (Some(f), Some(updated)) = (file.as_mut(), sbar_opt) {
                            *f = ProjectData::StatusBar(updated);
                        }
                    }
                    BrowserTab::IWAD => {
                        let mut sbar_opt = file.as_mut().and_then(|f| f.as_sbar_mut().cloned());
                        changed |= browser::draw_filtered_browser(
                            ui,
                            assets,
                            state,
                            &mut sbar_opt,
                            zoom,
                            false,
                            wizard_state,
                            confirmation_modal,
                            show_fonts,
                        );
                        if let (Some(f), Some(updated)) = (file.as_mut(), sbar_opt) {
                            *f = ProjectData::StatusBar(updated);
                        }
                    }
                    BrowserTab::Library => {
                        let mut sbar_opt = file.as_mut().and_then(|f| f.as_sbar_mut().cloned());
                        changed |= browser::draw_library_browser(ui, assets, &mut sbar_opt, zoom);
                        if let (Some(f), Some(updated)) = (file.as_mut(), sbar_opt) {
                            *f = ProjectData::StatusBar(updated);
                        }
                    }
                });
        });

        if has_footer && footer_top < top_rect.max.y {
            let footer_rect = egui::Rect::from_min_max(
                egui::pos2(top_rect.min.x, footer_top),
                egui::pos2(top_rect.max.x, top_rect.max.y),
            );

            top_ui.scope_builder(egui::UiBuilder::new().max_rect(footer_rect), |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Zoom").weak().size(12.0));
                    ui.add_sized(
                        [80.0, 20.0],
                        egui::Slider::new(&mut zoom, 0.5..=3.0).show_value(false),
                    );

                    if active_tab == BrowserTab::Graphics || active_tab == BrowserTab::IWAD {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(6.0);
                            if active_tab == BrowserTab::Graphics {
                                ui.menu_button("Import...", |ui| {
                                    if ui.button("Files").clicked() {
                                        let count =
                                            crate::io::import_images_dialog(ui.ctx(), assets);
                                        if count > 0 {
                                            changed = true;
                                            messages::log_event(
                                                state,
                                                EditorEvent::ImportImages(count),
                                            );
                                        }
                                        ui.close();
                                    }
                                    if ui.button("Folder").clicked() {
                                        let count =
                                            crate::io::import_folder_dialog(ui.ctx(), assets);
                                        if count > 0 {
                                            changed = true;
                                            messages::log_event(
                                                state,
                                                EditorEvent::ImportFolder(count),
                                            );
                                        }
                                        ui.close();
                                    }
                                });
                                ui.separator();
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
        d.insert_temp(egui::Id::new(LAST_TAB_STATE_KEY), last_tab);
        d.insert_temp(egui::Id::new(THUMB_ZOOM_KEY), zoom);
        d.insert_temp(egui::Id::new(SHOW_FONTS_KEY), show_fonts);
    });

    if !is_collapsed {
        ui.add_space(2.0);
    }

    let remaining_h = ui.available_height();
    let bottom_rect = egui::Rect::from_min_size(
        ui.cursor().min,
        egui::vec2(ui.available_width(), remaining_h),
    );

    ui.allocate_rect(bottom_rect, egui::Sense::hover());
    let mut bottom_ui = ui.new_child(egui::UiBuilder::new().max_rect(bottom_rect));
    bottom_ui.set_clip_rect(bottom_rect);

    if is_collapsed && anim_top_h <= header_h + 1.0 {
        bottom_ui.add_space(1.0);
    }

    if let Some(f) = file {
        let mut layer_ctx = crate::ui::properties::editor::LayerContext {
            selection,
            selection_pivot,
            assets,
            state,
            current_item_idx: current_bar_idx,
            wizard_state,
            confirmation_modal,
        };

        let (lump_actions, lump_changed) = f.draw_layer_list(&mut bottom_ui, &mut layer_ctx);
        actions.extend(lump_actions);
        changed |= lump_changed;
    } else {
        shared::draw_no_file_placeholder(&mut bottom_ui);
    }

    (actions, changed)
}
