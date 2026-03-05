use super::{RenderContext, RenderPass};
use crate::models::sbardef::{MinimapBackground, MinimapDef};
use eframe::egui;

pub(super) fn draw_minimap(ctx: &RenderContext, def: &MinimapDef, pos: egui::Pos2, alpha: f32) {
    let rect = egui::Rect::from_min_size(pos, egui::vec2(def.width as f32, def.height as f32));
    let screen_rect = ctx.to_screen_rect(rect);

    match ctx.pass {
        RenderPass::Background => {
            match def.background {
                MinimapBackground::Dark => {
                    let dim_color = egui::Color32::from_black_alpha(150).linear_multiply(alpha);
                    ctx.painter.rect_filled(screen_rect, 0.0, dim_color);
                }
                MinimapBackground::Black => {
                    ctx.painter.rect_filled(
                        screen_rect,
                        0.0,
                        egui::Color32::BLACK.linear_multiply(alpha),
                    );
                }
                MinimapBackground::Off => {}
            }

            let id = crate::assets::AssetId::new("_MINIMAP_PLACEHOLDER");
            if let Some(tex) = ctx.assets.textures.get(&id) {
                let safe_scale = def.scale.max(0.01);
                let tex_size = tex.size_vec2();

                let uv_w = (def.width as f32 / tex_size.x) * (4.0 / safe_scale);
                let uv_h = (def.height as f32 / tex_size.y) * (4.0 / safe_scale);

                let uv_rect =
                    egui::Rect::from_center_size(egui::pos2(0.5, 0.5), egui::vec2(uv_w, uv_h));

                ctx.painter.image(
                    tex.id(),
                    screen_rect,
                    uv_rect,
                    egui::Color32::WHITE.linear_multiply(alpha),
                );
            }
        }
        RenderPass::Foreground => {
            ctx.painter.rect_stroke(
                screen_rect,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::YELLOW),
                egui::StrokeKind::Middle,
            );
        }
    }
}
