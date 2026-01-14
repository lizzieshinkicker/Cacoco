use super::{RenderContext, get_alignment_anchor_offset};
use crate::assets::AssetId;
use crate::model::*;
use eframe::egui;

/// Renders a single static Graphic element into the viewport.
pub(super) fn draw_graphic(ctx: &RenderContext, def: &GraphicDef, mut pos: egui::Pos2, alpha: f32) {
    if def.midoffset != 0 {
        pos.x += def.midoffset as f32;
    }

    let patch_id = AssetId::new(&def.patch);
    draw_simple_graphic_patch(ctx, patch_id, pos, def.common.alignment, alpha, &def.crop);
}

/// A low-level primitive for drawing a Doom patch or flat by its AssetId.
pub(super) fn draw_simple_graphic_patch(
    ctx: &RenderContext,
    patch_id: AssetId,
    pos: egui::Pos2,
    alignment: Alignment,
    alpha: f32,
    crop: &Option<CropDef>,
) {
    if let Some(tex) = ctx.assets.textures.get(&patch_id) {
        let mut size = tex.size_vec2();
        let (base_scale_x, base_scale_y) = ctx.get_native_scale_factor();

        size.x /= base_scale_x;
        size.y /= base_scale_y;

        let (off_x, off_y) = ctx
            .assets
            .offsets
            .get(&patch_id)
            .map(|(x, y)| (*x as f32, *y as f32))
            .unwrap_or((0.0, 0.0));

        let mut final_off_x = off_x;
        let mut final_off_y = off_y;

        if alignment.contains(Alignment::NO_LEFT_OFFSET) {
            final_off_x = 0.0;
        }
        if alignment.contains(Alignment::NO_TOP_OFFSET) {
            final_off_y = 0.0;
        }

        let mut uv_rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

        if let Some(c) = crop {
            let mut crop_l = c.left as f32;
            let mut crop_t = c.top as f32;
            let raw_sz = tex.size_vec2();

            if c.center {
                crop_l += (raw_sz.x / 2.0).floor();
                crop_t += (raw_sz.y / 2.0).floor();
            }

            let crop_w = (c.width as f32).max(1.0).min(raw_sz.x - crop_l);
            let crop_h = (c.height as f32).max(1.0).min(raw_sz.y - crop_t);

            uv_rect = egui::Rect::from_min_max(
                egui::pos2(crop_l / raw_sz.x, crop_t / raw_sz.y),
                egui::pos2((crop_l + crop_w) / raw_sz.x, (crop_t + crop_h) / raw_sz.y),
            );

            size = egui::vec2(crop_w / base_scale_x, crop_h / base_scale_y);
        }

        let self_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, size);
        let align_offset = get_alignment_anchor_offset(alignment, self_rect);

        let final_pos = egui::pos2(
            pos.x + align_offset.x + final_off_x,
            pos.y + align_offset.y + final_off_y,
        );

        let screen_pos = ctx.to_screen(final_pos);
        let (sc_x, sc_y) = ctx.get_render_scale();

        let screen_size = egui::vec2(size.x * sc_x, size.y * sc_y);
        let tint = egui::Color32::from_white_alpha((255.0 * alpha) as u8);

        ctx.painter.image(
            tex.id(),
            egui::Rect::from_min_size(screen_pos, screen_size),
            uv_rect,
            tint,
        );
    } else {
        draw_missing_patch_placeholder(ctx, patch_id, pos, alignment);
    }
}

/// Renders a red-stroked box and the asset name for missing textures.
fn draw_missing_patch_placeholder(
    ctx: &RenderContext,
    patch_id: AssetId,
    pos: egui::Pos2,
    alignment: Alignment,
) {
    let size = egui::vec2(16.0, 16.0);
    let self_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, size);
    let align_offset = get_alignment_anchor_offset(alignment, self_rect);

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

    let name = ctx
        .assets
        .names
        .get(&patch_id)
        .cloned()
        .unwrap_or_else(|| "UNKNOWN".to_string());

    ctx.painter.text(
        screen_pos,
        egui::Align2::LEFT_TOP,
        format!("?{}", name),
        egui::FontId::monospace(8.0 * ctx.proj.final_scale_y),
        egui::Color32::RED,
    );
}
