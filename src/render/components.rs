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
                ComponentType::LevelTitle => "MAP01: ENTRYWAY".to_string(),
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
    let parts = ["K: 0/19", "I: 0/9", "S: 0/5"];
    let mut cur_pos = pos;

    if def.vertical {
        for p in parts {
            draw_text_line(
                ctx,
                p,
                &def.font,
                cur_pos,
                def.common.alignment,
                false,
                alpha,
            );
            cur_pos.y += 8.0;
        }
    } else {
        for p in parts {
            draw_text_line(
                ctx,
                p,
                &def.font,
                cur_pos,
                def.common.alignment,
                false,
                alpha,
            );
            cur_pos.x += measure_text_line(ctx, p, &def.font, false) + 8.0;
        }
    }
}
