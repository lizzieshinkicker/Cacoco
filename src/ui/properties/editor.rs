use eframe::egui;
use crate::assets::AssetStore;
use crate::state::PreviewState;
use super::font_cache::FontCache;
use super::preview::PreviewContent;

/// A trait for any SBARDEF element that can be edited in the properties panel.
pub trait PropertiesUI {
    /// Draws the fields specific to this element type (e.g., "Patch", "Font", etc.).
    fn draw_specific_fields(&mut self, ui: &mut egui::Ui, fonts: &FontCache, assets: &AssetStore, state: &PreviewState);

    /// Returns the content needed for the preview panel at the top.
    fn get_preview_content(&self, ui: &egui::Ui, fonts: &FontCache, state: &PreviewState) -> Option<PreviewContent>;

    /// Check if we have anything additional to show in the properties.
    fn has_specific_fields(&self) -> bool { true }
}