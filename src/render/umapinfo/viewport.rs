use crate::document::actions::DocumentAction;
use crate::models::umap_graph::UmapGraph;
use crate::models::umapinfo::UmapInfoFile;
use crate::ui::properties::editor::ViewportContext;

use super::connectors_draw::{detect_hovered_connector, draw_connectors};
use super::edges::{EdgePath, calculate_all_edge_paths, draw_edge_arrows, draw_edge_lines};
use super::grid::draw_grid;
use super::interaction::handle_center_request;
use super::interaction::handle_interaction;
use super::nodes::{calculate_node_rects, detect_hovered_node, draw_nodes};

/// The primary orchestrator for the UMAPINFO flowchart viewport.
pub fn draw_umapinfo_viewport(
    ui: &mut eframe::egui::Ui,
    file: &UmapInfoFile,
    ctx: &mut ViewportContext,
) -> Vec<DocumentAction> {
    let graph = UmapGraph::build(file);

    handle_center_request(ui, file, &graph, ctx);

    draw_grid(ui.painter(), ctx);

    let node_rects = calculate_node_rects(&graph, ctx);

    let hovered_connector = detect_hovered_connector(&graph, &node_rects, ctx);
    let hovered_node = if hovered_connector.is_some() {
        None
    } else {
        detect_hovered_node(&graph, &node_rects, ctx)
    };

    if hovered_node.is_some() || hovered_connector.is_some() {
        ui.ctx()
            .set_cursor_icon(eframe::egui::CursorIcon::PointingHand);
    }

    let mut signature = Vec::new();
    for n in &graph.nodes {
        signature.push((n.id.clone(), n.x as i32, n.y as i32));
    }
    for e in &graph.edges {
        signature.push((format!("{}-{}", e.source, e.target), 0, 0));
    }

    let cache_id = ui.make_persistent_id("umap_edge_cache");
    let sig_id = cache_id.with("sig");

    let cached_sig: Option<Vec<(String, i32, i32)>> = ui.data(|d| d.get_temp(sig_id));

    let edge_paths: Vec<EdgePath> = if cached_sig == Some(signature.clone()) {
        ui.data(|d| d.get_temp(cache_id)).unwrap_or_default()
    } else {
        let new_paths = calculate_all_edge_paths(&graph);
        ui.data_mut(|d| {
            d.insert_temp(sig_id, signature);
            d.insert_temp(cache_id, new_paths.clone());
        });
        new_paths
    };

    draw_edge_lines(ui.painter(), &edge_paths, ctx);

    draw_nodes(
        ui.painter(),
        file,
        &graph,
        &node_rects,
        ctx,
        hovered_node.as_deref(),
    );

    draw_edge_arrows(ui.painter(), &edge_paths, ctx);

    draw_connectors(
        ui.painter(),
        file,
        &graph,
        &node_rects,
        ctx,
        hovered_connector,
    );

    handle_interaction(ui, file, &graph, &node_rects, ctx)
}
