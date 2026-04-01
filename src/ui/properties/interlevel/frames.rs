use crate::assets::{AssetId, AssetStore};
use crate::models::interlevel::{InterlevelAnim, InterlevelFrame};
use crate::ui::layers::thumbnails;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

pub(super) enum InterlevelFrameAction {
    MoveSelection(Vec<usize>, usize),
    Add(usize, String),
    Replace(usize, String),
}

pub(super) fn get_active_frame_index(
    frames: &[InterlevelFrame],
    current_time: f64,
) -> Option<usize> {
    if frames.is_empty() {
        return None;
    }

    let mut total_duration = 0.0;
    let mut has_infinite = false;
    for f in frames {
        let f_type = f.frame_type & 0x7;
        if f_type == 1 {
            has_infinite = true;
            break;
        }
        total_duration += f.duration;
    }

    if total_duration > 0.0 && !has_infinite {
        let anim_time = current_time % total_duration;
        let mut accumulator = 0.0;
        for (idx, frame) in frames.iter().enumerate() {
            accumulator += frame.duration;
            if anim_time < accumulator {
                return Some(idx);
            }
        }
    } else if has_infinite {
        let mut accumulator = 0.0;
        for (idx, frame) in frames.iter().enumerate() {
            let f_type = frame.frame_type & 0x7;
            accumulator += frame.duration;
            if f_type == 1 || current_time < accumulator {
                return Some(idx);
            }
        }
    }
    Some(0)
}

