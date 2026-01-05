use crate::assets::AssetStore;
use crate::document::{LayerAction, determine_insertion_point};
use crate::model::*;
use crate::render::projection::ViewportProjection;
use crate::render::{self, RenderPass};
use crate::state::PreviewState;
use crate::ui::shared::VIEWPORT_RECT_ID;
use crate::ui::viewport_controller::ViewportController;
use eframe::egui;
use std::collections::HashSet;

/// Draws the main HUD viewport, handling rendering and delegating interaction logic.
pub fn draw_viewport(
    ui: &mut egui::Ui,
    file: &Option<SBarDefFile>,
    assets: &AssetStore,
    preview_state: &mut PreviewState,
    controller: &mut ViewportController,
    selection: &HashSet<Vec<usize>>,
    current_bar_idx: usize,
) -> Vec<LayerAction> {
    let mut actions = Vec::new();

    let mut estimate_rect = ui.available_rect_before_wrap();
    estimate_rect.min.y += 32.0;

    let temp_proj = ViewportProjection::new(
        estimate_rect,
        preview_state.engine.widescreen_mode,
        preview_state.engine.aspect_correction,
        if preview_state.engine.auto_zoom {
            None
        } else {
            Some(preview_state.engine.zoom_level)
        },
        preview_state.engine.pan_offset,
    );

    ui.vertical(|ui| {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.heading(format!("Viewport ({}x Scale)", temp_proj.final_scale_x));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(
                    &mut preview_state.engine.widescreen_mode,
                    "Widescreen (16:9)",
                );
                ui.checkbox(
                    &mut preview_state.engine.aspect_correction,
                    "Aspect Correct (4:3)",
                );

                ui.separator();

                if ui
                    .checkbox(&mut preview_state.engine.auto_zoom, "Auto Fit")
                    .changed()
                {
                    if preview_state.engine.auto_zoom {
                        preview_state.engine.pan_offset = egui::Vec2::ZERO;
                    }
                }

                if !preview_state.engine.auto_zoom {
                    ui.label(
                        egui::RichText::new(format!("{}x", preview_state.engine.zoom_level))
                            .strong(),
                    );

                    let btn_size = egui::vec2(20.0, 20.0);
                    if ui.add_sized(btn_size, egui::Button::new("-")).clicked() {
                        preview_state.engine.zoom_level =
                            (preview_state.engine.zoom_level - 1).max(1);
                    }
                    if ui.add_sized(btn_size, egui::Button::new("+")).clicked() {
                        preview_state.engine.zoom_level =
                            (preview_state.engine.zoom_level + 1).min(8);
                    }
                }
            });
        });
        ui.add_space(4.0);
    });

    let background_rect = ui.available_rect_before_wrap();

    ui.data_mut(|d| {
        d.insert_temp(egui::Id::new(VIEWPORT_RECT_ID), background_rect);
    });

    ui.painter()
        .rect_filled(background_rect, 0.0, egui::Color32::from_rgb(15, 15, 15));

    let file_ref = match file {
        Some(f) => f,
        None => {
            ui.scope_builder(egui::UiBuilder::new().max_rect(background_rect), |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label("No file loaded.");
                });
            });
            return actions;
        }
    };

    let viewport_res = ui.interact(
        background_rect,
        ui.make_persistent_id("viewport_interact"),
        egui::Sense::click_and_drag(),
    );

    let is_panning = ui.input(|i| i.key_down(egui::Key::Space));

    if !preview_state.engine.auto_zoom && viewport_res.hovered() {
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::MouseWheel { delta, .. } = event {
                    let old_zoom = preview_state.engine.zoom_level;
                    let new_zoom = if delta.y > 0.0 {
                        (old_zoom + 1).min(8)
                    } else if delta.y < 0.0 {
                        (old_zoom - 1).max(1)
                    } else {
                        old_zoom
                    };

                    if new_zoom != old_zoom {
                        if let Some(mouse_pos) = i.pointer.latest_pos() {
                            let calc_visual_pan = |z: i32, raw_pan: egui::Vec2| {
                                let correction = if preview_state.engine.aspect_correction {
                                    1.2
                                } else {
                                    1.0
                                };
                                let base_w = if preview_state.engine.widescreen_mode {
                                    428.0
                                } else {
                                    320.0
                                };
                                let scaled_w = base_w * (z as f32);
                                let scaled_h = 200.0 * (z as f32 * correction);

                                let mut visual = raw_pan;
                                if scaled_w <= background_rect.width() {
                                    visual.x = 0.0;
                                }
                                if scaled_h <= background_rect.height() {
                                    visual.y = 0.0;
                                }
                                visual
                            };

                            let current_visual_pan =
                                calc_visual_pan(old_zoom, preview_state.engine.pan_offset);

                            let old_proj = ViewportProjection::new(
                                background_rect,
                                preview_state.engine.widescreen_mode,
                                preview_state.engine.aspect_correction,
                                Some(old_zoom),
                                current_visual_pan,
                            );
                            let virt_anchor = old_proj.to_virtual(mouse_pos);

                            preview_state.engine.zoom_level = new_zoom;

                            let new_proj_temp = ViewportProjection::new(
                                background_rect,
                                preview_state.engine.widescreen_mode,
                                preview_state.engine.aspect_correction,
                                Some(new_zoom),
                                current_visual_pan,
                            );

                            let new_screen_pos = new_proj_temp.to_screen(virt_anchor);
                            let pan_adjustment = mouse_pos - new_screen_pos;

                            preview_state.engine.pan_offset = current_visual_pan + pan_adjustment;
                        } else {
                            preview_state.engine.zoom_level = new_zoom;
                        }
                    }
                }
            }
        });
    }

    let mut effective_pan = preview_state.engine.pan_offset;

    if !preview_state.engine.auto_zoom {
        let correction = if preview_state.engine.aspect_correction {
            1.2
        } else {
            1.0
        };
        let base_w = if preview_state.engine.widescreen_mode {
            crate::constants::DOOM_W_WIDE
        } else {
            crate::constants::DOOM_W
        };
        let base_h = crate::constants::DOOM_H;

        let scaled_w = base_w * (preview_state.engine.zoom_level as f32);
        let scaled_h = base_h * (preview_state.engine.zoom_level as f32 * correction);

        if scaled_w <= background_rect.width() {
            effective_pan.x = 0.0;
        }
        if scaled_h <= background_rect.height() {
            effective_pan.y = 0.0;
        }
    }

    let zoom_override = if preview_state.engine.auto_zoom {
        None
    } else {
        Some(preview_state.engine.zoom_level)
    };

    let proj = ViewportProjection::new(
        background_rect,
        preview_state.engine.widescreen_mode,
        preview_state.engine.aspect_correction,
        zoom_override,
        effective_pan,
    );

    if is_panning {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        if viewport_res.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            preview_state.engine.pan_offset += ui.input(|i| i.pointer.delta());
        }
    }

    actions.extend(controller.handle_selection_drag(
        ui,
        &proj,
        selection,
        &viewport_res,
        is_panning,
    ));

    let final_clip_rect = proj.screen_rect.intersect(background_rect);

    let mut screen_ui = ui.new_child(egui::UiBuilder::new().max_rect(background_rect));
    screen_ui.set_clip_rect(final_clip_rect);

    screen_ui
        .painter()
        .rect_filled(proj.screen_rect, 0.0, egui::Color32::BLACK);

    let bar_idx = if current_bar_idx < file_ref.data.status_bars.len() {
        current_bar_idx
    } else {
        0
    };

    let mut bar_height = 0.0;
    let mut is_fullscreen = true;
    let mut fill_flat_name = None;

    if let Some(bar) = file_ref.data.status_bars.get(bar_idx) {
        bar_height = bar.height as f32;
        is_fullscreen = bar.fullscreen_render;
        fill_flat_name = bar.fill_flat.clone();
    }

    let h_view = if is_fullscreen {
        200.0
    } else {
        200.0 - bar_height
    };
    let y_center = h_view / 2.0;
    let y_offset_from_top = y_center - 100.0;

    if let Some(tex) = assets
        .textures
        .get(&crate::assets::AssetId::new("_BG_MASTER"))
    {
        let mut uv_rect = if preview_state.engine.widescreen_mode {
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
        } else {
            let margin = (1.0 - 0.75) / 2.0;
            egui::Rect::from_min_max(egui::pos2(margin, 0.0), egui::pos2(1.0 - margin, 1.0))
        };

        uv_rect.min.y += (-y_offset_from_top) / 200.0;
        uv_rect.max.y -= (y_center + 100.0 - h_view) / 200.0;

        let mut draw_rect = proj.screen_rect;
        draw_rect.max.y -= (200.0 - h_view) * proj.final_scale_y;

        screen_ui
            .painter()
            .image(tex.id(), draw_rect, uv_rect, egui::Color32::WHITE);
    }

    if !is_fullscreen && bar_height > 0.0 {
        let flat_key = fill_flat_name.unwrap_or_else(|| "GRNROCK".to_string());
        let id = crate::assets::AssetId::new(&flat_key);

        if let Some(tex) = assets.textures.get(&id) {
            let tile_size_px = 64.0 * proj.final_scale_x;
            let bar_area_rect = egui::Rect::from_min_max(
                egui::pos2(
                    proj.screen_rect.left(),
                    proj.screen_rect.bottom() - (bar_height * proj.final_scale_y),
                ),
                egui::pos2(proj.screen_rect.right(), proj.screen_rect.bottom()),
            );

            let mut y = bar_area_rect.min.y;
            while y < bar_area_rect.max.y {
                let mut x = bar_area_rect.min.x;
                while x < bar_area_rect.max.x {
                    let draw_w = (bar_area_rect.max.x - x).min(tile_size_px);
                    let draw_h = (bar_area_rect.max.y - y).min(tile_size_px);

                    screen_ui.painter().image(
                        tex.id(),
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(draw_w, draw_h)),
                        egui::Rect::from_min_max(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(draw_w / tile_size_px, draw_h / tile_size_px),
                        ),
                        egui::Color32::WHITE,
                    );
                    x += tile_size_px;
                }
                y += tile_size_px;
            }
        }
    }

    {
        let mut world_clip_rect = proj.screen_rect;
        world_clip_rect.max.y -= (200.0 - h_view) * proj.final_scale_y;

        let mut weapon_ui = screen_ui.new_child(egui::UiBuilder::new());
        weapon_ui.set_clip_rect(world_clip_rect.intersect(final_clip_rect));
        render_player_weapon(&weapon_ui, preview_state, assets, &proj, y_offset_from_top);
    }

    if let Some(bar) = file_ref.data.status_bars.get(bar_idx) {
        let root_y = if is_fullscreen {
            0.0
        } else {
            200.0 - bar_height
        };
        ui.ctx().request_repaint();

        let mouse_pos = ui
            .input(|i| i.pointer.latest_pos())
            .unwrap_or(egui::pos2(-1000.0, -1000.0));
        preview_state.editor.virtual_mouse_pos = proj.to_virtual(mouse_pos);

        let ctx = render::RenderContext {
            painter: screen_ui.painter(),
            assets,
            file: file_ref,
            state: preview_state,
            time: ui.input(|i| i.time),
            fps: preview_state.editor.display_fps,
            mouse_pos: preview_state.editor.virtual_mouse_pos,
            selection,
            pass: RenderPass::Background,
            proj: &proj,
            is_dragging: controller.is_dragging,
            is_viewport_clicked: viewport_res.contains_pointer()
                && ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary)),
        };

        let render_status_bar = |render_ctx: &render::RenderContext| {
            for (idx, child) in bar.children.iter().enumerate() {
                let mut path = vec![bar_idx, idx];
                render::draw_element_wrapper(
                    render_ctx,
                    child,
                    egui::pos2(proj.origin_x, root_y),
                    &mut path,
                );
            }
        };

        render_status_bar(&ctx);

        if !selection.is_empty() && preview_state.editor.strobe_timer > 0.0 {
            let fg_ctx = render::RenderContext {
                pass: RenderPass::Foreground,
                ..ctx
            };
            render_status_bar(&fg_ctx);
        }

        if let Some(asset_keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
            if viewport_res.contains_pointer() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::None);
                screen_ui.painter().rect_stroke(
                    proj.screen_rect,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::YELLOW),
                    egui::StrokeKind::Inside,
                );

                if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
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
                                patch: AssetStore::stem(key),
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
                            determine_insertion_point(file_ref, selection, bar_idx);
                        for key in asset_keys.iter() {
                            let new_element = ElementWrapper {
                                data: Element::Graphic(GraphicDef {
                                    common: CommonAttrs {
                                        x: final_x,
                                        y: final_y,
                                        ..Default::default()
                                    },
                                    patch: AssetStore::stem(key),
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
    v_shift: f32,
) {
    let (weapon_lump_name, constant_offset) = match state.editor.display_weapon_slot {
        1 => {
            if state.inventory.has_chainsaw {
                (Some("SAWGC0"), 0.0)
            } else {
                (Some("PUNGA0"), 0.0)
            }
        }
        2 => (Some("PISGA0"), 0.0),
        3 => {
            if state.editor.display_super_shotgun {
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
        let id = crate::assets::AssetId::new(lump);
        if let Some(tex) = assets.textures.get(&id) {
            let tex_size = tex.size_vec2();
            let scaled_size = egui::vec2(
                tex_size.x * proj.final_scale_x,
                tex_size.y * proj.final_scale_y,
            );
            let draw_x = proj.screen_rect.center().x - (scaled_size.x / 2.0);
            let total_offset_y =
                (state.editor.weapon_offset_y + constant_offset + v_shift) * proj.final_scale_y;
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
