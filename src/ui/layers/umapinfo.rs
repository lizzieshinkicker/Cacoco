use super::thumbnails::ListRow;
use crate::assets::AssetStore;
use crate::document::actions::{DocumentAction, UmapAction};
use crate::models::umapinfo::{UmapField, UmapInfoFile};
use crate::ui::context_menu::ContextMenu;
use eframe::egui;
use std::collections::HashSet;

/// Key for storing the center-on-node request in egui context.
const CENTER_ON_NODE_KEY: &str = "umap_center_on_node";

/// Renders the scrollable list of maps defined in the project.
pub fn draw_umapinfo_layers_list(
    ui: &mut egui::Ui,
    file: &mut UmapInfoFile,
    selection: &mut HashSet<Vec<usize>>,
    current_idx: &mut usize,
    _assets: &AssetStore,
    actions: &mut Vec<DocumentAction>,
    _confirmation_modal: &mut Option<crate::app::ConfirmationRequest>,
) {
    ui.spacing_mut().item_spacing.y = 1.0;

    for (i, map) in file.data.maps.iter().enumerate() {
        let is_selected = selection.contains(&vec![i]);
        let is_active = *current_idx == i;

        let subtitle = map
            .fields
            .iter()
            .find_map(|f| {
                if let UmapField::LevelName(name) = f {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Untitled".to_string());

        let response = ListRow::new(&map.mapname)
            .subtitle(subtitle)
            .fallback("M")
            .selected(is_selected)
            .active(is_active)
            .show(ui);

        if response.clicked() {
            selection.clear();
            selection.insert(vec![i]);
            *current_idx = i;
            ui.ctx().data_mut(|d| {
                d.insert_temp(egui::Id::new(CENTER_ON_NODE_KEY), i);
            });
        }

        let just_opened = ContextMenu::check(ui, &response);
        if let Some(menu) = ContextMenu::get(ui, response.id) {
            if !is_selected {
                selection.clear();
                selection.insert(vec![i]);
                *current_idx = i;
            }

            ContextMenu::show(ui, menu, just_opened, |ui| {
                if ContextMenu::button(ui, "Delete Map Entry", true) {
                    actions.push(DocumentAction::UndoSnapshot);
                    actions.push(DocumentAction::Umap(UmapAction::DeleteMap(i)));
                    ContextMenu::close(ui);
                }
            });
        }
    }
}
