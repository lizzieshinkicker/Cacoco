use super::RenderContext;
use super::graphic::draw_simple_graphic_patch;
use crate::model::*;
use eframe::egui;

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
    let mut patch_to_draw = &def.frames[0].lump;

    for frame in &def.frames {
        time_accumulator += frame.duration;
        if anim_time < time_accumulator {
            patch_to_draw = &frame.lump;
            break;
        }
    }

    draw_simple_graphic_patch(
        ctx,
        &patch_to_draw.to_uppercase(),
        pos,
        def.common.alignment,
        alpha,
    );
}
