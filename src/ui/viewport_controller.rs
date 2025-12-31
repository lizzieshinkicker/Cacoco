use crate::document::LayerAction;
use crate::render::projection::ViewportProjection;
use eframe::egui;
use std::collections::HashSet;

/// Manages the persistent interaction state of the HUD viewport.
///
/// The controller handles coordinate mapping, element translation (dragging),
/// and processing drag-and-drop payloads from the asset browser.
#[derive(Default)]
pub struct ViewportController {
    /// Accumulates sub-pixel movement during mouse drags to prevent rounding jitter.
    pub move_accumulator: egui::Vec2,
    /// Tracks if a drag operation was initiated in this frame.
    pub is_dragging: bool,
}

impl ViewportController {
    /// Resets the interaction state. Called when a drag operation finishes.
    pub fn reset(&mut self) {
        self.move_accumulator = egui::Vec2::ZERO;
        self.is_dragging = false;
    }

    /// Processes primary mouse dragging to translate the current selection.
    pub fn handle_selection_drag(
        &mut self,
        ui: &egui::Ui,
        proj: &ViewportProjection,
        selection: &HashSet<Vec<usize>>,
        viewport_res: &egui::Response,
    ) -> Vec<LayerAction> {
        let mut actions = Vec::new();

        if viewport_res.dragged_by(egui::PointerButton::Primary) && !selection.is_empty() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            self.is_dragging = true;

            if viewport_res.drag_started() {
                actions.push(LayerAction::UndoSnapshot);
            }

            let delta = ui.input(|i| i.pointer.delta());

            self.move_accumulator.x += delta.x / proj.final_scale_x;
            self.move_accumulator.y += delta.y / proj.final_scale_y;

            let move_x = self.move_accumulator.x.trunc() as i32;
            let move_y = self.move_accumulator.y.trunc() as i32;

            if move_x != 0 || move_y != 0 {
                self.move_accumulator.x -= move_x as f32;
                self.move_accumulator.y -= move_y as f32;

                actions.push(LayerAction::TranslateSelection {
                    paths: selection.iter().cloned().collect(),
                    dx: move_x,
                    dy: move_y,
                });
            }
        } else if viewport_res.drag_stopped() {
            self.reset();
        }

        actions
    }
}
