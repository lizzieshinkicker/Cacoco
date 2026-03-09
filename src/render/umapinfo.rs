use crate::document::actions::{DocumentAction, UmapAction};
use crate::models::umap_graph::{EdgeType, NodeType, UmapGraph};
use crate::models::umapinfo::UmapInfoFile;
use crate::ui::properties::editor::ViewportContext;
use eframe::egui;
use std::collections::HashMap;

/// The primary orchestrator for the UMAPINFO flowchart viewport.
pub fn draw_umapinfo_viewport(
    ui: &mut egui::Ui,
    file: &UmapInfoFile,
    ctx: &mut ViewportContext,
) -> Vec<DocumentAction> {
    let graph = UmapGraph::build(file);

    draw_grid(ui.painter(), ctx);

    let node_rects = calculate_node_rects(&graph, ctx);

    draw_edges(ui.painter(), &graph, &node_rects, ctx);
    draw_nodes(ui.painter(), file, &graph, &node_rects, ctx);

    handle_interaction(ui, file, &graph, &node_rects, ctx)
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
) {
    for node in &graph.nodes {
        let rect = node_rects.get(&node.id).unwrap();

        let (bg_color, stroke_color, title, subtitle) = match &node.node_type {
            NodeType::Map { levelname } => {
                let is_selected = ctx.selection.iter().any(|p| {
                    if let Some(map) = file.data.maps.get(p[0]) {
                        map.mapname.to_uppercase() == node.id
                    } else {
                        false
                    }
                });
                (
                    if is_selected {
                        egui::Color32::from_rgb(60, 80, 120)
                    } else {
                        egui::Color32::from_rgb(45, 45, 45)
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
            NodeType::Episode { name, patch } => (
                egui::Color32::from_rgb(30, 60, 30),
                egui::Color32::from_rgb(80, 150, 80),
                if patch.is_empty() {
                    "Episode".to_string()
                } else {
                    patch.clone()
                },
                Some(name.clone()),
            ),
            NodeType::InterText { is_secret } => (
                egui::Color32::from_rgb(60, 40, 80),
                egui::Color32::from_rgb(150, 100, 200),
                "Intermission Text".to_string(),
                if *is_secret {
                    Some("(Secret)".to_string())
                } else {
                    None
                },
            ),
            NodeType::Terminal { end_type } => (
                egui::Color32::from_rgb(80, 30, 30),
                egui::Color32::from_rgb(200, 80, 80),
                "Terminal".to_string(),
                Some(end_type.clone()),
            ),
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

                        if matches!(node.node_type, NodeType::Map { .. }) {
                            if let Some(idx) = file
                                .data
                                .maps
                                .iter()
                                .position(|m| m.mapname.to_uppercase() == node.id)
                            {
                                actions.push(DocumentAction::Tree(
                                    crate::document::actions::TreeAction::Select(vec![vec![idx]]),
                                ));
                            }
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
