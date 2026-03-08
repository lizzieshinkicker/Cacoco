use super::editor::{LayerContext, LumpUI, PropertyContext, ViewportContext};
use crate::assets::AssetStore;
use crate::document::DocumentAction;
use crate::document::actions::UmapAction;
use crate::models::umapinfo::{MapEntry, UmapField, UmapInfoFile};
use crate::state::PreviewState;
use crate::ui::context_menu::ContextMenu;
use crate::ui::properties::common;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

/// Metadata for a UMAPINFO field type to facilitate UI lookups and creation.
struct FieldMetadata {
    label: &'static str,
    key_name: &'static str,
    group: &'static str,
}

const FIELD_REGISTRY: &[FieldMetadata] = &[
    FieldMetadata {
        label: "Level Name",
        key_name: "levelname",
        group: "Metadata",
    },
    FieldMetadata {
        label: "Author",
        key_name: "author",
        group: "Metadata",
    },
    FieldMetadata {
        label: "Label/Prefix",
        key_name: "label",
        group: "Metadata",
    },
    FieldMetadata {
        label: "Sky Texture",
        key_name: "skytexture",
        group: "Graphics",
    },
    FieldMetadata {
        label: "Level Pic",
        key_name: "levelpic",
        group: "Graphics",
    },
    FieldMetadata {
        label: "Exit Backdrop",
        key_name: "exitpic",
        group: "Graphics",
    },
    FieldMetadata {
        label: "Enter Backdrop",
        key_name: "enterpic",
        group: "Graphics",
    },
    FieldMetadata {
        label: "Music",
        key_name: "music",
        group: "Audio",
    },
    FieldMetadata {
        label: "Inter. Music",
        key_name: "intermusic",
        group: "Audio",
    },
    FieldMetadata {
        label: "Next Map",
        key_name: "next",
        group: "Progression",
    },
    FieldMetadata {
        label: "Secret Exit",
        key_name: "nextsecret",
        group: "Progression",
    },
    FieldMetadata {
        label: "Par Time",
        key_name: "partime",
        group: "Progression",
    },
    FieldMetadata {
        label: "End Game",
        key_name: "endgame",
        group: "Completion",
    },
    FieldMetadata {
        label: "End Bunny",
        key_name: "endbunny",
        group: "Completion",
    },
    FieldMetadata {
        label: "End Cast",
        key_name: "endcast",
        group: "Completion",
    },
    FieldMetadata {
        label: "No Intermission",
        key_name: "nointermission",
        group: "Completion",
    },
    FieldMetadata {
        label: "Inter. Text",
        key_name: "intertext",
        group: "Advanced",
    },
    FieldMetadata {
        label: "Inter. Text (Secret)",
        key_name: "intertextsecret",
        group: "Advanced",
    },
    FieldMetadata {
        label: "Boss Action",
        key_name: "bossaction",
        group: "Advanced",
    },
    FieldMetadata {
        label: "Boss EdNum",
        key_name: "bossactionednum",
        group: "Advanced",
    },
    FieldMetadata {
        label: "Episode Menu",
        key_name: "episode",
        group: "Advanced",
    },
];

/// Renders the high-level editor for UMAPINFO map entries in the Properties Panel.
pub fn draw_umapinfo_editor(
    ui: &mut egui::Ui,
    file: &mut UmapInfoFile,
    selection_path: &[usize],
    _assets: &AssetStore,
    _state: &PreviewState,
) -> bool {
    let mut changed = false;

    if let Some(map) = file.data.maps.get_mut(selection_path[0]) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 180.0).max(0.0) / 2.0);
                ui.label("Map ID:");
                if ui
                    .add(egui::TextEdit::singleline(&mut map.mapname).desired_width(100.0))
                    .changed()
                {
                    map.mapname = map.mapname.to_uppercase();
                    changed = true;
                }
            });
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Keys: {}", map.fields.len())).weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let id = ui.make_persistent_id("add_umap_key_menu");
                    let btn = shared::combobox_button(ui, "Add Key...", 110.0);
                    if btn.clicked() {
                        ContextMenu::open(ui, id, btn.rect.left_bottom());
                    }

                    if let Some(menu) = ContextMenu::get(ui, id) {
                        ContextMenu::show(ui, menu, btn.clicked(), |ui| {
                            changed |= draw_add_field_menu(ui, map);
                        });
                    }
                });
            });
            ui.separator();
            ui.add_space(4.0);

            let mut to_remove = None;
            for (idx, field) in map.fields.iter_mut().enumerate() {
                ui.push_id(idx, |ui| {
                    changed |= draw_field_card(ui, field, &mut to_remove, idx);
                });
                ui.add_space(4.0);
            }

            if let Some(idx) = to_remove {
                map.fields.remove(idx);
                changed = true;
            }
        });
    }
    changed
}

