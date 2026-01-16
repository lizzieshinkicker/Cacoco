use crate::assets::{AssetId, AssetStore};
use crate::models::sbardef::{CanvasDef, CarouselDef, CropDef, ExportTarget, GraphicDef};
use crate::state::PreviewState;
use eframe::egui;

use super::editor::PropertiesUI;
use super::font_cache::FontCache;
use super::preview::PreviewContent;

impl PropertiesUI for GraphicDef {
    /// Renders the specialized editor for SBARDEF Graphic elements.
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        _: &FontCache,
        assets: &AssetStore,
        _: &PreviewState,
    ) -> bool {
        let mut changed = false;
        let target = ui.data(|d| {
            d.get_temp::<ExportTarget>(egui::Id::new("cacoco_current_target"))
                .unwrap_or_default()
        });

        let label_w = 50.0;
        let field_w = 100.0;
        let total_w = label_w + field_w;

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - total_w).max(0.0) / 2.0);
            ui.add_sized([label_w, 18.0], egui::Label::new("Patch:"));
            let edit = egui::TextEdit::singleline(&mut self.patch).desired_width(field_w);
            if ui.add(edit).changed() {
                self.patch = AssetStore::stem(&self.patch);
                changed = true;
            }
        });

        ui.add_space(4.0);

        let id = AssetId::new(&self.patch);
        let (dw, dh) = assets
            .textures
            .get(&id)
            .map(|t| (t.size()[0] as i32, t.size()[1] as i32))
            .unwrap_or((0, 0));

        changed |= draw_crop_editor(ui, &mut self.crop, dw, dh, target);

        changed
    }

    /// Returns the lump name of the defined patch for the preview panel.
    fn get_preview_content(
        &self,
        _: &egui::Ui,
        _: &FontCache,
        _: &PreviewState,
    ) -> Option<PreviewContent> {
        Some(PreviewContent::Image(self.patch.clone()))
    }
}

/// Helper for drawing the cropping interface. Shared across graphical elements.
pub fn draw_crop_editor(
    ui: &mut egui::Ui,
    crop: &mut Option<CropDef>,
    default_w: i32,
    default_h: i32,
    target: ExportTarget,
) -> bool {
    let mut changed = false;
    let mut is_enabled = crop.is_some();

    if target == ExportTarget::Basic && !is_enabled {
        return false;
    }

    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 130.0).max(0.0) / 2.0);

        let mut check_ui = ui.add_enabled(
            target == ExportTarget::Extended,
            egui::Checkbox::new(&mut is_enabled, "Enable Cropping"),
        );

        if target == ExportTarget::Basic {
            check_ui =
                check_ui.on_disabled_hover_text("Cropping is only available for Extended targets.");
        }

        if check_ui.changed() {
            if is_enabled {
                *crop = Some(CropDef {
                    width: default_w,
                    height: default_h,
                    left: 0,
                    top: 0,
                    center: false,
                });
            } else {
                *crop = None;
            }
            changed = true;
        }
    });

    if let Some(c) = crop {
        ui.add_space(4.0);

        let label_w = 60.0;
        let field_w = 50.0;
        let total_w = label_w + field_w;

        let mut draw_row = |label: &str, val: &mut i32, min: i32, max: i32| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - total_w).max(0.0) / 2.0);
                ui.add_sized([label_w, 18.0], egui::Label::new(label));
                changed |= ui
                    .add_sized([field_w, 18.0], egui::DragValue::new(val).range(min..=max))
                    .changed();
            });
        };

        draw_row("Width:", &mut c.width, 0, 4096);
        draw_row("Height:", &mut c.height, 0, 4096);
        draw_row("Left:", &mut c.left, -2048, 2048);
        draw_row("Top:", &mut c.top, -2048, 2048);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 120.0).max(0.0) / 2.0);
            if ui.checkbox(&mut c.center, "Center Offset").changed() {
                if c.center {
                    c.left -= default_w / 2;
                    c.top -= default_h / 2;
                } else {
                    c.left += default_w / 2;
                    c.top += default_h / 2;
                }
                changed = true;
            }
        });
    }
    changed
}

impl PropertiesUI for CanvasDef {
    fn has_specific_fields(&self) -> bool {
        false
    }
}
impl PropertiesUI for CarouselDef {
    fn has_specific_fields(&self) -> bool {
        false
    }
}
