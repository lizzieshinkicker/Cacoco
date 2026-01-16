use crate::app::{CacocoApp, ConfirmationRequest, PendingAction};
use crate::assets::AssetId;
use crate::document;
use crate::ui::messages::{self, EditorEvent};
use eframe::egui;

/// Renders the onboarding screen for users who haven't selected an IWAD yet.
pub fn draw_onboarding_screen(ctx: &egui::Context, app: &mut CacocoApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        let id = AssetId::new("HICACOCO");
        if let Some(tex) = app.assets.textures.get(&id) {
            let size = tex.size_vec2() * 2.0;
            let rect =
                egui::Rect::from_center_size(ui.max_rect().center() - egui::vec2(0.0, 150.0), size);
            ui.painter().image(
                tex.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }

        egui::Window::new("Initial Setup")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 50.0))
            .show(ctx, |ui| {
                ui.set_width(400.0);
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.heading("Welcome to Cacoco!");
                    ui.add_space(12.0);

                    let desc = app
                        .config
                        .base_wad_path
                        .as_deref()
                        .unwrap_or("Click to browse for DOOM2.WAD...");

                    if crate::ui::menu::draw_menu_card(ui, "Select Base DOOM II IWAD", desc) {
                        if let Some(p) = crate::io::load_iwad_dialog(ctx, &mut app.assets) {
                            app.config.base_wad_path = Some(p);
                            app.config.save();
                            app.iwad_verified = true;
                        }
                    }
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(
                            "A base Doom II IWAD is required to render graphics correctly.",
                        )
                        .weak()
                        .size(11.0),
                    );
                    ui.add_space(8.0);
                });
            });
    });
}

/// Renders various confirmation dialogs for destructive actions.
pub fn draw_confirmation_modal(
    ctx: &egui::Context,
    app: &mut CacocoApp,
    request: &ConfirmationRequest,
) {
    let mut close_modal = false;
    let mut confirmed = false;

    let title = match request {
        ConfirmationRequest::DiscardChanges(_) => "Unsaved Changes",
        ConfirmationRequest::DowngradeTarget(_) => "Confirm Downgrade",
        _ => "Confirm Deletion",
    };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(320.0);
            ui.vertical_centered(|ui| match request {
                ConfirmationRequest::DeleteStatusBar(_) => {
                    ui.label("Delete this status bar layout?");
                }
                ConfirmationRequest::DeleteLayers(paths) => {
                    ui.label(format!("Delete {} selected layers?", paths.len()));
                }
                ConfirmationRequest::DeleteAssets(items) => {
                    ui.label(format!("Delete {} assets from project?", items.len()));
                }
                ConfirmationRequest::DiscardChanges(_) => {
                    ui.label("Discard unsaved changes and continue?");
                }
                ConfirmationRequest::DowngradeTarget(_) => {
                    ui.label(
                        egui::RichText::new("Warning: Extended Features Detected")
                            .color(egui::Color32::from_rgb(200, 100, 100))
                            .strong(),
                    );
                    ui.add_space(8.0);
                    ui.label("Switching to Basic (KEX) target will strip extended features (Components, Lists, Conditionals) and enforce mandatory layout slots.");
                    ui.add_space(4.0);
                    ui.label("This action is permanent for the current session state.");
                }
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("  Cancel  ").clicked() {
                    close_modal = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (text, color) = match request {
                        ConfirmationRequest::DiscardChanges(_) => {
                            ("Discard", egui::Color32::from_rgb(110, 40, 40))
                        }
                        ConfirmationRequest::DowngradeTarget(_) => {
                            ("Downgrade", egui::Color32::from_rgb(110, 40, 40))
                        }
                        _ => ("Delete", egui::Color32::from_rgb(110, 40, 40)),
                    };

                    let btn = egui::Button::new(text).fill(color);
                    if ui.add(btn).clicked() {
                        confirmed = true;
                        close_modal = true;
                    }
                });
            });
        });

    if confirmed {
        match request {
            ConfirmationRequest::DeleteStatusBar(idx) => {
                app.execute_actions(vec![
                    document::LayerAction::UndoSnapshot,
                    document::LayerAction::DeleteStatusBar(*idx),
                ]);
                messages::log_event(&mut app.preview_state, EditorEvent::Delete);
            }
            ConfirmationRequest::DeleteLayers(paths) => {
                app.execute_actions(vec![
                    document::LayerAction::UndoSnapshot,
                    document::LayerAction::DeleteSelection(paths.clone()),
                ]);
                messages::log_event(&mut app.preview_state, EditorEvent::Delete);
            }
            ConfirmationRequest::DeleteAssets(items) => {
                for key in items {
                    let id = AssetId::new(key);
                    app.assets.textures.remove(&id);
                    app.assets.raw_files.remove(&id);
                    app.assets.offsets.remove(&id);
                    app.assets.names.remove(&id);
                }
                if let Some(doc) = &mut app.doc {
                    doc.dirty = true;
                }
                messages::log_event(&mut app.preview_state, EditorEvent::AssetsDeleted);
            }
            ConfirmationRequest::DiscardChanges(pending) => {
                if let Some(doc) = &mut app.doc {
                    doc.dirty = false;
                }
                match pending {
                    PendingAction::New => app.new_project(
                        ctx,
                        crate::models::ProjectData::StatusBar(
                            crate::models::sbardef::SBarDefFile::new_empty(),
                        ),
                    ),
                    PendingAction::Load(path) => {
                        if path.is_empty() {
                            app.open_project_ui(ctx);
                        } else if let Some(loaded) = crate::io::load_project_from_path(ctx, &path) {
                            app.load_project(ctx, loaded, &path);
                        }
                    }
                    PendingAction::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                }
            }
            ConfirmationRequest::DowngradeTarget(t) => {
                if let Some(doc) = &mut app.doc {
                    doc.execute_actions(vec![document::LayerAction::UndoSnapshot]);
                    doc.file.set_target(*t);
                    doc.file.normalize_for_target();
                    doc.dirty = true;
                }
            }
        }
    }
    if close_modal {
        app.confirmation_modal = None;
    }
}