fn draw_interlevel_frame_row(
    ui: &mut egui::Ui,
    idx: usize,
    frame: &mut InterlevelFrame,
    assets: &AssetStore,
    actions: &mut Vec<InterlevelFrameAction>,
    selection: &mut HashSet<usize>,
    pivot: &mut Option<usize>,
    is_active: bool,
) -> bool {
    let mut changed = false;
    let row_height = 54.0;
    let is_selected = selection.contains(&idx);
    let spacing_offset = ui.spacing().item_spacing.y * 0.5;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click_and_drag(),
    );

    if response.clicked() {
        shared::handle_list_selection(ui, is_selected, idx, selection, pivot);
    }

    if response.drag_started() {
        if !is_selected {
            selection.clear();
            selection.insert(idx);
        }
        egui::DragAndDrop::set_payload(ui.ctx(), "INTERLEVEL_FRAME_SELECTION");
    }

    if ui.rect_contains_pointer(rect) {
        if egui::DragAndDrop::payload::<&'static str>(ui.ctx())
            .is_some_and(|p| *p == "INTERLEVEL_FRAME_SELECTION")
        {
            let pos = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
            let rel_y = pos.y - rect.top();
            let top_half = rel_y < (row_height / 2.0);
            let target_idx = if top_half { idx } else { idx + 1 };

            let is_source =
                selection.contains(&idx) || (top_half && idx > 0 && selection.contains(&(idx - 1)));
            if !is_source {
                let y = if top_half {
                    rect.top() - spacing_offset
                } else {
                    rect.bottom() + spacing_offset
                };
                shared::draw_yellow_line(ui, rect, y);
                if ui.input(|i| i.pointer.any_released()) {
                    actions.push(InterlevelFrameAction::MoveSelection(
                        selection.iter().cloned().collect(),
                        target_idx,
                    ));
                }
            }
        }

        if let Some(asset_keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
            let pos = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
            let rel_y = pos.y - rect.top();
            let margin = row_height * 0.25;

            if rel_y > margin && rel_y < (row_height - margin) {
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(2.0, egui::Color32::GREEN),
                    egui::StrokeKind::Inside,
                );
                if ui.input(|i| i.pointer.any_released()) {
                    actions.push(InterlevelFrameAction::Replace(idx, asset_keys[0].clone()));
                }
            } else {
                let top_half = rel_y < (row_height / 2.0);
                let y = if top_half {
                    rect.top() - spacing_offset
                } else {
                    rect.bottom() + spacing_offset
                };
                let mut target_idx = if top_half { idx } else { idx + 1 };
                shared::draw_yellow_line(ui, rect, y);

                if ui.input(|i| i.pointer.any_released()) {
                    for key in asset_keys.iter() {
                        actions.push(InterlevelFrameAction::Add(target_idx, key.clone()));
                        target_idx += 1;
                    }
                }
            }
        }
    }

    let mut bg = if is_active {
        egui::Color32::from_rgba_unmultiplied(0, 255, 255, 10)
    } else {
        egui::Color32::TRANSPARENT
    };
    if response.hovered() {
        bg = ui.visuals().widgets.hovered.bg_fill;
    }
    let stroke = if is_selected {
        ui.visuals().selection.stroke
    } else {
        egui::Stroke::NONE
    };
    ui.painter()
        .rect(rect, 4.0, bg, stroke, egui::StrokeKind::Outside);

    let thumb_rect = egui::Rect::from_center_size(
        egui::pos2(rect.min.x + 22.0, rect.center().y),
        egui::vec2(thumbnails::THUMB_SIZE, thumbnails::THUMB_SIZE),
    );
    let mut thumb_ui = ui.new_child(egui::UiBuilder::new().max_rect(thumb_rect));

    let id = AssetId::new(&frame.image);
    thumbnails::draw_thumbnail_widget(
        &mut thumb_ui,
        assets.textures.get(&id),
        Some("?"),
        false,
        false,
    );

    let content_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + 48.0, rect.min.y + 4.0),
        egui::pos2(rect.max.x - 4.0, rect.max.y - 4.0),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&frame.image).strong());

                let mut base_type = frame.frame_type & 0x7;
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .radio_value(&mut base_type, 4, "Rnd")
                        .on_hover_text("Random Duration")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut base_type, 2, "Fix")
                        .on_hover_text("Fixed Duration")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut base_type, 1, "Inf")
                        .on_hover_text("Infinite Duration")
                        .changed()
                    {
                        changed = true;
                    }
                });

                if changed {
                    frame.frame_type = (frame.frame_type & !0x7) | base_type;
                }
            });

            ui.horizontal(|ui| {
                let base_type = frame.frame_type & 0x7;

                if base_type == 2 || base_type == 4 {
                    changed |=
                        crate::ui::properties::common::draw_tic_drag_value(ui, &mut frame.duration);
                }

                if base_type == 4 {
                    ui.label("Max:");
                    changed |= crate::ui::properties::common::draw_tic_drag_value(
                        ui,
                        &mut frame.maxduration,
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut widescreen = (frame.frame_type & 0x8000000) != 0;
                    if ui
                        .checkbox(&mut widescreen, "Wide")
                        .on_hover_text("Center for Widescreen")
                        .changed()
                    {
                        frame.frame_type = (frame.frame_type & !0x8000000)
                            | (if widescreen { 0x8000000 } else { 0 });
                        changed = true;
                    }

                    let mut random_first = (frame.frame_type & 0x1000) != 0;
                    if ui
                        .checkbox(&mut random_first, "RFF")
                        .on_hover_text("Random First Frame Offset")
                        .changed()
                    {
                        frame.frame_type =
                            (frame.frame_type & !0x1000) | (if random_first { 0x1000 } else { 0 });
                        changed = true;
                    }
                });
            });
        });
    });

    if response.dragged() {
        let label = if selection.len() > 1 {
            format!("{} frames", selection.len())
        } else {
            frame.image.clone()
        };
        shared::draw_drag_ghost(
            ui.ctx(),
            |ui| {
                thumbnails::draw_thumbnail_widget(
                    ui,
                    assets.textures.get(&id),
                    Some("?"),
                    false,
                    false,
                );
            },
            &label,
        );
    }
    changed
}

