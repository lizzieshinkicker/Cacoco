use crate::document::actions::{DocumentAction, TreeAction};
use crate::models::umap_graph::{NodeType, UmapNode};
use crate::models::umapinfo::{UmapField, UmapInfoFile};
use crate::ui::properties::editor::ViewportContext;

use super::connectors::{
    ConnectorType, detect_connector_hit, map_has_normal_exit, map_has_secret_exit,
};
use super::constants::{CENTER_ON_NODE_KEY, CENTER_TARGET_KEY, NODE_HEIGHT, NODE_WIDTH};

#[derive(Debug, Clone)]
pub struct NodeHit {
    pub node: UmapNode,
}

/// Detects if a point hits any node's body.
/// Returns the first hit found (iterates in reverse order for z-order).
pub fn detect_node_hit(
    point: eframe::egui::Pos2,
    graph: &crate::models::umap_graph::UmapGraph,
    node_rects: &std::collections::HashMap<String, eframe::egui::Rect>,
) -> Option<NodeHit> {
    for node in graph.nodes.iter().rev() {
        if let Some(rect) = node_rects.get(&node.id) {
            if rect.contains(point) {
                return Some(NodeHit { node: node.clone() });
            }
        }
    }
    None
}

/// Checks if a position is on the empty field (not on any node).
pub fn is_position_on_blank(
    point: eframe::egui::Pos2,
    graph: &crate::models::umap_graph::UmapGraph,
    node_rects: &std::collections::HashMap<String, eframe::egui::Rect>,
) -> bool {
    for node in graph.nodes.iter() {
        if let Some(rect) = node_rects.get(&node.id) {
            if rect.contains(point) {
                return false;
            }
        }
    }
    true
}

/// Extracts the exit target name from a map for the given connector type.
pub fn get_exit_target_name(
    file: &UmapInfoFile,
    map_id: &str,
    conn_type: ConnectorType,
) -> Option<String> {
    let map = file
        .data
        .maps
        .iter()
        .find(|m| m.mapname.to_uppercase() == map_id.to_uppercase())?;

    match conn_type {
        ConnectorType::NormalExit => map.fields.iter().find_map(|f| {
            if let UmapField::Next(t) = f {
                Some(t.clone())
            } else {
                None
            }
        }),
        ConnectorType::SecretExit => map.fields.iter().find_map(|f| {
            if let UmapField::NextSecret(t) = f {
                Some(t.clone())
            } else {
                None
            }
        }),
    }
}

/// Extracts the map ID from a node ID if it's a non-MAP node type.
/// Returns Some(map_id) for Episode, InterText, and Terminal nodes.
pub fn extract_map_id_from_node(node: &UmapNode) -> Option<String> {
    if let NodeType::Map { .. } = &node.node_type {
        return Some(node.id.clone());
    }

    let id = &node.id;
    if let Some(pos) = id.find("::") {
        Some(id[..pos].to_uppercase())
    } else {
        None
    }
}

/// Finds the index of the Map entry in the file that corresponds to the given node.
/// For Map nodes, returns the index of that map.
/// For other node types (Episode, InterText, Terminal), returns the index of the parent Map.
pub fn find_map_index_for_node(file: &UmapInfoFile, node: &UmapNode) -> Option<usize> {
    let map_id = extract_map_id_from_node(node)?;
    file.data
        .maps
        .iter()
        .position(|m| m.mapname.to_uppercase() == map_id)
}

/// Helper to create a selection action for a map index.
pub fn select_map_action(idx: usize) -> DocumentAction {
    DocumentAction::Tree(TreeAction::Select(vec![vec![idx]]))
}

