use eframe::egui;
use crate::model::*;
use super::RenderContext;
use super::graphic::draw_simple_graphic_patch;

pub(super) fn draw_face(
    ctx: &RenderContext,
    def: &FaceDef,
    pos: egui::Pos2,
    alpha: f32,
    ouch: bool,
) {
    let mut look_dir = 1;

    if !ouch {
        let face_center_x = pos.x + 12.0;
        let dx = ctx.mouse_pos.x - face_center_x;
        let threshold = 30.0;

        if dx > threshold {
            look_dir = 0;
        } else if dx < -threshold {
            look_dir = 2;
        }
    }

    let patch = ctx.state.get_face_sprite(ouch, look_dir);
    draw_simple_graphic_patch(ctx, &patch, pos, def.common.alignment, alpha);
}

pub(super) fn draw_face_background(
    ctx: &RenderContext,
    def: &FaceDef,
    pos: egui::Pos2,
    alpha: f32
) {
    draw_simple_graphic_patch(ctx, "STFB0", pos, def.common.alignment, alpha);
}