use crate::models::umap_graph::{NodeType, UmapNode};
use crate::models::umapinfo::{UmapField, UmapInfoFile};

use super::constants::CONNECTOR_RADIUS_BASE;

/// Represents which connector, if any, is being hovered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorType {
    /// Normal exit (green) - bottom of node
    NormalExit,
    /// Secret exit (pink) - right side of node
    SecretExit,
}

/// Checks if a node can have a normal exit connector
pub fn node_has_normal_exit(node: &UmapNode) -> bool {
    matches!(&node.node_type, NodeType::Map { .. })
}

/// Checks if a node can have a secret exit connector
pub fn node_has_secret_exit(node: &UmapNode) -> bool {
    match &node.node_type {
        NodeType::Map { .. } => true,
        NodeType::InterText { is_secret } => *is_secret,
        _ => false,
    }
}

/// Checks if a map has a normal exit configured
pub fn map_has_normal_exit(file: &UmapInfoFile, map_id: &str) -> bool {
    let upper = map_id.to_uppercase();
    if let Some(map) = file
        .data
        .maps
        .iter()
        .find(|m| m.mapname.to_uppercase() == upper)
    {
        map.fields.iter().any(|f| matches!(f, UmapField::Next(_)))
    } else {
        false
    }
}

/// Checks if a map has a secret exit configured
pub fn map_has_secret_exit(file: &UmapInfoFile, map_id: &str) -> bool {
    let upper = map_id.to_uppercase();
    if let Some(map) = file
        .data
        .maps
        .iter()
        .find(|m| m.mapname.to_uppercase() == upper)
    {
        map.fields
            .iter()
            .any(|f| matches!(f, UmapField::NextSecret(_)))
    } else {
        false
    }
}

/// Calculates the screen position of a normal exit connector (bottom center)
pub fn get_normal_exit_pos(rect: &eframe::egui::Rect) -> eframe::egui::Pos2 {
    rect.center_bottom()
}

/// Calculates the screen position of a secret exit connector (right center)
pub fn get_secret_exit_pos(rect: &eframe::egui::Rect) -> eframe::egui::Pos2 {
    rect.right_center()
}

/// Checks if a point is within a connector's hit area (scaled with zoom)
pub fn is_point_in_connector(
    point: eframe::egui::Pos2,
    connector_pos: eframe::egui::Pos2,
    scale: f32,
) -> bool {
    let radius = CONNECTOR_RADIUS_BASE * scale;
    let dx = point.x - connector_pos.x;
    let dy = point.y - connector_pos.y;
    (dx * dx + dy * dy) <= (radius * radius)
}

/// Result of detecting a connector hit - contains node ID, connector type, and position
#[derive(Debug, Clone)]
pub struct ConnectorHit {
    /// The node ID that owns this connector
    pub node_id: String,
    /// The type of connector hit
    pub connector_type: ConnectorType,
    /// The screen position of the connector
    pub position: eframe::egui::Pos2,
}

/// Detects if a point hits any connector on any node.
pub fn detect_connector_hit(
    point: eframe::egui::Pos2,
    graph: &crate::models::umap_graph::UmapGraph,
    node_rects: &std::collections::HashMap<String, eframe::egui::Rect>,
    scale: f32,
) -> Option<ConnectorHit> {
    for node in graph.nodes.iter().rev() {
        let Some(rect) = node_rects.get(&node.id) else {
            continue;
        };

        if node_has_normal_exit(node) {
            let conn_pos = get_normal_exit_pos(rect);
            if is_point_in_connector(point, conn_pos, scale) {
                return Some(ConnectorHit {
                    node_id: node.id.clone(),
                    connector_type: ConnectorType::NormalExit,
                    position: conn_pos,
                });
            }
        }

        if node_has_secret_exit(node) {
            let conn_pos = get_secret_exit_pos(rect);
            if is_point_in_connector(point, conn_pos, scale) {
                return Some(ConnectorHit {
                    node_id: node.id.clone(),
                    connector_type: ConnectorType::SecretExit,
                    position: conn_pos,
                });
            }
        }
    }
    None
}
