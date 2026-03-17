use crate::app::CacocoApp;
use eframe::egui;

pub fn draw_resource_manager(ctx: &egui::Context, app: &mut CacocoApp) {
    let mut is_open = app.resources_open;

    egui::Window::new("Resource Manager")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(true)
        .min_width(400.0)
        .show(ctx, |ui| {
            ui.label("External resources added here are used for previewing in the editor.");
            ui.label(
                egui::RichText::new("These are NOT saved into your project file.")
                    .weak()
                    .italics(),
            );
            ui.add_space(8.0);

            let mut trigger_reload = false;

            ui.heading("Global Resources");
            let mut to_remove = None;

            egui::ScrollArea::vertical()
                .id_salt("global_res_scroll")
                .max_height(200.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if app.config.resource_paths.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("No resources loaded.").weak().italics());
                        });
                    } else {
                        for (i, path) in app.config.resource_paths.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if crate::ui::menu::draw_delete_card(ui, 24.0) {
                                    to_remove = Some(i);
                                }
                                ui.label(crate::ui::shared::truncate_path(path, 60));
                            });
                        }
                    }
                });

            if let Some(idx) = to_remove {
                app.config.resource_paths.remove(idx);
                app.config.save();
                trigger_reload = true;
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Add Resource...").clicked() {
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

            if trigger_reload {
                app.reload_resources(ctx);
            }
        });

    app.resources_open = is_open;
}
