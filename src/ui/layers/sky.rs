use super::thumbnails::ListRow;
use crate::assets::AssetStore;
use crate::document::actions::DocumentAction;
use crate::models::skydefs::SkyDefsFile;
use crate::ui::context_menu::ContextMenu;
use eframe::egui;
use std::collections::HashSet;

pub fn draw_sky_layers_list(
    ui: &mut egui::Ui,
    file: &mut SkyDefsFile,
    selection: &mut HashSet<Vec<usize>>,
    current_idx: &mut usize,
    assets: &AssetStore,
    _actions: &mut Vec<DocumentAction>,
    confirmation_modal: &mut Option<crate::app::ConfirmationRequest>,
) {
    ui.spacing_mut().item_spacing.y = 1.0;

    for (i, def) in file.data.skies.iter().enumerate() {
        let is_selected = selection.contains(&vec![i]);
        let is_active = *current_idx == i;

        let id = assets.resolve_sky_id(&def.name);
        let texture = assets.textures.get(&id);

        let response = ListRow::new(&def.name)
            .subtitle(format!("Type: {:?}", def.sky_type))
            .texture(texture)
            .fallback("?")
            .selected(is_selected)
            .active(is_active)
            .show(ui);

        if response.clicked() {
            selection.clear();
            selection.insert(vec![i]);
            *current_idx = i;
        }

        if response.drag_started() {
            egui::DragAndDrop::set_payload(ui.ctx(), vec![i]);
        }

        let just_opened = ContextMenu::check(ui, &response);
        if let Some(menu) = ContextMenu::get(ui, response.id) {
            if !is_selected {
                selection.clear();
                selection.insert(vec![i]);
                *current_idx = i;
            }

            ContextMenu::show(ui, menu, just_opened, |ui| {
                if ContextMenu::button(ui, "Delete Sky", true) {
                    *confirmation_modal = Some(crate::app::ConfirmationRequest::DeleteSky(i));
                    ContextMenu::close(ui);
                }
            });
        }
    }
}
