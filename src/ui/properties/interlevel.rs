use crate::assets::{AssetId, AssetStore};
use crate::constants::DOOM_TICS_PER_SEC;
use crate::document::actions::{DocumentAction, InterlevelAction, TreeAction};
use crate::models::interlevel::{InterlevelCondition, InterlevelDefFile, InterlevelFrame};
use crate::ui::colors;
use crate::ui::context_menu::ContextMenu;
use crate::ui::layers::thumbnails::{self, ListRow};
use crate::ui::properties::editor::{
    LayerContext, LumpUI, PropertyContext, TickContext, ViewportContext,
};
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

const PROP_TAB_KEY: &str = "cacoco_interlevel_tab_state";
const SCREEN_IDX_KEY: &str = "cacoco_ilvl_screen_idx";

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
enum PropertyTab {
    Properties,
    Conditions,
}

enum InterlevelFrameAction {
    MoveSelection(Vec<usize>, usize),
    Add(usize, String),
    Replace(usize, String),
}

fn condition_name(val: i32) -> &'static str {
    match val {
        0 => "None",
        1 => "Current Map > Param",
        2 => "Current Map == Param",
        3 => "Map (Param) Visited",
        4 => "Not a Secret Map",
        5 => "Any Secret Map Visited",
        6 => "Is Tally Screen",
        7 => "Is Entering Screen",
        _ => "Unknown",
    }
}

fn draw_interlevel_conditions(
    ui: &mut egui::Ui,
    conditions: &mut Vec<InterlevelCondition>,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("Active Rules: {}", conditions.len())).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(2.0);
            if ui.button("Add Condition").clicked() {
                conditions.push(InterlevelCondition {
                    condition: 0,
                    param: 0,
                });
                changed = true;
            }
            if !conditions.is_empty() {
                if ui.button("Clear All").clicked() {
                    conditions.clear();
                    changed = true;
                }
            }
        });
    });

    ui.separator();
    ui.add_space(4.0);

    let mut to_remove = None;
    for (i, cond) in conditions.iter_mut().enumerate() {
        let frame = egui::Frame::new()
            .inner_margin(4.0)
            .corner_radius(4.0)
            .fill(egui::Color32::from_white_alpha(5))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(15)));

        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new("X").min_size(egui::vec2(18.0, 18.0)))
                        .on_hover_text("Remove Condition")
                        .clicked()
                    {
                        to_remove = Some(i);
                    }

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let id = ui.make_persistent_id(format!("ilvl_cond_{}", i));
                        let current_name = condition_name(cond.condition);

                        let btn_res = shared::combobox_button(ui, current_name, 160.0);
                        if btn_res.clicked() {
                            ContextMenu::open(ui, id, btn_res.rect.left_bottom());
                        }

                        if let Some(menu) = ContextMenu::get(ui, id) {
                            ContextMenu::show(ui, menu, btn_res.clicked(), |ui| {
                                ui.set_min_width(160.0);
                                for val in 0..=7 {
                                    if crate::ui::properties::common::custom_menu_item(
                                        ui,
                                        condition_name(val),
                                        cond.condition == val,
                                    ) {
                                        cond.condition = val;
                                        changed = true;
                                        ContextMenu::close(ui);
                                    }
                                }
                            });
                        }

                        if matches!(cond.condition, 1 | 2 | 3) {
                            ui.add_space(8.0);
                            ui.label("Map:");
                            changed |= ui.add(egui::DragValue::new(&mut cond.param)).changed();
                        }
                    });
                });
            });
        });
        ui.add_space(4.0);
    }

    if let Some(idx) = to_remove {
        conditions.remove(idx);
        changed = true;
    }

    changed
}

