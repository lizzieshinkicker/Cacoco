use std::collections::HashMap;

use crate::models::umap_graph::{NodeType, UmapGraph};
use crate::models::umapinfo::UmapInfoFile;
use crate::ui::properties::editor::ViewportContext;

use super::constants::{NODE_HEIGHT, NODE_WIDTH, lighten_color};

/// Generates screen-space bounding boxes for every node in the graph.
pub fn calculate_node_rects(
    graph: &UmapGraph,
    ctx: &ViewportContext,
) -> HashMap<String, eframe::egui::Rect> {
    let mut node_rects = HashMap::new();
    for node in &graph.nodes {
        let (w, h) = (NODE_WIDTH, NODE_HEIGHT);
        let rect = eframe::egui::Rect::from_min_size(
            ctx.proj.to_screen(eframe::egui::pos2(node.x, node.y)),
            eframe::egui::vec2(w * ctx.proj.final_scale_x, h * ctx.proj.final_scale_y),
        );
        node_rects.insert(node.id.clone(), rect);
    }
    node_rects
}

/// Detects which node, if any, is currently hovered by the mouse.
pub fn detect_hovered_node<'a>(
    graph: &'a UmapGraph,
    node_rects: &HashMap<String, eframe::egui::Rect>,
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

/// Renders the node boxes and their internal text.
pub fn draw_nodes(
    painter: &eframe::egui::Painter,
    file: &UmapInfoFile,
    graph: &UmapGraph,
    node_rects: &HashMap<String, eframe::egui::Rect>,
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
                    eframe::egui::Color32::from_rgb(60, 80, 120)
                } else {
                    eframe::egui::Color32::from_rgb(45, 45, 45)
                };
                (
                    if is_hovered {
                        lighten_color(base_color, hover_lighten)
                    } else {
                        base_color
                    },
                    if is_selected {
                        eframe::egui::Color32::WHITE
                    } else {
                        eframe::egui::Color32::from_gray(80)
                    },
                    node.id.clone(),
                    Some(levelname.clone()),
                )
            }
            NodeType::Episode { name, patch } => {
                let base_color = eframe::egui::Color32::from_rgb(30, 60, 30);
                (
                    if is_hovered {
                        lighten_color(base_color, hover_lighten)
                    } else {
                        base_color
                    },
                    eframe::egui::Color32::from_rgb(80, 150, 80),
                    if patch.is_empty() {
                        "Episode".to_string()
                    } else {
                        patch.clone()
                    },
                    Some(name.clone()),
                )
            }
            NodeType::InterText { is_secret } => {
                let base_color = eframe::egui::Color32::from_rgb(60, 40, 80);
                (
                    if is_hovered {
                        lighten_color(base_color, hover_lighten)
                    } else {
                        base_color
                    },
                    eframe::egui::Color32::from_rgb(150, 100, 200),
                    "Intermission Text".to_string(),
                    if *is_secret {
                        Some("(Secret)".to_string())
                    } else {
                        None
                    },
                )
            }
            NodeType::Terminal { end_type } => {
                let base_color = eframe::egui::Color32::from_rgb(80, 30, 30);
                (
                    if is_hovered {
                        lighten_color(base_color, hover_lighten)
                    } else {
                        base_color
                    },
                    eframe::egui::Color32::from_rgb(200, 80, 80),
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
            eframe::egui::Stroke::new(1.0, stroke_color),
            eframe::egui::StrokeKind::Inside,
        );

        if let Some(sub) = subtitle {
            painter.text(
                rect.center() - eframe::egui::vec2(0.0, 6.0 * ctx.proj.final_scale_y),
                eframe::egui::Align2::CENTER_CENTER,
                title,
                eframe::egui::FontId::proportional(12.0 * ctx.proj.final_scale_x),
                eframe::egui::Color32::WHITE,
            );
            painter.text(
                rect.center() + eframe::egui::vec2(0.0, 6.0 * ctx.proj.final_scale_y),
                eframe::egui::Align2::CENTER_CENTER,
                sub,
                eframe::egui::FontId::proportional(10.0 * ctx.proj.final_scale_x),
                eframe::egui::Color32::from_gray(180),
            );
        } else {
            painter.text(
                rect.center(),
                eframe::egui::Align2::CENTER_CENTER,
                title,
                eframe::egui::FontId::proportional(14.0 * ctx.proj.final_scale_x),
                eframe::egui::Color32::WHITE,
            );
        }
    }
}