/// Centers the viewport on whatever map is selected from the list of maps.
pub fn handle_center_request(
    ui: &mut eframe::egui::Ui,
    file: &UmapInfoFile,
    graph: &crate::models::umap_graph::UmapGraph,
    ctx: &mut ViewportContext,
) {
    let center_idx: Option<usize> = ui
        .ctx()
        .data(|d| d.get_temp(eframe::egui::Id::new(CENTER_ON_NODE_KEY)));

    if let Some(idx) = center_idx {
        if let Some(map) = file.data.maps.get(idx) {
            let map_id = map.mapname.to_uppercase();

            for node in &graph.nodes {
                if node.id == map_id {
                    let viewport_rect = ctx.viewport_res.rect;
                    let center = viewport_rect.center();

                    let zoom = ctx.proj.final_scale_x;

                    let node_screen_w = NODE_WIDTH * zoom;
                    let node_screen_h = NODE_HEIGHT * zoom;

                    let node_screen_x = node.x * zoom;
                    let node_screen_y = node.y * zoom;

                    let new_pan_offset = eframe::egui::vec2(
                        center.x - viewport_rect.min.x - node_screen_x - node_screen_w * 0.5,
                        center.y - viewport_rect.min.y - node_screen_y - node_screen_h * 0.5,
                    );

                    ui.ctx().data_mut(|d| {
                        d.insert_temp(eframe::egui::Id::new(CENTER_TARGET_KEY), new_pan_offset);
                    });

                    ui.ctx().data_mut(|d| {
                        d.remove::<usize>(eframe::egui::Id::new(CENTER_ON_NODE_KEY));
                    });

                    break;
                }
            }
        }
    }

    let target: Option<eframe::egui::Vec2> = ui
        .ctx()
        .data(|d| d.get_temp(eframe::egui::Id::new(CENTER_TARGET_KEY)));

    if let Some(target_offset) = target {
        let current_offset = ctx.state.sim.engine.pan_offset;
        let diff = target_offset - current_offset;
        let dist = diff.length();

        if dist > 0.5 {
            let t = (10.0 * ui.input(|i| i.stable_dt)).min(1.0);
            ctx.state.sim.engine.pan_offset = current_offset + diff * t;
            ui.ctx().request_repaint();
        } else {
            ctx.state.sim.engine.pan_offset = target_offset;
            ui.ctx().data_mut(|d| {
                d.remove::<eframe::egui::Vec2>(eframe::egui::Id::new(CENTER_TARGET_KEY));
            });
        }
    }
}

