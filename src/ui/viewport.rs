use crate::assets::AssetStore;
use crate::document::{determine_insertion_point, LayerAction};
use crate::model::*;
use crate::render::projection::ViewportProjection;
use crate::render::{self, RenderPass};
use crate::state::PreviewState;
use crate::ui::shared::VIEWPORT_RECT_ID;
use eframe::egui;
use std::collections::HashSet;

pub fn draw_viewport(
    ui: &mut egui::Ui,
    file: &Option<SBarDefFile>,
    assets: &AssetStore,
    preview_state: &mut PreviewState,
    selection: &HashSet<Vec<usize>>,
    current_bar_idx: usize,
) -> Vec<LayerAction> {
    let mut actions = Vec::new();

    let mut predict_rect = ui.available_rect_before_wrap();
    predict_rect.min.y += 32.0;

    let proj = ViewportProjection::new(
        predict_rect,
        preview_state.engine.widescreen_mode,
        preview_state.engine.aspect_correction,
    );

    ui.add_space(-6.0);
    ui.horizontal(|ui| {
        ui.heading(format!("Viewport ({}x Scale)", proj.final_scale_x));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(&mut preview_state.engine.widescreen_mode, "Widescreen (16:9)");
            ui.checkbox(
                &mut preview_state.engine.aspect_correction,
                "Aspect Correct (4:3)",
            );
        });
    });

    let background_rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(background_rect, 0.0, egui::Color32::from_rgb(20, 20, 20));

    ui.data_mut(|d| d.insert_temp(egui::Id::new(VIEWPORT_RECT_ID), background_rect));

    let file_ref = match file {
        Some(f) => f,
        None => {
            ui.scope_builder(egui::UiBuilder::new().max_rect(background_rect), |ui| {
                ui.centered_and_justified(|ui| ui.label("No file loaded."));
            });
            return actions;
        }
    };

    let sense = egui::Sense::click_and_drag();
    let viewport_res =
        ui.interact(background_rect, ui.make_persistent_id("viewport_interact"), sense);

    let is_dragging = viewport_res.dragged_by(egui::PointerButton::Primary);

    if is_dragging && !selection.is_empty() {
        if egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()).is_none() {
            if viewport_res.drag_started() {
                actions.push(LayerAction::UndoSnapshot);
            }

            ui.ctx().set_cursor_icon(egui::CursorIcon::None);

            let delta = ui.input(|i| i.pointer.delta());
            let accum_id = ui.make_persistent_id("move_accumulator");
            let mut accum: egui::Vec2 = ui.data(|d| d.get_temp(accum_id).unwrap_or_default());

            accum.x += delta.x / proj.final_scale_x;
            accum.y += delta.y / proj.final_scale_y;

            let move_x = accum.x.trunc() as i32;
            let move_y = accum.y.trunc() as i32;

            accum.x -= move_x as f32;
            accum.y -= move_y as f32;
            ui.data_mut(|d| d.insert_temp(accum_id, accum));

            if move_x != 0 || move_y != 0 {
                actions.push(LayerAction::TranslateSelection {
                    paths: selection.iter().cloned().collect(),
                    dx: move_x,
                    dy: move_y,
                });
            }
        }
    } else if viewport_res.drag_stopped() {
        let accum_id = ui.make_persistent_id("move_accumulator");
        ui.data_mut(|d| d.remove::<egui::Vec2>(accum_id));
    }

    ui.painter()
        .rect_filled(proj.screen_rect, 0.0, egui::Color32::BLACK);

    if let Some(tex) = assets.textures.get("_BG_MASTER") {
        let uv_rect = if preview_state.engine.widescreen_mode {
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
        } else {
            let margin = (1.0 - 0.75) / 2.0;
            egui::Rect::from_min_max(egui::pos2(margin, 0.0), egui::pos2(1.0 - margin, 1.0))
        };
        ui.painter()
            .image(tex.id(), proj.screen_rect, uv_rect, egui::Color32::WHITE);
    }

    let mut screen_ui = ui.new_child(egui::UiBuilder::new().max_rect(proj.screen_rect));
    screen_ui.set_clip_rect(proj.screen_rect.intersect(ui.clip_rect()));

    render_player_weapon(&screen_ui, preview_state, assets, &proj);

    let bar_idx = if current_bar_idx < file_ref.data.status_bars.len() {
        current_bar_idx
    } else {
        0
    };

    if let Some(bar) = file_ref.data.status_bars.get(bar_idx) {
        let root_y = if bar.fullscreen_render {
            0.0
        } else {
            200.0 - bar.height as f32
        };
        ui.ctx().request_repaint();

        let mouse_pos = ui
            .input(|i| i.pointer.latest_pos())
            .unwrap_or(egui::pos2(-1000.0, -1000.0));

        preview_state.virtual_mouse_pos = proj.to_virtual(mouse_pos);

        let ctx = render::RenderContext {
            painter: screen_ui.painter(),
            assets,
            file: file_ref,
            state: preview_state,
            time: ui.input(|i| i.time),
            fps: preview_state.display_fps,
            mouse_pos: preview_state.virtual_mouse_pos,
            selection,
            pass: RenderPass::Background,
            proj: &proj,
            is_dragging,
        };

        for (idx, child) in bar.children.iter().enumerate() {
            let mut path = vec![bar_idx, idx];
            render::draw_element_wrapper(&ctx, child, egui::pos2(proj.origin_x, root_y), &mut path);
        }

        if !selection.is_empty() && preview_state.strobe_timer > 0.0 {
            let fg_ctx = render::RenderContext {
                pass: RenderPass::Foreground,
                ..ctx
            };
            for (idx, child) in bar.children.iter().enumerate() {
                let mut path = vec![bar_idx, idx];
                render::draw_element_wrapper(
                    &fg_ctx,
                    child,
                    egui::pos2(proj.origin_x, root_y),
                    &mut path,
                );
            }
        }

        if let Some(asset_keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
            let pointer_pos = ui.input(|i| i.pointer.latest_pos());
            let is_over_viewport = pointer_pos.map_or(false, |pos| proj.screen_rect.contains(pos));

            if is_over_viewport {
                ui.ctx().set_cursor_icon(egui::CursorIcon::None);
                ui.painter().rect_stroke(
                    proj.screen_rect,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    egui::StrokeKind::Inside,
                );

                if let Some(pos) = pointer_pos {
                    let virtual_pos = proj.to_virtual(pos);
                    let final_x = (virtual_pos.x - proj.origin_x).round() as i32;
                    let final_y = (virtual_pos.y - root_y).round() as i32;

                    for (i, key) in asset_keys.iter().enumerate() {
                        let preview_el = ElementWrapper {
                            data: Element::Graphic(GraphicDef {
                                common: CommonAttrs {
                                    x: final_x + (i as i32 * 4),
                                    y: final_y + (i as i32 * 4),
                                    ..Default::default()
                                },
                                patch: key.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        };
                        render::draw_element_wrapper(
                            &ctx,
                            &preview_el,
                            egui::pos2(proj.origin_x, root_y),
                            &mut vec![],
                        );
                    }

                    if ui.input(|i| i.pointer.any_released()) {
                        let (parent_path, mut insert_idx) =
                            determine_insertion_point(selection, bar_idx);
                        for key in asset_keys.iter() {
                            let new_element = ElementWrapper {
                                data: Element::Graphic(GraphicDef {
                                    common: CommonAttrs {
                                        x: final_x,
                                        y: final_y,
                                        ..Default::default()
                                    },
                                    patch: key.clone(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            };
                            actions.push(LayerAction::Add {
                                parent_path: parent_path.clone(),
                                insert_idx,
                                element: new_element,
                            });
                            insert_idx += 1;
                        }
                        egui::DragAndDrop::clear_payload(ui.ctx());
                    }
                }
            }
        }
    }

    actions
}

fn render_player_weapon(
    ui: &egui::Ui,
    state: &PreviewState,
    assets: &AssetStore,
    proj: &ViewportProjection,
) {
    let (weapon_lump_name, constant_offset) = match state.display_weapon_slot {
        1 => {
            if state.inventory.has_chainsaw {
                (Some("SAWGC0"), 0.0)
            } else {
                (Some("PUNGA0"), 0.0)
            }
        }
        2 => (Some("PISGA0"), 0.0),
        3 => {
            if state.display_super_shotgun {
                (Some("SHT2A0"), 0.0)
            } else {
                (Some("SHTGA0"), 0.0)
            }
        }
        4 => (Some("CHGGA0"), 32.0),
        5 => (Some("MISGA0"), 32.0),
        6 => (Some("PLSGA0"), 0.0),
        7 => (Some("BFGGA0"), 32.0),
        _ => (None, 0.0),
    };

    if let Some(lump) = weapon_lump_name {
        if let Some(tex) = assets.textures.get(lump) {
            let tex_size = tex.size_vec2();
            let scaled_size =
                egui::vec2(tex_size.x * proj.final_scale_x, tex_size.y * proj.final_scale_y);
            let draw_x = proj.screen_rect.center().x - (scaled_size.x / 2.0);
            let total_offset_y = (state.weapon_offset_y + constant_offset) * proj.final_scale_y;
            let draw_y = (proj.screen_rect.max.y - scaled_size.y) + total_offset_y;

            ui.painter().image(
                tex.id(),
                egui::Rect::from_min_size(egui::pos2(draw_x, draw_y), scaled_size),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }
}