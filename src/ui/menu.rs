use crate::app::PendingAction;
use crate::assets::AssetStore;
use crate::config::{AppConfig, SourcePortConfig};
use crate::document::SBarDocument;
use crate::io::{self, LoadedProject};
use crate::library;
use crate::ui::context_menu::ContextMenu;
use crate::ui::shared;
use eframe::egui;

/// Result of a menu interaction that requires application-level handling.
pub enum MenuAction {
    None,
    LoadProject(LoadedProject, String),
    LoadTemplate(&'static library::Template),
    NewEmpty,
    Open,
    RequestDiscard(PendingAction),
    SaveDone(String),
    ExportDone(String),
    SetTarget(crate::model::ExportTarget),
}

/// Draws the primary application menu bar (File, Run, Target).
pub fn draw_menu_bar(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    doc: &mut Option<SBarDocument>,
    config: &mut AppConfig,
    assets: &mut AssetStore,
    settings_open: &mut bool,
) -> MenuAction {
    let mut action = MenuAction::None;

    let template_modal_id = ui.make_persistent_id("template_selector_open");
    let mut template_open = ui.data(|d| d.get_temp::<bool>(template_modal_id).unwrap_or(false));

    let file_id = ui.make_persistent_id("file_menu_area");
    let run_id = ui.make_persistent_id("run_menu_area");
    let target_id = ui.make_persistent_id("target_menu_area");

    let mut open_file = false;
    let mut open_run = false;
    let mut open_target = false;

    let dirty = doc.as_ref().map_or(false, |d| d.dirty);

    let current_target = doc
        .as_ref()
        .map_or(crate::model::ExportTarget::Extended, |d| d.file.target);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let btn_w = (ui.available_width() - 8.0) / 3.0;

        let file_res = ui.add_sized([btn_w, 28.0], |ui: &mut egui::Ui| {
            shared::section_header_button(ui, "File", None, ContextMenu::get(ui, file_id).is_some())
        });
        if file_res.clicked() {
            open_file = true;
            trigger_menu(ui, file_id, file_res.rect.left_bottom());
        }

        let run_res = ui.add_sized([btn_w, 28.0], |ui: &mut egui::Ui| {
            shared::section_header_button(ui, "Run", None, ContextMenu::get(ui, run_id).is_some())
        });
        if run_res.clicked() {
            open_run = true;
            trigger_menu(ui, run_id, run_res.rect.left_bottom());
        }

        let target_label = match current_target {
            crate::model::ExportTarget::Basic => "Basic",
            crate::model::ExportTarget::Extended => "Extended",
        };
        let target_res = ui.add_sized([btn_w, 28.0], |ui: &mut egui::Ui| {
            shared::section_header_button(
                ui,
                target_label,
                None,
                ContextMenu::get(ui, target_id).is_some(),
            )
        });
        if target_res.clicked() {
            open_target = true;
            trigger_menu(ui, target_id, target_res.rect.left_bottom());
        }
    });

    if let Some(menu) = ContextMenu::get(ui, file_id) {
        ContextMenu::show(ui, menu, open_file, |ui| {
            if ContextMenu::button(ui, "New Project...", true) {
                if dirty {
                    action = MenuAction::RequestDiscard(PendingAction::New);
                } else {
                    template_open = true;
                }
                ContextMenu::close(ui);
            }
            if ContextMenu::button(ui, "Open Project...", true) {
                if dirty {
                    action = MenuAction::RequestDiscard(PendingAction::Load("".to_string()));
                } else {
                    action = MenuAction::Open;
                }
                ContextMenu::close(ui);
            }

            if !config.recent_files.is_empty() {
                ui.separator();
                ui.label(egui::RichText::new("  Recent Files").weak().size(10.0));
                let mut file_to_load = None;
                for path in config.recent_files.iter() {
                    if ContextMenu::button(ui, &shared::truncate_path(path, 30), true) {
                        file_to_load = Some(path.clone());
                    }
                }
                if let Some(path) = file_to_load {
                    if dirty {
                        action = MenuAction::RequestDiscard(PendingAction::Load(path));
                    } else if let Some(loaded) = io::load_project_from_path(ctx, &path) {
                        action = MenuAction::LoadProject(loaded, path);
                    }
                    ContextMenu::close(ui);
                }
            }

            ui.separator();
            if ContextMenu::button(ui, "Save", doc.is_some()) {
                action = MenuAction::SaveDone("SILENT".to_string());
                ContextMenu::close(ui);
            }
            if ContextMenu::button(ui, "Save As...", doc.is_some()) {
                if let Some(d) = doc {
                    if let Some(path) = io::save_pk3_dialog(&d.file, assets, d.path.clone()) {
                        action = MenuAction::SaveDone(path);
                    }
                }
                ContextMenu::close(ui);
            }
            if ContextMenu::button(ui, "Export JSON...", doc.is_some()) {
                if let Some(d) = doc {
                    let sanitized = d.file.to_sanitized_json(assets);
                    if let Some(path) = io::save_json_dialog(&sanitized, d.path.clone()) {
                        action = MenuAction::ExportDone(path);
                    }
                }
                ContextMenu::close(ui);
            }
            if ContextMenu::button(ui, "Export WAD...", doc.is_some()) {
                if let Some(d) = doc {
                    let sanitized = d.file.to_sanitized_json(assets);
                    if let Some(path) = io::save_wad_dialog(&sanitized, assets, d.path.clone()) {
                        action = MenuAction::ExportDone(path);
                    }
                }
                ContextMenu::close(ui);
            }

            ui.separator();
            if ContextMenu::button(ui, "Settings...", true) {
                *settings_open = true;
                ContextMenu::close(ui);
            }
            ui.separator();
            if ContextMenu::button(ui, "Quit", true) {
                if dirty {
                    action = MenuAction::RequestDiscard(PendingAction::Quit);
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                ContextMenu::close(ui);
            }
        });
    }

    if let Some(menu) = ContextMenu::get(ui, run_id) {
        ContextMenu::show(ui, menu, open_run, |ui| {
            let has_file = doc.is_some();
            if config.source_ports.is_empty() {
                if ContextMenu::button(ui, "Add New Port...", true) {
                    *settings_open = true;
                    ContextMenu::close(ui);
                }
            } else {
                for port in &config.source_ports {
                    if ContextMenu::button(ui, &format!("Launch in {}", port.name), has_file) {
                        if let (Some(d), Some(iwad)) = (doc.as_ref(), config.base_wad_path.as_ref())
                        {
                            let sanitized = d.file.to_sanitized_json(assets);
                            io::launch_game(&sanitized, assets, &port.command, iwad, d.file.target);
                        }
                        ContextMenu::close(ui);
                    }
                }
                ui.separator();
                if ContextMenu::button(ui, "Add New Port...", true) {
                    *settings_open = true;
                    ContextMenu::close(ui);
                }
            }
        });
    }

    if let Some(menu) = ContextMenu::get(ui, target_id) {
        ContextMenu::show(ui, menu, open_target, |ui| {
            if ContextMenu::button(
                ui,
                "Basic (KEX / 1.0.0)",
                current_target != crate::model::ExportTarget::Basic,
            ) {
                action = MenuAction::SetTarget(crate::model::ExportTarget::Basic);
                ContextMenu::close(ui);
            }
            if ContextMenu::button(
                ui,
                "Extended (1.2.0+)",
                current_target != crate::model::ExportTarget::Extended,
            ) {
                action = MenuAction::SetTarget(crate::model::ExportTarget::Extended);
                ContextMenu::close(ui);
            }
            ui.separator();
            ui.label(
                egui::RichText::new("Basic targets will sanitize \nSBARDEF and export as .WAD")
                    .weak()
                    .size(10.0),
            );
        });
    }

    if template_open {
        let mut close_window = false;
        egui::Window::new("Create New Project")
            .open(&mut template_open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_width(480.0);
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;

                        if draw_menu_card(ui, "Empty", "Start from scratch.") {
                            if dirty {
                                action = MenuAction::RequestDiscard(PendingAction::New);
                            } else {
                                action = MenuAction::NewEmpty;
                            }
                            close_window = true;
                        }

                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Templates").weak().italics().size(11.0));

                        for template in library::TEMPLATES {
                            if draw_menu_card(ui, template.name, template.description) {
                                if dirty {
                                    action = MenuAction::RequestDiscard(PendingAction::Template(
                                        template,
                                    ));
                                } else {
                                    action = MenuAction::LoadTemplate(template);
                                }
                                close_window = true;
                            }
                        }
                    });

                ui.add_space(12.0);
                ui.separator();

                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    if ui.button("  Cancel  ").clicked() {
                        close_window = true;
                    }
                    ui.add_space(8.0);
                });
            });

        if close_window {
            template_open = false;
        }
    }

    ui.data_mut(|d| d.insert_temp(template_modal_id, template_open));
    action
}

