use super::font_cache::FontCache;
use super::preview::PreviewContent;
use crate::assets::AssetStore;
use crate::state::PreviewState;
use eframe::egui;

/// A trait for any SBARDEF element that provides a user interface for editing its properties.
///
/// This trait decouples the UI logic from the core data model. Implementing this trait
/// allows an element type to define its own specialized widgets for the Properties panel
/// and its own logic for the visual preview window.
pub trait PropertiesUI {
    /// Draws the UI widgets specifically associated with this element's data type.
    ///
    /// This is called within the "Properties" tab of the sidebar.
    ///
    /// # Returns
    /// `true` if any data was modified by the user during this frame, triggering a "dirty" state.
    fn draw_specific_fields(
        &mut self,
        _ui: &mut egui::Ui,
        _fonts: &FontCache,
        _assets: &AssetStore,
        _state: &PreviewState,
    ) -> bool {
        false
    }

    /// Generates a description of the content to be displayed in the property preview window.
    ///
    /// This allows the UI to render a scaled version of the element (like a specific patch
    /// or a sample of a font) at the top of the properties panel.
    fn get_preview_content(
        &self,
        _ui: &egui::Ui,
        _fonts: &FontCache,
        _state: &PreviewState,
    ) -> Option<PreviewContent> {
        None
    }

    /// Returns `true` if this element has unique fields beyond the standard transform/conditions.
    ///
    /// If this returns `false`, the specialized editor section in the sidebar will be
    /// hidden to keep the UI clean.
    fn has_specific_fields(&self) -> bool {
        true
    }
}
