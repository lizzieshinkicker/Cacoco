use crate::assets::{AssetId, AssetStore};
use crate::constants::DOOM_TICS_PER_SEC;
use crate::model::{AnimationDef, FrameDef};
use crate::state::PreviewState;
use crate::ui::layers::thumbnails;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

use super::editor::PropertiesUI;
use super::font_cache::FontCache;
use super::preview::PreviewContent;

enum FrameAction {
    MoveSelection(Vec<usize>, usize),
    Add(usize, String),
    Replace(usize, String),
}

impl PropertiesUI for AnimationDef {
    /// Renders the frame list editor for Animation elements.
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        _fonts: &FontCache,
        assets: &AssetStore,
        _state: &PreviewState,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.heading("Frames");
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(format!("{:.0} tics/sec", DOOM_TICS_PER_SEC))
                    .weak()
                    .italics(),
            );

            self.framerate = DOOM_TICS_PER_SEC;

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !self.frames.is_empty() && ui.button("Clear").clicked() {
                    self.frames.clear();
                    changed = true;
                }
                if ui.button("Add").clicked() {
                    self.frames.push(FrameDef {
                        lump: "HICACOCO".to_string(),
                        duration: 1.0 / DOOM_TICS_PER_SEC,
                    });
                    changed = true;
                }
            });
        });
        ui.separator();

        let total_duration: f64 = self.frames.iter().map(|f| f.duration).sum();
        let mut active_idx = None;
        if total_duration > 0.0 {
            let anim_time = ui.input(|i| i.time) % total_duration;
            let mut accumulator = 0.0;
            for (idx, frame) in self.frames.iter().enumerate() {
                accumulator += frame.duration;
                if anim_time < accumulator {
                    active_idx = Some(idx);
                    break;
                }
            }
        }

        let sel_id = ui.make_persistent_id("anim_frame_selection");
        let pivot_id = ui.make_persistent_id("anim_frame_pivot");
        let mut selection: HashSet<usize> = ui.data(|d| d.get_temp(sel_id).unwrap_or_default());
        let mut pivot: Option<usize> = ui.data(|d| d.get_temp(pivot_id));

        let mut actions = Vec::new();
        ui.spacing_mut().item_spacing.y = 1.0;

        if self.frames.is_empty() {
            changed |= draw_empty_frame_dropzone(ui, &mut actions);
        } else {
            for (idx, frame) in self.frames.iter_mut().enumerate() {
                let is_active = active_idx == Some(idx);
                ui.push_id(idx, |ui| {
                    changed |= draw_frame_row(
                        ui,
                        idx,
                        frame,
                        assets,
                        &mut actions,
                        &mut selection,
                        &mut pivot,
                        is_active,
                    );
                });
            }
        }

        for action in actions {
            changed = true;
            match action {
                FrameAction::MoveSelection(sources, mut target_idx) => {
                    let mut sorted_src = sources.clone();
                    sorted_src.sort();
                    let mut src_desc = sorted_src.clone();
                    src_desc.sort_by(|a, b| b.cmp(a));

                    let mut moved_items = Vec::new();
                    for src in src_desc {
                        if src < target_idx {
                            target_idx -= 1;
                        }
                        if src < self.frames.len() {
                            moved_items.push(self.frames.remove(src));
                        }
                    }
                    moved_items.reverse();

                    let safe_idx = target_idx.min(self.frames.len());
                    selection.clear();
                    for (i, item) in moved_items.into_iter().enumerate() {
                        self.frames.insert(safe_idx + i, item);
                        selection.insert(safe_idx + i);
                    }
                }
                FrameAction::Add(i, lump) => {
                    self.frames.insert(
                        i.min(self.frames.len()),
                        FrameDef {
                            lump,
                            duration: 1.0 / DOOM_TICS_PER_SEC,
                        },
                    );
                    selection.clear();
                    selection.insert(i.min(self.frames.len() - 1));
                }
                FrameAction::Replace(i, lump) => {
                    if i < self.frames.len() {
                        self.frames[i].lump = lump;
                    }
                }
            }
        }

        ui.data_mut(|d| {
            d.insert_temp(sel_id, selection);
            d.insert_temp(pivot_id, pivot);
        });

        changed
    }

    fn get_preview_content(
        &self,
        _: &egui::Ui,
        _: &FontCache,
        _: &PreviewState,
    ) -> Option<PreviewContent> {
        let first = self
            .frames
            .first()
            .map(|f| f.lump.clone())
            .unwrap_or_default();
        Some(PreviewContent::Image(first))
    }
}

