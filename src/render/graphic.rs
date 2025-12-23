use crate::model::*;
use eframe::egui;
use super::{get_alignment_anchor_offset, RenderContext};

pub(super) fn draw_graphic(
    ctx: &RenderContext,
    def: &GraphicDef,
    mut pos: egui::Pos2,
    alpha: f32
) {
    if def.midoffset != 0 {
        pos.x += def.midoffset as f32;
    }

    let patch_name = def.patch.to_uppercase();
    draw_simple_graphic_patch(ctx, &patch_name, pos, def.common.alignment, alpha);
}

pub(super) fn draw_simple_graphic_patch(
    ctx: &RenderContext,
    patch_name: &str,
    pos: egui::Pos2,
    alignment: Alignment,
    alpha: f32,
) {
    if let Some(tex) = ctx.assets.textures.get(patch_name) {
        let size = tex.size_vec2();

        let (off_x, off_y) = ctx
            .assets
            .offsets
            .get(patch_name)
            .map(|(x, y)| (*x as f32, *y as f32))
            .unwrap_or((0.0, 0.0));

        let align_offset = get_alignment_anchor_offset(alignment, size.x, size.y);

        let final_pos = egui::pos2(
            pos.x + align_offset.x - off_x,
            pos.y + align_offset.y - off_y
        );

        let screen_pos = ctx.to_screen(final_pos);

        let screen_size = egui::vec2(
            size.x * ctx.proj.final_scale_x,
            size.y * ctx.proj.final_scale_y,
        );

        let tint = egui::Color32::from_white_alpha((255.0 * alpha) as u8);

        ctx.painter.image(
            tex.id(),
            egui::Rect::from_min_size(screen_pos, screen_size),
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    } else {
        let size = egui::vec2(16.0, 16.0);
        let align_offset = get_alignment_anchor_offset(alignment, size.x, size.y);
        let screen_pos = ctx.to_screen(pos + align_offset);

        let screen_size = egui::vec2(
            size.x * ctx.proj.final_scale_x,
            size.y * ctx.proj.final_scale_y,
        );

        ctx.painter.rect_stroke(
            egui::Rect::from_min_size(screen_pos, screen_size),
            0.0,
            egui::Stroke::new(1.0, egui::Color32::RED),
            egui::StrokeKind::Middle,
        );

        ctx.painter.text(
            screen_pos,
            egui::Align2::LEFT_TOP,
            format!("?{}", patch_name),
            egui::FontId::monospace(8.0 * ctx.proj.final_scale_y),
            egui::Color32::RED,
        );
    }
}