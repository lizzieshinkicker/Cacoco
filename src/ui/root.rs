use crate::app::{CacocoApp, ConfirmationRequest, PendingAction};
use crate::ui::font_wizard;
use crate::{document, ui};
use eframe::egui;
use std::path::Path;

pub fn draw_root_ui(ctx: &egui::Context, app: &mut CacocoApp) {
    if !app.iwad_verified {
        draw_onboarding_screen(ctx, app);
        return;
    }

    if ctx.input(|i| i.viewport().close_requested()) {
        if app.dirty {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            app.confirmation_modal = Some(ConfirmationRequest::DiscardChanges(PendingAction::Quit));
        }
    }

    let dirty_indicator = if app.dirty { "*" } else { "" };

    let file_display = if app.current_file.is_some() {
        if let Some(path_str) = &app.opened_file_path {
            let name = Path::new(path_str)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(path_str);
            format!("[{}{}] ", dirty_indicator, name)
        } else {
            format!("[{}New Project] ", dirty_indicator)
        }
    } else {
        "".to_string()
    };

    let flavor_title = ctx.data(|d| d.get_temp::<String>(egui::Id::new("random_title")));
    if let Some(flavor) = flavor_title {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "{}Cacoco - {}",
            file_display, flavor
        )));
    }

    if let Some(action) = app.hotkeys.check(ctx) {
        handle_action(app, action, ctx);
    }

    app.cheat_engine.process_input(ctx, &mut app.preview_state);
    app.preview_state.update(ctx.input(|i| i.stable_dt));

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        let action = ui::draw_menu_bar(
            ui,
            ctx,
            &mut app.current_file,
            app.opened_file_path.clone(),
            &mut app.config,
            &mut app.assets,
            &mut app.settings_open,
            app.dirty,
        );

        match action {
            ui::MenuAction::LoadProject(loaded, path) => {
                app.load_project(ctx, loaded, &path);
            }
            ui::MenuAction::LoadTemplate(template) => {
                app.apply_template(ctx, template);
            }
            ui::MenuAction::NewEmpty => {
                app.new_project(ctx);
            }
            ui::MenuAction::Open => {
                app.open_project_ui(ctx);
            }
            ui::MenuAction::RequestDiscard(pending) => {
                app.confirmation_modal = Some(ConfirmationRequest::DiscardChanges(pending));
            }
            ui::MenuAction::SaveDone(path) => {
                if path == "SILENT" {
                    handle_action(app, crate::hotkeys::Action::Save, ctx);
                } else {
                    app.opened_file_path = Some(path.clone());
                    app.add_to_recent(&path);
                    app.dirty = false;
                    app.preview_state.push_message(format!("Saved: {}", path));
                }
            }
            ui::MenuAction::ExportDone(path) => {
                app.add_to_recent(&path);
                app.preview_state
                    .push_message(format!("Exported: {}", path));
            }
            ui::MenuAction::PickPortAndRun => {
                handle_pick_port_and_run(app);
            }
            _ => {}
        }
    });

    egui::SidePanel::left("left_side_panel")
        .resizable(false)
        .exact_width(289.0)
        .show(ctx, |ui| {
            let available_height = ui.available_height();

            ui.vertical(|ui| {
                ui.set_height(available_height);

                let mut props_changed = false;
                egui::ScrollArea::vertical()
                    .id_salt("properties_scroll")
                    .max_height(available_height * 0.60)
                    .show(ui, |ui| {
                        props_changed = ui::draw_properties_panel(
                            ui,
                            &mut app.current_file,
                            &app.selection,
                            &app.assets,
                            &app.preview_state,
                        );
                    });

                if props_changed {
                    app.dirty = true;
                }

                ui.separator();
                ui::draw_gamestate_panel(ui, &mut app.preview_state, &app.assets);
            });
        });

    egui::SidePanel::right("right_side_panel")
        .resizable(false)
        .exact_width(320.0)
        .show(ctx, |ui| {
            let (actions, layers_changed) = ui::draw_layers_panel(
                ui,
                &mut app.current_file,
                &mut app.selection,
                &mut app.selection_pivot,
                &mut app.assets,
                &mut app.current_statusbar_idx,
                &mut app.preview_state,
                &mut app.font_wizard,
                &mut app.confirmation_modal,
            );

            if layers_changed {
                app.dirty = true;
            }

            app.execute_actions(actions);
        });

    egui::TopBottomPanel::bottom("gamestate_panel")
        .resizable(false)
        .exact_height(140.0)
        .show(ctx, |ui| {
            ui::draw_vitals_panel(ui, &mut app.preview_state, &app.assets);
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        let actions = ui::draw_viewport(
            ui,
            &app.current_file,
            &app.assets,
            &mut app.preview_state,
            &app.selection,
            app.current_statusbar_idx,
        );

        app.execute_actions(actions);
    });

    if app.settings_open {
        ui::draw_settings_window(
            ctx,
            &mut app.settings_open,
            &mut app.config,
            &mut app.assets,
        );
    }

    if let Some(f) = &mut app.current_file {
        if font_wizard::draw_font_wizard(ctx, &mut app.font_wizard, f, &app.assets) {
            app.dirty = true;
        }
    }

    if let Some(request) = app.confirmation_modal.clone() {
        draw_confirmation_modal(ctx, app, &request);
    }
}

