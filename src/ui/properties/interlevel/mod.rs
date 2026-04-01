use crate::assets::AssetStore;
use crate::document::actions::{DocumentAction, InterlevelAction, TreeAction};
use crate::models::interlevel::{InterlevelAnim, InterlevelDefFile, InterlevelLayer};
use crate::ui::colors;
use crate::ui::layers::thumbnails::ListRow;
use crate::ui::properties::editor::{
    LayerContext, LumpUI, PropertyContext, TickContext, ViewportContext,
};
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

mod conditions;
mod frames;
pub mod simulator;
mod viewport;

pub(super) const SCREEN_IDX_KEY: &str = "cacoco_ilvl_screen_idx";
const PROP_TAB_KEY: &str = "cacoco_interlevel_tab_state";

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
enum PropertyTab {
    Properties,
    Conditions,
}

/// Renders the Properties/Conditions tab for a Layer element.
fn draw_layer_properties(
    ui: &mut egui::Ui,
    layer: &mut InterlevelLayer,
    current_tab: PropertyTab,
) -> bool {
    let mut changed = false;
    match current_tab {
        PropertyTab::Properties => {
            ui.vertical_centered(|ui| {
                changed |=
                    crate::ui::properties::common::draw_name_field(ui, &mut layer._cacoco_name);
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!(
                        "Layer contains {} animations.",
                        layer.anims.len()
                    ))
                    .weak(),
                );
            });
        }
        PropertyTab::Conditions => {
            changed |= conditions::draw_interlevel_conditions(ui, &mut layer.conditions);
        }
    }
    changed
}

/// Renders the Properties/Conditions tab for an Animation element.
fn draw_anim_properties(
    ui: &mut egui::Ui,
    anim: &mut InterlevelAnim,
    ctx: &PropertyContext,
    current_tab: PropertyTab,
) -> bool {
    let mut changed = false;
    match current_tab {
        PropertyTab::Properties => {
            changed |= crate::ui::properties::common::draw_name_field(ui, &mut anim._cacoco_name);
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 140.0).max(0.0) / 2.0);
                ui.label("X:");
                changed |= ui.add(egui::DragValue::new(&mut anim.x)).changed();
                ui.add_space(10.0);
                ui.label("Y:");
                changed |= ui.add(egui::DragValue::new(&mut anim.y)).changed();
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            changed |= frames::draw_interlevel_frames_editor(ui, anim, ctx.assets);
        }
        PropertyTab::Conditions => {
            changed |= conditions::draw_interlevel_conditions(ui, &mut anim.conditions);
        }
    }
    changed
}

fn draw_interlevel_move_buttons(
    ui: &mut egui::Ui,
    screen_idx: usize,
    path: Vec<usize>,
    actions: &mut Vec<DocumentAction>,
) {
    if ui.button("Move Up").clicked() {
        actions.push(DocumentAction::Interlevel(InterlevelAction::MoveUp {
            screen_idx,
            path: path.clone(),
        }));
        ui.close();
    }
    if ui.button("Move Down").clicked() {
        actions.push(DocumentAction::Interlevel(InterlevelAction::MoveDown {
            screen_idx,
            path: path.clone(),
        }));
        ui.close();
    }
}

/// Generates context menu actions for a Layer.
fn handle_layer_context_menu(
    ui: &mut egui::Ui,
    screen_idx: usize,
    l_idx: usize,
    path: Vec<usize>,
    actions: &mut Vec<DocumentAction>,
) {
    if ui.button("Add Animation").clicked() {
        actions.push(DocumentAction::UndoSnapshot);
        actions.push(DocumentAction::Interlevel(InterlevelAction::AddAnim {
            screen_idx,
            layer_idx: l_idx,
        }));
        ui.close();
    }
    if ui.button("Duplicate Layer").clicked() {
        actions.push(DocumentAction::UndoSnapshot);
        actions.push(DocumentAction::Interlevel(InterlevelAction::Duplicate {
            screen_idx,
            path: path.clone(),
        }));
        ui.close();
    }
    ui.separator();
    draw_interlevel_move_buttons(ui, screen_idx, path.clone(), actions);
    ui.separator();
    if ui.button("Delete Layer").clicked() {
        actions.push(DocumentAction::UndoSnapshot);
        actions.push(DocumentAction::Interlevel(InterlevelAction::Delete {
            screen_idx,
            paths: vec![path],
        }));
        ui.close();
    }
}

