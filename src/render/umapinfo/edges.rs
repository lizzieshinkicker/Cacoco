use std::collections::HashMap;

use crate::models::umap_graph::{EdgeType, UmapGraph};
use crate::ui::properties::editor::ViewportContext;

/// Renders the bezier curve connections between nodes.
pub fn draw_edges(
    painter: &eframe::egui::Painter,
    graph: &UmapGraph,
    node_rects: &HashMap<String, eframe::egui::Rect>,
    ctx: &ViewportContext,
) {
    for edge in &graph.edges {
        if let (Some(src_rect), Some(dst_rect)) =
            (node_rects.get(&edge.source), node_rects.get(&edge.target))
        {
            let is_secret = edge.edge_type == EdgeType::Secret;
            let color = if is_secret {
                eframe::egui::Color32::from_rgb(200, 100, 200)
            } else {
                eframe::egui::Color32::from_rgb(100, 200, 100)
            };

            let (src_pt, dst_pt, cp1, cp2) = if is_secret {
                let s = src_rect.right_center();
                let d = dst_rect.left_center();
                let dist = (d.x - s.x).abs().max(40.0) * 0.5;
                let c1 = s + eframe::egui::vec2(dist, 0.0);
                let c2 = d - eframe::egui::vec2(dist, 0.0);
                (s, d, c1, c2)
            } else {
                let s = src_rect.center_bottom();
                let d = dst_rect.center_top();
                let dist = (d.y - s.y).abs().max(40.0) * 0.5;
                let c1 = s + eframe::egui::vec2(0.0, dist);
                let c2 = d - eframe::egui::vec2(0.0, dist);
                (s, d, c1, c2)
            };

            painter.add(eframe::egui::epaint::CubicBezierShape::from_points_stroke(
                [src_pt, cp1, cp2, dst_pt],
                false,
                eframe::egui::Color32::TRANSPARENT,
                eframe::egui::Stroke::new(2.5 * ctx.proj.final_scale_x, color),
            ));
        }
    }
}
