use crate::assets::{AssetId, AssetStore};
use crate::document::actions::{DocumentAction, TreeAction};
use crate::document::determine_insertion_point;
use crate::models::ProjectData;
use crate::models::sbardef::*;
use crate::render::projection::ViewportProjection;
use crate::render::{self, RenderPass};
use crate::state::PreviewState;
use crate::ui::properties::editor::ViewportContext;
use crate::ui::shared::VIEWPORT_RECT_ID;
use crate::ui::viewport_controller::ViewportController;
use eframe::egui;
use std::collections::HashSet;

/// Draws the main viewport, handling background rendering and interaction logic.
pub fn draw_viewport(
    ui: &mut egui::Ui,
    project: &Option<ProjectData>,
    assets: &AssetStore,
    preview_state: &mut PreviewState,
    controller: &mut ViewportController,
    selection: &HashSet<Vec<usize>>,
    current_bar_idx: usize,
    active_mode: &mut crate::app::ProjectMode,
) -> Vec<DocumentAction> {
    let mut actions = Vec::new();
    let mut estimate_rect = ui.available_rect_before_wrap();
    estimate_rect.min.y += 32.0;
    let temp_proj = ViewportProjection::from_engine(estimate_rect, &preview_state.sim.engine);

    actions.extend(draw_viewport_header(
        ui,
        active_mode,
        preview_state,
        &temp_proj,
    ));

    let is_umap = *active_mode == crate::app::ProjectMode::UmapInfo;

    let background_rect = ui.available_rect_before_wrap();
    ui.data_mut(|d| {
        d.insert_temp(egui::Id::new(VIEWPORT_RECT_ID), background_rect);
    });

    ui.painter()
        .rect_filled(background_rect, 0.0, egui::Color32::from_rgb(15, 15, 15));

    if project.is_none() {
        ui.scope_builder(egui::UiBuilder::new().max_rect(background_rect), |ui| {
            ui.centered_and_justified(|ui| {
                ui.label("Lump not present in project.");
            });
        });
        return actions;
    }

    let viewport_res = ui.interact(
        background_rect,
        egui::Id::new("cacoco_viewport_interact_global"),
        egui::Sense::click_and_drag(),
    );

    let is_panning = ui.input(|i| i.key_down(egui::Key::Space));

    handle_viewport_navigation(ui, &viewport_res, preview_state, is_umap, background_rect);

    if !is_umap && preview_state.sim.engine.auto_zoom {
        preview_state.sim.engine.pan_offset = egui::Vec2::ZERO;
    }

    let proj = if is_umap {
        let umap_zoom: f32 = ui
            .ctx()
            .data(|d| d.get_temp(egui::Id::new("umap_zoom")).unwrap_or(1.0));
        ViewportProjection {
            screen_rect: egui::Rect::from_min_size(
                background_rect.min + preview_state.sim.engine.pan_offset,
                background_rect.size(),
            ),
            final_scale_x: umap_zoom,
            final_scale_y: umap_zoom,
            origin_x: 0.0,
        }
    } else {
        ViewportProjection::from_engine(background_rect, &preview_state.sim.engine)
    };

    if is_panning {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        if viewport_res.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            preview_state.sim.engine.pan_offset += ui.input(|i| i.pointer.delta());
        }
    }

    let mouse_pos = ui
        .input(|i| i.pointer.latest_pos())
        .unwrap_or(egui::pos2(-1000.0, -1000.0));
    preview_state.interaction.virtual_mouse_pos = proj.to_virtual(mouse_pos);

    let selection_mode = ui.input(|i| i.modifiers.alt || i.modifiers.command || i.modifiers.ctrl);
    let container_mode = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
    let primary_pressed = ui.input(|i| i.pointer.primary_pressed());
    let primary_down = ui.input(|i| i.pointer.primary_down());

    let final_clip_rect = if is_umap {
        background_rect
    } else {
        proj.screen_rect.intersect(background_rect)
    };

    let mut viewport_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("cacoco_viewport_child_ui_stable")
            .max_rect(background_rect),
    );
    viewport_ui.set_clip_rect(final_clip_rect);

    preview_state.interaction.hovered_path = None;

    if let Some(lump) = project {
        let mut vp_ctx = ViewportContext {
            assets,
            state: preview_state,
            proj: &proj,
            selection,
            current_item_idx: current_bar_idx,
            is_panning,
            container_mode,
            selection_mode,
            primary_pressed,
            primary_down,
            viewport_res: &viewport_res,
        };
        actions.extend(lump.render_viewport(&mut viewport_ui, &mut vp_ctx));
    } else {
        viewport_ui
            .painter()
            .rect_filled(proj.screen_rect, 0.0, egui::Color32::BLACK);
    }

    if !primary_down {
        preview_state.interaction.grabbed_path = None;
    }

    let mut effective_selection = selection.clone();
    if let Some(grab) = &preview_state.interaction.grabbed_path {
        effective_selection.clear();
        effective_selection.insert(grab.clone());
    }

    actions.extend(controller.handle_selection_drag(
        ui,
        &proj,
        &effective_selection,
        &viewport_res,
        is_panning,
        *active_mode,
        preview_state,
    ));

    actions.extend(handle_asset_drop(
        ui,
        &viewport_res,
        project,
        assets,
        preview_state,
        &proj,
        selection,
        current_bar_idx,
        controller,
    ));

    actions
}

