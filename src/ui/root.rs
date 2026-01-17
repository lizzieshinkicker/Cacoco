use crate::app::{CacocoApp, ConfirmationRequest, PendingAction, ProjectMode};
use crate::ui::font_wizard;
use crate::ui::messages::{self, EditorEvent};
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

    let mut modes_in_project = HashSet::new();
    if let Some(d) = &app.doc {
        for lump in &d.lumps {
            modes_in_project.insert(ProjectMode::from_data(lump));
        }
    }

    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::new("active_doc_exists"), app.doc.is_some());
        d.insert_temp(egui::Id::new("active_mode"), app.active_mode);
        d.insert_temp(egui::Id::new("modes_in_project"), modes_in_project);
    });

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
                    if app.doc.is_some() {
                        let has_focus = ui.ctx().memory(|mem| mem.focused().is_some());
                        let tab_pressed = ui.input(|i| i.key_pressed(egui::Key::Tab));
                        let mouse_clicked =
                            ui.input(|i| i.pointer.any_pressed()) && ui.ui_contains_pointer();

                        if mouse_clicked || (has_focus && tab_pressed) {
                            app.execute_actions(vec![document::LayerAction::UndoSnapshot]);
                        }
                    }

                    if let Some(doc) = &mut app.doc {
                        let mut active_lump = doc.get_lump(app.active_mode).cloned();

                        if ui::draw_properties_panel(
                            ui,
                            &mut active_lump,
                            &doc.selection,
                            &app.assets,
                            &app.preview_state,
                        ) {
                            if let Some(updated) = active_lump {
                                if let Some(lump_ref) = doc.get_lump_mut(app.active_mode) {
                                    *lump_ref = updated;
                                    doc.dirty = true;
                                }
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
                let mut active_lump = doc.get_lump(app.active_mode).cloned();

                let (actions, layers_changed) = ui::draw_layers_panel(
                    ui,
                    &mut active_lump,
                    &mut doc.selection,
                    &mut doc.selection_pivot,
                    &mut app.assets,
                    &mut app.current_statusbar_idx,
                    &mut app.preview_state,
                    &mut app.font_wizard,
                    &mut app.confirmation_modal,
                );

                if let Some(updated) = active_lump {
                    if let Some(lump_ref) = doc.get_lump_mut(app.active_mode) {
                        *lump_ref = updated;
                    }
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
                    &mut 0,
                    &mut app.preview_state,
                    &mut app.font_wizard,
                    &mut app.confirmation_modal,
                );
            }
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        let active_lump = app
            .doc
            .as_ref()
            .and_then(|d| d.get_lump(app.active_mode).cloned());
        let empty_selection = HashSet::new();
        let selection = app.doc.as_ref().map_or(&empty_selection, |d| &d.selection);

        let actions = ui::draw_viewport(
            ui,
            &active_lump,
            &app.assets,
            &mut app.preview_state,
            &mut app.viewport_ctrl,
            selection,
            app.current_statusbar_idx,
            &mut app.active_mode,
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
        if let Some(crate::models::ProjectData::StatusBar(sbar)) = doc.get_lump_mut(app.active_mode)
        {
            if font_wizard::draw_font_wizard(ctx, &mut app.font_wizard, sbar, &app.assets) {
                doc.dirty = true;
            }
        }
    }

    if let Some(request) = app.confirmation_modal.clone() {
        ui::modals::draw_confirmation_modal(ctx, app, &request);
    }

    if let Some(target) =
        ctx.data(|d| d.get_temp::<crate::app::CreationModal>(egui::Id::new("creation_modal_type")))
    {
        app.creation_modal = target;
        ctx.data_mut(|d| {
            d.remove::<crate::app::CreationModal>(egui::Id::new("creation_modal_type"))
        });
    }

    if app.creation_modal != crate::app::CreationModal::None {
        ui::menu::draw_creation_wizard(ctx, app);
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

    let mut tab_idx: Option<usize> = ui.data(|d| d.get_temp(tab_id).unwrap_or(None));
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

/// Dispatches keyboard shortcuts to application actions.
fn handle_action(app: &mut CacocoApp, action: crate::hotkeys::Action, ctx: &egui::Context) {
    use crate::document::LayerAction;
    use crate::hotkeys::Action;
    use crate::models::ProjectData;

    match action {
        Action::Undo => {
            if let Some(doc) = &mut app.doc {
                doc.undo();
                messages::log_event(&mut app.preview_state, EditorEvent::Undo);
            }
        }
        Action::Redo => {
            if let Some(doc) = &mut app.doc {
                doc.redo();
                messages::log_event(&mut app.preview_state, EditorEvent::Redo);
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
                if let Some(sbar) = doc.get_lump(ProjectMode::SBarDef).and_then(|l| l.as_sbar()) {
                    let needs_dialog = match &doc.path {
                        Some(p) => !Path::new(p).is_absolute(),
                        None => true,
                    };
                    if needs_dialog {
                        if let Some(p) =
                            crate::io::save_pk3_dialog(sbar, &app.assets, doc.path.clone())
                        {
                            doc.path = Some(p.clone());
                            doc.dirty = false;
                            app.add_to_recent(&p);
                            messages::log_event(
                                &mut app.preview_state,
                                EditorEvent::ProjectSaved(p),
                            );
                        }
                    } else {
                        let p = doc.path.as_ref().unwrap();
                        if crate::io::save_pk3_silent(sbar, &app.assets, p).is_ok() {
                            doc.dirty = false;
                            messages::log_event(
                                &mut app.preview_state,
                                EditorEvent::ProjectSaved(p.clone()),
                            );
                        }
                    }
                }
            }
        }
        Action::ExportJSON => {
            if let Some(doc) = &app.doc {
                if let Some(lump) = doc.get_lump(app.active_mode) {
                    let sanitized = lump.to_sanitized_json(&app.assets);
                    if let Some(p) = crate::io::save_json_dialog(&sanitized, doc.path.clone()) {
                        app.add_to_recent(&p);
                        messages::log_event(
                            &mut app.preview_state,
                            EditorEvent::ProjectExported(p),
                        );
                    }
                }
            }
        }
        Action::Copy => {
            if let Some(doc) = &mut app.doc {
                let mut new_clipboard = Vec::new();
                let mut new_bar_clipboard = Vec::new();

                if let Some(ProjectData::StatusBar(sbar)) = doc.get_lump(app.active_mode) {
                    let paths: Vec<Vec<usize>> = doc.selection.iter().cloned().collect();
                    let mut roots: Vec<Vec<usize>> = paths
                        .iter()
                        .filter(|p| !paths.iter().any(|o| p.len() > o.len() && p.starts_with(o)))
                        .cloned()
                        .collect();
                    roots.sort();

                    for path in roots {
                        if path.len() == 1 {
                            if let Some(bar) = sbar.data.status_bars.get(path[0]) {
                                new_bar_clipboard.push(bar.clone());
                            }
                        } else if let Some(el) = sbar.get_element(&path) {
                            new_clipboard.push(el.clone());
                        }
                    }
                }

                let count = new_clipboard.len() + new_bar_clipboard.len();

                doc.history.clipboard = new_clipboard;
                doc.history.bar_clipboard = new_bar_clipboard;
                messages::log_event(&mut app.preview_state, EditorEvent::ClipboardCopy(count));
            }
        }
        Action::Paste => {
            if let Some(doc) = &mut app.doc {
                if let Some(ProjectData::StatusBar(sbar)) = doc.get_lump(app.active_mode) {
                    if !doc.history.bar_clipboard.is_empty() {
                        let count = doc.history.bar_clipboard.len();
                        let pasted = doc.history.prepare_bar_clipboard_for_paste();
                        doc.execute_actions(
                            vec![
                                LayerAction::UndoSnapshot,
                                LayerAction::PasteStatusBars(pasted),
                            ],
                            app.active_mode,
                        );
                        messages::log_event(
                            &mut app.preview_state,
                            EditorEvent::ClipboardPaste(count),
                        );
                    } else if !doc.history.clipboard.is_empty() {
                        let count = doc.history.clipboard.len();
                        let pasted = doc.history.prepare_clipboard_for_paste();
                        let (p, i) = document::determine_insertion_point(
                            sbar,
                            &doc.selection,
                            app.current_statusbar_idx,
                        );
                        doc.execute_actions(
                            vec![
                                LayerAction::UndoSnapshot,
                                LayerAction::Paste {
                                    parent_path: p,
                                    insert_idx: i,
                                    elements: pasted,
                                },
                            ],
                            app.active_mode,
                        );
                        messages::log_event(
                            &mut app.preview_state,
                            EditorEvent::ClipboardPaste(count),
                        );
                    }
                }
            }
        }
        Action::Duplicate => {
            if let Some(doc) = &mut app.doc {
                if !doc.selection.is_empty() {
                    doc.history.take_snapshot(&doc.lumps, &doc.selection);
                    let paths: Vec<Vec<usize>> = doc.selection.iter().cloned().collect();
                    let mut actions = Vec::new();
                    for path in paths {
                        if path.len() == 1 {
                            actions.push(LayerAction::DuplicateStatusBar(path[0]));
                        } else {
                            actions.push(LayerAction::DuplicateSelection(vec![path]));
                        }
                    }
                    doc.execute_actions(actions, app.active_mode);
                    messages::log_event(&mut app.preview_state, EditorEvent::Duplicate);
                }
            }
        }
        Action::Delete => {
            if let Some(doc) = &mut app.doc {
                if let Some(ProjectData::StatusBar(sbar)) = doc.get_lump(app.active_mode) {
                    if !doc.selection.is_empty() {
                        let paths: Vec<Vec<usize>> = doc.selection.iter().cloned().collect();

                        if let Some(bar_path) = paths.iter().find(|p| p.len() == 1) {
                            let bar_idx = bar_path[0];
                            if !sbar.data.status_bars[bar_idx].children.is_empty() {
                                app.confirmation_modal =
                                    Some(ConfirmationRequest::DeleteStatusBar(bar_idx));
                                return;
                            } else if sbar.data.status_bars.len() > 1 {
                                doc.execute_actions(
                                    vec![LayerAction::DeleteStatusBar(bar_idx)],
                                    app.active_mode,
                                );
                                messages::log_event(&mut app.preview_state, EditorEvent::Delete);
                                return;
                            }
                        }

                        let mut needs_conf = false;
                        for path in &paths {
                            if let Some(el) = sbar.get_element(path) {
                                if !el.children().is_empty() {
                                    needs_conf = true;
                                    break;
                                }
                            }
                        }

                        if needs_conf {
                            app.confirmation_modal = Some(ConfirmationRequest::DeleteLayers(paths));
                        } else {
                            doc.execute_actions(
                                vec![
                                    LayerAction::UndoSnapshot,
                                    LayerAction::DeleteSelection(paths),
                                ],
                                app.active_mode,
                            );
                            messages::log_event(&mut app.preview_state, EditorEvent::Delete);
                        }
                    }
                }
            }
        }
        Action::Deselect => {
            if let Some(doc) = &mut app.doc {
                doc.selection.clear();
                doc.selection_pivot = None;
            }
        }
    }
}

/// Dispatches menu actions to application logic.
fn handle_menu_action(app: &mut CacocoApp, action: ui::MenuAction, ctx: &egui::Context) {
    match action {
        ui::MenuAction::NewProject => {
            app.doc = None;
            app.creation_modal = crate::app::CreationModal::LumpSelector;
        }
        ui::MenuAction::LoadProject(loaded, path) => app.load_project(ctx, loaded, &path),
        ui::MenuAction::Open => app.open_project_ui(ctx),
        ui::MenuAction::RequestDiscard(pending) => {
            app.confirmation_modal = Some(ConfirmationRequest::DiscardChanges(pending));
        }
        ui::MenuAction::SetTarget(t) => {
            if let Some(doc) = &mut app.doc {
                use crate::models::sbardef::ExportTarget::*;

                let current_is_extended = doc
                    .get_lump(app.active_mode)
                    .map_or(false, |l| l.target() == Extended);

                if current_is_extended && t == Basic {
                    if let Some(lump) = doc.get_lump(app.active_mode) {
                        if lump.determine_target() == Extended {
                            app.confirmation_modal = Some(ConfirmationRequest::DowngradeTarget(t));
                            return;
                        }
                    }
                }

                doc.execute_actions(vec![document::LayerAction::UndoSnapshot], app.active_mode);
                if let Some(l) = doc.get_lump_mut(app.active_mode) {
                    l.set_target(t);
                    l.normalize_for_target();
                }
                doc.dirty = true;
            }
        }
        ui::MenuAction::SaveDone(path) => {
            if path == "SILENT" {
                handle_action(app, crate::hotkeys::Action::Save, ctx);
            } else if let Some(doc) = &mut app.doc {
                doc.path = Some(path.clone());
                doc.dirty = false;
                app.add_to_recent(&path);
                messages::log_event(&mut app.preview_state, EditorEvent::ProjectSaved(path));
            }
        }
        ui::MenuAction::ExportDone(path) => {
            app.add_to_recent(&path);
            messages::log_event(&mut app.preview_state, EditorEvent::ProjectExported(path));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{CacocoApp, ConfirmationRequest, PendingAction, ProjectMode};
    use crate::hotkeys::Action;
    use crate::models::ProjectData;
    use crate::models::sbardef::{
        CanvasDef, Element, ElementWrapper, ExportTarget, ListDef, SBarDefFile, StatusBarLayout,
    };
    use crate::ui::MenuAction;

    #[test]
    fn test_downgrade_trigger_modal() {
        let ctx = egui::Context::default();
        let mut app = CacocoApp::default();

        let list_element = ElementWrapper {
            data: Element::List(ListDef::default()),
            ..Default::default()
        };
        let mut sbar = SBarDefFile::new_empty();
        sbar.target = ExportTarget::Extended;
        sbar.data.status_bars[0].children.push(list_element);

        app.doc = Some(document::ProjectDocument::new(
            ProjectData::StatusBar(sbar),
            None,
        ));
        app.active_mode = ProjectMode::SBarDef;

        handle_menu_action(&mut app, MenuAction::SetTarget(ExportTarget::Basic), &ctx);

        assert!(app.confirmation_modal.is_some());
        if let Some(ConfirmationRequest::DowngradeTarget(target)) = app.confirmation_modal {
            assert_eq!(target, ExportTarget::Basic);
        } else {
            panic!("Expected DowngradeTarget modal, but found a different type!");
        }
    }

    #[test]
    fn test_discard_changes_on_open() {
        let ctx = egui::Context::default();
        let mut app = CacocoApp::default();

        app.doc = Some(document::ProjectDocument::new(
            ProjectData::StatusBar(SBarDefFile::new_empty()),
            None,
        ));
        app.doc.as_mut().unwrap().dirty = true;

        handle_action(&mut app, Action::Open, &ctx);

        assert!(app.confirmation_modal.is_some());
        if let Some(ConfirmationRequest::DiscardChanges(PendingAction::Load(path))) =
            &app.confirmation_modal
        {
            assert!(path.is_empty());
        } else {
            panic!("Expected DiscardChanges(Load) modal!");
        }
    }

    #[test]
    fn test_delete_status_bar_confirmation_and_execution() {
        let ctx = egui::Context::default();
        let mut app = CacocoApp::default();

        let mut sbar = SBarDefFile::new_empty();
        sbar.data.status_bars.push(StatusBarLayout::default());
        sbar.data.status_bars[0]
            .children
            .push(ElementWrapper::default());

        app.doc = Some(document::ProjectDocument::new(
            ProjectData::StatusBar(sbar),
            None,
        ));
        app.active_mode = ProjectMode::SBarDef;

        app.doc.as_mut().unwrap().selection.insert(vec![0]);

        handle_action(&mut app, Action::Delete, &ctx);

        let idx_to_delete = match app.confirmation_modal {
            Some(ConfirmationRequest::DeleteStatusBar(idx)) => idx,
            _ => panic!(
                "Expected DeleteStatusBar modal! The current code is likely sending DeleteLayers instead."
            ),
        };

        app.execute_actions(vec![
            document::LayerAction::UndoSnapshot,
            document::LayerAction::DeleteStatusBar(idx_to_delete),
        ]);

        let final_count = app.doc.as_ref().unwrap().lumps[0]
            .as_sbar()
            .unwrap()
            .data
            .status_bars
            .len();

        assert_eq!(
            final_count, 1,
            "The layout was not actually removed from the document after confirmation!"
        );
    }

    #[test]
    fn test_delete_layer_with_children_confirmation() {
        let ctx = egui::Context::default();
        let mut app = CacocoApp::default();

        let mut parent = ElementWrapper {
            data: Element::Canvas(CanvasDef::default()),
            ..Default::default()
        };
        parent
            .get_common_mut()
            .children
            .push(ElementWrapper::default());

        let mut sbar = SBarDefFile::new_empty();
        sbar.data.status_bars[0].children.push(parent);

        app.doc = Some(document::ProjectDocument::new(
            ProjectData::StatusBar(sbar),
            None,
        ));
        app.active_mode = ProjectMode::SBarDef;

        app.doc.as_mut().unwrap().selection.insert(vec![0, 0]);

        handle_action(&mut app, Action::Delete, &ctx);

        assert!(app.confirmation_modal.is_some());
        if let Some(ConfirmationRequest::DeleteLayers(paths)) = &app.confirmation_modal {
            assert_eq!(paths[0], vec![0, 0]);
        } else {
            panic!("Expected DeleteLayers modal for a non-empty layer group!");
        }
    }

    #[test]
    fn test_modes_in_project_broadcasting() {
        let ctx = egui::Context::default();
        let mut app = CacocoApp::default();

        let mut doc =
            document::ProjectDocument::new(ProjectData::StatusBar(SBarDefFile::new_empty()), None);
        doc.lumps.push(ProjectData::Sky(
            crate::models::skydefs::SkyDefsFile::new_empty(),
        ));
        app.doc = Some(doc);

        app.iwad_verified = true;

        let input = egui::RawInput::default();
        let _ = ctx.run(input, |ctx| {
            draw_root_ui(ctx, &mut app);
        });

        let modes: HashSet<ProjectMode> = ctx.data(|d| {
            d.get_temp(egui::Id::new("modes_in_project"))
                .unwrap_or_default()
        });

        assert!(
            modes.contains(&ProjectMode::SBarDef),
            "SBARDEF was not registered in the project modes!"
        );
        assert!(
            modes.contains(&ProjectMode::SkyDefs),
            "SKYDEFS was not registered in the project modes!"
        );
    }

    #[test]
    fn test_add_lump_preserves_existing_data() {
        let ctx = egui::Context::default();
        let mut app = CacocoApp::default();

        app.new_project(&ctx, ProjectData::StatusBar(SBarDefFile::new_empty()));

        let sky_lump = ProjectData::Sky(crate::models::skydefs::SkyDefsFile::new_empty());
        app.add_lump_to_project(sky_lump);

        let doc = app.doc.as_ref().unwrap();
        assert_eq!(
            doc.lumps.len(),
            2,
            "Adding a lump should increase the count, not replace the project!"
        );
        assert!(doc.get_lump(ProjectMode::SBarDef).is_some());
        assert!(doc.get_lump(ProjectMode::SkyDefs).is_some());
    }
}
