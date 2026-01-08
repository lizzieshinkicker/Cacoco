use crate::assets::AssetId;
use crate::conditions;
use crate::model::*;
use crate::render::text::measure_text_size;
use crate::render::{RenderContext, draw_element_wrapper, hit_test};
use eframe::egui;

/// Internal helper to calculate the layout positions for list children.
pub(super) fn get_list_layout(
    ctx: &RenderContext,
    def: &ListDef,
    pos: egui::Pos2,
    parent_visible: bool,
    current_path: &mut Vec<usize>,
) -> (egui::Vec2, Vec<(usize, egui::Pos2, egui::Vec2)>) {
    let mut layout_children = Vec::new();
    for (idx, child) in def.common.children.iter().enumerate() {
        current_path.push(idx);

        let my_conditions_met =
            conditions::resolve(&child.get_common().conditions, ctx.state, ctx.assets);
        let visible_in_game = parent_visible && my_conditions_met;
        let is_selected_branch = ctx.selection.contains(current_path)
            || ctx.selection.iter().any(|s| current_path.starts_with(s));
        let is_ancestor_of_selection = ctx.selection.iter().any(|s| s.starts_with(current_path));

        if visible_in_game || is_selected_branch || is_ancestor_of_selection {
            layout_children.push((idx, child, estimate_element_tree_size(ctx, child)));
        }
        current_path.pop();
    }

    if layout_children.is_empty() {
        return (egui::Vec2::ZERO, Vec::new());
    }

    let mut total_size = egui::Vec2::ZERO;
    for (_, _, sz) in &layout_children {
        if def.horizontal {
            total_size.x += sz.x;
            total_size.y = total_size.y.max(sz.y);
        } else {
            total_size.y += sz.y;
            total_size.x = total_size.x.max(sz.x);
        }
    }

    let spacing = def.spacing as f32;
    if def.horizontal {
        total_size.x += spacing * (layout_children.len() as f32 - 1.0);
    } else {
        total_size.y += spacing * (layout_children.len() as f32 - 1.0);
    }

    let mut global_block_offset = egui::Vec2::ZERO;
    if def.common.alignment.contains(Alignment::RIGHT) {
        global_block_offset.x = -total_size.x;
    } else if def.common.alignment.contains(Alignment::H_CENTER) {
        global_block_offset.x = -(total_size.x / 2.0).floor();
    }
    if def.common.alignment.contains(Alignment::BOTTOM) {
        global_block_offset.y = -total_size.y;
    } else if def.common.alignment.contains(Alignment::V_CENTER) {
        global_block_offset.y = -(total_size.y / 2.0).floor();
    }

    let mut current_stack_pos = 0.0;
    let mut results = Vec::new();

    for (idx, _child, child_size) in layout_children {
        let mut child_draw_pos = pos + global_block_offset;

        if def.horizontal {
            child_draw_pos.x += current_stack_pos;
            if def.common.alignment.contains(Alignment::BOTTOM) {
                child_draw_pos.y += total_size.y - child_size.y;
            } else if def.common.alignment.contains(Alignment::V_CENTER) {
                child_draw_pos.y += ((total_size.y - child_size.y) / 2.0).floor();
            }
            current_stack_pos += child_size.x + spacing;
        } else {
            child_draw_pos.y += current_stack_pos;
            if def.common.alignment.contains(Alignment::RIGHT) {
                child_draw_pos.x += total_size.x - child_size.x;
            } else if def.common.alignment.contains(Alignment::H_CENTER) {
                child_draw_pos.x += ((total_size.x - child_size.x) / 2.0).floor();
            }
            current_stack_pos += child_size.y + spacing;
        }
        results.push((idx, child_draw_pos, child_size));
    }

    (total_size, results)
}

pub(super) fn draw_list(
    ctx: &RenderContext,
    def: &ListDef,
    pos: egui::Pos2,
    _alpha: f32,
    current_path: &mut Vec<usize>,
    parent_visible: bool,
) {
    let (_, layout) = get_list_layout(ctx, def, pos, parent_visible, current_path);

    for (idx, child_draw_pos, _) in layout {
        let child = &def.common.children[idx];
        current_path.push(idx);

        let mut local_child = child.clone();
        let current_align = local_child.get_common().alignment;
        let mut forced_align = Alignment::TOP | Alignment::LEFT;

        if current_align.contains(Alignment::WIDESCREEN_LEFT) {
            forced_align |= Alignment::WIDESCREEN_LEFT;
        }
        if current_align.contains(Alignment::WIDESCREEN_RIGHT) {
            forced_align |= Alignment::WIDESCREEN_RIGHT;
        }
        if current_align.contains(Alignment::NO_LEFT_OFFSET) {
            forced_align |= Alignment::NO_LEFT_OFFSET;
        }
        if current_align.contains(Alignment::NO_TOP_OFFSET) {
            forced_align |= Alignment::NO_TOP_OFFSET;
        }

        local_child.get_common_mut().alignment = forced_align;

        draw_element_wrapper(
            ctx,
            &local_child,
            child_draw_pos,
            current_path,
            parent_visible,
        );
        current_path.pop();
    }
}

pub fn hit_test_list(
    ctx: &RenderContext,
    def: &ListDef,
    pos: egui::Pos2,
    current_path: &mut Vec<usize>,
    parent_visible: bool,
    container_mode: bool,
) -> Option<Vec<usize>> {
    let (_, layout) = get_list_layout(ctx, def, pos, parent_visible, current_path);

    for (idx, child_draw_pos, _) in layout.into_iter().rev() {
        let child = &def.common.children[idx];
        current_path.push(idx);

        let mut local_child = child.clone();
        local_child.get_common_mut().alignment = Alignment::TOP | Alignment::LEFT;

        if let Some(hit) = hit_test(
            ctx,
            &local_child,
            child_draw_pos,
            current_path,
            parent_visible,
            container_mode,
        ) {
            current_path.pop();
            return Some(hit);
        }
        current_path.pop();
    }
    None
}

/// Recursively calculates the visual bounds of an element tree.
fn estimate_element_tree_size(ctx: &RenderContext, element: &ElementWrapper) -> egui::Vec2 {
    if let Element::List(l) = &element.data {
        let (size, _) = get_list_layout(ctx, l, egui::Pos2::ZERO, true, &mut Vec::new());
        return size;
    }

    let (base_sc_x, base_sc_y) = ctx.get_native_scale_factor();
    let scale_factor_x = 1.0 / base_sc_x;
    let scale_factor_y = 1.0 / base_sc_y;

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
            measure_text_size(ctx, sample_text, &n.font, true)
        }
        Element::String(s) => measure_text_size(ctx, "Sample Text", &s.font, false),
        Element::Face(_) | Element::FaceBackground(_) => egui::vec2(24.0, 29.0),
        Element::Component(c) => measure_text_size(ctx, "Sample Text", &c.font, false),
        Element::Canvas(_) | Element::Native(_) | Element::Carousel(_) => egui::Vec2::ZERO,
        Element::List(_) => unreachable!(),
    };

    if ctx.is_native {
        size.x *= scale_factor_x;
        size.y *= scale_factor_y;
    }

    for child in element.children() {
        let child_size = estimate_element_tree_size(ctx, child);
        let common = child.get_common();

        let offset_x = common.x as f32 * scale_factor_x;
        let offset_y = common.y as f32 * scale_factor_y;

        let end_x = offset_x + child_size.x;
        let end_y = offset_y + child_size.y;

        size.x = size.x.max(end_x);
        size.y = size.y.max(end_y);
    }

    size
}
