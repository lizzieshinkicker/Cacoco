use crate::assets::AssetStore;
use crate::conditions;
use crate::document::LayerAction;
use crate::model::{Element, ElementWrapper, GraphicDef, SBarDefFile};
use crate::state::PreviewState;
use crate::ui::context_menu::ContextMenu;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

use super::colors;
use super::thumbnails;

const ROW_HEIGHT: f32 = 42.0;

enum DropTarget {
    Sibling(usize, f32),
    Child,
}

pub fn draw_layer_tree_root(
    ui: &mut egui::Ui,
    file: &SBarDefFile,
    bar_idx: usize,
    selection: &mut HashSet<Vec<usize>>,
    selection_pivot: &mut Option<Vec<usize>>,
    assets: &AssetStore,
    state: &PreviewState,
    actions: &mut Vec<LayerAction>,
    confirmation_modal: &mut Option<crate::app::ConfirmationRequest>,
) {
    ui.style_mut().spacing.item_spacing.y = 1.0;

    handle_auto_scroll(ui);

    if let Some(bar) = file.data.status_bars.get(bar_idx) {
        draw_layer_tree_recursive(
            ui,
            &bar.children,
            vec![bar_idx],
            selection,
            selection_pivot,
            assets,
            file,
            state,
            actions,
            confirmation_modal,
        );
    }
}

pub fn draw_layer_tree_recursive(
    ui: &mut egui::Ui,
    elements: &[ElementWrapper],
    current_path: Vec<usize>,
    selection: &mut HashSet<Vec<usize>>,
    selection_pivot: &mut Option<Vec<usize>>,
    assets: &AssetStore,
    file: &SBarDefFile,
    state: &PreviewState,
    actions: &mut Vec<LayerAction>,
    confirmation_modal: &mut Option<crate::app::ConfirmationRequest>,
) {
    for (idx, element) in elements.iter().enumerate() {
        let mut my_path = current_path.clone();
        my_path.push(idx);

        let base_id = ui.make_persistent_id(element.uid);
        let mut folder_state = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            base_id.with("collapse"),
            true,
        );

        let response = draw_layer_row(
            ui,
            element,
            &my_path,
            &current_path,
            idx,
            selection,
            selection_pivot,
            state,
            &mut folder_state,
            assets,
            file,
            actions,
            idx == 0,
            idx == elements.len() - 1,
            confirmation_modal,
        );

        let is_text_helper = element._cacoco_text.is_some();
        let has_real_children = !element.children().is_empty();

        if response.double_clicked() && has_real_children && !is_text_helper {
            folder_state.toggle(ui);
        }

        if has_real_children && !is_text_helper {
            folder_state.show_body_indented(&response, ui, |ui| {
                draw_layer_tree_recursive(
                    ui,
                    element.children(),
                    my_path,
                    selection,
                    selection_pivot,
                    assets,
                    file,
                    state,
                    actions,
                    confirmation_modal,
                );
            });
        }
    }
}

fn draw_layer_row(
    ui: &mut egui::Ui,
    element: &ElementWrapper,
    my_path: &[usize],
    parent_path: &[usize],
    my_idx: usize,
    selection: &mut HashSet<Vec<usize>>,
    selection_pivot: &mut Option<Vec<usize>>,
    state: &PreviewState,
    folder_state: &mut egui::collapsing_header::CollapsingState,
    assets: &AssetStore,
    file: &SBarDefFile,
    actions: &mut Vec<LayerAction>,
    is_first: bool,
    is_last: bool,
    confirmation_modal: &mut Option<crate::app::ConfirmationRequest>,
) -> egui::Response {
    let is_selected = selection.contains(my_path);
    let common = element.get_common();
    let is_visible = conditions::resolve(&common.conditions, state);
    let is_container = matches!(element.data, Element::Canvas(_) | Element::Carousel(_));

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width() - 8.0, ROW_HEIGHT),
        egui::Sense::click_and_drag(),
    );

    if response.clicked() {
        handle_selection(ui, selection, selection_pivot, my_path, file);
    }

    if response.drag_started() {
        if !is_selected {
            selection.clear();
            selection.insert(my_path.to_vec());
        }
        egui::DragAndDrop::set_payload(ui.ctx(), my_path.to_vec());
    }

    handle_drop_logic(
        ui,
        rect,
        my_path,
        parent_path,
        my_idx,
        is_container,
        selection,
        element,
        actions,
    );

    render_row_visuals(ui, rect, &response, element, is_selected);

    let thumb_response = render_row_contents(
        ui,
        rect,
        element,
        folder_state,
        assets,
        file,
        state,
        is_visible,
        selection,
        my_path,
    );

    let combined_res = response.clone().union(thumb_response);
    handle_context_menu(
        ui,
        &combined_res,
        my_path,
        selection,
        actions,
        is_first,
        is_last,
        file,
        confirmation_modal,
    );

    if response.dragged() {
        render_drag_ghost(
            ui,
            element,
            assets,
            file,
            state,
            is_visible,
            selection.len(),
        );
    }

    response
}

