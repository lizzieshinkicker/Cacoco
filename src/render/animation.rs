use super::RenderContext;
use super::graphic::draw_simple_graphic_patch;
use crate::assets::AssetId;
use crate::models::sbardef::*;
use eframe::egui;

/// Renders an animation sequence by selecting a frame based on elapsed time.
pub(super) fn draw_animation(ctx: &RenderContext, def: &AnimationDef, pos: egui::Pos2, alpha: f32) {
    if def.frames.is_empty() {
        return;
    }

    let total_duration: f64 = def.frames.iter().map(|f| f.duration).sum();
    if total_duration <= 0.0 {
        return;
    }

    let anim_time = ctx.time % total_duration;

    let mut time_accumulator = 0.0;
    let mut patch_name = &def.frames[0].lump;

    for frame in &def.frames {
        time_accumulator += frame.duration;
        if anim_time < time_accumulator {
            patch_name = &frame.lump;
            break;
        }
    }

    let patch_id = AssetId::new(patch_name);

    draw_simple_graphic_patch(ctx, patch_id, pos, def.common.alignment, alpha, &None);
}
