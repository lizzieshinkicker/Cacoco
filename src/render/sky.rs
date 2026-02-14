use crate::assets::{AssetId, AssetStore};
use crate::models::skydefs::{SkyDef, SkyType};
use crate::render::projection::ViewportProjection;
use crate::state::PreviewState;
use eframe::egui;

pub fn draw_sky_view(
    painter: &egui::Painter,
    sky: &SkyDef,
    assets: &AssetStore,
    state: &PreviewState,
    proj: &ViewportProjection,
    time: f64,
) {
    let mut main_tex_id = assets.resolve_sky_id(&sky.name);

    if sky.sky_type == SkyType::Fire {
        let dynamic_key = format!("_FIRE_ANIM_{}", sky.name);
        let dynamic_id = AssetId::new(&dynamic_key);
        if assets.textures.contains_key(&dynamic_id) {
            main_tex_id = dynamic_id;
        }
    }

    draw_single_sky_layer(
        painter,
        main_tex_id,
        sky.mid,
        sky.scrollx,
        sky.scrolly,
        sky.scalex,
        sky.scaley,
        assets,
        state,
        proj,
        time,
    );

    if sky.sky_type == SkyType::WithForeground {
        if let Some(fore) = &sky.foregroundtex {
            let fore_id = assets.resolve_sky_id(&fore.name);
            draw_single_sky_layer(
                painter,
                fore_id,
                fore.mid,
                fore.scrollx,
                fore.scrolly,
                fore.scalex,
                fore.scaley,
                assets,
                state,
                proj,
                time,
            );
        }
    }
}

fn draw_single_sky_layer(
    painter: &egui::Painter,
    id: AssetId,
    mid: f32,
    scroll_x: f32,
    scroll_y: f32,
    scale_x: f32,
    scale_y: f32,
    assets: &AssetStore,
    state: &PreviewState,
    proj: &ViewportProjection,
    time: f64,
) {
    if let Some(tex) = assets.textures.get(&id) {
        let rect = proj.screen_rect;
        let sky_tex_w = tex.size()[0] as f32;
        let sky_tex_h = tex.size()[1] as f32;

        if sky_tex_w <= 0.0 || sky_tex_h <= 0.0 {
            return;
        }

        let world_scale_x = scale_x.max(0.01);
        let world_scale_y = scale_y.max(0.01);

        let view_width_texels = 256.0 / world_scale_x;
        let scroll_x_val = time as f32 * scroll_x;
        let yaw_offset = (state.viewer.sky_yaw as f32) + scroll_x_val;

        let scroll_y_val = time as f32 * scroll_y;
        let v_center_texel = mid + scroll_y_val + state.viewer.weapon_offset_y;
        let v_half_extent_texels = 80.0 / world_scale_y;

        let mut mesh = egui::Mesh::with_texture(tex.id());
        let uv_max_x = yaw_offset / sky_tex_w;
        let uv_min_x = (yaw_offset + view_width_texels) / sky_tex_w;
        let uv_min_y = (v_center_texel - v_half_extent_texels) / sky_tex_h;
        let uv_max_y = (v_center_texel + v_half_extent_texels) / sky_tex_h;

        mesh.add_rect_with_uv(
            rect,
            egui::Rect::from_min_max(
                egui::pos2(uv_min_x, uv_min_y),
                egui::pos2(uv_max_x, uv_max_y),
            ),
            egui::Color32::WHITE,
        );
        painter.add(mesh);
    }
}
