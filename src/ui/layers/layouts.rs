use crate::document::LayerAction;
use crate::models::sbardef::SBarDefFile;
use crate::ui::context_menu::ContextMenu;
use crate::ui::layers::thumbnails::ListRow;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

pub fn draw_layouts_browser(
    ui: &mut egui::Ui,
    file: &mut SBarDefFile,
    selection: &mut HashSet<Vec<usize>>,
    current_bar_idx: &mut usize,
    actions: &mut Vec<LayerAction>,
    confirmation_modal: &mut Option<crate::app::ConfirmationRequest>,
) -> bool {
    let mut changed = false;

    if shared::heading_action_button(ui, "Layouts", Some("New Layout"), false).clicked() {
        actions.push(LayerAction::UndoSnapshot);
        actions.push(LayerAction::AddStatusBar);
        changed = true;
    }

    let mut move_request = None;
    let mut duplicate_request = None;
    let mut delete_request = None;

    let bar_count = file.data.status_bars.len();

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;

        for i in 0..bar_count {
            let is_active = *current_bar_idx == i;
            let is_selected = selection.contains(&vec![i]);
            let bar = &file.data.status_bars[i];

            let system_label = bar._cacoco_system_locked.clone();
            let is_system = system_label.is_some();

            let label = system_label.unwrap_or_else(|| {
                bar.name
                    .clone()
                    .unwrap_or_else(|| format!("Status Bar #{}", i))
            });

            let height_str = if bar.fullscreen_render {
                "Fullscreen".to_string()
            } else {
                format!("{}px", bar.height)
            };
            let sub = format!("{}, Children: {}", height_str, bar.children.len());
            let thumb_label = format!("#{}", i);

            let mut response = ui
                .horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.scope(|ui| {
                        ui.set_width(ui.available_width() - 4.0);
                        ListRow::new(label)
                            .subtitle(sub)
                            .fallback(&thumb_label)
                            .active(is_active)
                            .selected(is_selected)
                            .dimmed(is_system)
                            .system(is_system)
                            .show(ui)
                    })
                    .inner
                })
                .inner;

            if is_system {
                response = response.on_hover_text(
                    "This layout slot is mandatory for the KEX port and cannot be modified.",
                );
            }

            if response.clicked() {
                selection.clear();
                selection.insert(vec![i]);
                *current_bar_idx = i;
            }

            if !is_system && response.drag_started() {
                egui::DragAndDrop::set_payload(ui.ctx(), i);
            }

            if let Some(source_idx) = egui::DragAndDrop::payload::<usize>(ui.ctx()) {
                if ui.rect_contains_pointer(response.rect) && *source_idx != i {
                    let pos = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
                    let is_top = pos.y < response.rect.center().y;

                    let target_idx = if is_top { i } else { i + 1 };

                    if target_idx > 0 && target_idx < bar_count {
                        let target_y = if is_top {
                            response.rect.top()
                        } else {
                            response.rect.bottom()
                        };
                        shared::draw_yellow_line(ui, response.rect, target_y);

                        if ui.input(|i| i.pointer.any_released()) {
                            move_request = Some((*source_idx, target_idx));
                        }
                    }
                }
            }

            let just_opened = ContextMenu::check(ui, &response);
            if let Some(menu) = ContextMenu::get(ui, response.id) {
                ContextMenu::show(ui, menu, just_opened, |ui| {
                    if is_system {
                        ui.label(
                            egui::RichText::new("🔒 System Locked Slot")
                                .color(egui::Color32::from_rgb(200, 100, 100))
                                .size(10.0),
                        );
                        ui.separator();
                    }

                    if ContextMenu::button(ui, "Duplicate", !is_system) {
                        duplicate_request = Some(i);
                        ContextMenu::close(ui);
                    }
                    ui.separator();
                    if ContextMenu::button(ui, "Delete", !is_system) {
                        if bar.children.is_empty() {
                            delete_request = Some(i);
                        } else {
                            *confirmation_modal =
                                Some(crate::app::ConfirmationRequest::DeleteStatusBar(i));
                        }
                        ContextMenu::close(ui);
                    }
                });
            }
        }

        ui.add_space(2.0);
    });

    if let Some((source, target)) = move_request {
        actions.push(LayerAction::UndoSnapshot);
        actions.push(LayerAction::MoveStatusBar { source, target });
        changed = true;
    }
    if let Some(idx) = duplicate_request {
        actions.push(LayerAction::UndoSnapshot);
        actions.push(LayerAction::DuplicateStatusBar(idx));
        changed = true;
    }
    if let Some(idx) = delete_request {
        actions.push(LayerAction::UndoSnapshot);
        actions.push(LayerAction::DeleteStatusBar(idx));
        changed = true;
    }

    if let Some(source_idx) = egui::DragAndDrop::payload::<usize>(ui.ctx()) {
        shared::draw_drag_ghost(
            ui.ctx(),
            |ui| {
                ui.label(format!("#{}", source_idx));
            },
            "Reordering Layout",
        );
    }

    changed
}