fn draw_onboarding_screen(ctx: &egui::Context, app: &mut CacocoApp) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(tex) = app.assets.textures.get("HICACOCO") {
            let size = tex.size_vec2() * 2.0;
            let rect = egui::Rect::from_center_size(
                ui.max_rect().center() - egui::vec2(0.0, 150.0),
                size,
            );
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

                    if ui::menu::draw_menu_card(ui, "Select Base DOOM II IWAD", desc) {
                        if let Some(path) = crate::io::load_iwad_dialog(ctx, &mut app.assets) {
                            app.config.base_wad_path = Some(path);
                            app.config.save();
                            app.iwad_verified = true;
                        }
                    }

                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new(
                            "To render graphics and fonts correctly, you must select a base Doom II IWAD.",
                        )
                            .weak()
                            .size(11.0),
                    );
                    ui.label(
                        egui::RichText::new("(...It's usually named DOOM2.WAD!)")
                            .weak()
                            .size(11.0),
                    );
                    ui.add_space(8.0);
                });
            });
    });
}

fn handle_pick_port_and_run(app: &mut CacocoApp) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Executable", &["exe"])
        .set_title("Select Source Port to Launch")
        .pick_file()
    {
        let path_str = path.to_string_lossy().into_owned();

        if !app.config.source_ports.contains(&path_str) {
            app.config.source_ports.push(path_str.clone());
            app.config.save();
        }

        if let (Some(f), Some(iwad)) = (&app.current_file, &app.config.base_wad_path) {
            crate::io::launch_game(f, &app.assets, &path_str, iwad);
        }
    }
}

fn draw_confirmation_modal(
    ctx: &egui::Context,
    app: &mut CacocoApp,
    request: &ConfirmationRequest,
) {
    let mut close_modal = false;
    let mut confirmed = false;

    let window_title = if matches!(request, ConfirmationRequest::DiscardChanges(_)) {
        "Unsaved Changes"
    } else {
        "Confirm Deletion"
    };

    egui::Window::new(window_title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(280.0);

            ui.vertical_centered(|ui| match request {
                ConfirmationRequest::DeleteStatusBar(_) => {
                    ui.label("Are you sure you want to delete this status bar?");
                    ui.label(
                        egui::RichText::new(
                            "This will permanently remove all layers and components contained within this layout.",
                        )
                            .weak()
                            .size(11.0),
                    );
                }
                ConfirmationRequest::DeleteAssets(items) => {
                    let count = items.len();
                    let main_msg = if count == 1 {
                        "Are you sure you want to delete this asset?".to_string()
                    } else {
                        format!("Are you sure you want to delete {} assets?", count)
                    };

                    ui.label(main_msg);
                    ui.label(
                        egui::RichText::new(
                            "This action is permanent. Any layers using these assets will show a missing graphic placeholder.",
                        )
                            .weak()
                            .size(11.0),
                    );
                }
                ConfirmationRequest::DiscardChanges(_) => {
                    ui.label("You have unsaved changes. Do you want to discard them and continue?");
                }
            });

            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui.button("  Cancel  ").clicked() {
                    close_modal = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn_text = if matches!(request, ConfirmationRequest::DiscardChanges(_)) {
                        "Discard & Continue"
                    } else {
                        "Confirm Deletion"
                    };

                    let btn = egui::Button::new(btn_text)
                        .fill(egui::Color32::from_rgb(110, 40, 40));

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
                if let Some(f) = &mut app.current_file {
                    app.history.take_snapshot(f, &app.selection);
                    f.data.status_bars.remove(*idx);
                    if app.current_statusbar_idx >= f.data.status_bars.len() {
                        app.current_statusbar_idx = f.data.status_bars.len().saturating_sub(1);
                    }
                    app.dirty = true;
                }
            }
            ConfirmationRequest::DeleteAssets(items) => {
                for key in items {
                    app.assets.textures.remove(key);
                    app.assets.raw_files.remove(key);
                    app.assets.offsets.remove(key);
                }
                app.dirty = true;
                app.preview_state
                    .push_message("Deleted assets from project.");
            }
            ConfirmationRequest::DiscardChanges(pending) => {
                app.dirty = false;
                match pending {
                    PendingAction::New => app.new_project(ctx),
                    PendingAction::Load(path) => {
                        if path.is_empty() {
                            app.open_project_ui(ctx);
                        } else if let Some(loaded) = crate::io::load_project_from_path(ctx, &path) {
                            app.load_project(ctx, loaded, &path);
                        }
                    }
                    PendingAction::Template(t) => app.apply_template(ctx, t),
                    PendingAction::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                }
            }
        }
    }

    if close_modal {
        app.confirmation_modal = None;
    }
}

