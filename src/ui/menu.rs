use crate::app::PendingAction;
use crate::assets::AssetStore;
use crate::config::AppConfig;
use crate::io::{self, LoadedProject};
use crate::library;
use crate::model::SBarDefFile;
use eframe::egui;
use std::path::Path;

pub enum MenuAction {
    None,
    LoadProject(LoadedProject, String),
    LoadTemplate(&'static library::Template),
    NewEmpty,
    Open,
    RequestDiscard(PendingAction),
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
    dirty: bool,
) -> MenuAction {
    let mut action = MenuAction::None;

    let template_modal_id = ui.make_persistent_id("template_selector_open");
    let mut template_open = ui.data(|d| d.get_temp::<bool>(template_modal_id).unwrap_or(false));

    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New...").clicked() {
                if dirty {
                    action = MenuAction::RequestDiscard(PendingAction::New);
                } else {
                    template_open = true;
                }
                ui.close();
            }

            if ui.button("Open...").on_hover_text("Ctrl+O").clicked() {
                if dirty {
                    action = MenuAction::RequestDiscard(PendingAction::Load("".to_string()));
                } else {
                    action = MenuAction::Open;
                }
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
                    if dirty {
                        action = MenuAction::RequestDiscard(PendingAction::Load(path_to_load));
                    } else if let Some(loaded) = io::load_project_from_path(ctx, &path_to_load) {
                        action = MenuAction::LoadProject(loaded, path_to_load);
                    } else {
                        config.recent_files.retain(|p| p != &path_to_load);
                        config.save();
                    }
                }
            }

            ui.separator();
            if ui.button("Save").on_hover_text("Ctrl+S").clicked() {
                action = MenuAction::SaveDone("SILENT".to_string());
                ui.close();
            }
            if ui.button("Save As...").clicked() {
                if let Some(f) = current_file {
                    if let Some(path) = io::save_pk3_dialog(f, assets, opened_file_path.clone()) {
                        action = MenuAction::SaveDone(path);
                    }
                }
                ui.close();
            }

            if ui
                .button("Export JSON...")
                .on_hover_text("Ctrl+E")
                .clicked()
            {
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
                if dirty {
                    action = MenuAction::RequestDiscard(PendingAction::Quit);
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        });

        ui.menu_button("Run", |ui| {
            let has_file = current_file.is_some();

            if config.source_ports.is_empty() {
                if ui
                    .add_enabled(has_file, egui::Button::new("Launch (No port set...)"))
                    .clicked()
                {
                    action = MenuAction::PickPortAndRun;
                    ui.close();
                }
            } else {
                for path_str in &config.source_ports {
                    let name = get_port_name(path_str);
                    if ui
                        .add_enabled(has_file, egui::Button::new(format!("Launch in {}", name)))
                        .clicked()
                    {
                        if let (Some(f), Some(iwad)) =
                            (current_file.as_ref(), config.base_wad_path.as_ref())
                        {
                            io::launch_game(f, assets, path_str, iwad);
                        }
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Add New Port...").clicked() {
                    action = MenuAction::PickPortAndRun;
                    ui.close();
                }
            }

            if !has_file {
                ui.separator();
                ui.label("⚠ No file loaded");
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
                    if ui.button("+ Add Port").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Executable", &["exe"])
                            .set_title("Select Source Port")
                            .pick_file()
                        {
                            let path_str = path.to_string_lossy().into_owned();
                            if !config.source_ports.contains(&path_str) {
                                config.source_ports.push(path_str);
                            }
                        }
                    }
                });
            });
            ui.separator();
            ui.add_space(4.0);

            let mut to_remove = None;
            egui::ScrollArea::vertical()
                .max_height(240.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    for (idx, path_str) in config.source_ports.iter().enumerate() {
                        let name = get_port_name(path_str);

                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            let total_w = ui.available_width();
                            let delete_w = 44.0;
                            let card_w = total_w - delete_w - 4.0;

                            ui.allocate_ui(egui::vec2(card_w, 54.0), |ui| {
                                draw_menu_card(ui, &name, path_str);
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
    let height = 54.0;
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

pub fn get_port_name(path_str: &str) -> String {
    let stem = Path::new(path_str)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Port");

    if stem.is_empty() {
        return "Unknown Port".to_string();
    }

    let mut chars = stem.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
