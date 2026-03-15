use std::collections::HashMap;

use crate::models::umap_graph::UmapGraph;
use crate::models::umapinfo::UmapInfoFile;
use crate::ui::properties::editor::ViewportContext;

use super::connectors::{
    ConnectorType, get_normal_exit_pos, get_secret_exit_pos, is_point_in_connector,
    map_has_normal_exit, map_has_secret_exit, node_has_normal_exit, node_has_secret_exit,
};
use super::constants::{
    CONNECTOR_RADIUS_BASE, NORMAL_EXIT_COLOR, SECRET_EXIT_COLOR, lighten_color,
};

/// Detects which connector, if any, is currently hovered by the mouse.
pub fn detect_hovered_connector<'a>(
    graph: &'a UmapGraph,
    node_rects: &HashMap<String, eframe::egui::Rect>,
    ctx: &ViewportContext,
) -> Option<(&'a str, ConnectorType)> {
    if let Some(pos) = ctx.viewport_res.hover_pos() {
        for node in graph.nodes.iter() {
            if let Some(rect) = node_rects.get(&node.id) {
                if node_has_normal_exit(node) {
                    let connector_pos = get_normal_exit_pos(rect);
                    if is_point_in_connector(pos, connector_pos, ctx.proj.final_scale_x) {
                        return Some((node.id.as_str(), ConnectorType::NormalExit));
                    }
                }
                if node_has_secret_exit(node) {
                    let connector_pos = get_secret_exit_pos(rect);
                    if is_point_in_connector(pos, connector_pos, ctx.proj.final_scale_x) {
                        return Some((node.id.as_str(), ConnectorType::SecretExit));
                    }
                }
            }
        }
    }
    None
}

/// Draws the exit connectors on nodes.
/// Green normal exit on bottom, pink secret exit on right.
pub fn draw_connectors(
    painter: &eframe::egui::Painter,
    file: &UmapInfoFile,
    graph: &UmapGraph,
    node_rects: &HashMap<String, eframe::egui::Rect>,
    ctx: &ViewportContext,
    hovered_connector: Option<(&str, ConnectorType)>,
) {
    let connector_radius = CONNECTOR_RADIUS_BASE * ctx.proj.final_scale_x;

    for node in &graph.nodes {
        let rect = match node_rects.get(&node.id) {
            Some(r) => r,
            None => continue,
        };

        if node_has_normal_exit(node) {
            let pos = get_normal_exit_pos(rect);
            let _has_exit = map_has_normal_exit(file, &node.id);
            let is_hovered =
                hovered_connector == Some((node.id.as_str(), ConnectorType::NormalExit));

            let color = if is_hovered {
                lighten_color(NORMAL_EXIT_COLOR, 0.3)
            } else {
                NORMAL_EXIT_COLOR
            };

            painter.circle_filled(pos, connector_radius, color);
        }

        if node_has_secret_exit(node) {
            let pos = get_secret_exit_pos(rect);
            let _has_exit = map_has_secret_exit(file, &node.id);
            let is_hovered =
                hovered_connector == Some((node.id.as_str(), ConnectorType::SecretExit));

            let color = if is_hovered {
                lighten_color(SECRET_EXIT_COLOR, 0.3)
            } else {
                SECRET_EXIT_COLOR
            };

            painter.circle_filled(pos, connector_radius, color);
        }
    }
}
