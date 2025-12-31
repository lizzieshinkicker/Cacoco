use crate::assets::AssetId;
use crate::model::*;
use crate::render::{RenderContext, draw_element_wrapper, text::measure_text_line};
use eframe::egui;

/// Renders a SBARDEF List element, stacking children based on its spacing rules.
pub(super) fn draw_list(
    ctx: &RenderContext,
    def: &ListDef,
    pos: egui::Pos2,
    _alpha: f32,
    current_path: &mut Vec<usize>,
) {
    let mut current_offset = 0.0;

    let mut children_indices: Vec<usize> = (0..def.common.children.len()).collect();
    if def.reverse {
        children_indices.reverse();
    }

    for &idx in &children_indices {
        let child = &def.common.children[idx];
        current_path.push(idx);

        let child_size = estimate_element_tree_size(ctx, child);

        let mut child_pos = pos;
        if def.horizontal {
            child_pos.x += current_offset;
            current_offset += child_size.x + def.spacing as f32;
        } else {
            child_pos.y += current_offset;
            current_offset += child_size.y + def.spacing as f32;
        }

        draw_element_wrapper(ctx, child, child_pos, current_path);
        current_path.pop();
    }
}

/// Recursively calculates the visual bounds of an element tree.
///
/// Used by the List container to arrange children without overlap.
fn estimate_element_tree_size(ctx: &RenderContext, element: &ElementWrapper) -> egui::Vec2 {
    let mut size = match &element.data {
        Element::Graphic(g) => {
            let id = AssetId::new(&g.patch);
            if let Some(tex) = ctx.assets.textures.get(&id) {
                tex.size_vec2()
            } else {
                egui::vec2(16.0, 16.0)
            }
        }
        Element::Animation(a) => {
            if let Some(frame) = a.frames.first() {
                let id = AssetId::new(&frame.lump);
                if let Some(tex) = ctx.assets.textures.get(&id) {
                    tex.size_vec2()
                } else {
                    egui::vec2(16.0, 16.0)
                }
            } else {
                egui::Vec2::ZERO
            }
        }
        Element::Number(n) | Element::Percent(n) => {
            let sample_text = if matches!(element.data, Element::Percent(_)) {
                "100%"
            } else {
                "100"
            };
            let w = measure_text_line(ctx, sample_text, &n.font, true);
            egui::vec2(w, 12.0)
        }
        Element::String(s) => {
            let w = measure_text_line(ctx, "Sample Text", &s.font, false);
            egui::vec2(w, 12.0)
        }
        Element::Face(_) | Element::FaceBackground(_) => egui::vec2(24.0, 29.0),
        Element::Component(_) => egui::vec2(64.0, 12.0),
        Element::Canvas(_) | Element::List(_) | Element::Carousel(_) => egui::Vec2::ZERO,
    };

    for child in element.children() {
        let child_size = estimate_element_tree_size(ctx, child);
        let common = child.get_common();
        let end_x = common.x as f32 + child_size.x;
        let end_y = common.y as f32 + child_size.y;
        size.x = size.x.max(end_x);
        size.y = size.y.max(end_y);
    }

    size
}
