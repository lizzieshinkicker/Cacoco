use crate::ui::properties::editor::ViewportContext;

/// Renders the infinite background dot grid. Culls to a rational amount of dots.
pub fn draw_grid(painter: &eframe::egui::Painter, ctx: &ViewportContext) {
    let virtual_spacing = 20.0;
    let dot_color = eframe::egui::Color32::from_gray(30);
    let viewport_rect = ctx.viewport_res.rect;

    let virt_min = ctx.proj.to_virtual(viewport_rect.min);
    let virt_max = ctx.proj.to_virtual(viewport_rect.max);

    let start_vx = (virt_min.x / virtual_spacing).floor() * virtual_spacing;
    let start_vy = (virt_min.y / virtual_spacing).floor() * virtual_spacing;
    let end_vx = (virt_max.x / virtual_spacing).ceil() * virtual_spacing;
    let end_vy = (virt_max.y / virtual_spacing).ceil() * virtual_spacing;

    let dot_radius = (1.5 * ctx.proj.final_scale_x).clamp(0.5, 3.0);
    let cols = ((end_vx - start_vx) / virtual_spacing).max(0.0) as usize + 1;
    let rows = ((end_vy - start_vy) / virtual_spacing).max(0.0) as usize + 1;
    let dot_count = cols * rows;

    const DOT_LIMIT: usize = 5_000;
    if dot_count <= DOT_LIMIT {
        let mut vx = start_vx;
        while vx <= end_vx {
            let mut vy = start_vy;
            while vy <= end_vy {
                let center = ctx.proj.to_screen(eframe::egui::pos2(vx, vy));
                painter.circle_filled(center, dot_radius, dot_color);
                vy += virtual_spacing;
            }
            vx += virtual_spacing;
        }
    }
}
