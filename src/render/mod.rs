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
pub mod palette;
pub mod patch;
pub mod projection;
pub mod text;

#[derive(Clone, Copy, PartialEq)]
pub enum RenderPass {
    Background,
    Foreground,
}

pub struct RenderContext<'a> {
    pub painter: &'a egui::Painter,
    pub assets: &'a AssetStore,
    pub file: &'a SBarDefFile,
    pub state: &'a PreviewState,
    pub time: f64,
    pub fps: f32,
    pub mouse_pos: egui::Pos2,
    pub selection: &'a HashSet<Vec<usize>>,
    pub pass: RenderPass,
    pub proj: &'a projection::ViewportProjection,
    pub is_dragging: bool,
    pub is_viewport_clicked: bool,
}

impl<'a> RenderContext<'a> {
    pub fn to_screen(&self, pos: egui::Pos2) -> egui::Pos2 {
        self.proj.to_screen(pos)
    }

    pub fn get_number_font(&self, name: &str) -> Option<&NumberFontDef> {
        self.file
            .data
            .number_fonts
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(name))
    }
}

pub fn draw_element_wrapper(
    ctx: &RenderContext,
    element: &ElementWrapper,
    parent_pos: egui::Pos2,
    current_path: &mut Vec<usize>,
) {
    let common = element.get_common();

    let is_selected_branch = ctx.selection.contains(current_path)
        || ctx.selection.iter().any(|s| current_path.starts_with(s));
    let is_strobing = ctx.state.strobe_timer > 0.0;

    match ctx.pass {
        RenderPass::Background => {
            if is_selected_branch && is_strobing {
                return;
            }
        }
        RenderPass::Foreground => {
            if !is_strobing {
                return;
            }

            if !is_selected_branch {
                let pos = resolve_position(ctx, common, parent_pos);
                recurse_children(ctx, &common.children, pos, current_path);
                return;
            }
        }
    }

    let conditions_met = conditions::resolve(&common.conditions, ctx.state);
    if !is_selected_branch && !conditions_met {
        return;
    }

    let mut alpha = if !conditions_met { 0.05 } else { 1.0 };
    if common.translucency {
        alpha *= 0.5;
    }

    if is_selected_branch && is_strobing {
        let dur = 0.5;
        let prog = (dur - ctx.state.strobe_timer) / dur;
        let wave = (prog * std::f32::consts::PI * 4.0).cos();
        alpha *= 0.70 + (wave * 0.30);
    }

    let pos = resolve_position(ctx, common, parent_pos);

    match &element.data {
        Element::Canvas(c) => canvas::draw_canvas(ctx, c, pos, alpha),
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
        Element::Component(c) => components::draw_component(ctx, c, pos, alpha),
        Element::Carousel(_) => {}
    }

    recurse_children(ctx, &common.children, pos, current_path);
}

fn recurse_children(
    ctx: &RenderContext,
    children: &[ElementWrapper],
    pos: egui::Pos2,
    path: &mut Vec<usize>,
) {
    for (idx, child) in children.iter().enumerate() {
        path.push(idx);
        draw_element_wrapper(ctx, child, pos, path);
        path.pop();
    }
}

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
        let dl = common.alignment.contains(Alignment::DYNAMIC_LEFT);
        let dr = common.alignment.contains(Alignment::DYNAMIC_RIGHT);

        if dl && !dr {
            pos.x -= ctx.proj.origin_x;
        } else if dr && !dl {
            pos.x += ctx.proj.origin_x;
        }
    }
    pos
}

pub(super) fn get_alignment_anchor_offset(align: Alignment, w: f32, h: f32) -> egui::Vec2 {
    let mut dx = 0.0;
    let mut dy = 0.0;
    if align.contains(Alignment::RIGHT) {
        dx = -w;
    } else if align.contains(Alignment::H_CENTER) {
        dx = -w / 2.0;
    }
    if align.contains(Alignment::BOTTOM) {
        dy = -h;
    } else if align.contains(Alignment::V_CENTER) {
        dy = -h / 2.0;
    }
    egui::vec2(dx, dy)
}