/// Generates context menu actions for an Animation.
fn handle_anim_context_menu(
    ui: &mut egui::Ui,
    screen_idx: usize,
    anim_path: Vec<usize>,
    actions: &mut Vec<DocumentAction>,
) {
    if ui.button("Duplicate Animation").clicked() {
        actions.push(DocumentAction::UndoSnapshot);
        actions.push(DocumentAction::Interlevel(InterlevelAction::Duplicate {
            screen_idx,
            path: anim_path.clone(),
        }));
        ui.close();
    }
    ui.separator();
    draw_interlevel_move_buttons(ui, screen_idx, anim_path.clone(), actions);
    ui.separator();
    if ui.button("Delete Animation").clicked() {
        actions.push(DocumentAction::UndoSnapshot);
        actions.push(DocumentAction::Interlevel(InterlevelAction::Delete {
            screen_idx,
            paths: vec![anim_path],
        }));
        ui.close();
    }
}

impl LumpUI for InterlevelDefFile {
    fn draw_properties(&mut self, ui: &mut egui::Ui, ctx: &PropertyContext) -> bool {
        let mut changed = false;

        let screen_idx = ui.ctx().data(|d| {
            d.get_temp::<usize>(egui::Id::new(SCREEN_IDX_KEY))
                .unwrap_or(0)
        });
        let screen_idx = screen_idx.min(self.screens.len().saturating_sub(1));
        if self.screens.is_empty() {
            return false;
        }
        let screen = &mut self.screens[screen_idx];

        let selection = ctx.selection.iter().next();

        let mut current_tab = ui.data(|d| {
            d.get_temp(egui::Id::new(PROP_TAB_KEY))
                .unwrap_or(PropertyTab::Properties)
        });

        if let Some(path) = selection {
            if path.len() == 2 || path.len() == 3 {
                ui.columns(2, |uis| {
                    if shared::section_header_button(
                        &mut uis[0],
                        "Properties",
                        None,
                        current_tab == PropertyTab::Properties,
                    )
                    .clicked()
                    {
                        current_tab = PropertyTab::Properties;
                    }
                    if shared::section_header_button(
                        &mut uis[1],
                        "Conditions",
                        None,
                        current_tab == PropertyTab::Conditions,
                    )
                    .clicked()
                    {
                        current_tab = PropertyTab::Conditions;
                    }
                });
                ui.add_space(3.0);
                ui.separator();
                ui.add_space(4.0);
            }
        }

        match selection {
            None => {
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 250.0).max(0.0) / 2.0);
                    ui.label("Background Image:");
                    if ui
                        .add_sized(
                            [120.0, 18.0],
                            egui::TextEdit::singleline(&mut screen.data.backgroundimage),
                        )
                        .changed()
                    {
                        screen.data.backgroundimage =
                            AssetStore::stem(&screen.data.backgroundimage);
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - 250.0).max(0.0) / 2.0);
                    ui.add_sized([114.0, 18.0], egui::Label::new("Music Lump:"));
                    if ui
                        .add_sized(
                            [120.0, 18.0],
                            egui::TextEdit::singleline(&mut screen.data.music),
                        )
                        .changed()
                    {
                        screen.data.music = AssetStore::stem(&screen.data.music);
                        changed = true;
                    }
                });
            }
            Some(path) => match path.as_slice() {
                [_s, l] => {
                    if let Some(layer) = screen.data.layers.get_mut(*l) {
                        changed |= draw_layer_properties(ui, layer, current_tab);
                    }
                }
                [_s, l, a] => {
                    if let Some(layer) = screen.data.layers.get_mut(*l) {
                        if let Some(anim) = layer.anims.get_mut(*a) {
                            changed |= draw_anim_properties(ui, anim, ctx, current_tab);
                        }
                    }
                }
                _ => {}
            },
        }

        ui.data_mut(|d| d.insert_temp(egui::Id::new(PROP_TAB_KEY), current_tab));
        changed
    }

    fn tick(&self, _ctx: &mut TickContext) {}

    fn draw_layer_list(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut LayerContext,
    ) -> (Vec<DocumentAction>, bool) {
        let mut actions = Vec::new();

        let screen_idx = *ctx.current_item_idx;
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(SCREEN_IDX_KEY), screen_idx));

        let screen_idx = screen_idx.min(self.screens.len().saturating_sub(1));
        if self.screens.is_empty() {
            return (actions, false);
        }
        let screen = &mut self.screens[screen_idx];

        let header_res = shared::heading_action_button(ui, "Layers", Some("Add Layer"), false);
        if header_res.clicked() {
            actions.push(DocumentAction::UndoSnapshot);
            actions.push(DocumentAction::Interlevel(InterlevelAction::AddLayer {
                screen_idx,
            }));
        }

        let current_time = ui.input(|i| i.time);

        egui::ScrollArea::vertical()
            .id_salt("interlevel_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (l_idx, layer) in screen.data.layers.iter().enumerate() {
                    let local_path = vec![l_idx];
                    let full_path = vec![screen_idx, l_idx];
                    let is_selected = ctx.selection.contains(&full_path);

                    let row = ListRow::new(layer.display_name(l_idx))
                        .selected(is_selected)
                        .fallback("📦")
                        .show(ui);

                    if row.clicked() {
                        actions.push(DocumentAction::Tree(TreeAction::Select(vec![
                            full_path.clone(),
                        ])));
                    }

                    row.context_menu(|ui| {
                        handle_layer_context_menu(
                            ui,
                            screen_idx,
                            l_idx,
                            local_path.clone(),
                            &mut actions,
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.vertical(|ui| {
                            for (a_idx, anim) in layer.anims.iter().enumerate() {
                                let local_anim_path = vec![l_idx, a_idx];
                                let full_anim_path = vec![screen_idx, l_idx, a_idx];
                                let anim_selected = ctx.selection.contains(&full_anim_path);

                                let mut tex = None;
                                if !anim.frames.is_empty() {
                                    let active_frame_idx =
                                        frames::get_active_frame_index(&anim.frames, current_time)
                                            .unwrap_or(0);
                                    if let Some(active_frame) = anim.frames.get(active_frame_idx) {
                                        let frame_id =
                                            crate::assets::AssetId::new(&active_frame.image);
                                        tex = ctx.assets.textures.get(&frame_id);
                                    }
                                }

                                let mut title = anim.display_name(a_idx);
                                if let Some(f) = anim.frames.first() {
                                    title.push_str(&format!(" ({})", f.image));
                                }

                                let a_row = ListRow::new(title)
                                    .selected(anim_selected)
                                    .texture(tex)
                                    .fallback("🎞")
                                    .show(ui);

                                if a_row.clicked() {
                                    actions.push(DocumentAction::Tree(TreeAction::Select(vec![
                                        full_anim_path.clone(),
                                    ])));
                                }

                                a_row.context_menu(|ui| {
                                    handle_anim_context_menu(
                                        ui,
                                        screen_idx,
                                        local_anim_path.clone(),
                                        &mut actions,
                                    );
                                });
                            }
                        });
                    });
                }
            });

        (actions, false)
    }

    fn header_info(&self, selection: &HashSet<Vec<usize>>) -> (String, String, egui::Color32) {
        let base_color = colors::as_header_bg(colors::LUMP_INTERLEVEL);

        if let Some(path) = selection.iter().next() {
            match path.as_slice() {
                [_s, _l] => {
                    return (
                        "Animation Layer".to_string(),
                        crate::ui::properties::descriptions::get_interlevel_layer_desc()
                            .to_string(),
                        colors::as_header_bg(colors::HEADER_LAYER),
                    );
                }
                [_s, _l, _a] => {
                    return (
                        "Animation Sequence".to_string(),
                        crate::ui::properties::descriptions::get_interlevel_anim_desc().to_string(),
                        colors::as_header_bg(colors::HEADER_ANIM),
                    );
                }
                _ => {}
            }
        }
        (
            "Interlevel".to_string(),
            "ID24 Intermission Editor".to_string(),
            base_color,
        )
    }

    fn get_preview_content(
        &self,
        ui: &egui::Ui,
        ctx: &PropertyContext,
    ) -> Option<crate::ui::properties::preview::PreviewContent> {
        let screen_idx = ui.ctx().data(|d| {
            d.get_temp::<usize>(egui::Id::new(SCREEN_IDX_KEY))
                .unwrap_or(0)
        });
        let screen_idx = screen_idx.min(self.screens.len().saturating_sub(1));
        if self.screens.is_empty() {
            return None;
        }
        let screen = &self.screens[screen_idx];

        if ctx.selection.is_empty() {
            return Some(crate::ui::properties::preview::PreviewContent::Image(
                screen.data.backgroundimage.clone(),
            ));
        }

        let path = ctx.selection.iter().next()?;
        match path.as_slice() {
            [_s, l, a] => {
                if let Some(layer) = screen.data.layers.get(*l) {
                    if let Some(anim) = layer.anims.get(*a) {
                        if let Some(frame) = anim.frames.first() {
                            return Some(crate::ui::properties::preview::PreviewContent::Image(
                                frame.image.clone(),
                            ));
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn render_viewport(&self, ui: &mut egui::Ui, ctx: &mut ViewportContext) -> Vec<DocumentAction> {
        viewport::render_viewport_impl(self, ui, ctx)
    }
}
