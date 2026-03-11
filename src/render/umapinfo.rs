use crate::document::actions::{DocumentAction, UmapAction};
use crate::models::umap_graph::{EdgeType, NodeType, UmapGraph, UmapNode};
use crate::models::umapinfo::UmapInfoFile;
use crate::ui::context_menu::ContextMenu;
use crate::ui::properties::editor::ViewportContext;
use eframe::egui;
use std::collections::HashMap;

/// Key for storing the center-on-node request in egui context.
const CENTER_ON_NODE_KEY: &str = "umap_center_on_node";

/// Lightens a color by a given amount (0.0 to 1.0).
fn lighten_color(color: egui::Color32, amount: f32) -> egui::Color32 {
    let [r, g, b, _a] = color.to_array();
    let add = (amount * 255.0) as u8;
    egui::Color32::from_rgb(
        r.saturating_add(add).min(255),
        g.saturating_add(add).min(255),
        b.saturating_add(add).min(255),
    )
}

/// Key for storing the target pan offset for smooth scrolling.
const CENTER_TARGET_KEY: &str = "umap_center_target";

/// Node dimensions in virtual coordinates (from calculate_node_rects).
const NODE_WIDTH: f32 = 120.0;
const NODE_HEIGHT: f32 = 40.0;

/// Handles the request to center the viewport on a specific map node.
/// This is triggered when a map is selected from the List of Maps.
fn handle_center_request(
    ui: &mut egui::Ui,
    file: &UmapInfoFile,
    graph: &UmapGraph,
    ctx: &mut ViewportContext,
) {
    let center_idx: Option<usize> = ui
        .ctx()
        .data(|d| d.get_temp(egui::Id::new(CENTER_ON_NODE_KEY)));

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

                    let new_pan_offset = egui::vec2(
                        center.x - viewport_rect.min.x - node_screen_x - node_screen_w * 0.5,
                        center.y - viewport_rect.min.y - node_screen_y - node_screen_h * 0.5,
                    );

                    ui.ctx().data_mut(|d| {
                        d.insert_temp(egui::Id::new(CENTER_TARGET_KEY), new_pan_offset);
                    });

                    ui.ctx().data_mut(|d| {
                        d.remove::<usize>(egui::Id::new(CENTER_ON_NODE_KEY));
                    });

                    break;
                }
            }
        }
    }

    let target: Option<egui::Vec2> = ui
        .ctx()
        .data(|d| d.get_temp(egui::Id::new(CENTER_TARGET_KEY)));

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
                d.remove::<egui::Vec2>(egui::Id::new(CENTER_TARGET_KEY));
            });
        }
    }
}

/// The primary orchestrator for the UMAPINFO flowchart viewport.
pub fn draw_umapinfo_viewport(
    ui: &mut egui::Ui,
    file: &UmapInfoFile,
    ctx: &mut ViewportContext,
) -> Vec<DocumentAction> {
    let graph = UmapGraph::build(file);

    handle_center_request(ui, file, &graph, ctx);

    draw_grid(ui.painter(), ctx);

    let node_rects = calculate_node_rects(&graph, ctx);

    let hovered_node = detect_hovered_node(&graph, &node_rects, ctx);

    if hovered_node.is_some() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    draw_edges(ui.painter(), &graph, &node_rects, ctx);
    draw_nodes(
        ui.painter(),
        file,
        &graph,
        &node_rects,
        ctx,
        hovered_node.as_deref(),
    );

    handle_interaction(ui, file, &graph, &node_rects, ctx)
}

/// Detects which node, if any, is currently hovered by the mouse.
fn detect_hovered_node<'a>(
    graph: &'a UmapGraph,
    node_rects: &HashMap<String, egui::Rect>,
    ctx: &ViewportContext,
) -> Option<&'a str> {
    if let Some(pos) = ctx.viewport_res.hover_pos() {
        for node in graph.nodes.iter() {
            if let Some(rect) = node_rects.get(&node.id) {
                if rect.contains(pos) {
                    return Some(node.id.as_str());
                }
            }
        }
    }
    None
}