fn draw_viewport_header(
    ui: &mut egui::Ui,
    active_mode: &mut crate::app::ProjectMode,
    preview_state: &mut PreviewState,
    temp_proj: &ViewportProjection,
) -> Vec<DocumentAction> {
    let mut actions = Vec::new();
    let is_umap = *active_mode == crate::app::ProjectMode::UmapInfo;

    ui.vertical(|ui| {
        ui.add_space(-1.0);
        ui.horizontal(|ui| {
            let mode_id = ui.make_persistent_id("viewport_mode_dropdown");
            let mode_label = match active_mode {
                crate::app::ProjectMode::SBarDef => "SBARDEF",
                crate::app::ProjectMode::SkyDefs => "SKYDEFS",
                crate::app::ProjectMode::Interlevel => "INTERLEVEL",
                crate::app::ProjectMode::Finale => "FINALE",
                crate::app::ProjectMode::UmapInfo => "UMAPINFO",
            };

            let mode_res = ui.add_sized([110.0, 28.0], |ui: &mut egui::Ui| {
                crate::ui::shared::section_header_button(
                    ui,
                    mode_label,
                    None,
                    crate::ui::context_menu::ContextMenu::get(ui, mode_id).is_some(),
                )
            });

            if mode_res.clicked() {
                crate::ui::context_menu::ContextMenu::open(
                    ui,
                    mode_id,
                    mode_res.rect.left_bottom(),
                );
            }

            if let Some(menu) = crate::ui::context_menu::ContextMenu::get(ui, mode_id) {
                crate::ui::context_menu::ContextMenu::show(ui, menu, mode_res.clicked(), |ui| {
                    use crate::app::{CreationModal, ProjectMode};

                    let modes_in_project: HashSet<ProjectMode> = ui.ctx().data(|d| {
                        d.get_temp(egui::Id::new("modes_in_project"))
                            .unwrap_or_default()
                    });

                    ui.label(egui::RichText::new("Lumps in Project").weak().size(10.0));

                    let all_modes = [
                        (ProjectMode::SBarDef, "SBARDEF"),
                        (ProjectMode::SkyDefs, "SKYDEFS"),
                        (ProjectMode::UmapInfo, "UMAPINFO"),
                    ];

                    for (m, lbl) in all_modes {
                        if modes_in_project.contains(&m) {
                            let is_active = *active_mode == m;
                            let prefix = if is_active { "✔ " } else { "  " };

                            if crate::ui::context_menu::ContextMenu::button(
                                ui,
                                &format!("{}{}", prefix, lbl),
                                true,
                            ) {
                                if !is_active {
                                    ui.ctx().data_mut(|d| {
                                        d.remove::<HashSet<Vec<usize>>>(egui::Id::new("selection"));
                                        d.insert_temp(egui::Id::new("umap_zoom"), 1.0f32);
                                    });

                                    preview_state.interaction.grabbed_path = None;
                                    preview_state.interaction.hovered_path = None;

                                    preview_state.sim.engine.pan_offset = egui::Vec2::ZERO;
                                    preview_state.sim.engine.auto_zoom = true;
                                    preview_state.sim.engine.zoom_level = 1;

                                    *active_mode = m;
                                }
                                crate::ui::context_menu::ContextMenu::close(ui);
                            }
                        }
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("Add to Project").weak().size(10.0));

                    let mut add_lump_row = |lbl: &str, m: ProjectMode, target: CreationModal| {
                        let already_exists = modes_in_project.contains(&m);
                        if !already_exists {
                            if crate::ui::context_menu::ContextMenu::button(ui, lbl, true) {
                                ui.data_mut(|d| {
                                    d.insert_temp(egui::Id::new("creation_modal_type"), target)
                                });
                                crate::ui::context_menu::ContextMenu::close(ui);
                            }
                        }
                    };

                    add_lump_row("+ SBARDEF", ProjectMode::SBarDef, CreationModal::SBarDef);
                    add_lump_row("+ SKYDEFS", ProjectMode::SkyDefs, CreationModal::SkyDefs);
                    add_lump_row("+ UMAPINFO", ProjectMode::UmapInfo, CreationModal::UmapInfo);
                });
            }

            ui.add_space(8.0);

            if is_umap {
                ui.heading("Flowchart");
            } else {
                let viewport_label = if preview_state.sim.engine.auto_zoom {
                    format!("Viewport ({}x Scale)", temp_proj.final_scale_x)
                } else {
                    "Viewport".to_string()
                };
                ui.heading(viewport_label);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if is_umap {
                    if ui
                        .button("Reset Layout")
                        .on_hover_text("Automatically arrange nodes to their default state.")
                        .clicked()
                    {
                        actions.push(DocumentAction::UndoSnapshot);
                        actions.push(DocumentAction::Umap(
                            crate::document::actions::UmapAction::ResetLayout,
                        ));
                    }
                } else {
                    ui.checkbox(
                        &mut preview_state.sim.engine.widescreen_mode,
                        "Widescreen (16:9)",
                    );
                    ui.checkbox(
                        &mut preview_state.sim.engine.aspect_correction,
                        "Aspect Correct",
                    );
                    ui.separator();

                    if ui
                        .checkbox(&mut preview_state.sim.engine.auto_zoom, "Auto Fit")
                        .changed()
                    {
                        if preview_state.sim.engine.auto_zoom {
                            preview_state.sim.engine.pan_offset = egui::Vec2::ZERO;
                        }
                    }

                    if !preview_state.sim.engine.auto_zoom {
                        ui.label(
                            egui::RichText::new(format!(
                                "{}x",
                                preview_state.sim.engine.zoom_level
                            ))
                            .strong(),
                        );
                        let btn_size = egui::vec2(20.0, 20.0);
                        if ui.add_sized(btn_size, egui::Button::new("-")).clicked() {
                            preview_state.sim.engine.zoom_level =
                                (preview_state.sim.engine.zoom_level - 1).max(1);
                        }
                        if ui.add_sized(btn_size, egui::Button::new("+")).clicked() {
                            preview_state.sim.engine.zoom_level =
                                (preview_state.sim.engine.zoom_level + 1).min(8);
                        }
                    }
                }
            });
        });
        ui.add_space(4.0);
    });

    actions
}