pub fn draw_settings_window(
    ctx: &egui::Context,
    settings_open: &mut bool,
    config: &mut AppConfig,
    assets: &mut AssetStore,
) {
    let mut is_open = *settings_open;
    egui::Window::new("Settings")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(550.0);
            ui.add_space(4.0);

            ui.heading("Global Paths");
            let iwad_desc = config
                .base_wad_path
                .as_deref()
                .unwrap_or("Click to browse for DOOM2.WAD...");
            if draw_menu_card(ui, "Base IWAD", iwad_desc) {
                if let Some(new_path) = io::load_iwad_dialog(ctx, assets) {
                    config.base_wad_path = Some(new_path);
                }
            }

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.heading("Source Ports");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("+ Add Command").clicked() {
                        config.source_ports.push(SourcePortConfig {
                            name: "New Port".to_string(),
                            command: "flatpak run ...".to_string(),
                        });
                    }

                    if ui.button("+ Add Port").clicked() {
                        let mut dialog = rfd::FileDialog::new().set_title("Select Source Port");
                        if cfg!(windows) {
                            dialog = dialog.add_filter("Executable", &["exe"]);
                        }

                        if let Some(path) = dialog.pick_file() {
                            let path_str = path.to_string_lossy().into_owned();
                            config.source_ports.push(SourcePortConfig {
                                name: SourcePortConfig::infer_name(&path_str),
                                command: path_str,
                            });
                        }
                    }
                });
            });
            ui.separator();
            ui.add_space(4.0);

            let mut to_remove = None;
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;

                    for (idx, port) in config.source_ports.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            let total_w = ui.available_width();
                            let delete_w = 44.0;
                            let card_w = total_w - delete_w - 4.0;

                            ui.allocate_ui(egui::vec2(card_w, 70.0), |ui| {
                                let frame = egui::Frame::NONE
                                    .inner_margin(8.0)
                                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                                    .corner_radius(4.0);

                                frame.show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Name:");
                                            ui.add(
                                                egui::TextEdit::singleline(&mut port.name)
                                                    .desired_width(f32::INFINITY)
                                                    .font(egui::FontId::proportional(14.0))
                                                    .text_color(ui.visuals().strong_text_color()),
                                            );
                                        });
                                        ui.add_space(4.0);
                                        ui.horizontal(|ui| {
                                            ui.label("Cmd: ");
                                            ui.add(
                                                egui::TextEdit::singleline(&mut port.command)
                                                    .hint_text("Executable path or flatpak command")
                                                    .desired_width(f32::INFINITY)
                                                    .font(egui::FontId::monospace(11.0)),
                                            );
                                        });
                                    });
                                });
                            });

                            if draw_delete_card(ui, delete_w) {
                                to_remove = Some(idx);
                            }
                        });
                    }
                });

            if let Some(idx) = to_remove {
                config.source_ports.remove(idx);
            }

            ui.add_space(24.0);
            ui.separator();

            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                if ui.button("   Save & Close   ").clicked() {
                    config.save();
                    *settings_open = false;
                }
                ui.add_space(8.0);
            });
        });

    if !is_open {
        *settings_open = false;
    }
}

