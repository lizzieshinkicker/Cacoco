use crate::assets::{AssetId, AssetStore};
use eframe::egui;

/// Represents the content data to be visualized in the property preview window.
pub enum PreviewContent {
    /// Renders a single static image from a patch name.
    Image(String),
    /// Renders a line of text using a specific font stem and type.
    Text {
        text: String,
        stem: Option<String>,
        is_number_font: bool,
    },
}

/// Renders a high-resolution preview box for the currently selected element.
///
/// This panel handles the automatic scaling and centering of textures
/// and text samples, using pre-hashed AssetIds for performance.
pub fn draw_preview_panel(ui: &mut egui::Ui, assets: &AssetStore, content: PreviewContent) {
    let height = 96.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::hover(),
    );

    ui.painter()
        .rect_filled(rect, 4.0, egui::Color32::from_black_alpha(50));
    ui.painter().rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)),
        egui::StrokeKind::Inside,
    );

    let mut textures: Vec<&egui::TextureHandle> = Vec::new();

    match content {
        PreviewContent::Image(name) => {
            let id = AssetId::new(&name);
            if let Some(tex) = assets.textures.get(&id) {
                textures.push(tex);
            }
        }
        PreviewContent::Text {
            text,
            stem,
            is_number_font,
        } => {
            if let Some(s) = stem {
                for char in text.chars() {
                    let id = assets.resolve_patch_id(&s, char, is_number_font);
                    if let Some(tex) = assets.textures.get(&id) {
                        textures.push(tex);
                    }
                }
            }
        }
    }

    if textures.is_empty() {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "(No Preview Available)",
            egui::FontId::proportional(14.0),
            egui::Color32::GRAY,
        );
        return;
    }

    let total_width: f32 = textures.iter().map(|t| t.size_vec2().x).sum();
    let max_height: f32 = textures
        .iter()
        .map(|t| t.size_vec2().y)
        .reduce(f32::max)
        .unwrap_or(10.0);

    let height_scale = (rect.height() - 16.0) / max_height;
    let width_scale = (rect.width() - 16.0) / total_width;

    let raw_scale = height_scale.min(width_scale).min(4.0);
    let scale = if raw_scale >= 1.0 {
        raw_scale.floor()
    } else {
        raw_scale
    };

    let draw_width = total_width * scale;
    let start_x = (rect.center().x - (draw_width / 2.0)).floor();
    let center_y = rect.center().y;

    let mut current_x = start_x;
    for tex in textures {
        let size = tex.size_vec2() * scale;
        let y_pos = (center_y - (size.y / 2.0)).floor();
        let dest_rect = egui::Rect::from_min_size(egui::pos2(current_x, y_pos), size);

        ui.painter().image(
            tex.id(),
            dest_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        current_x += size.x;
    }
}