fn draw_interlevel_frame_row(
    ui: &mut egui::Ui,
    idx: usize,
    frame: &mut InterlevelFrame,
    assets: &AssetStore,
    actions: &mut Vec<InterlevelFrameAction>,
    selection: &mut HashSet<usize>,
    pivot: &mut Option<usize>,
    is_active: bool,
) -> bool {
    let mut changed = false;
    let row_height = 54.0;
    let is_selected = selection.contains(&idx);
    let spacing_offset = ui.spacing().item_spacing.y * 0.5;

    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click_and_drag(),
    );

    if response.clicked() {
        let modifiers = ui.input(|i| i.modifiers);
        if modifiers.ctrl || modifiers.command {
            if is_selected {
                selection.remove(&idx);
            } else {
                selection.insert(idx);
                *pivot = Some(idx);
            }
        } else if modifiers.shift {
            if let Some(p) = *pivot {
                let min = p.min(idx);
                let max = p.max(idx);
                selection.clear();
                for i in min..=max {
                    selection.insert(i);
                }
            } else {
                selection.insert(idx);
                *pivot = Some(idx);
            }
        } else {
            selection.clear();
            selection.insert(idx);
            *pivot = Some(idx);
        }
    }

    if response.drag_started() {
        if !is_selected {
            selection.clear();
            selection.insert(idx);
        }
        egui::DragAndDrop::set_payload(ui.ctx(), "INTERLEVEL_FRAME_SELECTION");
    }

    if ui.rect_contains_pointer(rect) {
        if egui::DragAndDrop::payload::<&'static str>(ui.ctx())
            .is_some_and(|p| *p == "INTERLEVEL_FRAME_SELECTION")
        {
            let pos = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
            let rel_y = pos.y - rect.top();
            let top_half = rel_y < (row_height / 2.0);
            let target_idx = if top_half { idx } else { idx + 1 };

            let is_source =
                selection.contains(&idx) || (top_half && idx > 0 && selection.contains(&(idx - 1)));
            if !is_source {
                let y = if top_half {
                    rect.top() - spacing_offset
                } else {
                    rect.bottom() + spacing_offset
                };
                shared::draw_yellow_line(ui, rect, y);
                if ui.input(|i| i.pointer.any_released()) {
                    actions.push(InterlevelFrameAction::MoveSelection(
                        selection.iter().cloned().collect(),
                        target_idx,
                    ));
                }
            }
        }

        if let Some(asset_keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
            let pos = ui.input(|i| i.pointer.latest_pos().unwrap_or_default());
            let rel_y = pos.y - rect.top();
            let margin = row_height * 0.25;

            if rel_y > margin && rel_y < (row_height - margin) {
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(2.0, egui::Color32::GREEN),
                    egui::StrokeKind::Inside,
                );
                if ui.input(|i| i.pointer.any_released()) {
                    actions.push(InterlevelFrameAction::Replace(idx, asset_keys[0].clone()));
                }
            } else {
                let top_half = rel_y < (row_height / 2.0);
                let y = if top_half {
                    rect.top() - spacing_offset
                } else {
                    rect.bottom() + spacing_offset
                };
                let mut target_idx = if top_half { idx } else { idx + 1 };
                shared::draw_yellow_line(ui, rect, y);

                if ui.input(|i| i.pointer.any_released()) {
                    for key in asset_keys.iter() {
                        actions.push(InterlevelFrameAction::Add(target_idx, key.clone()));
                        target_idx += 1;
                    }
                }
            }
        }
    }

    let mut bg = if is_active {
        egui::Color32::from_rgba_unmultiplied(0, 255, 255, 10)
    } else {
        egui::Color32::TRANSPARENT
    };
    if response.hovered() {
        bg = ui.visuals().widgets.hovered.bg_fill;
    }
    let stroke = if is_selected {
        ui.visuals().selection.stroke
    } else {
        egui::Stroke::NONE
    };
    ui.painter()
        .rect(rect, 4.0, bg, stroke, egui::StrokeKind::Outside);

    let thumb_rect = egui::Rect::from_center_size(
        egui::pos2(rect.min.x + 22.0, rect.center().y),
        egui::vec2(thumbnails::THUMB_SIZE, thumbnails::THUMB_SIZE),
    );
    let mut thumb_ui = ui.new_child(egui::UiBuilder::new().max_rect(thumb_rect));

    let id = AssetId::new(&frame.image);
    thumbnails::draw_thumbnail_widget(
        &mut thumb_ui,
        assets.textures.get(&id),
        Some("?"),
        false,
        false,
    );

    let content_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + 48.0, rect.min.y + 4.0),
        egui::pos2(rect.max.x - 4.0, rect.max.y - 4.0),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&frame.image).strong());

                let mut base_type = frame.frame_type & 0x7;
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .radio_value(&mut base_type, 4, "Rnd")
                        .on_hover_text("Random Duration")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut base_type, 2, "Fix")
                        .on_hover_text("Fixed Duration")
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .radio_value(&mut base_type, 1, "Inf")
                        .on_hover_text("Infinite Duration")
                        .changed()
                    {
                        changed = true;
                    }
                });

                if changed {
                    frame.frame_type = (frame.frame_type & !0x7) | base_type;
                }
            });

            ui.horizontal(|ui| {
                let base_type = frame.frame_type & 0x7;

                if base_type == 2 || base_type == 4 {
                    let mut tics = (frame.duration * DOOM_TICS_PER_SEC).round() as i32;
                    if ui
                        .add(
                            egui::DragValue::new(&mut tics)
                                .suffix(" tics")
                                .range(1..=3500),
                        )
                        .changed()
                    {
                        frame.duration = tics as f64 / DOOM_TICS_PER_SEC;
                        changed = true;
                    }
                }

                if base_type == 4 {
                    ui.label("Max:");
                    let mut max_tics = (frame.maxduration * DOOM_TICS_PER_SEC).round() as i32;
                    if ui
                        .add(
                            egui::DragValue::new(&mut max_tics)
                                .suffix(" tics")
                                .range(1..=3500),
                        )
                        .changed()
                    {
                        frame.maxduration = max_tics as f64 / DOOM_TICS_PER_SEC;
                        changed = true;
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut widescreen = (frame.frame_type & 0x8000000) != 0;
                    if ui
                        .checkbox(&mut widescreen, "Wide")
                        .on_hover_text("Center for Widescreen")
                        .changed()
                    {
                        frame.frame_type = (frame.frame_type & !0x8000000)
                            | (if widescreen { 0x8000000 } else { 0 });
                        changed = true;
                    }

                    let mut random_first = (frame.frame_type & 0x1000) != 0;
                    if ui
                        .checkbox(&mut random_first, "RFF")
                        .on_hover_text("Random First Frame Offset")
                        .changed()
                    {
                        frame.frame_type =
                            (frame.frame_type & !0x1000) | (if random_first { 0x1000 } else { 0 });
                        changed = true;
                    }
                });
            });
        });
    });

    if response.dragged() {
        let label = if selection.len() > 1 {
            format!("{} frames", selection.len())
        } else {
            frame.image.clone()
        };
        shared::draw_drag_ghost(
            ui.ctx(),
            |ui| {
                thumbnails::draw_thumbnail_widget(
                    ui,
                    assets.textures.get(&id),
                    Some("?"),
                    false,
                    false,
                );
            },
            &label,
        );
    }
    changed
}