/// Renders the infinite background dot grid. Culls to a rational amount of dots.
fn draw_grid(painter: &egui::Painter, ctx: &ViewportContext) {
    let virtual_spacing = 20.0;
    let dot_color = egui::Color32::from_gray(30);
    let viewport_rect = ctx.viewport_res.rect;

    let virt_min = ctx.proj.to_virtual(viewport_rect.min);
    let virt_max = ctx.proj.to_virtual(viewport_rect.max);

    let start_vx = (virt_min.x / virtual_spacing).floor() * virtual_spacing;
    let start_vy = (virt_min.y / virtual_spacing).floor() * virtual_spacing;
    let end_vx = (virt_max.x / virtual_spacing).ceil() * virtual_spacing;
    let end_vy = (virt_max.y / virtual_spacing).ceil() * virtual_spacing;

    let dot_radius = (1.5 * ctx.proj.final_scale_x).clamp(0.5, 3.0);
    let cols = ((end_vx - start_vx) / virtual_spacing).max(0.0) as usize + 1;
    let rows = ((end_vy - start_vy) / virtual_spacing).max(0.0) as usize + 1;
    let dot_count = cols * rows;

    const DOT_LIMIT: usize = 5_000;
    if dot_count <= DOT_LIMIT {
        let mut vx = start_vx;
        while vx <= end_vx {
            let mut vy = start_vy;
            while vy <= end_vy {
                let center = ctx.proj.to_screen(egui::pos2(vx, vy));
                painter.circle_filled(center, dot_radius, dot_color);
                vy += virtual_spacing;
            }
            vx += virtual_spacing;
        }
    }
}

/// Generates screen-space bounding boxes for every node in the graph.
fn calculate_node_rects(graph: &UmapGraph, ctx: &ViewportContext) -> HashMap<String, egui::Rect> {
    let mut node_rects = HashMap::new();
    for node in &graph.nodes {
        let (w, h) = (120.0, 40.0);
        let rect = egui::Rect::from_min_size(
            ctx.proj.to_screen(egui::pos2(node.x, node.y)),
            egui::vec2(w * ctx.proj.final_scale_x, h * ctx.proj.final_scale_y),
        );
        node_rects.insert(node.id.clone(), rect);
    }
    node_rects
}

/// Renders the bezier curve connections between nodes.
fn draw_edges(
    painter: &egui::Painter,
    graph: &UmapGraph,
    node_rects: &HashMap<String, egui::Rect>,
    ctx: &ViewportContext,
) {
    for edge in &graph.edges {
        if let (Some(src_rect), Some(dst_rect)) =
            (node_rects.get(&edge.source), node_rects.get(&edge.target))
        {
            let is_secret = edge.edge_type == EdgeType::Secret;
            let color = if is_secret {
                egui::Color32::from_rgb(200, 100, 200)
            } else {
                egui::Color32::from_rgb(100, 200, 100)
            };

            let (src_pt, dst_pt, cp1, cp2) = if is_secret {
                let s = src_rect.right_center();
                let d = dst_rect.left_center();
                let dist = (d.x - s.x).abs().max(40.0) * 0.5;
                let c1 = s + egui::vec2(dist, 0.0);
                let c2 = d - egui::vec2(dist, 0.0);
                (s, d, c1, c2)
            } else {
                let s = src_rect.center_bottom();
                let d = dst_rect.center_top();
                let dist = (d.y - s.y).abs().max(40.0) * 0.5;
                let c1 = s + egui::vec2(0.0, dist);
                let c2 = d - egui::vec2(0.0, dist);
                (s, d, c1, c2)
            };

            painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                [src_pt, cp1, cp2, dst_pt],
                false,
                egui::Color32::TRANSPARENT,
                egui::Stroke::new(2.5 * ctx.proj.final_scale_x, color),
            ));
        }
    }
}

