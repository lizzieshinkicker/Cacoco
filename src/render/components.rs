use super::RenderContext;
use super::text::{draw_text_line, measure_text_line};
use crate::model::*;
use eframe::egui;

/// Entry point for rendering specialized engine components (Clock, FPS, etc.).
pub(super) fn draw_component(ctx: &RenderContext, def: &ComponentDef, pos: egui::Pos2, alpha: f32) {
    match def.type_ {
        ComponentType::StatTotals => {
            render_stat_totals(ctx, def, pos, alpha);
        }
        ComponentType::Coordinates => {
            render_coordinates(ctx, def, pos, alpha);
        }
        _ => {
            let text = match def.type_ {
                ComponentType::Time => {
                    let ts = ctx.time as u64;
                    format!("{:02}:{:02}:{:02}", ts / 3600, (ts % 3600) / 60, ts % 60)
                }
                ComponentType::LevelTitle => {
                    let map_format_id = egui::Id::new("use_doom2_map_format");
                    let is_doom2 = ctx
                        .painter
                        .ctx()
                        .data(|d| d.get_temp::<bool>(map_format_id).unwrap_or(true));

                    if is_doom2 {
                        format!("MAP{:02}: ENTRYWAY", ctx.state.world.level)
                    } else {
                        format!(
                            "E{}M{}: ENTRYWAY",
                            ctx.state.world.episode, ctx.state.world.level
                        )
                    }
                }
                ComponentType::FpsCounter => format!("{:.0}", ctx.fps),
                ComponentType::Message => ctx
                    .state
                    .editor
                    .message_log
                    .last()
                    .cloned()
                    .unwrap_or_default(),
                _ => format!("[{:?}]", def.type_),
            };
            draw_text_line(
                ctx,
                &text,
                &def.font,
                pos,
                def.common.alignment,
                false,
                alpha,
            );
        }
    }
}

/// Helper to calculate the multiplier needed to convert Raw Pixels into Virtual Units
fn get_scale_adj(ctx: &RenderContext) -> f32 {
    1.0 / ctx.get_native_scale_factor().0
}

/// Renders the Kills/Items/Secrets block, either horizontally or vertically.
fn render_stat_totals(ctx: &RenderContext, def: &ComponentDef, pos: egui::Pos2, alpha: f32) {
    let p = &ctx.state.player;
    let parts = [
        format!("K: {}/{}", p.kills, p.max_kills),
        format!("I: {}/{}", p.items, p.max_items),
        format!("S: {}/{}", p.secrets, p.max_secrets),
    ];

    let mut cur_pos = pos;
    let scale_adj = get_scale_adj(ctx);

    let line_height = 8.0 * scale_adj;
    let spacing = 8.0 * scale_adj;

    if def.vertical {
        for part in &parts {
            draw_text_line(
                ctx,
                part,
                &def.font,
                cur_pos,
                def.common.alignment,
                false,
                alpha,
            );
            cur_pos.y += line_height;
        }
    } else {
        for part in &parts {
            draw_text_line(
                ctx,
                part,
                &def.font,
                cur_pos,
                def.common.alignment,
                false,
                alpha,
            );
            let width = measure_text_line(ctx, part, &def.font, false) * scale_adj;
            cur_pos.x += width + spacing;
        }
    }
}

fn render_coordinates(ctx: &RenderContext, def: &ComponentDef, pos: egui::Pos2, alpha: f32) {
    let parts = [
        format!("X: {:.0}", ctx.mouse_pos.x),
        format!("Y: {:.0}", ctx.mouse_pos.y),
        "Z: 0".to_string(),
    ];

    let mut cur_pos = pos;
    let scale_adj = get_scale_adj(ctx);
    let line_height = 8.0 * scale_adj;

    if def.vertical {
        for part in &parts {
            draw_text_line(
                ctx,
                part,
                &def.font,
                cur_pos,
                def.common.alignment,
                false,
                alpha,
            );
            cur_pos.y += line_height;
        }
    } else {
        let text = format!("X: {:.0} Y: {:.0} Z: 0", ctx.mouse_pos.x, ctx.mouse_pos.y);
        draw_text_line(
            ctx,
            &text,
            &def.font,
            pos,
            def.common.alignment,
            false,
            alpha,
        );
    }
}
