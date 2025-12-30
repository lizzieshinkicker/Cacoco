use super::font_cache::FontCache;
use super::preview::PreviewContent;
use crate::assets::AssetStore;
use crate::state::PreviewState;
use eframe::egui;

/// A trait for any SBARDEF element that can be edited in the properties panel.
pub trait PropertiesUI {
    /// Draws the fields specific to this element type.
    fn draw_specific_fields(
        &mut self,
        _ui: &mut egui::Ui,
        _fonts: &FontCache,
        _assets: &AssetStore,
        _state: &PreviewState,
    ) -> bool {
        false
    }

    /// Returns the content needed for the preview panel.
    fn get_preview_content(
        &self,
        _ui: &egui::Ui,
        _fonts: &FontCache,
        _state: &PreviewState,
    ) -> Option<PreviewContent> {
        None
    }

    /// Check if we have anything additional to show in the properties.
    fn has_specific_fields(&self) -> bool {
        true
    }
}
