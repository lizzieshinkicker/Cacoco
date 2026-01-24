use crate::assets::AssetStore;
use crate::models::skydefs::SkyDef;
use crate::render::projection::ViewportProjection;
use crate::state::PreviewState;
use eframe::egui;

/// Renders a SKYDEFS entry using the ID24 cylindrical projection.
pub fn draw_sky_view(
    painter: &egui::Painter,
    sky: &SkyDef,
    assets: &AssetStore,
    state: &PreviewState,
    proj: &ViewportProjection,
    time: f64,
) {
    let id = assets.resolve_sky_id(&sky.name);

    if let Some(tex) = assets.textures.get(&id) {
        let rect = proj.screen_rect;
        let sky_tex_w = tex.size()[0] as f32;
        let sky_tex_h = tex.size()[1] as f32;

        if sky_tex_w <= 0.0 || sky_tex_h <= 0.0 {
            return;
        }

        let world_scale_x = sky.scalex.max(0.01);
        let world_scale_y = sky.scaley.max(0.01);

        let view_width_texels = 256.0 / world_scale_x;
        let scroll_x = time as f32 * sky.scrollx;
        let yaw_offset = (state.editor.sky_yaw as f32) + scroll_x;

        let scroll_y = time as f32 * sky.scrolly;
        let v_center_texel = sky.mid + scroll_y + state.editor.weapon_offset_y;
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
