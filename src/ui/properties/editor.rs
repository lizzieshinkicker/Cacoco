use super::font_cache::FontCache;
use super::preview::PreviewContent;
use crate::app::ConfirmationRequest;
use crate::assets::AssetStore;
use crate::document::DocumentAction;
use crate::models::sbardef::ExportTarget;
use crate::state::PreviewState;
use crate::ui::font_wizard::FontWizardState;
use eframe::egui;
use std::collections::HashSet;

/// Bundles common data needed by all property editors.
pub struct PropertyContext<'a> {
    pub selection: &'a HashSet<Vec<usize>>,
    pub assets: &'a AssetStore,
    pub state: &'a PreviewState,
    pub target: ExportTarget,
}

/// Data passed during the per-frame update phase.
pub struct TickContext<'a> {
    pub ctx: &'a egui::Context,
    pub assets: &'a mut AssetStore,
    pub state: &'a mut PreviewState,
    pub time: f64,
}

/// Data passed to draw the sidebar layer/list tree.
pub struct LayerContext<'a> {
    pub selection: &'a mut HashSet<Vec<usize>>,
    pub selection_pivot: &'a mut Option<Vec<usize>>,
    pub assets: &'a mut AssetStore,
    pub state: &'a mut PreviewState,
    pub current_item_idx: &'a mut usize,
    #[allow(dead_code)]
    pub wizard_state: &'a mut Option<FontWizardState>,
    pub confirmation_modal: &'a mut Option<ConfirmationRequest>,
}

/// A trait for any Cacoco-handled lump that provides a user interface for editing its properties.
pub trait LumpUI {
    /// Draws the property editor for this lump. Returns true if data was modified.
    fn draw_properties(&mut self, ui: &mut egui::Ui, ctx: &PropertyContext) -> bool;

    /// Optional: Per-frame logic (animations, simulations).
    fn tick(&self, _ctx: &mut TickContext) {}

    /// Draws the sidebar list. Returns (Actions to execute, has data changed).
    fn draw_layer_list(
        &mut self,
        _ui: &mut egui::Ui,
        _ctx: &mut LayerContext,
    ) -> (Vec<DocumentAction>, bool) {
        (Vec::new(), false)
    }

    /// Returns the header information (Title, Description, Background Color) for the panel.
    fn header_info(&self, selection: &HashSet<Vec<usize>>) -> (String, String, egui::Color32);

    /// (Optional) Returns visual content to be rendered in the top preview panel.
    fn get_preview_content(&self, _: &egui::Ui, _: &PropertyContext) -> Option<PreviewContent> {
        None
    }

    /// Renders the lump content into the viewport and handles internal interactions.
    fn render_viewport(
        &self,
        _ui: &mut egui::Ui,
        _ctx: &mut ViewportContext,
    ) -> Vec<DocumentAction> {
        Vec::new()
    }
}

/// A trait for any element that provides a user interface for editing its properties.
///
/// Allows an element type to define its own specialized widgets for the Properties panel
/// and its own logic for the visual preview window.
pub trait PropertiesUI {
    /// Draws the UI widgets specifically associated with this element's data type.
    ///
    /// Returns `true` if any data was modified by the user during this frame,
    /// triggering a "dirty" state.
    fn draw_specific_fields(
        &mut self,
        _ui: &mut egui::Ui,
        _fonts: &FontCache,
        _assets: &AssetStore,
        _state: &PreviewState,
    ) -> bool {
        false
    }

    /// Renders a scaled version of the element (like a specific patch
    /// or a sample of a font) at the top of the properties panel in the property preview window.
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

/// Bundles data and interaction state for viewport rendering.
pub struct ViewportContext<'a> {
    pub assets: &'a AssetStore,
    pub state: &'a mut PreviewState,
    pub proj: &'a crate::render::projection::ViewportProjection,
    pub selection: &'a HashSet<Vec<usize>>,
    pub current_item_idx: usize,
    pub is_panning: bool,
    pub container_mode: bool,
    pub selection_mode: bool,
    pub primary_pressed: bool,
    pub primary_down: bool,
    pub viewport_res: &'a egui::Response,
}
