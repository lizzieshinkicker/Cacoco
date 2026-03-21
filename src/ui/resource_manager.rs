use crate::app::CacocoApp;
use eframe::egui;

pub fn draw_resource_manager(ctx: &egui::Context, app: &mut CacocoApp) {
    let mut is_open = app.resources_open;

    egui::Window::new("Resource Manager")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(295.0);
            ui.add_space(4.0);

            ui.vertical_centered(|ui| {
                ui.label("External assets used for previewing in the editor.");
                ui.label(
                    egui::RichText::new("These will not be saved into your project file.")
                        .weak()
                        .italics(),
                );
            });
            ui.add_space(8.0);

            let mut trigger_reload = false;

            ui.horizontal(|ui| {
                ui.heading("Global Resources");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("+ Add").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter(
                                "Doom Resources",
                                &["wad", "WAD", "pk3", "PK3", "zip", "ZIP"],
                            )
                            .pick_file()
                        {
                            let path_str = path.to_string_lossy().into_owned();
                            if !app.config.resource_paths.contains(&path_str) {
                                app.config.resource_paths.push(path_str);
                                app.config.save();
                                trigger_reload = true;
                            }
                        }
                    }
                });
            });
            ui.separator();
            ui.add_space(4.0);

            let mut to_remove = None;

            egui::ScrollArea::vertical()
                .id_salt("global_res_scroll")
                .max_height(200.0)
                .auto_shrink([true, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    if app.config.resource_paths.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("No resources loaded.").weak().italics());
                        });
                    } else {
                        let total_w = ui.available_width();
                        let delete_w = 24.0;
                        let card_w = total_w - delete_w - 4.0;

                        for (i, path) in app.config.resource_paths.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;

                                ui.allocate_ui(egui::vec2(card_w, 37.0), |ui| {
                                    let frame = egui::Frame::NONE
                                        .inner_margin(8.0)
                                        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                                        .corner_radius(4.0);

                                    frame.show(ui, |ui| {
                                        ui.set_min_size(egui::vec2(
                                            ui.available_width(),
                                            ui.available_height(),
                                        ));

                                        ui.with_layout(
                                            egui::Layout::left_to_right(egui::Align::Center),
                                            |ui| {
                                                ui.label(crate::ui::shared::truncate_path(
                                                    path, 37,
                                                ));
                                            },
                                        );
                                    });
                                });

                                if crate::ui::menu::draw_delete_card(ui, delete_w, 35.0) {
                                    to_remove = Some(i);
                                }
                            });
                        }
                    }
                });

            if let Some(idx) = to_remove {
                app.config.resource_paths.remove(idx);
                app.config.save();
                trigger_reload = true;
            }

            if trigger_reload {
                app.reload_resources(ctx);
            }
        });

    app.resources_open = is_open;
}