fn handle_action(app: &mut CacocoApp, action: crate::hotkeys::Action, ctx: &egui::Context) {
    use crate::document::LayerAction;
    use crate::hotkeys::Action;

    match action {
        Action::Undo => {
            if let Some(f) = &mut app.current_file {
                app.history.undo(f, &mut app.selection);
                app.dirty = true;
                app.preview_state.push_message("Undo performed.");
            }
        }
        Action::Redo => {
            if let Some(f) = &mut app.current_file {
                app.history.redo(f, &mut app.selection);
                app.dirty = true;
                app.preview_state.push_message("Redo performed.");
            }
        }
        Action::Open => {
            if app.dirty {
                app.confirmation_modal = Some(ConfirmationRequest::DiscardChanges(
                    PendingAction::Load("".to_string()),
                ));
            } else {
                app.open_project_ui(ctx);
            }
        }
        Action::Save => {
            if let Some(f) = &app.current_file {
                let needs_dialog = match &app.opened_file_path {
                    Some(p) => !Path::new(p).is_absolute(),
                    None => true,
                };

                if needs_dialog {
                    if let Some(path) =
                        crate::io::save_pk3_dialog(f, &app.assets, app.opened_file_path.clone())
                    {
                        app.opened_file_path = Some(path.clone());
                        app.add_to_recent(&path);
                        app.dirty = false;
                        app.preview_state.push_message(format!("Saved: {}", path));
                    }
                } else {
                    let path = app.opened_file_path.as_ref().unwrap();
                    if let Err(e) = crate::io::save_pk3_silent(f, &app.assets, path) {
                        app.preview_state
                            .push_message(format!("Save Failed: {}", e));
                    } else {
                        app.dirty = false;
                        app.preview_state.push_message(format!("Saved: {}", path));
                    }
                }
            }
        }
        Action::ExportJSON => {
            if let Some(f) = &app.current_file {
                let name = app.opened_file_path.clone();
                if let Some(path) = crate::io::save_json_dialog(f, name) {
                    app.add_to_recent(&path);
                    app.preview_state
                        .push_message(format!("Exported: {}", path));
                }
            }
        }
        Action::Copy => {
            if let Some(f) = &mut app.current_file {
                app.history.clipboard.clear();
                let paths: Vec<Vec<usize>> = app.selection.iter().cloned().collect();

                let mut filtered_paths: Vec<Vec<usize>> = paths
                    .iter()
                    .filter(|p| {
                        !paths
                            .iter()
                            .any(|other| p.len() > other.len() && p.starts_with(other))
                    })
                    .cloned()
                    .collect();

                filtered_paths.sort();

                for path in filtered_paths {
                    if let Some(el) = f.get_element_mut(&path) {
                        app.history.clipboard.push(el.clone());
                    }
                }
                app.preview_state.push_message(format!(
                    "Clipboard: Copied {} elements.",
                    app.history.clipboard.len()
                ));
            }
        }
        Action::Paste => {
            if !app.history.clipboard.is_empty() {
                if let Some(f) = &mut app.current_file {
                    app.history.take_snapshot(f, &app.selection);
                    let pasted_elements = app.history.prepare_clipboard_for_paste();

                    if app.current_statusbar_idx >= f.data.status_bars.len() {
                        app.current_statusbar_idx = f.data.status_bars.len().saturating_sub(1);
                    }

                    let (parent_path, insert_idx) = document::determine_insertion_point(
                        &app.selection,
                        app.current_statusbar_idx,
                    );

                    app.preview_state.push_message(format!(
                        "Clipboard: Pasted {} elements.",
                        pasted_elements.len()
                    ));

                    let action = LayerAction::Paste {
                        parent_path,
                        insert_idx,
                        elements: pasted_elements,
                    };
                    app.dirty = true;
                    document::execute_layer_action(f, action, &mut app.selection);
                }
            }
        }
        Action::Duplicate => {
            if !app.selection.is_empty() {
                if let Some(f) = &mut app.current_file {
                    app.history.take_snapshot(f, &app.selection);
                    let paths: Vec<Vec<usize>> = app.selection.iter().cloned().collect();
                    app.dirty = true;
                    document::execute_layer_action(
                        f,
                        LayerAction::DuplicateSelection(paths),
                        &mut app.selection,
                    );
                    app.preview_state.push_message("Duplicate performed.");
                }
            }
        }
        Action::Delete => {
            if !app.selection.is_empty() {
                if let Some(f) = &mut app.current_file {
                    app.history.take_snapshot(f, &app.selection);
                    let paths: Vec<Vec<usize>> = app.selection.iter().cloned().collect();
                    app.dirty = true;
                    document::execute_layer_action(
                        f,
                        LayerAction::DeleteSelection(paths),
                        &mut app.selection,
                    );
                    app.preview_state.push_message("Delete performed.");
                }
            }
        }
    }
}
