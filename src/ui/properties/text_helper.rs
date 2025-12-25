use crate::model::{ElementWrapper, GraphicDef, CommonAttrs, Element};
use crate::assets::AssetStore;
use crate::ui::properties::font_cache::FontCache;
use crate::ui::properties::common;
use eframe::egui;

pub fn rebake_text(element: &mut ElementWrapper, assets: &AssetStore, fonts: &FontCache) {
    let (text, font_name, spacing) = if let Some(helper) = &element._cacoco_text {
        (helper.text.clone(), helper.font.clone(), helper.spacing)
    } else {
        return;
    };

    let (stem, is_number_font) = if let Some(s) = fonts.get_hud_stem(&font_name) {
        (s, false)
    } else if let Some(s) = fonts.get_number_stem(&font_name) {
        (s, true)
    } else {
        (font_name.clone(), false)
    };

    element.get_common_mut().children.clear();

    let mut current_x = 0;

    for c in text.chars() {
        if c == ' ' {
            current_x += 4 + spacing;
            continue;
        }

        let patch_name = assets.resolve_patch_name(&stem, c, is_number_font);

        let width = if let Some(tex) = assets.textures.get(&patch_name) {
            tex.size()[0] as i32
        } else {
            8
        };

        let letter_graphic = ElementWrapper {
            data: Element::Graphic(GraphicDef {
                common: CommonAttrs {
                    x: current_x,
                    y: 0,
                    ..Default::default()
                },
                patch: patch_name,
                ..Default::default()
            }),
            ..Default::default()
        };

        element.get_common_mut().children.push(letter_graphic);
        current_x += width + spacing;
    }
}

pub fn draw_text_helper_editor(
    ui: &mut egui::Ui,
    element: &mut ElementWrapper,
    fonts: &FontCache,
    assets: &AssetStore
) -> bool {
    let mut bake_needed = false;
    let mut changed = false;
    let helper = element._cacoco_text.as_mut().unwrap();

    ui.horizontal(|ui| {
        ui.label("Text:");
        if ui.text_edit_singleline(&mut helper.text).changed() {
            bake_needed = true;
            changed = true;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Font:");
        let old_font = helper.font.clone();

        egui::ComboBox::from_id_salt("text_helper_font")
            .selected_text(helper.font.clone())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                let count = fonts.hud_font_names.len() + fonts.number_font_names.len();
                let h = (count as f32 * 42.0).min(250.0);
                ui.set_min_height(h);

                for (i, name) in fonts.hud_font_names.iter().enumerate() {
                    let stem = fonts.get_hud_stem(name);
                    changed |= common::draw_font_selection_row(ui, &mut helper.font, name, stem.as_ref(), assets, false, i);
                }

                for (i, name) in fonts.number_font_names.iter().enumerate() {
                    let stem = fonts.get_number_stem(name);
                    changed |= common::draw_font_selection_row(ui, &mut helper.font, name, stem.as_ref(), assets, true, i);
                }
            });

        if helper.font != old_font {
            bake_needed = true;
            changed = true;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Spacing:");
        if ui.add(egui::DragValue::new(&mut helper.spacing)).changed() {
            bake_needed = true;
            changed = true;
        }
    });

    ui.add_space(8.0);
    if ui.button("Explode to Graphics").clicked() {
        element._cacoco_text = None;
        changed = true;
    }

    if bake_needed {
        rebake_text(element, assets, fonts);
    }

    changed
}