fn draw_frame_row(
    ui: &mut egui::Ui,
    idx: usize,
    frame: &mut FrameDef,
    assets: &AssetStore,
    actions: &mut Vec<FrameAction>,
    selection: &mut HashSet<usize>,
    pivot: &mut Option<usize>,
    is_active: bool,
) -> bool {
    let mut changed = false;
    let row_height = 42.0;
    let is_selected = selection.contains(&idx);
    let spacing_offset = ui.spacing().item_spacing.y * 0.5;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click_and_drag(),
    );

    if response.clicked() {
        let modifiers = ui.input(|i| i.modifiers);
        if modifiers.ctrl || modifiers.command {
            if is_selected {
                selection.remove(&idx);
            } else {
                selection.insert(idx);
                *pivot = Some(idx);
            }
        } else if modifiers.shift {
            if let Some(p) = *pivot {
                let min = p.min(idx);
                let max = p.max(idx);
                selection.clear();
                for i in min..=max {
                    selection.insert(i);
                }
            } else {
                selection.insert(idx);
                *pivot = Some(idx);
            }
        } else {
            selection.clear();
            selection.insert(idx);
            *pivot = Some(idx);
        }
    }

    if response.drag_started() {
        if !is_selected {
            selection.clear();
            selection.insert(idx);
        }
        egui::DragAndDrop::set_payload(ui.ctx(), "FRAME_SELECTION");
    }

    if ui.rect_contains_pointer(rect) {
        if egui::DragAndDrop::payload::<&'static str>(ui.ctx())
            .is_some_and(|p| *p == "FRAME_SELECTION")
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
                    actions.push(FrameAction::MoveSelection(
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
                    actions.push(FrameAction::Replace(idx, asset_keys[0].clone()));
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
                        actions.push(FrameAction::Add(target_idx, key.clone()));
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

    let id = AssetId::new(&frame.lump);
    thumbnails::draw_thumbnail_widget(&mut thumb_ui, assets.textures.get(&id), Some("?"), false);

    ui.painter().text(
        egui::pos2(rect.min.x + 44.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &frame.lump,
        egui::FontId::proportional(14.0),
        ui.visuals().text_color(),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            let mut tics = (frame.duration * DOOM_TICS_PER_SEC).round() as i32;
            if ui
                .add(
                    egui::DragValue::new(&mut tics)
                        .suffix(" tics")
                        .range(1..=3500),
                )
                .changed()
            {
                frame.duration = tics as f64 / DOOM_TICS_PER_SEC;
                changed = true;
            }
        });
    });

    if response.dragged() {
        let label = if selection.len() > 1 {
            format!("{} frames", selection.len())
        } else {
            frame.lump.clone()
        };
        let id = AssetId::new(&frame.lump);
        shared::draw_drag_ghost(
            ui.ctx(),
            |ui| {
                thumbnails::draw_thumbnail_widget(ui, assets.textures.get(&id), Some("?"), false);
            },
            &label,
        );
    }
    changed
}

fn draw_empty_frame_dropzone(ui: &mut egui::Ui, actions: &mut Vec<FrameAction>) -> bool {
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
                    actions.push(FrameAction::Add(9999, key.clone()));
                }
                return true;
            }
        }
    }
    false
}
