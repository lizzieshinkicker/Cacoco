use crate::assets::{AssetId, AssetStore};
use crate::constants::{DOOM_W, DOOM_W_WIDE};
use crate::model::FaceDef;
use crate::state::PreviewState;
use crate::ui::shared::VIEWPORT_RECT_ID;
use eframe::egui;

use super::editor::PropertiesUI;
use super::font_cache::FontCache;
use super::graphics::draw_crop_editor;
use super::preview::PreviewContent;

impl PropertiesUI for FaceDef {
    /// Face elements only have cropping if the user explicitly enables it.
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        _: &FontCache,
        assets: &AssetStore,
        _: &PreviewState,
    ) -> bool {
        let id = AssetId::new("STFST01");
        let (dw, dh) = assets
            .textures
            .get(&id)
            .map(|t| (t.size()[0] as i32, t.size()[1] as i32))
            .unwrap_or((24, 29));

        draw_crop_editor(ui, &mut self.crop, dw, dh)
    }

    /// Simulates the STF behavior in the preview panel based on mouse position.
    fn get_preview_content(
        &self,
        ui: &egui::Ui,
        _: &FontCache,
        state: &PreviewState,
    ) -> Option<PreviewContent> {
        let screen_w = if state.engine.widescreen_mode {
            DOOM_W_WIDE
        } else {
            DOOM_W
        };

        let anchor_x =
            -crate::render::get_alignment_anchor_offset(self.common.alignment, screen_w, 0.0).x;

        let face_center_x = anchor_x + (self.common.x as f32) + 12.0;
        let dx = state.editor.virtual_mouse_pos.x - face_center_x;
        let threshold = 30.0;

        let look_dir = if dx > threshold {
            0 // Right
        } else if dx < -threshold {
            2 // Left
        } else {
            1 // Forward
        };

        let is_button_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        let viewport_rect: Option<egui::Rect> = ui
            .ctx()
            .data(|d| d.get_temp(egui::Id::new(VIEWPORT_RECT_ID)));
        let pointer_pos = ui.input(|i| i.pointer.latest_pos());

        let is_ouched = if let (Some(rect), Some(pos)) = (viewport_rect, pointer_pos) {
            is_button_down && rect.contains(pos)
        } else {
            false
        };

        Some(PreviewContent::Image(state.player.get_face_sprite(
            is_ouched,
            look_dir,
            state.editor.pain_timer,
            state.editor.evil_timer,
        )))
    }

    fn has_specific_fields(&self) -> bool {
        self.crop.is_some()
    }
}