/// Renders an individual Field Card using the unified UI style.
fn draw_field_card(
    ui: &mut egui::Ui,
    field: &mut UmapField,
    remove_idx: &mut Option<usize>,
    idx: usize,
) -> bool {
    let mut changed = false;
    let frame = egui::Frame::NONE
        .fill(egui::Color32::from_white_alpha(5))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(15)))
        .inner_margin(6.0)
        .corner_radius(4.0);

    frame.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let del_btn = egui::Button::new("X").min_size(egui::vec2(18.0, 18.0));
                    if ui.add(del_btn).on_hover_text("Remove Key").clicked() {
                        *remove_idx = Some(idx);
                    }

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let id = ui.make_persistent_id(format!("change_field_type_{}", idx));
                        let label = FIELD_REGISTRY
                            .iter()
                            .find(|m| m.key_name == field.key_name())
                            .map(|m| m.label)
                            .unwrap_or(field.key_name());

                        let btn_res = shared::combobox_button(ui, label, ui.available_width());

                        if btn_res.clicked() {
                            ContextMenu::open(ui, id, btn_res.rect.left_bottom());
                        }

                        if let Some(menu) = ContextMenu::get(ui, id) {
                            ContextMenu::show(ui, menu, btn_res.clicked(), |ui| {
                                changed |= draw_change_field_menu(ui, field);
                            });
                        }
                    });
                });
            });

            shared::draw_separator_line(ui);
            ui.add_space(2.0);

            if let Some(s) = field.as_string_mut() {
                if ui
                    .add(egui::TextEdit::singleline(s).desired_width(ui.available_width()))
                    .changed()
                {
                    changed = true;
                }
            } else {
                match field {
                    UmapField::Label(s) => {
                        ui.horizontal(|ui| {
                            let mut is_clear = s == "clear";
                            if ui.checkbox(&mut is_clear, "Clear").changed() {
                                *s = if is_clear {
                                    "clear".to_string()
                                } else {
                                    String::new()
                                };
                                changed = true;
                            }
                            if !is_clear {
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(s)
                                            .desired_width(ui.available_width()),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            }
                        });
                    }
                    UmapField::InterText(lines) | UmapField::InterTextSecret(lines) => {
                        ui.vertical(|ui| {
                            let mut is_clear = lines.len() == 1 && lines[0] == "clear";
                            if ui.checkbox(&mut is_clear, "Clear / Disable").changed() {
                                if is_clear {
                                    *lines = vec!["clear".to_string()];
                                } else {
                                    *lines = vec!["New intermission text...".to_string()];
                                }
                                changed = true;
                            }

                            if !is_clear {
                                let mut text_buf = lines.join("\n");
                                if ui
                                    .add(
                                        egui::TextEdit::multiline(&mut text_buf)
                                            .desired_width(ui.available_width())
                                            .desired_rows(3),
                                    )
                                    .changed()
                                {
                                    *lines = text_buf.lines().map(|s| s.to_string()).collect();
                                    changed = true;
                                }
                            }
                        });
                    }
                    UmapField::ParTime(v) => {
                        if ui.add(egui::DragValue::new(v).suffix("s")).changed() {
                            changed = true;
                        }
                    }
                    UmapField::EndGame(v)
                    | UmapField::EndBunny(v)
                    | UmapField::EndCast(v)
                    | UmapField::NoIntermission(v) => {
                        if ui.checkbox(v, "Enabled").changed() {
                            changed = true;
                        }
                    }
                    UmapField::BossAction {
                        thing,
                        special,
                        tag,
                    }
                    | UmapField::BossActionEdNum {
                        ednum: thing,
                        special,
                        tag,
                    } => {
                        ui.horizontal(|ui| {
                            ui.label("ID:");
                            if ui
                                .add(egui::TextEdit::singleline(thing).desired_width(70.0))
                                .changed()
                            {
                                changed = true;
                            }
                            ui.label("Spc:");
                            if ui.add(egui::DragValue::new(special)).changed() {
                                changed = true;
                            }
                            ui.label("Tag:");
                            if ui.add(egui::DragValue::new(tag)).changed() {
                                changed = true;
                            }
                        });
                    }
                    UmapField::Episode { patch, name, key } => {
                        ui.horizontal(|ui| {
                            ui.label("Pat:");
                            ui.add(egui::TextEdit::singleline(patch).desired_width(40.0));
                            ui.label("Nam:");
                            ui.add(egui::TextEdit::singleline(name).desired_width(60.0));
                            ui.label("Key:");
                            ui.add(egui::TextEdit::singleline(key).desired_width(20.0));
                        });
                    }
                    _ => {}
                }
            }
        });
    });
    changed
}