/// Processes all mouse inputs, drag-and-drop, selection, and viewport panning.
pub fn handle_interaction(
    ui: &mut eframe::egui::Ui,
    file: &UmapInfoFile,
    graph: &crate::models::umap_graph::UmapGraph,
    node_rects: &std::collections::HashMap<String, eframe::egui::Rect>,
    ctx: &mut ViewportContext,
) -> Vec<DocumentAction> {
    let mut actions = Vec::new();

    let drag_id = eframe::egui::Id::new("umap_node_drag_global");
    let start_pos_id = eframe::egui::Id::new("umap_drag_start_ptr_global");
    let node_start_id = eframe::egui::Id::new("umap_node_start_v_global");
    let bg_pan_id = eframe::egui::Id::new("umap_bg_pan_global");
    let connector_drag_data_id = eframe::egui::Id::new("umap_connector_drag_data");

    let mut dragged_node: Option<String> = ui.ctx().data(|d| d.get_temp(drag_id));
    let mut start_ptr: Option<eframe::egui::Pos2> = ui.ctx().data(|d| d.get_temp(start_pos_id));
    let mut node_start: Option<eframe::egui::Pos2> = ui.ctx().data(|d| d.get_temp(node_start_id));
    let mut is_bg_panning: bool = ui.ctx().data(|d| d.get_temp(bg_pan_id).unwrap_or(false));

    let active_connector_drag: Option<(String, ConnectorType, eframe::egui::Pos2)> =
        ui.ctx().data(|d| d.get_temp(connector_drag_data_id));

    let _is_primary_down = ui.input(|i| i.pointer.primary_down());

    let interact_pos = ui.input(|i| i.pointer.interact_pos());

    if active_connector_drag.is_none() && !ctx.is_panning {
        if let Some(pos) = interact_pos.or(ctx.viewport_res.hover_pos()) {
            let just_pressed = ui.input(|i| {
                i.events.iter().any(|e| {
                    matches!(
                        e,
                        eframe::egui::Event::PointerButton {
                            button: eframe::egui::PointerButton::Primary,
                            pressed: true,
                            ..
                        }
                    )
                })
            });

            if just_pressed {
                let scale = ctx.proj.final_scale_x;
                if let Some(hit) = detect_connector_hit(pos, graph, node_rects, scale) {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            connector_drag_data_id,
                            (hit.node_id, hit.connector_type, hit.position),
                        );
                    });
                    actions.push(DocumentAction::UndoSnapshot);
                    return actions;
                }
            }
        }
    }

    if ctx
        .viewport_res
        .drag_started_by(eframe::egui::PointerButton::Primary)
        && !ctx.is_panning
    {
        let click_pos = interact_pos.or(ctx.viewport_res.hover_pos());

        if let Some(pos) = click_pos {
            let hit_connector =
                detect_connector_hit(pos, graph, node_rects, ctx.proj.final_scale_x).is_some();

            if !hit_connector {
                if let Some(node_hit) = detect_node_hit(pos, graph, node_rects) {
                    dragged_node = Some(node_hit.node.id.clone());
                    start_ptr = Some(pos);
                    node_start = Some(eframe::egui::pos2(node_hit.node.x, node_hit.node.y));

                    if let Some(idx) = find_map_index_for_node(file, &node_hit.node) {
                        actions.push(select_map_action(idx));
                    }

                    actions.push(DocumentAction::UndoSnapshot);
                } else {
                    is_bg_panning = true;
                }
            }
        }
    }

    let primary_pressed_now = ui.input(|i| i.pointer.primary_down());
    let was_dragging = ctx
        .viewport_res
        .dragged_by(eframe::egui::PointerButton::Primary);
    if primary_pressed_now && !was_dragging && !ctx.is_panning && dragged_node.is_none() {
        let click_pos = interact_pos.or(ctx.viewport_res.hover_pos());
        if let Some(pos) = click_pos {
            let hit_connector =
                detect_connector_hit(pos, graph, node_rects, ctx.proj.final_scale_x).is_some();

            if !hit_connector {
                if let Some(node_hit) = detect_node_hit(pos, graph, node_rects) {
                    if let Some(idx) = find_map_index_for_node(file, &node_hit.node) {
                        actions.push(select_map_action(idx));
                    }
                }
            }
        }
    }

    let bg_menu_id = eframe::egui::Id::new("umap_bg_context_menu");
    let menu_valid_id = eframe::egui::Id::new("umap_bg_menu_valid");

    let viewport_rect = ctx.viewport_res.rect;
    let bg_response = ui.interact(viewport_rect, bg_menu_id, eframe::egui::Sense::click());

    let just_opened = crate::ui::context_menu::ContextMenu::check(ui, &bg_response);

    if just_opened {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let is_blank = is_position_on_blank(pos, graph, node_rects);
            ui.data_mut(|d| d.insert_temp(menu_valid_id, is_blank));
        }
    }

    let menu_valid: bool = ui
        .ctx()
        .data(|d| d.get_temp(menu_valid_id).unwrap_or(false));

    if let Some(menu) = crate::ui::context_menu::ContextMenu::get(ui, bg_menu_id) {
        if menu_valid {
            let click_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or_default();
            let virtual_pos = ctx.proj.to_virtual(click_pos);

            crate::ui::context_menu::ContextMenu::show(ui, menu, just_opened, |ui| {
                if crate::ui::context_menu::ContextMenu::button(ui, "New Map", true) {
                    actions.push(DocumentAction::UndoSnapshot);
                    actions.push(DocumentAction::Umap(
                        crate::document::actions::UmapAction::AddMap {
                            x: virtual_pos.x,
                            y: virtual_pos.y,
                        },
                    ));
                    crate::ui::context_menu::ContextMenu::close(ui);
                }
            });
        }
        if !crate::ui::context_menu::ContextMenu::get(ui, bg_menu_id).is_some() {
            ui.data_mut(|d| d.remove::<bool>(menu_valid_id));
        }
    }

    if is_bg_panning && active_connector_drag.is_none() {
        if ctx
            .viewport_res
            .dragged_by(eframe::egui::PointerButton::Primary)
        {
            ui.ctx().set_cursor_icon(eframe::egui::CursorIcon::Grabbing);
            ctx.state.sim.engine.pan_offset += ui.input(|i| i.pointer.delta());
            ui.ctx().request_repaint();
        }
    }

    if let Some(ref node_id) = dragged_node {
        if ctx
            .viewport_res
            .dragged_by(eframe::egui::PointerButton::Primary)
        {
            if let (Some(ptr_start), Some(n_start), Some(current_ptr)) =
                (start_ptr, node_start, ui.input(|i| i.pointer.latest_pos()))
            {
                let delta_screen = current_ptr - ptr_start;
                let delta_v = eframe::egui::vec2(
                    delta_screen.x / ctx.proj.final_scale_x,
                    delta_screen.y / ctx.proj.final_scale_y,
                );

                let target_vx = ((n_start.x + delta_v.x) / 20.0).round() * 20.0;
                let target_vy = ((n_start.y + delta_v.y) / 20.0).round() * 20.0;

                actions.push(DocumentAction::Umap(
                    crate::document::actions::UmapAction::UpdateNodePos(
                        node_id.clone(),
                        target_vx,
                        target_vy,
                    ),
                ));
                ui.ctx().request_repaint();
            }
        }
    }

    let _connector_drag_id = eframe::egui::Id::new("umap_connector_drag");

    let active_connector_drag: Option<(String, ConnectorType, eframe::egui::Pos2)> =
        ui.ctx().data(|d| d.get_temp(connector_drag_data_id));

    let connector_context_id = eframe::egui::Id::new("umap_connector_context");

    let click_pos = ui
        .input(|i| i.pointer.interact_pos())
        .or(ctx.viewport_res.hover_pos());

    if active_connector_drag.is_none()
        && ctx
            .viewport_res
            .drag_started_by(eframe::egui::PointerButton::Primary)
        && !ctx.is_panning
        && dragged_node.is_none()
    {
        if let Some(pos) = click_pos {
            if let Some(hit) = detect_connector_hit(pos, graph, node_rects, ctx.proj.final_scale_x)
            {
                ui.ctx().data_mut(|d| {
                    d.insert_temp(
                        connector_drag_data_id,
                        (hit.node_id, hit.connector_type, hit.position),
                    );
                });
                actions.push(DocumentAction::UndoSnapshot);
            }
        }
    }

    if ctx
        .viewport_res
        .drag_started_by(eframe::egui::PointerButton::Secondary)
        && !ctx.is_panning
    {
        if let Some(pos) = ctx.viewport_res.hover_pos() {
            if let Some(hit) = detect_connector_hit(pos, graph, node_rects, ctx.proj.final_scale_x)
            {
                let map_id = hit.node_id;
                let has_exit = match hit.connector_type {
                    ConnectorType::NormalExit => map_has_normal_exit(file, &map_id),
                    ConnectorType::SecretExit => map_has_secret_exit(file, &map_id),
                };

                if has_exit {
                    let target_name =
                        get_exit_target_name(file, &map_id, hit.connector_type).unwrap_or_default();
                    ui.data_mut(|d| {
                        d.insert_temp(
                            connector_context_id,
                            (map_id, hit.connector_type, target_name),
                        )
                    });
                }
            }
        }
    }

    let connector_context_data: Option<(String, ConnectorType, String)> =
        ui.ctx().data(|d| d.get_temp(connector_context_id));
    if let Some((map_name, conn_type, target_name)) = connector_context_data {
        let _menu_response = ui.interact(
            eframe::egui::Rect::from_min_size(
                eframe::egui::pos2(0.0, 0.0),
                eframe::egui::vec2(1.0, 1.0),
            ),
            connector_context_id,
            eframe::egui::Sense::click(),
        );

        if let Some(menu) = crate::ui::context_menu::ContextMenu::get(ui, connector_context_id) {
            crate::ui::context_menu::ContextMenu::show(ui, menu, false, |ui| {
                let clear_label = match conn_type {
                    ConnectorType::NormalExit => format!("Clear Normal Exit (to {})", target_name),
                    ConnectorType::SecretExit => format!("Clear Secret Exit (to {})", target_name),
                };
                if crate::ui::context_menu::ContextMenu::button(ui, &clear_label, true) {
                    actions.push(DocumentAction::UndoSnapshot);
                    match conn_type {
                        ConnectorType::NormalExit => {
                            actions.push(DocumentAction::Umap(
                                crate::document::actions::UmapAction::ClearNormalExit(
                                    map_name.clone(),
                                ),
                            ));
                        }
                        ConnectorType::SecretExit => {
                            actions.push(DocumentAction::Umap(
                                crate::document::actions::UmapAction::ClearSecretExit(
                                    map_name.clone(),
                                ),
                            ));
                        }
                    }
                    crate::ui::context_menu::ContextMenu::close(ui);
                }
            });
        }

        if !ui.input(|i| i.pointer.primary_down())
            && !crate::ui::context_menu::ContextMenu::get(ui, connector_context_id).is_some()
        {
            ui.ctx()
                .data_mut(|d| d.remove::<(String, ConnectorType, String)>(connector_context_id));
        }
    }

    if let Some((source_node_id, conn_type, start_pos)) = active_connector_drag {
        if let Some(current_pos) = ui.input(|i| i.pointer.latest_pos()) {
            let color = match conn_type {
                ConnectorType::NormalExit => super::constants::NORMAL_EXIT_COLOR,
                ConnectorType::SecretExit => super::constants::SECRET_EXIT_COLOR,
            };

            ui.painter().line_segment(
                [start_pos, current_pos],
                eframe::egui::Stroke::new(2.0, color),
            );

            ui.ctx().request_repaint();
        }

        if ctx.viewport_res.drag_stopped() {
            if let Some(pos) = ctx.viewport_res.hover_pos() {
                for target_node in graph.nodes.iter() {
                    if let Some(rect) = node_rects.get(&target_node.id) {
                        if rect.contains(pos) {
                            if let NodeType::Map { .. } = &target_node.node_type {
                                let source_map_id = extract_map_id_from_node(
                                    &graph.nodes.iter().find(|n| n.id == source_node_id).unwrap(),
                                )
                                .unwrap_or(source_node_id.clone());

                                match conn_type {
                                    ConnectorType::NormalExit => {
                                        actions.push(DocumentAction::Umap(
                                            crate::document::actions::UmapAction::SetNormalExit {
                                                map_name: source_map_id,
                                                target: target_node.id.clone(),
                                            },
                                        ));
                                    }
                                    ConnectorType::SecretExit => {
                                        let source_map = if source_node_id.contains("::TEXT_SECRET")
                                        {
                                            extract_map_id_from_node(
                                                &graph
                                                    .nodes
                                                    .iter()
                                                    .find(|n| n.id == source_node_id)
                                                    .unwrap(),
                                            )
                                            .unwrap_or(source_node_id)
                                        } else {
                                            source_map_id
                                        };
                                        actions.push(DocumentAction::Umap(
                                            crate::document::actions::UmapAction::SetSecretExit {
                                                map_name: source_map,
                                                target: target_node.id.clone(),
                                            },
                                        ));
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }

            ui.ctx().data_mut(|d| {
                d.remove::<(String, ConnectorType, eframe::egui::Pos2)>(connector_drag_data_id)
            });
        }
    }

    if ctx.viewport_res.drag_stopped() {
        dragged_node = None;
        start_ptr = None;
        node_start = None;
        is_bg_panning = false;
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
            d.remove::<eframe::egui::Pos2>(start_pos_id);
        }
        if let Some(val) = node_start {
            d.insert_temp(node_start_id, val);
        } else {
            d.remove::<eframe::egui::Pos2>(node_start_id);
        }
        d.insert_temp(bg_pan_id, is_bg_panning);
    });

    actions
}
