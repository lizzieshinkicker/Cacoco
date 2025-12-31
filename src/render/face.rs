use super::RenderContext;
use super::graphic::draw_simple_graphic_patch;
use crate::assets::AssetId;
use crate::model::*;
use eframe::egui;

/// Renders the Doom player face (STF) based on current health and look direction.
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

    let patch_name = ctx.state.player.get_face_sprite(
        ouch,
        look_dir,
        ctx.state.editor.pain_timer,
        ctx.state.editor.evil_timer,
    );

    let patch_id = AssetId::new(&patch_name);

    draw_simple_graphic_patch(ctx, patch_id, pos, def.common.alignment, alpha, &def.crop);
}

/// Renders the static face background (multiplayer color block).
pub(super) fn draw_face_background(
    ctx: &RenderContext,
    def: &FaceDef,
    pos: egui::Pos2,
    alpha: f32,
) {
    let patch_id = AssetId::new("STFB0");
    draw_simple_graphic_patch(ctx, patch_id, pos, def.common.alignment, alpha, &def.crop);
}
