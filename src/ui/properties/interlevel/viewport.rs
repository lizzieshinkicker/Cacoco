use crate::assets::AssetId;
use crate::document::actions::DocumentAction;
use crate::models::interlevel::InterlevelDefFile;
use crate::ui::properties::editor::ViewportContext;
use eframe::egui;

use super::SCREEN_IDX_KEY;
use super::frames::get_active_frame_index;

fn eval_conditions(
    conds: &[crate::models::interlevel::InterlevelCondition],
    state: &crate::state::viewer::ViewerState,
) -> bool {
    if conds.is_empty() {
        return true;
    }
    for c in conds {
        let passed = match c.condition {
            0 => true,
            1 => state.ilvl_current_map > c.param,
            2 => state.ilvl_current_map == c.param,
            3 => {
                state.ilvl_current_map == c.param
                    || (state.ilvl_earlier_visited && state.ilvl_current_map > c.param)
            }
            4 => !state.ilvl_is_secret_map,
            5 => state.ilvl_secret_visited,
            6 => state.ilvl_is_tally,
            7 => !state.ilvl_is_tally,
            _ => false,
        };
        if !passed {
            return false;
        }
    }
    true
}

pub(super) fn render_viewport_impl(
    file: &InterlevelDefFile,
    ui: &mut egui::Ui,
    ctx: &mut ViewportContext,
) -> Vec<DocumentAction> {
    let screen_idx = ctx
        .current_item_idx
        .min(file.screens.len().saturating_sub(1));
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new(SCREEN_IDX_KEY), screen_idx));

    if file.screens.is_empty() {
        return Vec::new();
    }

    let screen = &file.screens[screen_idx];
    let painter = ui.painter();
    let bg_id = AssetId::new(&screen.data.backgroundimage);

    if let Some(tex) = ctx.assets.textures.get(&bg_id) {
        let screen_w = if ctx.state.sim.engine.widescreen_mode {
            crate::constants::DOOM_W_WIDE
        } else {
            crate::constants::DOOM_W
        };

        let center_x = screen_w / 2.0;
        let center_y = crate::constants::DOOM_H / 2.0;

        let virtual_rect = egui::Rect::from_center_size(
            egui::pos2(center_x, center_y),
            egui::vec2(tex.size()[0] as f32, tex.size()[1] as f32),
        );

        let screen_rect = egui::Rect::from_min_max(
            ctx.proj.to_screen(virtual_rect.min),
            ctx.proj.to_screen(virtual_rect.max),
        );
        painter.image(
            tex.id(),
            screen_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        painter.rect_filled(ctx.proj.screen_rect, 0.0, egui::Color32::BLACK);
    }

    let current_time = ui.input(|i| i.time);

    for layer in &screen.data.layers {
        if !eval_conditions(&layer.conditions, &ctx.state.viewer) {
            continue;
        }

        for anim in &layer.anims {
            if !eval_conditions(&anim.conditions, &ctx.state.viewer) {
                continue;
            }

            if anim.frames.is_empty() {
                continue;
            }

            let active_idx = get_active_frame_index(&anim.frames, current_time).unwrap_or(0);
            let active_frame = &anim.frames[active_idx];

            let frame_id = AssetId::new(&active_frame.image);
            if let Some(tex) = ctx.assets.textures.get(&frame_id) {
                let mut x = anim.x as f32;
                let y = anim.y as f32;

                if (active_frame.frame_type & 0x8000000) != 0 {
                    if ctx.state.sim.engine.widescreen_mode {
                        x += (crate::constants::DOOM_W_WIDE - crate::constants::DOOM_W) / 2.0;
                    }
                }

                let (left_offset, top_offset) =
                    ctx.assets.offsets.get(&frame_id).copied().unwrap_or((0, 0));

                let draw_x = x - (left_offset as f32);
                let draw_y = y - (top_offset as f32);

                let virtual_rect = egui::Rect::from_min_size(
                    egui::pos2(draw_x, draw_y),
                    egui::vec2(tex.size()[0] as f32, tex.size()[1] as f32),
                );

                let screen_rect = egui::Rect::from_min_max(
                    ctx.proj.to_screen(virtual_rect.min),
                    ctx.proj.to_screen(virtual_rect.max),
                );
                painter.image(
                    tex.id(),
                    screen_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }
    }

    ui.ctx().request_repaint();
    Vec::new()
}