/// Renders the categorized dropdown to add new UMAPINFO keys.
fn draw_add_field_menu(ui: &mut egui::Ui, map: &mut MapEntry) -> bool {
    let mut changed = false;
    let groups = [
        "Metadata",
        "Graphics",
        "Audio",
        "Progression",
        "Completion",
        "Advanced",
    ];

    for group in groups {
        ui.label(egui::RichText::new(group).weak().size(10.0));
        for meta in FIELD_REGISTRY.iter().filter(|m| m.group == group) {
            if common::custom_menu_item(ui, meta.label, false) {
                map.fields.push(create_default_field(meta.key_name));
                changed = true;
                ContextMenu::close(ui);
            }
        }
        ui.separator();
    }
    changed
}

/// Renders the scrollable dropdown to change an existing field's type.
fn draw_change_field_menu(ui: &mut egui::Ui, field: &mut UmapField) -> bool {
    let mut changed = false;
    let current_key = field.key_name();

    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            for meta in FIELD_REGISTRY {
                if common::custom_menu_item(ui, meta.label, current_key == meta.key_name) {
                    let old_val = field.as_string_mut().cloned();
                    let mut new_field = create_default_field(meta.key_name);
                    if let (Some(val), Some(new_val_ref)) = (old_val, new_field.as_string_mut()) {
                        *new_val_ref = val;
                    }

                    *field = new_field;
                    changed = true;
                    ContextMenu::close(ui);
                }
            }
        });
    changed
}

/// Internal factory to create a field with its standard spec-default value.
fn create_default_field(key: &str) -> UmapField {
    match key {
        "levelname" => UmapField::LevelName(String::new()),
        "author" => UmapField::Author(String::new()),
        "skytexture" => UmapField::SkyTexture(String::new()),
        "music" => UmapField::Music(String::new()),
        "exitpic" => UmapField::ExitPic(String::new()),
        "enterpic" => UmapField::EnterPic(String::new()),
        "levelpic" => UmapField::LevelPic(String::new()),
        "endpic" => UmapField::EndPic(String::new()),
        "interbackdrop" => UmapField::InterBackdrop(String::new()),
        "intermusic" => UmapField::InterMusic(String::new()),
        "next" => UmapField::Next(String::new()),
        "nextsecret" => UmapField::NextSecret(String::new()),
        "label" => UmapField::Label("clear".to_string()),
        "intertextsecret" => UmapField::InterTextSecret(vec!["clear".to_string()]),
        "partime" => UmapField::ParTime(0),
        "endgame" => UmapField::EndGame(true),
        "endbunny" => UmapField::EndBunny(true),
        "endcast" => UmapField::EndCast(true),
        "nointermission" => UmapField::NoIntermission(true),
        "intertext" => UmapField::InterText(vec!["clear".to_string()]),
        "bossaction" => UmapField::BossAction {
            thing: "Cyberdemon".into(),
            special: 0,
            tag: 0,
        },
        "bossactionednum" => UmapField::BossActionEdNum {
            ednum: "7".into(),
            special: 0,
            tag: 0,
        },
        "episode" => UmapField::Episode {
            patch: "M_EPI1".into(),
            name: "Episode 1".into(),
            key: "e".into(),
        },
        _ => UmapField::LevelName(String::new()),
    }
}

impl LumpUI for UmapInfoFile {
    fn draw_properties(&mut self, ui: &mut egui::Ui, ctx: &PropertyContext) -> bool {
        if let Some(path) = ctx.selection.iter().next() {
            return draw_umapinfo_editor(ui, self, path, ctx.assets, ctx.state);
        }
        false
    }

