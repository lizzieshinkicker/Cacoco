use super::RenderContext;
use super::text::{draw_text_line, measure_text_line};
use crate::model::*;
use eframe::egui;

pub(super) fn draw_component(ctx: &RenderContext, def: &ComponentDef, pos: egui::Pos2, alpha: f32) {
    match def.type_ {
        ComponentType::StatTotals => {
            render_stat_totals(ctx, def, pos, alpha);
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
                ComponentType::Coordinates => {
                    format!("X: {:.0} Y: {:.0} Z: 0", ctx.mouse_pos.x, ctx.mouse_pos.y)
                }
                ComponentType::Message => ctx.state.message_log.last().cloned().unwrap_or_default(),
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

fn render_stat_totals(ctx: &RenderContext, def: &ComponentDef, pos: egui::Pos2, alpha: f32) {
    let p = &ctx.state.player;
    let parts = [
        format!("K: {}/{}", p.kills, p.max_kills),
        format!("I: {}/{}", p.items, p.max_items),
        format!("S: {}/{}", p.secrets, p.max_secrets),
    ];

    let mut cur_pos = pos;

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
            cur_pos.y += 8.0;
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
            cur_pos.x += measure_text_line(ctx, part, &def.font, false) + 8.0;
        }
    }
}