fn handle_selection(
    ui: &egui::Ui,
    selection: &mut HashSet<Vec<usize>>,
    pivot: &mut Option<Vec<usize>>,
    my_path: &[usize],
    file: &SBarDefFile,
) {
    let modifiers = ui.input(|i| i.modifiers);
    if modifiers.ctrl || modifiers.command {
        if selection.contains(my_path) {
            selection.remove(my_path);
        } else {
            selection.insert(my_path.to_vec());
            *pivot = Some(my_path.to_vec());
        }
    } else if modifiers.shift {
        if let Some(p) = pivot {
            let bar_idx = my_path[0];
            if let Some(bar) = file.data.status_bars.get(bar_idx) {
                let visible = collect_visible_paths(ui, &bar.children, vec![bar_idx]);
                let s = visible.iter().position(|x| x == p);
                let e = visible.iter().position(|x| x == my_path);
                if let (Some(start), Some(end)) = (s, e) {
                    selection.clear();
                    for i in start.min(end)..=start.max(end) {
                        selection.insert(visible[i].clone());
                    }
                    return;
                }
            }
        }
        selection.insert(my_path.to_vec());
    } else {
        selection.clear();
        selection.insert(my_path.to_vec());
        *pivot = Some(my_path.to_vec());
    }
}

fn handle_drop_logic(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    my_path: &[usize],
    parent_path: &[usize],
    my_idx: usize,
    is_container: bool,
    selection: &HashSet<Vec<usize>>,
    element: &ElementWrapper,
    actions: &mut Vec<LayerAction>,
) {
    if let Some(dragged) = egui::DragAndDrop::payload::<Vec<usize>>(ui.ctx()) {
        let is_self = &*dragged == my_path;
        let is_parent = my_path.starts_with(&*dragged) && my_path.len() > dragged.len();

        if !is_self && !is_parent && ui.rect_contains_pointer(rect) {
            if let Some(target) = calculate_drop_target(ui, rect, my_idx, is_container) {
                match target {
                    DropTarget::Sibling(insert_idx, line_y) => {
                        shared::draw_yellow_line(ui, rect, line_y);
                        if ui.input(|i| i.pointer.any_released()) {
                            actions.push(LayerAction::UndoSnapshot);
                            actions.push(LayerAction::MoveSelection {
                                sources: selection.iter().cloned().collect(),
                                target_parent: parent_path.to_vec(),
                                insert_idx,
                            });
                            egui::DragAndDrop::clear_payload(ui.ctx());
                        }
                    }
                    DropTarget::Child => {
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            egui::Stroke::new(2.0, egui::Color32::GREEN),
                            egui::StrokeKind::Inside,
                        );
                        if ui.input(|i| i.pointer.any_released()) {
                            actions.push(LayerAction::UndoSnapshot);
                            actions.push(LayerAction::MoveSelection {
                                sources: selection.iter().cloned().collect(),
                                target_parent: my_path.to_vec(),
                                insert_idx: element.children().len(),
                            });
                            egui::DragAndDrop::clear_payload(ui.ctx());
                        }
                    }
                }
            }
        }
    }

    if let Some(asset_keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
        if ui.rect_contains_pointer(rect) {
            if let Some(target) = calculate_drop_target(ui, rect, my_idx, is_container) {
                match target {
                    DropTarget::Sibling(mut insert_idx, line_y) => {
                        shared::draw_yellow_line(ui, rect, line_y);
                        if ui.input(|i| i.pointer.any_released()) {
                            actions.push(LayerAction::UndoSnapshot);
                            for key in asset_keys.iter() {
                                actions.push(LayerAction::Add {
                                    parent_path: parent_path.to_vec(),
                                    insert_idx,
                                    element: wrap_graphic(key),
                                });
                                insert_idx += 1;
                            }
                            egui::DragAndDrop::clear_payload(ui.ctx());
                        }
                    }
                    DropTarget::Child => {
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            egui::Stroke::new(2.0, egui::Color32::GREEN),
                            egui::StrokeKind::Inside,
                        );
                        if ui.input(|i| i.pointer.any_released()) {
                            actions.push(LayerAction::UndoSnapshot);
                            let mut append_idx = element.children().len();
                            for key in asset_keys.iter() {
                                actions.push(LayerAction::Add {
                                    parent_path: my_path.to_vec(),
                                    insert_idx: append_idx,
                                    element: wrap_graphic(key),
                                });
                                append_idx += 1;
                            }
                            egui::DragAndDrop::clear_payload(ui.ctx());
                        }
                    }
                }
            }
        }
    }
}

