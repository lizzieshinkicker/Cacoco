use crate::app::{CacocoApp, ConfirmationRequest, PendingAction};
use crate::ui::font_wizard;
use crate::{document, ui};
use eframe::egui;
use std::collections::HashSet;
use std::path::Path;

/// The main application UI composition function.
///
/// This orchestrates the side panels, the central viewport, and modal windows.
pub fn draw_root_ui(ctx: &egui::Context, app: &mut CacocoApp) {
    if !app.iwad_verified {
        ui::modals::draw_onboarding_screen(ctx, app);
        return;
    }

    if ctx.input(|i| i.viewport().close_requested()) {
        let is_dirty = app.doc.as_ref().map_or(false, |d| d.dirty);
        if is_dirty {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            app.confirmation_modal = Some(ConfirmationRequest::DiscardChanges(PendingAction::Quit));
        }
    }

    update_window_title(ctx, app);

    if let Some(action) = app.hotkeys.check(ctx) {
        handle_action(app, action, ctx);
    }
    app.cheat_engine.process_input(ctx, &mut app.preview_state);
    app.preview_state.update(ctx.input(|i| i.stable_dt));

    egui::SidePanel::left("left_side_panel")
        .resizable(false)
        .exact_width(289.0)
        .show(ctx, |ui| {
            ui.add_space(5.0);

            let menu_action = ui::draw_menu_bar(
                ui,
                ctx,
                &mut app.doc,
                &mut app.config,
                &mut app.assets,
                &mut app.settings_open,
            );
            handle_menu_action(app, menu_action, ctx);

            ui.add_space(2.0);
            ui.separator();

            draw_left_sidebar_drawer(ui, app);

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show_inside(ui, |ui| {
                    if let Some(doc) = &mut app.doc {
                        let mut file_clone = Some(doc.file.clone());
                        if ui::draw_properties_panel(
                            ui,
                            &mut file_clone,
                            &doc.selection,
                            &app.assets,
                            &app.preview_state,
                        ) {
                            if let Some(updated) = file_clone {
                                doc.file = updated;
                                doc.dirty = true;
                            }
                        }
                    } else {
                        ui::draw_properties_panel(
                            ui,
                            &mut None,
                            &Default::default(),
                            &app.assets,
                            &app.preview_state,
                        );
                    }
                });
        });

    egui::SidePanel::right("right_side_panel")
        .resizable(false)
        .exact_width(320.0)
        .show(ctx, |ui| {
            if let Some(doc) = &mut app.doc {
                let mut file_clone = Some(doc.file.clone());
                let (actions, layers_changed) = ui::draw_layers_panel(
                    ui,
                    &mut file_clone,
                    &mut doc.selection,
                    &mut doc.selection_pivot,
                    &mut app.assets,
                    &mut app.current_statusbar_idx,
                    &mut app.preview_state,
                    &mut app.font_wizard,
                    &mut app.confirmation_modal,
                );

                if let Some(updated) = file_clone {
                    doc.file = updated;
                }
                if layers_changed {
                    doc.dirty = true;
                }
                app.execute_actions(actions);
            } else {
                ui::draw_layers_panel(
                    ui,
                    &mut None,
                    &mut Default::default(),
                    &mut None,
                    &mut app.assets,
                    &mut app.current_statusbar_idx,
                    &mut app.preview_state,
                    &mut app.font_wizard,
                    &mut app.confirmation_modal,
                );
            }
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        let file_opt = app.doc.as_ref().map(|d| d.file.clone());
        let empty_selection = HashSet::new();
        let selection = app.doc.as_ref().map_or(&empty_selection, |d| &d.selection);

        let actions = ui::draw_viewport(
            ui,
            &file_opt,
            &app.assets,
            &mut app.preview_state,
            &mut app.viewport_ctrl,
            selection,
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

    if let Some(doc) = &mut app.doc {
        if font_wizard::draw_font_wizard(ctx, &mut app.font_wizard, &mut doc.file, &app.assets) {
            doc.dirty = true;
        }
    }

    if let Some(request) = app.confirmation_modal.clone() {
        ui::modals::draw_confirmation_modal(ctx, app, &request);
    }
}

/// Helper to update the OS window title based on current document state.
fn update_window_title(ctx: &egui::Context, app: &CacocoApp) {
    let (is_dirty, path_opt) = app
        .doc
        .as_ref()
        .map_or((false, None), |d| (d.dirty, d.path.clone()));

    let indicator = if is_dirty { "*" } else { "" };
    let display = if app.doc.is_some() {
        if let Some(p) = path_opt {
            let name = Path::new(&p)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&p);
            format!("[{}{}] ", indicator, name)
        } else {
            format!("[{}New Project] ", indicator)
        }
    } else {
        "".to_string()
    };

    let flavor = ctx.data(|d| d.get_temp::<String>(egui::Id::new("random_title")));
    if let Some(text) = flavor {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "{display}Cacoco - {text}"
        )));
    }
}