fn handle_viewport_navigation(
    ui: &mut egui::Ui,
    viewport_res: &egui::Response,
    preview_state: &mut PreviewState,
    is_umap: bool,
    background_rect: egui::Rect,
) {
    if viewport_res.hovered() {
        let mut scroll_delta = egui::Vec2::ZERO;
        let mut pointer_pos = None;

        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::MouseWheel { delta, .. } = event {
                    scroll_delta = *delta;
                }
            }
            pointer_pos = i.pointer.latest_pos();
        });

        if scroll_delta.y != 0.0 {
            if is_umap {
                let umap_zoom: f32 = ui
                    .ctx()
                    .data(|d| d.get_temp(egui::Id::new("umap_zoom")).unwrap_or(1.0));
                let zoom_delta = (scroll_delta.y * 0.1).exp();
                let new_zoom = (umap_zoom * zoom_delta).clamp(0.05, 10.0);

                if let Some(pos) = pointer_pos {
                    let mouse_rel = pos - background_rect.min - preview_state.sim.engine.pan_offset;
                    preview_state.sim.engine.pan_offset -= mouse_rel * (new_zoom / umap_zoom - 1.0);
                }

                ui.ctx()
                    .data_mut(|d| d.insert_temp(egui::Id::new("umap_zoom"), new_zoom));
            } else {
                let current_visible_zoom =
                    ViewportProjection::from_engine(background_rect, &preview_state.sim.engine)
                        .final_scale_x
                        .floor() as i32;

                let old_zoom = if preview_state.sim.engine.auto_zoom {
                    current_visible_zoom.max(1)
                } else {
                    preview_state.sim.engine.zoom_level
                };

                let new_zoom = if scroll_delta.y > 0.0 {
                    (old_zoom + 1).min(8)
                } else {
                    (old_zoom - 1).max(1)
                };

                if new_zoom != old_zoom {
                    if let Some(pos) = pointer_pos {
                        let old_proj = ViewportProjection::from_engine(
                            background_rect,
                            &preview_state.sim.engine,
                        );
                        let virt_anchor = old_proj.to_virtual(pos);

                        preview_state.sim.engine.auto_zoom = false;
                        preview_state.sim.engine.zoom_level = new_zoom;

                        let new_proj_temp = ViewportProjection::from_engine(
                            background_rect,
                            &preview_state.sim.engine,
                        );
                        let new_screen_pos = new_proj_temp.to_screen(virt_anchor);
                        preview_state.sim.engine.pan_offset += pos - new_screen_pos;
                    } else {
                        preview_state.sim.engine.auto_zoom = false;
                        preview_state.sim.engine.zoom_level = new_zoom;
                    }
                }
            }
        }
    }

    if is_umap {
        let middle_anchor_id = egui::Id::new("umap_middle_pan_anchor");
        if viewport_res.drag_started_by(egui::PointerButton::Middle) {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                ui.ctx().data_mut(|d| d.insert_temp(middle_anchor_id, pos));
            }
        }

        if ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle)) {
            if let Some(anchor) = ui
                .ctx()
                .data(|d| d.get_temp::<egui::Pos2>(middle_anchor_id))
            {
                if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                    let delta = pos - anchor;
                    let dt = ui.input(|i| i.stable_dt);

                    preview_state.sim.engine.pan_offset -= delta * dt * 10.0;
                    ui.ctx().request_repaint();

                    ui.painter().line_segment(
                        [anchor, pos],
                        egui::Stroke::new(2.0, egui::Color32::from_white_alpha(50)),
                    );
                    ui.painter()
                        .circle_filled(anchor, 4.0, egui::Color32::from_white_alpha(100));
                    ui.ctx().set_cursor_icon(egui::CursorIcon::AllScroll);
                }
            }
        } else {
            ui.ctx()
                .data_mut(|d| d.remove::<egui::Pos2>(middle_anchor_id));
        }
    }
}