fn draw_empty_frame_dropzone(ui: &mut egui::Ui, actions: &mut Vec<InterlevelFrameAction>) -> bool {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 60.0), egui::Sense::hover());
    ui.painter().rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "Drop Graphics Here",
        egui::FontId::proportional(14.0),
        egui::Color32::from_gray(100),
    );

    if let Some(keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
        if ui.rect_contains_pointer(rect) {
            ui.painter()
                .rect_filled(rect, 4.0, egui::Color32::from_white_alpha(10));
            if ui.input(|i| i.pointer.any_released()) {
                for key in keys.iter() {
                    actions.push(InterlevelFrameAction::Add(9999, key.clone()));
                }
                return true;
            }
        }
    }
    false
}

pub(super) fn draw_interlevel_frames_editor(
    ui: &mut egui::Ui,
    anim: &mut InterlevelAnim,
    assets: &AssetStore,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.heading("Frames");
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Sequence of images to play.").weak());

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Add").clicked() {
                anim.frames.push(InterlevelFrame {
                    image: "HICACOCO".to_string(),
                    duration: 1.0,
                    frame_type: 2,
                    ..Default::default()
                });
                changed = true;
            }
            if !anim.frames.is_empty() && ui.button("Clear").clicked() {
                anim.frames.clear();
                changed = true;
            }
        });
    });
    ui.separator();

    let active_idx = get_active_frame_index(&anim.frames, ui.input(|i| i.time));

    let sel_id = ui.make_persistent_id("ilvl_frame_selection");
    let pivot_id = ui.make_persistent_id("ilvl_frame_pivot");
    let mut f_selection: HashSet<usize> = ui.data(|d| d.get_temp(sel_id).unwrap_or_default());
    let mut f_pivot: Option<usize> = ui.data(|d| d.get_temp(pivot_id));

    let mut frame_actions = Vec::new();
    ui.spacing_mut().item_spacing.y = 1.0;

    if anim.frames.is_empty() {
        changed |= draw_empty_frame_dropzone(ui, &mut frame_actions);
    } else {
        for (idx, frame) in anim.frames.iter_mut().enumerate() {
            let is_active = active_idx == Some(idx);
            ui.push_id(idx, |ui| {
                changed |= draw_interlevel_frame_row(
                    ui,
                    idx,
                    frame,
                    assets,
                    &mut frame_actions,
                    &mut f_selection,
                    &mut f_pivot,
                    is_active,
                );
            });
        }
    }

    for action in frame_actions {
        changed = true;
        match action {
            InterlevelFrameAction::MoveSelection(sources, mut target_idx) => {
                let mut sorted_src = sources.clone();
                sorted_src.sort();
                let mut src_desc = sorted_src.clone();
                src_desc.sort_by(|a, b| b.cmp(a));

                let mut moved_items = Vec::new();
                for src in src_desc {
                    if src < target_idx {
                        target_idx -= 1;
                    }
                    if src < anim.frames.len() {
                        moved_items.push(anim.frames.remove(src));
                    }
                }
                moved_items.reverse();

                let safe_idx = target_idx.min(anim.frames.len());
                f_selection.clear();
                for (i, item) in moved_items.into_iter().enumerate() {
                    anim.frames.insert(safe_idx + i, item);
                    f_selection.insert(safe_idx + i);
                }
            }
            InterlevelFrameAction::Add(i, lump) => {
                anim.frames.insert(
                    i.min(anim.frames.len()),
                    InterlevelFrame {
                        image: lump,
                        duration: 1.0,
                        frame_type: 2,
                        ..Default::default()
                    },
                );
                f_selection.clear();
                f_selection.insert(i.min(anim.frames.len() - 1));
            }
            InterlevelFrameAction::Replace(i, lump) => {
                if i < anim.frames.len() {
                    anim.frames[i].image = lump;
                }
            }
        }
    }

    ui.data_mut(|d| {
        d.insert_temp(sel_id, f_selection);
        d.insert_temp(pivot_id, f_pivot);
    });

    changed
}
