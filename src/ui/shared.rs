use crate::ui::layers::thumbnails;
use eframe::egui;

/// Unique key for storing the viewport bounds in egui temp data.
pub const VIEWPORT_RECT_ID: &str = "cacoco_viewport_rect";

/// Draws an image scaled and centered within a given rect, respecting aspect ratio.
pub fn draw_scaled_image(
    ui: &egui::Ui,
    rect: egui::Rect,
    tex: &egui::TextureHandle,
    tint: egui::Color32,
    max_scale: f32,
) {
    let tex_size = tex.size_vec2();
    if tex_size.x > 0.0 && tex_size.y > 0.0 {
        let scale = (rect.width() / tex_size.x)
            .min(rect.height() / tex_size.y)
            .min(max_scale);
        let final_size = tex_size * scale;
        let draw_rect = egui::Rect::from_center_size(rect.center(), final_size);

        ui.painter().image(
            tex.id(),
            draw_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    }
}

/// Standard frame style for drag-and-drop "ghost" elements.
pub fn drag_ghost_frame() -> egui::Frame {
    egui::Frame::default()
        .inner_margin(6.0)
        .corner_radius(4.0)
        .fill(egui::Color32::from_black_alpha(200))
        .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
}

/// Draws a standard Cacoco-style drag ghost that follows the pointer.
/// Will automatically suppress itself if the pointer is inside the Viewport.
pub fn draw_drag_ghost(ctx: &egui::Context, icon_content: impl FnOnce(&mut egui::Ui), label: &str) {
    let pointer_pos = ctx.input(|i| i.pointer.latest_pos());

    let viewport_rect: Option<egui::Rect> =
        ctx.data(|d| d.get_temp(egui::Id::new(VIEWPORT_RECT_ID)));
    if let (Some(vr), Some(pp)) = (viewport_rect, pointer_pos) {
        if vr.contains(pp) {
            return;
        }
    }

    if let Some(pos) = pointer_pos {
        egui::Area::new(egui::Id::new("cacoco_drag_ghost"))
            .interactable(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .fixed_pos(pos)
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                drag_ghost_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.allocate_ui(
                            egui::vec2(thumbnails::THUMB_SIZE, thumbnails::THUMB_SIZE),
                            |ui| {
                                icon_content(ui);
                            },
                        );
                        ui.label(
                            egui::RichText::new(label)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                    });
                });
            });
    }
}