/// Renders the node boxes and their internal text.
fn draw_nodes(
    painter: &egui::Painter,
    file: &UmapInfoFile,
    graph: &UmapGraph,
    node_rects: &HashMap<String, egui::Rect>,
    ctx: &ViewportContext,
    hovered_node: Option<&str>,
) {
    let hover_lighten = 0.25;

    for node in &graph.nodes {
        let rect = node_rects.get(&node.id).unwrap();
        let is_hovered = hovered_node == Some(node.id.as_str());

        let (bg_color, stroke_color, title, subtitle) = match &node.node_type {
            NodeType::Map { levelname } => {
                let is_selected = ctx.selection.iter().any(|p| {
                    if let Some(map) = file.data.maps.get(p[0]) {
                        map.mapname.to_uppercase() == node.id
                    } else {
                        false
                    }
                });
                let base_color = if is_selected {
                    egui::Color32::from_rgb(60, 80, 120)
                } else {
                    egui::Color32::from_rgb(45, 45, 45)
                };
                (
                    if is_hovered {
                        lighten_color(base_color, hover_lighten)
                    } else {
                        base_color
                    },
                    if is_selected {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_gray(80)
                    },
                    node.id.clone(),
                    Some(levelname.clone()),
                )
            }
            NodeType::Episode { name, patch } => {
                let base_color = egui::Color32::from_rgb(30, 60, 30);
                (
                    if is_hovered {
                        lighten_color(base_color, hover_lighten)
                    } else {
                        base_color
                    },
                    egui::Color32::from_rgb(80, 150, 80),
                    if patch.is_empty() {
                        "Episode".to_string()
                    } else {
                        patch.clone()
                    },
                    Some(name.clone()),
                )
            }
            NodeType::InterText { is_secret } => {
                let base_color = egui::Color32::from_rgb(60, 40, 80);
                (
                    if is_hovered {
                        lighten_color(base_color, hover_lighten)
                    } else {
                        base_color
                    },
                    egui::Color32::from_rgb(150, 100, 200),
                    "Intermission Text".to_string(),
                    if *is_secret {
                        Some("(Secret)".to_string())
                    } else {
                        None
                    },
                )
            }
            NodeType::Terminal { end_type } => {
                let base_color = egui::Color32::from_rgb(80, 30, 30);
                (
                    if is_hovered {
                        lighten_color(base_color, hover_lighten)
                    } else {
                        base_color
                    },
                    egui::Color32::from_rgb(200, 80, 80),
                    "Terminal".to_string(),
                    Some(end_type.clone()),
                )
            }
        };

        let rounding = 4.0 * ctx.proj.final_scale_x;
        painter.rect_filled(*rect, rounding, bg_color);
        painter.rect_stroke(
            *rect,
            rounding,
            egui::Stroke::new(1.0, stroke_color),
            egui::StrokeKind::Inside,
        );

        if let Some(sub) = subtitle {
            painter.text(
                rect.center() - egui::vec2(0.0, 6.0 * ctx.proj.final_scale_y),
                egui::Align2::CENTER_CENTER,
                title,
                egui::FontId::proportional(12.0 * ctx.proj.final_scale_x),
                egui::Color32::WHITE,
            );
            painter.text(
                rect.center() + egui::vec2(0.0, 6.0 * ctx.proj.final_scale_y),
                egui::Align2::CENTER_CENTER,
                sub,
                egui::FontId::proportional(10.0 * ctx.proj.final_scale_x),
                egui::Color32::from_gray(180),
            );
        } else {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                title,
                egui::FontId::proportional(14.0 * ctx.proj.final_scale_x),
                egui::Color32::WHITE,
            );
        }
    }
}