pub fn draw_menu_card(ui: &mut egui::Ui, title: &str, desc: &str) -> bool {
    let width = ui.available_width();
    let height = 54.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    let visuals = ui.style().interact(&response);
    let bg_color = if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().widgets.noninteractive.bg_fill
    };

    let stroke = if response.hovered() {
        ui.visuals().widgets.hovered.bg_stroke
    } else {
        ui.visuals().widgets.noninteractive.bg_stroke
    };

    ui.painter()
        .rect(rect, 4.0, bg_color, stroke, egui::StrokeKind::Outside);

    let text_pos = rect.left_top() + egui::vec2(12.0, 10.0);
    ui.painter().text(
        text_pos,
        egui::Align2::LEFT_TOP,
        title,
        egui::FontId::proportional(16.0),
        visuals.text_color(),
    );

    ui.painter().text(
        text_pos + egui::vec2(0.0, 20.0),
        egui::Align2::LEFT_TOP,
        desc,
        egui::FontId::proportional(11.0),
        ui.visuals().weak_text_color(),
    );

    response.clicked()
}

pub fn draw_delete_card(ui: &mut egui::Ui, width: f32) -> bool {
    let height = 70.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    let (bg_color, stroke_color) = if response.hovered() {
        (
            egui::Color32::from_rgb(110, 40, 40),
            ui.visuals().widgets.hovered.bg_stroke.color,
        )
    } else {
        (
            ui.visuals().widgets.noninteractive.bg_fill,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        )
    };

    ui.painter().rect(
        rect,
        4.0,
        bg_color,
        egui::Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Outside,
    );

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "🗑",
        egui::FontId::proportional(18.0),
        if response.hovered() {
            egui::Color32::WHITE
        } else {
            ui.visuals().weak_text_color()
        },
    );

    response.clicked()
}

fn trigger_menu(ui: &mut egui::Ui, id: egui::Id, pos: egui::Pos2) {
    ui.data_mut(|d| {
        d.insert_temp(egui::Id::new("cacoco_context_menu_id"), id);
        d.insert_temp(egui::Id::new("cacoco_context_menu_pos"), pos);
    });
}
