use crate::assets::AssetStore;
use crate::conditions;
use crate::model::*;
use crate::state::PreviewState;
use eframe::egui;
use std::collections::HashSet;

pub mod animation;
pub mod canvas;
pub mod components;
pub mod face;
pub mod graphic;
pub mod list;
pub mod palette;
pub mod patch;
pub mod projection;
pub mod text;

/// Defines whether an element is being drawn in the standard background pass
/// or the specialized foreground pass (used for selection highlighting).
#[derive(Clone, Copy, PartialEq)]
pub enum RenderPass {
    Background,
    Foreground,
}

/// The state container passed through the rendering tree.
/// Contains references to all project data and rendering projection.
pub struct RenderContext<'a> {
    pub painter: &'a egui::Painter,
    pub assets: &'a AssetStore,
    pub file: &'a SBarDefFile,
    pub state: &'a PreviewState,
    /// Absolute application time in seconds.
    pub time: f64,
    /// Current smoothed rendering framerate.
    pub fps: f32,
    /// Mouse position in virtual Doom coordinates (0-320 or 0-428).
    pub mouse_pos: egui::Pos2,
    /// The set of paths currently selected in the editor.
    pub selection: &'a HashSet<Vec<usize>>,
    pub pass: RenderPass,
    pub proj: &'a projection::ViewportProjection,
    /// True if the user is currently performing a drag operation in the viewport.
    pub is_dragging: bool,
    /// True if the primary mouse button is currently held down over the viewport.
    pub is_viewport_clicked: bool,
}

impl<'a> RenderContext<'a> {
    /// Maps a virtual coordinate (Doom space) to a physical screen pixel.
    pub fn to_screen(&self, pos: egui::Pos2) -> egui::Pos2 {
        self.proj.to_screen(pos)
    }
}

/// The main recursive entry point for drawing an SBARDEF element and its children.
pub fn draw_element_wrapper(
    ctx: &RenderContext,
    element: &ElementWrapper,
    parent_pos: egui::Pos2,
    current_path: &mut Vec<usize>,
    parent_visible: bool,
) {
    let common = element.get_common();

    let is_selected_branch = ctx.selection.contains(current_path)
        || ctx.selection.iter().any(|s| current_path.starts_with(s));

    let is_ancestor_of_selection = ctx.selection.iter().any(|s| s.starts_with(current_path));

    let is_strobing = ctx.state.editor.strobe_timer > 0.0;

    match ctx.pass {
        RenderPass::Background => {
            if is_selected_branch && is_strobing {
                return;
            }
        }
        RenderPass::Foreground => {
            if !is_strobing || (!is_selected_branch && !is_ancestor_of_selection) {
                return;
            }
        }
    }

    let my_conditions_met = conditions::resolve(&common.conditions, ctx.state, ctx.assets);
    let visible_in_game = parent_visible && my_conditions_met;

    if !visible_in_game && !is_selected_branch && !is_ancestor_of_selection {
        return;
    }

    let mut alpha = if !visible_in_game { 0.30 } else { 1.0 };
    if common.translucency {
        alpha *= 0.5;
    }

    let is_hovered_branch = ctx
        .state
        .editor
        .hovered_path
        .as_ref()
        .map_or(false, |h| current_path.starts_with(h));
    let is_grabbed_branch = ctx
        .state
        .editor
        .grabbed_path
        .as_ref()
        .map_or(false, |g| current_path.starts_with(g));

    let has_active_overlay =
        ctx.state.editor.hovered_path.is_some() || ctx.state.editor.grabbed_path.is_some();

    if is_hovered_branch || is_grabbed_branch {
        let wave = (ctx.time * std::f64::consts::PI * 4.0).cos() as f32;
        alpha *= 0.70 + (wave * 0.30);
    } else if is_selected_branch && !has_active_overlay && is_strobing {
        let dur = 0.5;
        let prog = (dur - ctx.state.editor.strobe_timer) / dur;
        let wave = (prog * std::f32::consts::PI * 4.0).cos();
        alpha *= 0.70 + (wave * 0.30);
    }

    let pos = resolve_position(ctx, common, parent_pos);

    let should_render_content = match ctx.pass {
        RenderPass::Background => true,
        RenderPass::Foreground => is_selected_branch,
    };

    if should_render_content {
        match &element.data {
            Element::Canvas(c) => canvas::draw_canvas(ctx, c, pos, alpha),
            Element::List(l) => list::draw_list(ctx, l, pos, alpha, current_path, visible_in_game),
            Element::Graphic(g) => graphic::draw_graphic(ctx, g, pos, alpha),
            Element::Animation(a) => animation::draw_animation(ctx, a, pos, alpha),
            Element::Face(f) => face::draw_face(
                ctx,
                f,
                pos,
                alpha,
                is_selected_branch && (ctx.is_dragging || ctx.is_viewport_clicked),
            ),
            Element::FaceBackground(fb) => face::draw_face_background(ctx, fb, pos, alpha),
            Element::Number(n) => text::draw_number(ctx, n, pos, false, alpha),
            Element::Percent(p) => text::draw_number(ctx, p, pos, true, alpha),
            Element::String(s) => text::draw_string(ctx, s, pos, alpha),
            Element::Component(c) => components::draw_component(ctx, c, pos, alpha),
            Element::Carousel(_) => {}
        }
    } else {
        if let Element::List(l) = &element.data {
            list::draw_list(ctx, l, pos, alpha, current_path, visible_in_game);
        }
    }

    if !matches!(element.data, Element::List(_)) {
        recurse_children(ctx, &common.children, pos, current_path, visible_in_game);
    }
}