/// Extracts the map ID from a node ID if it's a non-MAP node type.
/// Returns Some(map_id) for Episode, InterText, and Terminal nodes.
fn extract_map_id_from_node(node: &UmapNode) -> Option<String> {
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

/// Finds the index of the MAP entry in the file that corresponds to the given node.
/// For MAP nodes, returns the index of that map.
/// For other node types (Episode, InterText, Terminal), returns the index of the parent MAP.
fn find_map_index_for_node(file: &UmapInfoFile, node: &UmapNode) -> Option<usize> {
    let map_id = extract_map_id_from_node(node)?;
    file.data
        .maps
        .iter()
        .position(|m| m.mapname.to_uppercase() == map_id)
}

/// Helper to create a selection action for a map index.
fn select_map_action(idx: usize) -> DocumentAction {
    DocumentAction::Tree(crate::document::actions::TreeAction::Select(vec![vec![
        idx,
    ]]))
}

/// Processes all mouse inputs, drag-and-drop, selection, and viewport panning.
fn handle_interaction(
    ui: &mut egui::Ui,
    file: &UmapInfoFile,
    graph: &UmapGraph,
    node_rects: &HashMap<String, egui::Rect>,
    ctx: &mut ViewportContext,
) -> Vec<DocumentAction> {
    let mut actions = Vec::new();

    let drag_id = egui::Id::new("umap_node_drag_global");
    let start_pos_id = egui::Id::new("umap_drag_start_ptr_global");
    let node_start_id = egui::Id::new("umap_node_start_v_global");
    let bg_pan_id = egui::Id::new("umap_bg_pan_global");

    let mut dragged_node: Option<String> = ui.ctx().data(|d| d.get_temp(drag_id));
    let mut start_ptr: Option<egui::Pos2> = ui.ctx().data(|d| d.get_temp(start_pos_id));
    let mut node_start: Option<egui::Pos2> = ui.ctx().data(|d| d.get_temp(node_start_id));
    let mut is_bg_panning: bool = ui.ctx().data(|d| d.get_temp(bg_pan_id).unwrap_or(false));

    if ctx
        .viewport_res
        .drag_started_by(egui::PointerButton::Primary)
        && !ctx.is_panning
    {
        if let Some(pos) = ctx.viewport_res.hover_pos() {
            let mut hit_node = false;

            for node in graph.nodes.iter().rev() {
                if let Some(rect) = node_rects.get(&node.id) {
                    if rect.contains(pos) {
                        dragged_node = Some(node.id.clone());
                        start_ptr = Some(pos);
                        node_start = Some(egui::pos2(node.x, node.y));
                        hit_node = true;

                        if let Some(idx) = find_map_index_for_node(file, node) {
                            actions.push(select_map_action(idx));
                        }

                        actions.push(DocumentAction::UndoSnapshot);
                        break;
                    }
                }
            }

            if !hit_node {
                is_bg_panning = true;
            }
        }
    }

    let primary_pressed_now = ui.input(|i| i.pointer.primary_down());
    let was_dragging = ctx.viewport_res.dragged_by(egui::PointerButton::Primary);
    if primary_pressed_now && !was_dragging && !ctx.is_panning && dragged_node.is_none() {
        if let Some(pos) = ctx.viewport_res.hover_pos() {
            for node in graph.nodes.iter().rev() {
                if let Some(rect) = node_rects.get(&node.id) {
                    if rect.contains(pos) {
                        if let Some(idx) = find_map_index_for_node(file, node) {
                            actions.push(select_map_action(idx));
                        }
                        break;
                    }
                }
            }
        }
    }

    let bg_menu_id = egui::Id::new("umap_bg_context_menu");
    let menu_valid_id = egui::Id::new("umap_bg_menu_valid");
    
    let viewport_rect = ctx.viewport_res.rect;
    let bg_response = ui.interact(viewport_rect, bg_menu_id, egui::Sense::click());
    
    let just_opened = ContextMenu::check(ui, &bg_response);
    
    if just_opened {
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let mut is_blank = true;
            for node in graph.nodes.iter() {
                if let Some(rect) = node_rects.get(&node.id) {
                    if rect.contains(pos) {
                        is_blank = false;
                        break;
                    }
                }
            }
            ui.data_mut(|d| d.insert_temp(menu_valid_id, is_blank));
        }
    }
    
    let menu_valid: bool = ui.data(|d| d.get_temp(menu_valid_id).unwrap_or(false));
    
    if let Some(menu) = ContextMenu::get(ui, bg_menu_id) {
        if menu_valid {
            let click_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or_default();
            let virtual_pos = ctx.proj.to_virtual(click_pos);
            
            ContextMenu::show(ui, menu, just_opened, |ui| {
                if ContextMenu::button(ui, "New Map", true) {
                    actions.push(DocumentAction::UndoSnapshot);
                    actions.push(DocumentAction::Umap(UmapAction::AddMap { 
                        x: virtual_pos.x, 
                        y: virtual_pos.y 
                    }));
                    ContextMenu::close(ui);
                }
            });
        }
        if !ContextMenu::get(ui, bg_menu_id).is_some() {
            ui.data_mut(|d| d.remove::<bool>(menu_valid_id));
        }
    }

    if is_bg_panning {
        if ctx.viewport_res.dragged_by(egui::PointerButton::Primary) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            ctx.state.sim.engine.pan_offset += ui.input(|i| i.pointer.delta());
            ui.ctx().request_repaint();
        }
    }

    if let Some(ref node_id) = dragged_node {
        if ctx.viewport_res.dragged_by(egui::PointerButton::Primary) {
            if let (Some(ptr_start), Some(n_start), Some(current_ptr)) =
                (start_ptr, node_start, ui.input(|i| i.pointer.latest_pos()))
            {
                let delta_screen = current_ptr - ptr_start;
                let delta_v = egui::vec2(
                    delta_screen.x / ctx.proj.final_scale_x,
                    delta_screen.y / ctx.proj.final_scale_y,
                );

                let target_vx = ((n_start.x + delta_v.x) / 20.0).round() * 20.0;
                let target_vy = ((n_start.y + delta_v.y) / 20.0).round() * 20.0;

                actions.push(DocumentAction::Umap(UmapAction::UpdateNodePos(
                    node_id.clone(),
                    target_vx,
                    target_vy,
                )));
                ui.ctx().request_repaint();
            }
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
            d.remove::<egui::Pos2>(start_pos_id);
        }
        if let Some(val) = node_start {
            d.insert_temp(node_start_id, val);
        } else {
            d.remove::<egui::Pos2>(node_start_id);
        }
        d.insert_temp(bg_pan_id, is_bg_panning);
    });

    actions
}
