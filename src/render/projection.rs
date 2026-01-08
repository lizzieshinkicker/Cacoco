use crate::constants::{DOOM_H, DOOM_W, DOOM_W_WIDE};
use eframe::egui;

pub struct ViewportProjection {
    pub screen_rect: egui::Rect,
    pub final_scale_x: f32,
    pub final_scale_y: f32,
    pub origin_x: f32,
}

impl ViewportProjection {
    pub fn new(
        avail_rect: egui::Rect,
        widescreen: bool,
        aspect_correct: bool,
        zoom_override: Option<i32>,
        pan_offset: egui::Vec2,
    ) -> Self {
        let correction = if aspect_correct { 1.2 } else { 1.0 };
        let base_h = DOOM_H;
        let base_w = if widescreen { DOOM_W_WIDE } else { DOOM_W };

        let final_scale_x;
        let final_scale_y;

        if let Some(zoom) = zoom_override {
            final_scale_x = zoom as f32;
            final_scale_y = (zoom as f32) * correction;
        } else {
            let scale_x = avail_rect.width() / base_w;
            let scale_y = avail_rect.height() / (base_h * correction);
            let base_scale = scale_x.min(scale_y).floor().max(1.0);
            final_scale_x = base_scale;
            final_scale_y = base_scale * correction;
        }

        let virtual_w = base_w * final_scale_x;
        let virtual_h = base_h * final_scale_y;

        let offset_x = (avail_rect.center().x - (virtual_w / 2.0) + pan_offset.x).round();
        let offset_y = (avail_rect.center().y - (virtual_h / 2.0) + pan_offset.y).round();

        Self {
            screen_rect: egui::Rect::from_min_size(
                egui::pos2(offset_x, offset_y),
                egui::vec2(virtual_w, virtual_h),
            ),
            final_scale_x,
            final_scale_y,
            origin_x: if widescreen {
                (base_w - DOOM_W) / 2.0
            } else {
                0.0
            },
        }
    }

    pub fn from_engine(avail_rect: egui::Rect, engine: &crate::state::EngineContext) -> Self {
        Self::new(
            avail_rect,
            engine.widescreen_mode,
            engine.aspect_correction,
            if engine.auto_zoom {
                None
            } else {
                Some(engine.zoom_level)
            },
            engine.pan_offset,
        )
    }

    pub fn to_screen(&self, pos: egui::Pos2) -> egui::Pos2 {
        let virtual_x = pos.x.floor();
        let virtual_y = pos.y.floor();

        let x = virtual_x * self.final_scale_x;
        let y = virtual_y * self.final_scale_y;

        (self.screen_rect.min + egui::vec2(x, y)).floor()
    }

    pub fn to_virtual(&self, pos: egui::Pos2) -> egui::Pos2 {
        let local_x = (pos.x - self.screen_rect.min.x) / self.final_scale_x;
        let local_y = (pos.y - self.screen_rect.min.y) / self.final_scale_y;
        egui::pos2(local_x, local_y)
    }
}