fn handle_asset_drop(
    ui: &mut egui::Ui,
    viewport_res: &egui::Response,
    project: &Option<ProjectData>,
    assets: &AssetStore,
    preview_state: &mut PreviewState,
    proj: &ViewportProjection,
    selection: &HashSet<Vec<usize>>,
    current_bar_idx: usize,
    controller: &ViewportController,
) -> Vec<DocumentAction> {
    let mut actions = Vec::new();

    if let Some(asset_keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
        if viewport_res.contains_pointer() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::None);
            if let Some(ProjectData::StatusBar(sbar)) = project {
                if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                    let bar_idx =
                        current_bar_idx.min(sbar.data.status_bars.len().saturating_sub(1));
                    let bar = &sbar.data.status_bars[bar_idx];
                    let root_y = if bar.fullscreen_render {
                        0.0
                    } else {
                        200.0 - bar.height as f32
                    };
                    let virtual_pos = proj.to_virtual(pos);
                    let final_x = (virtual_pos.x - proj.origin_x).round() as i32;
                    let final_y = (virtual_pos.y - root_y).round() as i32;

                    let render_ctx = render::RenderContext {
                        painter: ui.painter(),
                        assets,
                        file: sbar,
                        state: preview_state,
                        time: ui.input(|i| i.time),
                        fps: preview_state.viewer.display_fps,
                        mouse_pos: preview_state.interaction.virtual_mouse_pos,
                        selection,
                        pass: RenderPass::Background,
                        proj,
                        is_dragging: controller.is_dragging,
                        is_viewport_clicked: true,
                        is_native: false,
                    };

                    for (i, key) in asset_keys.iter().enumerate() {
                        let preview_el =
                            wrap_graphic(key, final_x + (i as i32 * 4), final_y + (i as i32 * 4));
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
                            determine_insertion_point(sbar, selection, bar_idx);
                        for key in asset_keys.iter() {
                            actions.push(DocumentAction::Tree(TreeAction::Add {
                                parent_path: parent_path.clone(),
                                insert_idx,
                                element: wrap_graphic(key, final_x, final_y),
                            }));
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

/// Renders a generic ID24 background centered in the virtual 320x200 space.
pub(crate) fn render_id24_background(
    ui: &mut egui::Ui,
    lump: &str,
    assets: &AssetStore,
    proj: &ViewportProjection,
) {
    let id = AssetId::new(lump);
    if let Some(tex) = assets.textures.get(&id) {
        let uv_rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter()
            .image(tex.id(), proj.screen_rect, uv_rect, egui::Color32::WHITE);
    }
}

/// Renders the complex workspace for SBARDEF, including world view and status bar flats.
pub(crate) fn render_statusbar_workspace(
    ui: &mut egui::Ui,
    bar: &StatusBarLayout,
    assets: &AssetStore,
    state: &PreviewState,
    proj: &ViewportProjection,
) {
    let bar_height = bar.height as f32;
    let h_view = if bar.fullscreen_render {
        200.0
    } else {
        200.0 - bar_height
    };
    let y_center = h_view / 2.0;
    let y_offset_from_top = y_center - 100.0;

    if let Some(tex) = assets.textures.get(&AssetId::new("_BG_MASTER")) {
        let mut uv_rect = if state.sim.engine.widescreen_mode {
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0))
        } else {
            let margin = (1.0 - 0.75) / 2.0;
            egui::Rect::from_min_max(egui::pos2(margin, 0.0), egui::pos2(1.0 - margin, 1.0))
        };

        uv_rect.min.y += (-y_offset_from_top) / 200.0;
        uv_rect.max.y -= (y_center + 100.0 - h_view) / 200.0;

        let mut draw_rect = proj.screen_rect;
        draw_rect.max.y -= (200.0 - h_view) * proj.final_scale_y;

        ui.painter()
            .image(tex.id(), draw_rect, uv_rect, egui::Color32::WHITE);
    }

    if !bar.fullscreen_render && bar_height > 0.0 {
        let flat_key = bar
            .fill_flat
            .clone()
            .unwrap_or_else(|| "GRNROCK".to_string());
        if let Some(tex) = assets.textures.get(&AssetId::new(&flat_key)) {
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

                    ui.painter().image(
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

    let mut world_clip_rect = proj.screen_rect;
    world_clip_rect.max.y -= (200.0 - h_view) * proj.final_scale_y;

    ui.scope_builder(egui::UiBuilder::new().max_rect(world_clip_rect), |ui| {
        render_player_weapon(ui, state, assets, proj, y_offset_from_top);
    });
}

fn render_player_weapon(
    ui: &egui::Ui,
    state: &PreviewState,
    assets: &AssetStore,
    proj: &ViewportProjection,
    v_shift: f32,
) {
    let (weapon_lump_name, constant_offset) = match state.viewer.display_weapon_slot {
        1 => (
            Some(
                if state.sim.inventory.has_chainsaw
                    && state.sim.engine.slot_mapping == crate::state::SlotMapping::Vanilla
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
                if state.viewer.display_super_shotgun
                    && state.sim.engine.slot_mapping == crate::state::SlotMapping::Vanilla
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
        let id = AssetId::new(lump);
        if let Some(tex) = assets.textures.get(&id) {
            let tex_size = tex.size_vec2();
            let scaled_size = egui::vec2(
                tex_size.x * proj.final_scale_x,
                tex_size.y * proj.final_scale_y,
            );
            let draw_x = proj.screen_rect.center().x - (scaled_size.x / 2.0);
            let total_offset_y =
                (state.viewer.weapon_offset_y + constant_offset + v_shift) * proj.final_scale_y;
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