/// Renders the simulation drawer (Held Items / Context) in the left panel.
fn draw_left_sidebar_drawer(ui: &mut egui::Ui, app: &mut CacocoApp) {
    let tab_id = ui.make_persistent_id("sidebar_tab_idx");
    let last_tab_id = ui.make_persistent_id("sidebar_last_tab_idx");
    let heights_id = ui.make_persistent_id("sidebar_tab_heights");

    let mut tab_idx: Option<usize> = ui.data(|d| d.get_temp(tab_id).unwrap_or(Some(0)));
    let mut last_tab: usize = ui.data(|d| d.get_temp(last_tab_id).unwrap_or(0));
    let mut heights: [f32; 2] = ui.data(|d| d.get_temp(heights_id).unwrap_or([428.0, 428.0]));

    if let Some(current) = tab_idx {
        last_tab = current;
    }

    let header_h = 38.0;
    let target_h = if let Some(idx) = tab_idx {
        header_h + heights[idx]
    } else {
        header_h
    };

    let anim_h = ui.ctx().animate_value_with_time(
        ui.make_persistent_id("sidebar_drawer_anim"),
        target_h,
        0.1,
    );

    egui::TopBottomPanel::bottom("left_sidebar_footer")
        .frame(egui::Frame::NONE)
        .resizable(false)
        .exact_height(anim_h)
        .show_inside(ui, |ui| {
            ui.add_space(7.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let btn_w = (ui.available_width() - 4.0) / 2.0;

                let items_res = ui.add_sized([btn_w, 28.0], |ui: &mut egui::Ui| {
                    ui::shared::section_header_button(ui, "Held Items", None, tab_idx == Some(0))
                });
                if items_res.clicked() {
                    tab_idx = if tab_idx == Some(0) { None } else { Some(0) };
                }

                let ctx_res = ui.add_sized([btn_w, 28.0], |ui: &mut egui::Ui| {
                    ui::shared::section_header_button(ui, "Game Context", None, tab_idx == Some(1))
                });
                if ctx_res.clicked() {
                    tab_idx = if tab_idx == Some(1) { None } else { Some(1) };
                }
            });

            ui.separator();
            if anim_h > header_h + 1.0 {
                let rect = ui.available_rect_before_wrap();
                ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                    let inner_response = ui.vertical(|ui| {
                        ui.add_space(3.0);
                        if last_tab == 0 {
                            ui::draw_gamestate_panel(ui, &mut app.preview_state, &app.assets);
                        } else {
                            ui::draw_context_panel(ui, &mut app.preview_state, &app.assets);
                        }
                        ui.add_space(10.0);
                    });

                    if tab_idx.is_some() || anim_h > header_h + 10.0 {
                        let h = inner_response.response.rect.height();
                        if h > 10.0 {
                            heights[last_tab] = h;
                        }
                    }
                });
            }
        });

    ui.data_mut(|d| {
        d.insert_temp(tab_id, tab_idx);
        d.insert_temp(last_tab_id, last_tab);
        d.insert_temp(heights_id, heights);
    });
}

/// Helper to prompt for a source port and launch the current project.
fn handle_pick_port_and_run(app: &mut CacocoApp) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Executable", &["exe"])
        .set_title("Select Source Port")
        .pick_file()
    {
        let path_str = path.to_string_lossy().into_owned();
        if !app.config.source_ports.contains(&path_str) {
            app.config.source_ports.push(path_str.clone());
            app.config.save();
        }
        if let Some(doc) = &app.doc {
            if let Some(iwad) = &app.config.base_wad_path {
                crate::io::launch_game(&doc.file, &app.assets, &path_str, iwad);
            }
        }
    }
}

