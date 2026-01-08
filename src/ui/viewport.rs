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

    let temp_proj = ViewportProjection::from_engine(estimate_rect, &preview_state.engine);

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
                            let old_proj = ViewportProjection::from_engine(
                                background_rect,
                                &preview_state.engine,
                            );
                            let virt_anchor = old_proj.to_virtual(mouse_pos);

                            preview_state.engine.zoom_level = new_zoom;

                            let new_proj_temp = ViewportProjection::from_engine(
                                background_rect,
                                &preview_state.engine,
                            );

                            let new_screen_pos = new_proj_temp.to_screen(virt_anchor);
                            preview_state.engine.pan_offset += mouse_pos - new_screen_pos;
                        } else {
                            preview_state.engine.zoom_level = new_zoom;
                        }
                    }
                }
            }
        });
    }

    let proj = ViewportProjection::from_engine(background_rect, &preview_state.engine);

    if is_panning {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        if viewport_res.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            preview_state.engine.pan_offset += ui.input(|i| i.pointer.delta());
        }
    }

    let bar_idx = current_bar_idx.min(file_ref.data.status_bars.len().saturating_sub(1));
    let mouse_pos = ui
        .input(|i| i.pointer.latest_pos())
        .unwrap_or(egui::pos2(-1000.0, -1000.0));
    preview_state.editor.virtual_mouse_pos = proj.to_virtual(mouse_pos);

    let selection_mode = ui.input(|i| i.modifiers.alt || i.modifiers.command || i.modifiers.ctrl);
    let container_mode = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
    let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
    let primary_down = ui.input(|i| i.pointer.primary_down());

    let bar = &file_ref.data.status_bars[bar_idx];
    let root_y = if bar.fullscreen_render {
        0.0
    } else {
        200.0 - bar.height as f32
    };

    if selection_mode && !is_panning {
        let mut hit_result = None;
        {
            let hit_ctx = render::RenderContext {
                painter: ui.painter(),
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
                is_viewport_clicked: viewport_res.contains_pointer() && primary_down,
            };

            if viewport_res.hovered() {
                for (idx, child) in bar.children.iter().enumerate().rev() {
                    let mut path = vec![bar_idx, idx];
                    if let Some(hit) = render::hit_test(
                        &hit_ctx,
                        child,
                        egui::pos2(proj.origin_x, root_y),
                        &mut path,
                        true,
                        container_mode,
                    ) {
                        hit_result = Some(hit);
                        break;
                    }
                }
            }
        }

        if preview_state.editor.hovered_path != hit_result {
            preview_state.editor.hovered_path = hit_result;
        }

        if primary_pressed && viewport_res.hovered() {
            if let Some(path) = preview_state.editor.hovered_path.clone() {
                preview_state.editor.grabbed_path = Some(path.clone());
                if ui.input(|i| i.modifiers.shift) {
                    actions.push(LayerAction::ToggleSelection(vec![path]));
                } else {
                    actions.push(LayerAction::Select(vec![path]));
                }
            }
        }
    } else {
        preview_state.editor.hovered_path = None;
    }

    if !primary_down {
        preview_state.editor.grabbed_path = None;
    }

    let mut effective_selection = selection.clone();
    if let Some(grab) = &preview_state.editor.grabbed_path {
        effective_selection.clear();
        effective_selection.insert(grab.clone());
    }

    actions.extend(controller.handle_selection_drag(
        ui,
        &proj,
        &effective_selection,
        &viewport_res,
        is_panning,
    ));

    let final_clip_rect = proj.screen_rect.intersect(background_rect);
    let mut screen_ui = ui.new_child(egui::UiBuilder::new().max_rect(background_rect));
    screen_ui.set_clip_rect(final_clip_rect);

    screen_ui
        .painter()
        .rect_filled(proj.screen_rect, 0.0, egui::Color32::BLACK);

    let bar_height = bar.height as f32;
    let is_fullscreen = bar.fullscreen_render;

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
        let flat_key = bar
            .fill_flat
            .clone()
            .unwrap_or_else(|| "GRNROCK".to_string());
        let id = crate::assets::AssetId::new(&flat_key);

        if let Some(tex) = assets.textures.get(&id) {
            let tile_size_px = 64.0 * proj.final_scale_x;
            let bar_area_rect = egui::Rect::from_min_max(
                egui::pos2(
                    proj.screen_rect.left(),
                    proj.screen_rect.bottom() - (bar_height * proj.final_scale_y),
                ),
                proj.screen_rect.max,
            );

            for y_idx in 0..((bar_area_rect.height() / tile_size_px).ceil() as i32) {
                for x_idx in 0..((bar_area_rect.width() / tile_size_px).ceil() as i32) {
                    let r = egui::Rect::from_min_size(
                        bar_area_rect.min
                            + egui::vec2(x_idx as f32 * tile_size_px, y_idx as f32 * tile_size_px),
                        egui::vec2(tile_size_px, tile_size_px),
                    )
                    .intersect(bar_area_rect);
                    screen_ui.painter().image(
                        tex.id(),
                        r,
                        egui::Rect::from_min_max(
                            egui::pos2(0.0, 0.0),
                            egui::pos2(r.width() / tile_size_px, r.height() / tile_size_px),
                        ),
                        egui::Color32::WHITE,
                    );
                }
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

    let time = ui.input(|i| i.time);
    let fps = preview_state.editor.display_fps;

    let draw_all = |painter: &egui::Painter, pass: RenderPass| {
        let render_ctx = render::RenderContext {
            painter,
            assets,
            file: file_ref,
            state: preview_state,
            time,
            fps,
            mouse_pos: preview_state.editor.virtual_mouse_pos,
            selection,
            pass,
            proj: &proj,
            is_dragging: controller.is_dragging,
            is_viewport_clicked: viewport_res.contains_pointer() && primary_down,
        };

        for (idx, child) in bar.children.iter().enumerate() {
            let mut path = vec![bar_idx, idx];
            render::draw_element_wrapper(
                &render_ctx,
                child,
                egui::pos2(proj.origin_x, root_y),
                &mut path,
                true,
            );
        }
    };

    draw_all(screen_ui.painter(), RenderPass::Background);

    if !selection.is_empty() && preview_state.editor.strobe_timer > 0.0 {
        draw_all(screen_ui.painter(), RenderPass::Foreground);
    }

    if let Some(asset_keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
        if viewport_res.contains_pointer() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                let virtual_pos = proj.to_virtual(pos);
                let final_x = (virtual_pos.x - proj.origin_x).round() as i32;
                let final_y = (virtual_pos.y - root_y).round() as i32;

                let render_ctx = render::RenderContext {
                    painter: screen_ui.painter(),
                    assets,
                    file: file_ref,
                    state: preview_state,
                    time,
                    fps,
                    mouse_pos: preview_state.editor.virtual_mouse_pos,
                    selection,
                    pass: RenderPass::Background,
                    proj: &proj,
                    is_dragging: controller.is_dragging,
                    is_viewport_clicked: viewport_res.contains_pointer() && primary_down,
                };

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
                        &render_ctx,
                        &preview_el,
                        egui::pos2(proj.origin_x, root_y),
                        &mut vec![],
                        true,
                    );
                }

                if ui.input(|i| i.pointer.any_released()) {
                    let (parent_path, mut insert_idx) =
                        determine_insertion_point(file_ref, selection, bar_idx);
                    for key in asset_keys.iter() {
                        actions.push(LayerAction::Add {
                            parent_path: parent_path.clone(),
                            insert_idx,
                            element: wrap_graphic(key, final_x, final_y),
                        });
                        insert_idx += 1;
                    }
                    egui::DragAndDrop::clear_payload(ui.ctx());
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
    let (weapon_lump_name, constant_offset) = match state.selected_weapon_slot {
        1 => (
            Some(
                if state.inventory.has_chainsaw
                    && state.engine.slot_mapping == crate::state::SlotMapping::Vanilla
                {
                    "SAWGC0"
                } else {
                    "PUNGA0"
                },
            ),
            0.0,
        ),
        2 => (Some("PISGA0"), 0.0),
        3 => (
            Some(
                if state.use_super_shotgun
                    && state.engine.slot_mapping == crate::state::SlotMapping::Vanilla
                {
                    "SHT2A0"
                } else {
                    "SHTGA0"
                },
            ),
            0.0,
        ),
        4 => (Some("CHGGA0"), 32.0),
        5 => (Some("MISGA0"), 32.0),
        6 => (Some("PLSGA0"), 0.0),
        7 => (Some("BFGGA0"), 32.0),
        8 => (Some("SAWGC0"), 0.0),
        9 => (Some("SHT2A0"), 0.0),
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
