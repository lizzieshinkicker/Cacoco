use crate::document::actions::DocumentAction;
use crate::models::umap_graph::UmapGraph;
use crate::models::umapinfo::UmapInfoFile;
use crate::ui::properties::editor::ViewportContext;

use super::connectors_draw::{detect_hovered_connector, draw_connectors};
use super::edges::draw_edges;
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

    let hovered_node = detect_hovered_node(&graph, &node_rects, ctx);

    let hovered_connector = detect_hovered_connector(&graph, &node_rects, ctx);

    if hovered_node.is_some() || hovered_connector.is_some() {
        ui.ctx()
            .set_cursor_icon(eframe::egui::CursorIcon::PointingHand);
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
