use crate::assets::{AssetId, AssetStore};
use crate::document::actions::{DocumentAction, InterlevelAction};
use crate::models::interlevel::InterlevelDefFile;
use crate::ui::layers::thumbnails::ListRow;
use eframe::egui;

pub fn draw_screens_browser(
    ui: &mut egui::Ui,
    file: &mut InterlevelDefFile,
    selection: &mut std::collections::HashSet<Vec<usize>>,
    current_idx: &mut usize,
    assets: &AssetStore,
    actions: &mut Vec<DocumentAction>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.heading("Screens");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Add Screen").clicked() {
                actions.push(DocumentAction::Interlevel(InterlevelAction::AddScreen));
            }
        });
    });
    ui.separator();

    for (idx, screen) in file.screens.iter_mut().enumerate() {
        let is_active = *current_idx == idx;

        let bg_id = AssetId::new(&screen.data.backgroundimage);
        let tex = assets.textures.get(&bg_id);

        let row = ListRow::new(&screen.name)
            .active(is_active)
            .texture(tex)
            .fallback("📺")
            .show(ui);

        if row.clicked() {
            if !is_active {
                selection.clear();
                *current_idx = idx;
            }
        }

        row.context_menu(|ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                let mut name_buf = screen.name.clone();
                if ui
                    .add(egui::TextEdit::singleline(&mut name_buf).desired_width(80.0))
                    .changed()
                {
                    actions.push(DocumentAction::Interlevel(InterlevelAction::RenameScreen(
                        idx, name_buf,
                    )));
                    changed = true;
                }
            });
            ui.separator();
            if ui.button("Duplicate Screen").clicked() {
                actions.push(DocumentAction::Interlevel(
                    InterlevelAction::DuplicateScreen(idx),
                ));
                ui.close();
            }
            if ui.button("Delete Screen").clicked() {
                actions.push(DocumentAction::Interlevel(InterlevelAction::DeleteScreen(
                    idx,
                )));
                ui.close();
            }
        });
    }
    changed
}