fn wrap_graphic(patch: &str) -> ElementWrapper {
    ElementWrapper {
        data: Element::Graphic(GraphicDef {
            patch: patch.to_string(),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn render_row_visuals(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    response: &egui::Response,
    element: &ElementWrapper,
    is_selected: bool,
) {
    let base_bg = colors::get_layer_color(element)
        .map_or(egui::Color32::TRANSPARENT, |c| c.linear_multiply(0.08));

    let final_bg = if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        base_bg
    };

    let stroke = if is_selected {
        ui.visuals().selection.stroke
    } else {
        egui::Stroke::NONE
    };

    ui.painter()
        .rect(rect, 4.0, final_bg, stroke, egui::StrokeKind::Outside);
}

fn render_row_contents(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    element: &ElementWrapper,
    folder_state: &mut egui::collapsing_header::CollapsingState,
    assets: &AssetStore,
    file: &SBarDefFile,
    state: &PreviewState,
    is_visible: bool,
    selection: &mut HashSet<Vec<usize>>,
    my_path: &[usize],
) -> egui::Response {
    let mut cursor_x = rect.min.x + 4.0;
    let thumb_rect = egui::Rect::from_min_size(
        egui::pos2(cursor_x, rect.min.y + 3.0),
        egui::vec2(thumbnails::THUMB_SIZE, thumbnails::THUMB_SIZE),
    );
    let mut thumb_ui = ui.new_child(egui::UiBuilder::new().max_rect(thumb_rect));

    let is_selected = selection.contains(my_path);

    let res = if element._cacoco_text.is_some() {
        thumbnails::draw_thumbnail_widget(&mut thumb_ui, None, Some("T"), !is_visible)
    } else if matches!(element.data, Element::Canvas(_)) {
        let icon = if folder_state.is_open() {
            "📂"
        } else {
            "📁"
        };
        thumbnails::draw_thumbnail_widget(&mut thumb_ui, None, Some(icon), !is_visible)
    } else {
        thumbnails::draw_thumbnail(
            &mut thumb_ui,
            element,
            assets,
            file,
            state,
            is_visible,
            is_selected,
        )
    };

    if res.clicked() {
        if !element.children().is_empty() && element._cacoco_text.is_none() {
            folder_state.toggle(ui);
        } else {
            selection.clear();
            selection.insert(my_path.to_vec());
        }
    }

    cursor_x += thumbnails::THUMB_SIZE + 8.0;
    let color = if !is_visible {
        ui.visuals().weak_text_color()
    } else {
        ui.visuals().text_color()
    };

    ui.painter().text(
        egui::pos2(cursor_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        element.display_name(),
        egui::FontId::proportional(14.0),
        color,
    );
    res
}

fn handle_context_menu(
    ui: &egui::Ui,
    response: &egui::Response,
    my_path: &[usize],
    selection: &mut HashSet<Vec<usize>>,
    actions: &mut Vec<LayerAction>,
    is_first: bool,
    is_last: bool,
    file: &SBarDefFile,
    confirmation_modal: &mut Option<crate::app::ConfirmationRequest>,
) {
    let just_opened = ContextMenu::check(ui, response);
    if let Some(menu) = ContextMenu::get(ui, response.id) {
        if !selection.contains(my_path) {
            selection.clear();
            selection.insert(my_path.to_vec());
        }
        ContextMenu::show(ui, menu, just_opened, |ui| {
            let can_group = !selection.is_empty() && selection.iter().all(|p| p.len() >= 2);
            if ContextMenu::button(ui, "Group in New Canvas", can_group) {
                actions.push(LayerAction::UndoSnapshot);
                actions.push(LayerAction::GroupSelection(
                    selection.iter().cloned().collect(),
                ));
                ContextMenu::close(ui);
            }
            ui.separator();

            if ContextMenu::button(ui, "Duplicate", true) {
                actions.push(LayerAction::UndoSnapshot);
                actions.push(LayerAction::DuplicateSelection(
                    selection.iter().cloned().collect(),
                ));
                ContextMenu::close(ui);
            }
            ui.separator();
            let single = selection.len() == 1;
            if ContextMenu::button(ui, "Move Up", single && !is_first) {
                actions.push(LayerAction::UndoSnapshot);
                actions.push(LayerAction::MoveUp(my_path.to_vec()));
                ContextMenu::close(ui);
            }
            if ContextMenu::button(ui, "Move Down", single && !is_last) {
                actions.push(LayerAction::UndoSnapshot);
                actions.push(LayerAction::MoveDown(my_path.to_vec()));
                ContextMenu::close(ui);
            }
            ui.separator();
            if ContextMenu::button(ui, "Delete", true) {
                if deletion_needs_confirmation(file, selection) {
                    let paths: Vec<Vec<usize>> = selection.iter().cloned().collect();
                    *confirmation_modal =
                        Some(crate::app::ConfirmationRequest::DeleteLayers(paths));
                } else {
                    actions.push(LayerAction::UndoSnapshot);
                    actions.push(LayerAction::DeleteSelection(
                        selection.iter().cloned().collect(),
                    ));
                }
                ContextMenu::close(ui);
            }
        });
    }
}

fn render_drag_ghost(
    ui: &egui::Ui,
    element: &ElementWrapper,
    assets: &AssetStore,
    file: &SBarDefFile,
    state: &PreviewState,
    is_visible: bool,
    count: usize,
) {
    let label = if count > 1 {
        format!("{} items", count)
    } else {
        element.display_name()
    };

    shared::draw_drag_ghost(
        ui.ctx(),
        |ui| {
            if element._cacoco_text.is_some() {
                thumbnails::draw_thumbnail_widget(ui, None, Some("T"), false);
            } else if matches!(element.data, Element::Canvas(_)) {
                thumbnails::draw_thumbnail_widget(ui, None, Some("📁"), false);
            } else {
                thumbnails::draw_thumbnail(ui, element, assets, file, state, is_visible, true);
            }
        },
        &label,
    );
}

fn handle_auto_scroll(ui: &mut egui::Ui) {
    let has_payload = egui::DragAndDrop::payload::<Vec<usize>>(ui.ctx()).is_some()
        || egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()).is_some();

    if has_payload {
        if let Some(pos) = ui.ctx().input(|i| i.pointer.latest_pos()) {
            let clip = ui.clip_rect();

            if clip.contains(pos) {
                let scroll_margin = 30.0;
                let scroll_speed = 8.0;

                if pos.y < clip.min.y + scroll_margin {
                    ui.scroll_with_delta(egui::vec2(0.0, scroll_speed));
                } else if pos.y > clip.max.y - scroll_margin {
                    ui.scroll_with_delta(egui::vec2(0.0, -scroll_speed));
                }
            }
        }
    }
}

fn collect_visible_paths(
    ui: &egui::Ui,
    elements: &[ElementWrapper],
    current_path: Vec<usize>,
) -> Vec<Vec<usize>> {
    let mut paths = Vec::new();
    for (idx, element) in elements.iter().enumerate() {
        let mut my_path = current_path.clone();
        my_path.push(idx);
        paths.push(my_path.clone());

        let id = ui.make_persistent_id(element.uid);
        let folder = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id.with("collapse"),
            true,
        );

        if folder.is_open() && !element.children().is_empty() && element._cacoco_text.is_none() {
            paths.extend(collect_visible_paths(ui, element.children(), my_path));
        }
    }
    paths
}

fn calculate_drop_target(
    ui: &egui::Ui,
    rect: egui::Rect,
    my_idx: usize,
    is_container: bool,
) -> Option<DropTarget> {
    let pos = ui.ctx().input(|i| i.pointer.latest_pos())?;
    let off = ui.spacing().item_spacing.y * 0.5;

    if !is_container {
        if pos.y < rect.center().y {
            Some(DropTarget::Sibling(my_idx, rect.top() - off))
        } else {
            Some(DropTarget::Sibling(my_idx + 1, rect.bottom() + off))
        }
    } else {
        let h = rect.height();
        let rel_y = pos.y - rect.top();
        if rel_y < h * 0.25 {
            Some(DropTarget::Sibling(my_idx, rect.top() - off))
        } else if rel_y > h * 0.75 {
            Some(DropTarget::Sibling(my_idx + 1, rect.bottom() + off))
        } else {
            Some(DropTarget::Child)
        }
    }
}

/// Helper to check if any part of the selection contains children, requiring deletion confirmation.
pub fn deletion_needs_confirmation(file: &SBarDefFile, selection: &HashSet<Vec<usize>>) -> bool {
    for path in selection {
        if path.len() < 2 {
            continue;
        }
        if let Some(el) = file.get_element(path) {
            if !el.children().is_empty() {
                return true;
            }
        }
    }
    false
}