/// Traverses the element tree to find the topmost element at the given virtual position.
pub fn hit_test(
    ctx: &RenderContext,
    element: &ElementWrapper,
    parent_pos: egui::Pos2,
    current_path: &mut Vec<usize>,
    _parent_visible: bool,
    container_mode: bool,
) -> Option<Vec<usize>> {
    let common = element.get_common();
    let pos = resolve_position(ctx, common, parent_pos);

    if !matches!(element.data, Element::List(_)) {
        for (idx, child) in element.children().iter().enumerate().rev() {
            current_path.push(idx);
            let hit = hit_test(ctx, child, pos, current_path, true, container_mode);

            if hit.is_some() {
                if element._cacoco_text.is_some() {
                    let my_path = current_path[..current_path.len() - 1].to_vec();
                    current_path.pop();
                    return Some(my_path);
                }
                return hit;
            }
            current_path.pop();
        }
    } else if let Element::List(l) = &element.data {
        if let Some(hit) = list::hit_test_list(ctx, l, pos, current_path, true, container_mode) {
            if element._cacoco_text.is_some() {
                return Some(current_path.clone());
            }
            return Some(hit);
        }
    }

    let is_atomic_text = element._cacoco_text.is_some();
    let is_selectable = if container_mode {
        element.is_spec_container()
    } else {
        !element.is_spec_container() || is_atomic_text
    };

    if is_selectable {
        if let Some(rect) = get_element_bounds(ctx, element, pos) {
            if rect.contains(ctx.mouse_pos) {
                return Some(current_path.clone());
            }
        }
    }

    None
}

fn get_element_bounds(
    ctx: &RenderContext,
    element: &ElementWrapper,
    pos: egui::Pos2,
) -> Option<egui::Rect> {
    let size = match &element.data {
        Element::Graphic(g) => {
            let id = crate::assets::AssetId::new(&g.patch);
            ctx.assets.textures.get(&id).map(|t| t.size_vec2())?
        }
        Element::Number(n) | Element::Percent(n) => {
            text::measure_text_size(ctx, "000", &n.font, true)
        }
        Element::String(s) => text::measure_text_size(ctx, "Sample Text", &s.font, false),
        Element::Face(_) | Element::FaceBackground(_) => egui::vec2(24.0, 29.0),
        Element::Component(c) => text::measure_text_size(ctx, "Sample Component", &c.font, false),
        Element::Animation(a) => {
            let lump = a.frames.first().map(|f| &f.lump)?;
            let id = crate::assets::AssetId::new(lump);
            ctx.assets.textures.get(&id).map(|t| t.size_vec2())?
        }
        _ => return None,
    };

    let offset = get_alignment_anchor_offset(element.get_common().alignment, size.x, size.y);
    Some(egui::Rect::from_min_size(pos + offset, size))
}

/// Internal helper to iterate and draw child elements.
fn recurse_children(
    ctx: &RenderContext,
    children: &[ElementWrapper],
    pos: egui::Pos2,
    path: &mut Vec<usize>,
    parent_visible: bool,
) {
    for (idx, child) in children.iter().enumerate() {
        path.push(idx);
        draw_element_wrapper(ctx, child, pos, path, parent_visible);
        path.pop();
    }
}

/// Calculates the final virtual position of an element based on its local X/Y,
/// alignment, and widescreen configuration.
pub(super) fn resolve_position(
    ctx: &RenderContext,
    common: &CommonAttrs,
    parent_pos: egui::Pos2,
) -> egui::Pos2 {
    let mut pos = egui::pos2(
        parent_pos.x + common.x as f32,
        parent_pos.y + common.y as f32,
    );

    if ctx.proj.origin_x > 0.0 {
        let wl = common.alignment.contains(Alignment::WIDESCREEN_LEFT);
        let wr = common.alignment.contains(Alignment::WIDESCREEN_RIGHT);

        if wl && !wr {
            pos.x -= ctx.proj.origin_x;
        } else if wr && !wl {
            pos.x += ctx.proj.origin_x;
        }
    }

    egui::pos2(pos.x.floor(), pos.y.floor())
}

/// Returns a pixel offset vector based on the alignment flags and provided dimensions.
pub fn get_alignment_anchor_offset(align: Alignment, w: f32, h: f32) -> egui::Vec2 {
    let calc = |size: f32, max_bit: Alignment, mid_bit: Alignment| {
        if align.contains(max_bit) {
            -size
        } else if align.contains(mid_bit) {
            -(size / 2.0).floor()
        } else {
            0.0
        }
    };

    egui::vec2(
        calc(w, Alignment::RIGHT, Alignment::H_CENTER),
        calc(h, Alignment::BOTTOM, Alignment::V_CENTER),
    )
}
