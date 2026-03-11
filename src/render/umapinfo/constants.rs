pub const CENTER_ON_NODE_KEY: &str = "umap_center_on_node";
pub const CENTER_TARGET_KEY: &str = "umap_center_target";
pub const NODE_WIDTH: f32 = 120.0;
pub const NODE_HEIGHT: f32 = 40.0;
pub const CONNECTOR_RADIUS_BASE: f32 = 5.0;
pub const NORMAL_EXIT_COLOR: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(80, 200, 80); // Green
pub const SECRET_EXIT_COLOR: eframe::egui::Color32 = eframe::egui::Color32::from_rgb(220, 100, 180); // Pink

/// Lightens a color by a given amount (0.0 to 1.0).
pub fn lighten_color(color: eframe::egui::Color32, amount: f32) -> eframe::egui::Color32 {
    let [r, g, b, _a] = color.to_array();
    let add = (amount * 255.0) as u8;
    eframe::egui::Color32::from_rgb(
        r.saturating_add(add).min(255),
        g.saturating_add(add).min(255),
        b.saturating_add(add).min(255),
    )
}
