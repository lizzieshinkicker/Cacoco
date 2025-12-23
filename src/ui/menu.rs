use crate::assets::AssetStore;
use crate::config::AppConfig;
use crate::io::{self, LoadedProject};
use crate::library;
use crate::model::SBarDefFile;
use eframe::egui;

pub enum MenuAction {
    None,
    LoadProject(LoadedProject, String),
    LoadTemplate(&'static library::Template),
    NewEmpty,
    SaveDone(String),
    ExportDone(String),
    PickPortAndRun,
}

pub fn draw_menu_bar(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    current_file: &mut Option<SBarDefFile>,
    opened_file_path: Option<String>,
    config: &mut AppConfig,
    assets: &mut AssetStore,
    settings_open: &mut bool,
) -> MenuAction {
    let mut action = MenuAction::None;

    let template_modal_id = ui.make_persistent_id("template_selector_open");
    let mut template_open = ui.data(|d| d.get_temp::<bool>(template_modal_id).unwrap_or(false));

    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New...").clicked() {
                template_open = true;
                ui.close();
            }

            if !config.recent_files.is_empty() {
                ui.separator();
                ui.label("Recent Files");

                let mut file_to_load = None;
                for path in config.recent_files.iter() {
                    let path_str: &String = path;
                    let display_path = if path_str.len() > 50 {
                        format!("...{}", &path_str[path_str.len() - 47..])
                    } else {
                        path_str.clone()
                    };

                    if ui.button(display_path).clicked() {
                        file_to_load = Some(path_str.clone());
                        ui.close();
                    }
                }

                if let Some(path_to_load) = file_to_load {
                    if let Some(loaded) = io::load_project_from_path(ctx, &path_to_load) {
                        action = MenuAction::LoadProject(loaded, path_to_load);
                    } else {
                        config.recent_files.retain(|p| p != &path_to_load);
                        config.save();
                    }
                }
            }

            ui.separator();
            if ui.button("Save PK3...").on_hover_text("Ctrl+S").clicked() {
                if let Some(f) = current_file {
                    if let Some(path) = io::save_pk3_dialog(f, assets, opened_file_path.clone()) {
                        action = MenuAction::SaveDone(path);
                    }
                }
                ui.close();
            }
            if ui.button("Export JSON...").on_hover_text("Ctrl+E").clicked() {
                if let Some(f) = current_file {
                    if let Some(path) = io::save_json_dialog(f, opened_file_path.clone()) {
                        action = MenuAction::ExportDone(path);
                    }
                }
                ui.close();
            }
            ui.separator();
            if ui.button("Settings...").clicked() {
                *settings_open = true;
                ui.close();
            }
            ui.separator();
            if ui.button("Quit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });

        ui.menu_button("Run", |ui| {
            let has_file = current_file.is_some();
            let has_port = config.source_port_path.is_some();

            if ui
                .add_enabled(has_file, egui::Button::new("Launch in Source Port"))
                .clicked()
            {
                if !has_port {
                    action = MenuAction::PickPortAndRun;
                } else if let (Some(f), Some(port), Some(iwad)) = (
                    current_file.as_ref(),
                    &config.source_port_path,
                    config.base_wad_path.as_ref(),
                ) {
                    io::launch_game(f, assets, port, iwad);
                }
                ui.close();
            }

            if !has_file {
                ui.separator();
                ui.label("⚠ No file loaded");
            } else if !has_port {
                ui.separator();
                ui.label("Port not set (will prompt)");
            }
        });
    });

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

                        if draw_menu_card(
                            ui,
                            "Empty",
                            "Start from scratch.",
                        ) {
                            action = MenuAction::NewEmpty;
                            close_window = true;
                        }

                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Templates").weak().italics().size(11.0));

                        for template in library::TEMPLATES {
                            if draw_menu_card(ui, template.name, template.description) {
                                action = MenuAction::LoadTemplate(template);
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
            ui.set_width(480.0);
            ui.add_space(4.0);

            let port_desc = config
                .source_port_path
                .as_deref()
                .unwrap_or("Click to browse for executable...");
            if draw_menu_card(ui, "Source Port", port_desc) {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Executable", &["exe"])
                    .set_title("Select Source Port (e.g., dsda-doom.exe)")
                    .pick_file()
                {
                    config.source_port_path = Some(path.to_string_lossy().into_owned());
                }
            }

            ui.add_space(8.0);

            let iwad_desc = config
                .base_wad_path
                .as_deref()
                .unwrap_or("Click to browse for DOOM2.WAD...");
            if draw_menu_card(ui, "Base IWAD", iwad_desc) {
                if let Some(new_path) = io::load_iwad_dialog(ctx, assets) {
                    config.base_wad_path = Some(new_path);
                }
            }

            ui.add_space(12.0);
            ui.separator();

            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                if ui.button("  Save & Close  ").clicked() {
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