/// Dispatches keyboard shortcuts to application actions.
fn handle_action(app: &mut CacocoApp, action: crate::hotkeys::Action, ctx: &egui::Context) {
    use crate::document::LayerAction;
    use crate::hotkeys::Action;

    match action {
        Action::Undo => {
            if let Some(doc) = &mut app.doc {
                doc.undo();
            }
        }
        Action::Redo => {
            if let Some(doc) = &mut app.doc {
                doc.redo();
            }
        }
        Action::Open => {
            let is_dirty = app.doc.as_ref().map_or(false, |d| d.dirty);
            if is_dirty {
                app.confirmation_modal = Some(ConfirmationRequest::DiscardChanges(
                    PendingAction::Load("".to_string()),
                ));
            } else {
                app.open_project_ui(ctx);
            }
        }
        Action::Save => {
            if let Some(doc) = &mut app.doc {
                let needs_dialog = match &doc.path {
                    Some(p) => !Path::new(p).is_absolute(),
                    None => true,
                };
                if needs_dialog {
                    if let Some(p) =
                        crate::io::save_pk3_dialog(&doc.file, &app.assets, doc.path.clone())
                    {
                        doc.path = Some(p.clone());
                        doc.dirty = false;
                        app.add_to_recent(&p);
                    }
                } else {
                    let p = doc.path.as_ref().unwrap();
                    if crate::io::save_pk3_silent(&doc.file, &app.assets, p).is_ok() {
                        doc.dirty = false;
                    }
                }
            }
        }
        Action::ExportJSON => {
            if let Some(doc) = &app.doc {
                if let Some(p) = crate::io::save_json_dialog(&doc.file, doc.path.clone()) {
                    app.add_to_recent(&p);
                }
            }
        }
        Action::Copy => {
            if let Some(doc) = &mut app.doc {
                doc.history.clipboard.clear();
                doc.history.bar_clipboard.clear();
                let paths: Vec<Vec<usize>> = doc.selection.iter().cloned().collect();
                let mut roots: Vec<Vec<usize>> = paths
                    .iter()
                    .filter(|p| !paths.iter().any(|o| p.len() > o.len() && p.starts_with(o)))
                    .cloned()
                    .collect();
                roots.sort();

                for path in roots {
                    if path.len() == 1 {
                        if let Some(bar) = doc.file.data.status_bars.get(path[0]) {
                            doc.history.bar_clipboard.push(bar.clone());
                        }
                    } else if let Some(el) = doc.file.get_element_mut(&path) {
                        doc.history.clipboard.push(el.clone());
                    }
                }
            }
        }
        Action::Paste => {
            if let Some(doc) = &mut app.doc {
                if !doc.history.bar_clipboard.is_empty() {
                    let pasted = doc.history.prepare_bar_clipboard_for_paste();
                    doc.execute_actions(vec![LayerAction::PasteStatusBars(pasted)]);
                } else if !doc.history.clipboard.is_empty() {
                    let pasted = doc.history.prepare_clipboard_for_paste();
                    let (p, i) = document::determine_insertion_point(
                        &doc.selection,
                        app.current_statusbar_idx,
                    );
                    doc.execute_actions(vec![LayerAction::Paste {
                        parent_path: p,
                        insert_idx: i,
                        elements: pasted,
                    }]);
                }
            }
        }
        Action::Duplicate => {
            if let Some(doc) = &mut app.doc {
                if !doc.selection.is_empty() {
                    doc.history.take_snapshot(&doc.file, &doc.selection);
                    let paths: Vec<Vec<usize>> = doc.selection.iter().cloned().collect();
                    let mut bars = Vec::new();
                    let mut layers = Vec::new();
                    for path in paths {
                        if path.len() == 1 {
                            bars.push(LayerAction::DuplicateStatusBar(path[0]));
                        } else {
                            layers.push(path);
                        }
                    }
                    if !bars.is_empty() {
                        doc.execute_actions(bars);
                    }
                    if !layers.is_empty() {
                        doc.execute_actions(vec![LayerAction::DuplicateSelection(layers)]);
                    }
                }
            }
        }
        Action::Delete => {
            if let Some(doc) = &mut app.doc {
                if !doc.selection.is_empty() {
                    let paths: Vec<Vec<usize>> = doc.selection.iter().cloned().collect();
                    let mut bar_del = None;
                    let mut needs_conf = false;
                    for path in &paths {
                        if path.len() == 1 {
                            bar_del = Some(path[0]);
                        } else if let Some(el) = doc.file.get_element(path) {
                            if !el.children().is_empty() {
                                needs_conf = true;
                            }
                        }
                    }
                    if let Some(idx) = bar_del {
                        if !doc.file.data.status_bars[idx].children.is_empty() {
                            app.confirmation_modal =
                                Some(ConfirmationRequest::DeleteStatusBar(idx));
                        } else if doc.file.data.status_bars.len() > 1 {
                            doc.execute_actions(vec![LayerAction::DeleteStatusBar(idx)]);
                        }
                    } else if needs_conf {
                        app.confirmation_modal = Some(ConfirmationRequest::DeleteLayers(paths));
                    } else {
                        doc.execute_actions(vec![LayerAction::DeleteSelection(paths)]);
                    }
                }
            }
        }
    }
}

/// Dispatches menu actions to application logic.
fn handle_menu_action(app: &mut CacocoApp, action: ui::MenuAction, ctx: &egui::Context) {
    match action {
        ui::MenuAction::LoadProject(loaded, path) => app.load_project(ctx, loaded, &path),
        ui::MenuAction::LoadTemplate(template) => app.apply_template(ctx, template),
        ui::MenuAction::NewEmpty => app.new_project(ctx),
        ui::MenuAction::Open => app.open_project_ui(ctx),
        ui::MenuAction::RequestDiscard(pending) => {
            app.confirmation_modal = Some(ConfirmationRequest::DiscardChanges(pending));
        }
        ui::MenuAction::SaveDone(path) => {
            if path == "SILENT" {
                handle_action(app, crate::hotkeys::Action::Save, ctx);
            } else if let Some(doc) = &mut app.doc {
                doc.path = Some(path.clone());
                doc.dirty = false;
                app.add_to_recent(&path);
            }
        }
        ui::MenuAction::ExportDone(path) => {
            app.add_to_recent(&path);
        }
        ui::MenuAction::PickPortAndRun => {
            handle_pick_port_and_run(app);
        }
        _ => {}
    }
}