    fn draw_layer_list(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut LayerContext,
    ) -> (Vec<DocumentAction>, bool) {
        let mut actions = Vec::new();
        if shared::heading_action_button(ui, "Maps", Some("Add Map"), false).clicked() {
            actions.push(DocumentAction::UndoSnapshot);
            actions.push(DocumentAction::Umap(UmapAction::AddMap));
        }

        egui::ScrollArea::vertical()
            .id_salt("umapinfo_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                crate::ui::layers::umapinfo::draw_umapinfo_layers_list(
                    ui,
                    self,
                    ctx.selection,
                    ctx.current_item_idx,
                    ctx.assets,
                    &mut actions,
                    ctx.confirmation_modal,
                );
            });
        (actions, false)
    }

    fn header_info(&self, _selection: &HashSet<Vec<usize>>) -> (String, String, egui::Color32) {
        (
            "UMAPINFO".to_string(),
            "Universal Map Info Definition".to_string(),
            egui::Color32::from_rgb(60, 40, 40),
        )
    }

    fn render_viewport(&self, ui: &mut egui::Ui, ctx: &mut ViewportContext) -> Vec<DocumentAction> {
        let mut actions = Vec::new();
        let painter = ui.painter();

        let grid_spacing = 20.0 * ctx.proj.final_scale_x;
        let dot_color = egui::Color32::from_gray(30);
        let viewport_rect = ctx.viewport_res.rect;

        let start_x = viewport_rect.left() + (ctx.proj.screen_rect.left() % grid_spacing);
        let start_y = viewport_rect.top() + (ctx.proj.screen_rect.top() % grid_spacing);

        for x in 0..=(viewport_rect.width() / grid_spacing) as i32 + 1 {
            for y in 0..=(viewport_rect.height() / grid_spacing) as i32 + 1 {
                let center = egui::pos2(
                    start_x + x as f32 * grid_spacing,
                    start_y + y as f32 * grid_spacing,
                );
                if viewport_rect.contains(center) {
                    painter.circle_filled(center, 1.0 * ctx.proj.final_scale_x.max(0.5), dot_color);
                }
            }
        }

        let drag_id = egui::Id::new("umap_node_drag_global");
        let start_pos_id = egui::Id::new("umap_drag_start_ptr_global");
        let node_start_id = egui::Id::new("umap_node_start_v_global");

        let mut dragged_node: Option<String> = ui.ctx().data(|d| d.get_temp(drag_id));
        let mut start_ptr: Option<egui::Pos2> = ui.ctx().data(|d| d.get_temp(start_pos_id));
        let mut node_start: Option<egui::Pos2> = ui.ctx().data(|d| d.get_temp(node_start_id));

        let mut map_positions = std::collections::HashMap::new();
        let mut lower_positions = std::collections::HashMap::new();
        for (i, map) in self.data.maps.iter().enumerate() {
            let (vx, vy) = self.get_node_pos(&map.mapname, i);
            map_positions.insert(map.mapname.clone(), (vx, vy));
            lower_positions.insert(map.mapname.to_lowercase(), (vx, vy));
        }

        if ctx.viewport_res.drag_started() && !ctx.is_panning {
            if let Some(pos) = ctx.viewport_res.hover_pos() {
                for map in self.data.maps.iter().rev() {
                    let (vx, vy) = map_positions.get(&map.mapname).copied().unwrap();
                    let node_rect = egui::Rect::from_min_max(
                        ctx.proj.to_screen(egui::pos2(vx, vy)),
                        ctx.proj.to_screen(egui::pos2(vx + 120.0, vy + 40.0)),
                    );

                    if node_rect.contains(pos) {
                        dragged_node = Some(map.mapname.clone());
                        start_ptr = Some(pos);
                        node_start = Some(egui::pos2(vx, vy));
                        actions.push(DocumentAction::UndoSnapshot);
                        break;
                    }
                }
            }
        }

        if let Some(ref mapname) = dragged_node {
            if ctx.viewport_res.dragged() {
                if let (Some(ptr_start), Some(n_start), Some(current_ptr)) =
                    (start_ptr, node_start, ui.input(|i| i.pointer.latest_pos()))
                {
                    let delta_screen = current_ptr - ptr_start;
                    let delta_v = egui::vec2(
                        delta_screen.x / ctx.proj.final_scale_x,
                        delta_screen.y / ctx.proj.final_scale_y,
                    );

                    let mut target_vx = n_start.x + delta_v.x;
                    let mut target_vy = n_start.y + delta_v.y;

                    if !ui.input(|i| i.modifiers.alt) {
                        target_vx = (target_vx / 20.0).round() * 20.0;
                        target_vy = (target_vy / 20.0).round() * 20.0;
                    }

                    if let Some(&(current_vx, current_vy)) = map_positions.get(mapname) {
                        if (target_vx - current_vx).abs() > 0.01
                            || (target_vy - current_vy).abs() > 0.01
                        {
                            map_positions.insert(mapname.clone(), (target_vx, target_vy));
                            lower_positions.insert(mapname.to_lowercase(), (target_vx, target_vy));

                            actions.push(DocumentAction::Umap(UmapAction::UpdateNodePos(
                                mapname.clone(),
                                target_vx,
                                target_vy,
                            )));
                        }
                    }
                    ui.ctx().request_repaint();
                }
            }
        }

        if ctx.viewport_res.drag_stopped() {
            dragged_node = None;
            start_ptr = None;
            node_start = None;
        }

        ui.ctx().data_mut(|d| {
            if let Some(val) = dragged_node {
                d.insert_temp(drag_id, val);
            } else {
                d.remove::<String>(drag_id);
            }
            if let Some(val) = start_ptr {
                d.insert_temp(start_pos_id, val);
            } else {
                d.remove::<egui::Pos2>(start_pos_id);
            }
            if let Some(val) = node_start {
                d.insert_temp(node_start_id, val);
            } else {
                d.remove::<egui::Pos2>(node_start_id);
            }
        });

        for map in &self.data.maps {
            let src_pos = map_positions
                .get(&map.mapname)
                .copied()
                .unwrap_or((0.0, 0.0));

            for field in &map.fields {
                if let UmapField::Next(target) | UmapField::NextSecret(target) = field {
                    if let Some(&dst_pos) = lower_positions.get(&target.to_lowercase()) {
                        let is_secret = matches!(field, UmapField::NextSecret(_));

                        let src_v_pt = if is_secret {
                            egui::pos2(src_pos.0 + 60.0, src_pos.1 + 40.0)
                        } else {
                            egui::pos2(src_pos.0 + 120.0, src_pos.1 + 20.0)
                        };

                        let mut dst_v_pt = if is_secret {
                            egui::pos2(dst_pos.0 + 60.0, dst_pos.1)
                        } else {
                            egui::pos2(dst_pos.0, dst_pos.1 + 20.0)
                        };

                        if !is_secret {
                            if src_pos.1 > dst_pos.1 + 10.0 {
                                dst_v_pt.y += 8.0;
                            } else if src_pos.1 < dst_pos.1 - 10.0 {
                                dst_v_pt.y -= 8.0;
                            }
                        }

                        let src_screen = ctx.proj.to_screen(src_v_pt);
                        let dst_screen = ctx.proj.to_screen(dst_v_pt);
                        let color = if is_secret {
                            egui::Color32::from_rgb(200, 100, 200)
                        } else {
                            egui::Color32::from_rgb(100, 200, 100)
                        };

                        let (cp1, cp2) = if is_secret {
                            let dist = (dst_screen.y - src_screen.y).abs().max(60.0) * 0.5;
                            (
                                src_screen + egui::vec2(0.0, dist),
                                dst_screen - egui::vec2(0.0, dist),
                            )
                        } else {
                            let dist = (dst_screen.x - src_screen.x).abs().max(60.0) * 0.5;
                            (
                                src_screen + egui::vec2(dist, 0.0),
                                dst_screen - egui::vec2(dist, 0.0),
                            )
                        };

                        painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                            [src_screen, cp1, cp2, dst_screen],
                            false,
                            egui::Color32::TRANSPARENT,
                            egui::Stroke::new(2.5 * ctx.proj.final_scale_x, color),
                        ));
                    }
                }
            }
        }

        for (i, map) in self.data.maps.iter().enumerate() {
            let (vx, vy) = map_positions
                .get(&map.mapname)
                .copied()
                .unwrap_or((0.0, 0.0));
            let node_rect = egui::Rect::from_min_max(
                ctx.proj.to_screen(egui::pos2(vx, vy)),
                ctx.proj.to_screen(egui::pos2(vx + 120.0, vy + 40.0)),
            );

            let is_selected = ctx.current_item_idx == i;
            let bg_color = if is_selected {
                egui::Color32::from_rgb(60, 80, 120)
            } else {
                egui::Color32::from_rgb(45, 45, 45)
            };
            let rounding = 4.0 * ctx.proj.final_scale_x;

            painter.rect_filled(node_rect, rounding, bg_color);
            painter.rect_stroke(
                node_rect,
                rounding,
                egui::Stroke::new(
                    1.0,
                    if is_selected {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_gray(80)
                    },
                ),
                egui::StrokeKind::Inside,
            );

            painter.text(
                node_rect.center(),
                egui::Align2::CENTER_CENTER,
                &map.mapname,
                egui::FontId::proportional(14.0 * ctx.proj.final_scale_x),
                egui::Color32::WHITE,
            );
        }

        actions
    }
}