fn draw_empty_frame_dropzone(ui: &mut egui::Ui, actions: &mut Vec<InterlevelFrameAction>) -> bool {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 60.0), egui::Sense::hover());
    ui.painter().rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
        egui::StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "Drop Graphics Here",
        egui::FontId::proportional(14.0),
        egui::Color32::from_gray(100),
    );

    if let Some(keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
        if ui.rect_contains_pointer(rect) {
            ui.painter()
                .rect_filled(rect, 4.0, egui::Color32::from_white_alpha(10));
            if ui.input(|i| i.pointer.any_released()) {
                for key in keys.iter() {
                    actions.push(InterlevelFrameAction::Add(9999, key.clone()));
                }
                return true;
            }
        }
    }
    false
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
            if path.len() == 1 || path.len() == 2 {
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
                [l] => {
                    if let Some(layer) = screen.data.layers.get_mut(*l) {
                        match current_tab {
                            PropertyTab::Properties => {
                                ui.vertical_centered(|ui| {
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
                                changed |= draw_interlevel_conditions(ui, &mut layer.conditions);
                            }
                        }
                    }
                }
                [l, a] => {
                    if let Some(layer) = screen.data.layers.get_mut(*l) {
                        if let Some(anim) = layer.anims.get_mut(*a) {
                            match current_tab {
                                PropertyTab::Properties => {
                                    ui.horizontal(|ui| {
                                        ui.add_space((ui.available_width() - 140.0).max(0.0) / 2.0);
                                        ui.label("X:");
                                        changed |=
                                            ui.add(egui::DragValue::new(&mut anim.x)).changed();
                                        ui.add_space(10.0);
                                        ui.label("Y:");
                                        changed |=
                                            ui.add(egui::DragValue::new(&mut anim.y)).changed();
                                    });

                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.add_space(4.0);

                                    ui.horizontal(|ui| {
                                        ui.heading("Frames");
                                        ui.add_space(8.0);
                                        let frame_color =
                                            colors::as_header_bg(colors::HEADER_FRAME);
                                        crate::ui::properties::draw_static_header(
                                            ui,
                                            "Animation Frames",
                                            "Sequence of images to play.",
                                            frame_color,
                                        );

                                        ui.horizontal(|ui| {
                                            ui.add_space(ui.available_width() - 110.0);
                                            if !anim.frames.is_empty()
                                                && ui.button("Clear").clicked()
                                            {
                                                anim.frames.clear();
                                                changed = true;
                                            }
                                            if ui.button("Add").clicked() {
                                                anim.frames.push(InterlevelFrame {
                                                    image: "HICACOCO".to_string(),
                                                    duration: 1.0,
                                                    frame_type: 2,
                                                    ..Default::default()
                                                });
                                                changed = true;
                                            }
                                        });
                                    });
                                    ui.separator();

                                    let mut total_duration = 0.0;
                                    let mut has_infinite = false;
                                    for f in &anim.frames {
                                        let f_type = f.frame_type & 0x7;
                                        if f_type == 1 {
                                            has_infinite = true;
                                            break;
                                        }
                                        total_duration += f.duration;
                                    }

                                    let mut active_idx = None;
                                    let current_time = ui.input(|i| i.time);

                                    if total_duration > 0.0 && !has_infinite {
                                        let anim_time = current_time % total_duration;
                                        let mut accumulator = 0.0;
                                        for (idx, frame) in anim.frames.iter().enumerate() {
                                            accumulator += frame.duration;
                                            if anim_time < accumulator {
                                                active_idx = Some(idx);
                                                break;
                                            }
                                        }
                                    } else if has_infinite {
                                        let mut accumulator = 0.0;
                                        for (idx, frame) in anim.frames.iter().enumerate() {
                                            let f_type = frame.frame_type & 0x7;
                                            accumulator += frame.duration;
                                            if f_type == 1 || current_time < accumulator {
                                                active_idx = Some(idx);
                                                break;
                                            }
                                        }
                                    }

                                    let sel_id = ui.make_persistent_id("ilvl_frame_selection");
                                    let pivot_id = ui.make_persistent_id("ilvl_frame_pivot");
                                    let mut f_selection: HashSet<usize> =
                                        ui.data(|d| d.get_temp(sel_id).unwrap_or_default());
                                    let mut f_pivot: Option<usize> =
                                        ui.data(|d| d.get_temp(pivot_id));

                                    let mut frame_actions = Vec::new();
                                    ui.spacing_mut().item_spacing.y = 1.0;

                                    if anim.frames.is_empty() {
                                        changed |=
                                            draw_empty_frame_dropzone(ui, &mut frame_actions);
                                    } else {
                                        for (idx, frame) in anim.frames.iter_mut().enumerate() {
                                            let is_active = active_idx == Some(idx);
                                            ui.push_id(idx, |ui| {
                                                changed |= draw_interlevel_frame_row(
                                                    ui,
                                                    idx,
                                                    frame,
                                                    ctx.assets,
                                                    &mut frame_actions,
                                                    &mut f_selection,
                                                    &mut f_pivot,
                                                    is_active,
                                                );
                                            });
                                        }
                                    }

                                    for action in frame_actions {
                                        changed = true;
                                        match action {
                                            InterlevelFrameAction::MoveSelection(
                                                sources,
                                                mut target_idx,
                                            ) => {
                                                let mut sorted_src = sources.clone();
                                                sorted_src.sort();
                                                let mut src_desc = sorted_src.clone();
                                                src_desc.sort_by(|a, b| b.cmp(a));

                                                let mut moved_items = Vec::new();
                                                for src in src_desc {
                                                    if src < target_idx {
                                                        target_idx -= 1;
                                                    }
                                                    if src < anim.frames.len() {
                                                        moved_items.push(anim.frames.remove(src));
                                                    }
                                                }
                                                moved_items.reverse();

                                                let safe_idx = target_idx.min(anim.frames.len());
                                                f_selection.clear();
                                                for (i, item) in moved_items.into_iter().enumerate()
                                                {
                                                    anim.frames.insert(safe_idx + i, item);
                                                    f_selection.insert(safe_idx + i);
                                                }
                                            }
                                            InterlevelFrameAction::Add(i, lump) => {
                                                anim.frames.insert(
                                                    i.min(anim.frames.len()),
                                                    InterlevelFrame {
                                                        image: lump,
                                                        duration: 1.0,
                                                        frame_type: 2,
                                                        ..Default::default()
                                                    },
                                                );
                                                f_selection.clear();
                                                f_selection.insert(i.min(anim.frames.len() - 1));
                                            }
                                            InterlevelFrameAction::Replace(i, lump) => {
                                                if i < anim.frames.len() {
                                                    anim.frames[i].image = lump;
                                                }
                                            }
                                        }
                                    }

                                    ui.data_mut(|d| {
                                        d.insert_temp(sel_id, f_selection);
                                        d.insert_temp(pivot_id, f_pivot);
                                    });
                                }
                                PropertyTab::Conditions => {
                                    changed |= draw_interlevel_conditions(ui, &mut anim.conditions);
                                }
                            }
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

        egui::ScrollArea::vertical()
            .id_salt("interlevel_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (l_idx, layer) in screen.data.layers.iter().enumerate() {
                    let path = vec![l_idx];
                    let is_selected = ctx.selection.contains(&path);

                    let row = ListRow::new(format!("Layer {}", l_idx))
                        .selected(is_selected)
                        .fallback("📦")
                        .show(ui);

                    if row.clicked() {
                        actions.push(DocumentAction::Tree(TreeAction::Select(vec![path.clone()])));
                    }

                    row.context_menu(|ui| {
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
                        ui.separator();
                        if ui.button("Delete Layer").clicked() {
                            actions.push(DocumentAction::UndoSnapshot);
                            actions.push(DocumentAction::Interlevel(InterlevelAction::Delete {
                                screen_idx,
                                paths: vec![path.clone()],
                            }));
                            ui.close();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.vertical(|ui| {
                            for (a_idx, anim) in layer.anims.iter().enumerate() {
                                let anim_path = vec![l_idx, a_idx];
                                let anim_selected = ctx.selection.contains(&anim_path);

                                let mut title = format!("Animation {}", a_idx);
                                if let Some(f) = anim.frames.first() {
                                    title.push_str(&format!(" ({})", f.image));
                                }

                                let a_row = ListRow::new(title)
                                    .selected(anim_selected)
                                    .fallback("🎞")
                                    .show(ui);

                                if a_row.clicked() {
                                    actions.push(DocumentAction::Tree(TreeAction::Select(vec![
                                        anim_path.clone(),
                                    ])));
                                }

                                a_row.context_menu(|ui| {
                                    if ui.button("Duplicate Animation").clicked() {
                                        actions.push(DocumentAction::UndoSnapshot);
                                        actions.push(DocumentAction::Interlevel(
                                            InterlevelAction::Duplicate {
                                                screen_idx,
                                                path: anim_path.clone(),
                                            },
                                        ));
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("Move Up").clicked() {
                                        actions.push(DocumentAction::Interlevel(
                                            InterlevelAction::MoveUp {
                                                screen_idx,
                                                path: anim_path.clone(),
                                            },
                                        ));
                                        ui.close();
                                    }
                                    if ui.button("Move Down").clicked() {
                                        actions.push(DocumentAction::Interlevel(
                                            InterlevelAction::MoveDown {
                                                screen_idx,
                                                path: anim_path.clone(),
                                            },
                                        ));
                                        ui.close();
                                    }
                                    ui.separator();
                                    if ui.button("Delete Animation").clicked() {
                                        actions.push(DocumentAction::UndoSnapshot);
                                        actions.push(DocumentAction::Interlevel(
                                            InterlevelAction::Delete {
                                                screen_idx,
                                                paths: vec![anim_path.clone()],
                                            },
                                        ));
                                        ui.close();
                                    }
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
                [l] => {
                    return (
                        format!("Layer {}", l),
                        "A container for animations.".to_string(),
                        colors::as_header_bg(colors::HEADER_LAYER),
                    );
                }
                [_l, a] => {
                    return (
                        format!("Animation {}", a),
                        "A sequence of frames at a set position.".to_string(),
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
            [_l, a] => {
                if let Some(layer) = screen.data.layers.get(path[0]) {
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
        let screen_idx = ctx
            .current_item_idx
            .min(self.screens.len().saturating_sub(1));
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(SCREEN_IDX_KEY), screen_idx));
        if self.screens.is_empty() {
            return Vec::new();
        }
        let screen = &self.screens[screen_idx];

        let painter = ui.painter();
        let bg_id = AssetId::new(&screen.data.backgroundimage);

        if let Some(tex) = ctx.assets.textures.get(&bg_id) {
            let screen_w = if ctx.state.sim.engine.widescreen_mode {
                crate::constants::DOOM_W_WIDE
            } else {
                crate::constants::DOOM_W
            };

            let center_x = screen_w / 2.0;
            let center_y = crate::constants::DOOM_H / 2.0;

            let virtual_rect = egui::Rect::from_center_size(
                egui::pos2(center_x, center_y),
                egui::vec2(tex.size()[0] as f32, tex.size()[1] as f32),
            );

            let screen_rect = egui::Rect::from_min_max(
                ctx.proj.to_screen(virtual_rect.min),
                ctx.proj.to_screen(virtual_rect.max),
            );
            painter.image(
                tex.id(),
                screen_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            painter.rect_filled(ctx.proj.screen_rect, 0.0, egui::Color32::BLACK);
        }

        let current_time = ui.input(|i| i.time);

        for layer in &screen.data.layers {
            for anim in &layer.anims {
                if anim.frames.is_empty() {
                    continue;
                }

                let mut active_frame = &anim.frames[0];

                let mut total_duration = 0.0;
                let mut has_infinite = false;
                for f in &anim.frames {
                    let f_type = f.frame_type & 0x7;
                    if f_type == 1 {
                        has_infinite = true;
                        break;
                    }
                    total_duration += f.duration;
                }

                if total_duration > 0.0 && !has_infinite {
                    let anim_time = current_time % total_duration;
                    let mut accumulator = 0.0;
                    for f in &anim.frames {
                        accumulator += f.duration;
                        if anim_time < accumulator {
                            active_frame = f;
                            break;
                        }
                    }
                } else if has_infinite {
                    let mut accumulator = 0.0;
                    for f in &anim.frames {
                        let f_type = f.frame_type & 0x7;
                        accumulator += f.duration;
                        if f_type == 1 || current_time < accumulator {
                            active_frame = f;
                            break;
                        }
                    }
                }

                let frame_id = AssetId::new(&active_frame.image);
                if let Some(tex) = ctx.assets.textures.get(&frame_id) {
                    let mut x = anim.x as f32;
                    let y = anim.y as f32;

                    if (active_frame.frame_type & 0x8000000) != 0 {
                        if ctx.state.sim.engine.widescreen_mode {
                            x += (crate::constants::DOOM_W_WIDE - crate::constants::DOOM_W) / 2.0;
                        }
                    }

                    let (left_offset, top_offset) =
                        ctx.assets.offsets.get(&frame_id).copied().unwrap_or((0, 0));

                    let draw_x = x - (left_offset as f32);
                    let draw_y = y - (top_offset as f32);

                    let virtual_rect = egui::Rect::from_min_size(
                        egui::pos2(draw_x, draw_y),
                        egui::vec2(tex.size()[0] as f32, tex.size()[1] as f32),
                    );

                    let screen_rect = egui::Rect::from_min_max(
                        ctx.proj.to_screen(virtual_rect.min),
                        ctx.proj.to_screen(virtual_rect.max),
                    );
                    painter.image(
                        tex.id(),
                        screen_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            }
        }

        ui.ctx().request_repaint();
        Vec::new()
    }
}
