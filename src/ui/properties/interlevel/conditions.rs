use crate::models::interlevel::InterlevelCondition;
use crate::ui::context_menu::ContextMenu;
use crate::ui::shared;
use eframe::egui;

pub(super) fn condition_name(val: i32) -> &'static str {
    match val {
        0 => "None",
        1 => "Current Map > Param",
        2 => "Current Map == Param",
        3 => "Map (Param) Visited",
        4 => "Not a Secret Map",
        5 => "Any Secret Map Visited",
        6 => "Is Tally Screen",
        7 => "Is Entering Screen",
        _ => "Unknown",
    }
}

pub(super) fn is_condition_true(
    cond: &InterlevelCondition,
    state: &crate::state::PreviewState,
) -> bool {
    match cond.condition {
        0 => true,
        1 => state.viewer.ilvl_current_map > cond.param,
        2 => state.viewer.ilvl_current_map == cond.param,
        3 => {
            state.viewer.ilvl_current_map == cond.param
                || (state.viewer.ilvl_earlier_visited && state.viewer.ilvl_current_map > cond.param)
        }
        4 => !state.viewer.ilvl_is_secret_map,
        5 => state.viewer.ilvl_secret_visited,
        6 => state.viewer.ilvl_is_tally,
        7 => !state.viewer.ilvl_is_tally,
        _ => true,
    }
}

pub(super) fn draw_interlevel_conditions(
    ui: &mut egui::Ui,
    conditions: &mut Vec<InterlevelCondition>,
    assets: &crate::assets::AssetStore,
    state: &crate::state::PreviewState,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("Active Rules: {}", conditions.len())).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(2.0);
            if ui.button("Add Condition").clicked() {
                conditions.push(InterlevelCondition {
                    condition: 0,
                    param: 0,
                });
                changed = true;
            }
            if !conditions.is_empty() {
                if ui.button("Clear All").clicked() {
                    conditions.clear();
                    changed = true;
                }
            }
        });
    });

    ui.separator();
    ui.add_space(4.0);

    let mut to_remove = None;
    for (i, cond) in conditions.iter_mut().enumerate() {
        let is_true = is_condition_true(cond, state);
        let frame = shared::condition_box_frame();

        let response = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let box_size = 44.0;
                    let (rect, _) = ui
                        .allocate_exact_size(egui::vec2(box_size, box_size), egui::Sense::hover());
                    ui.painter()
                        .rect_filled(rect, 4.0, egui::Color32::from_gray(45));

                    let tex_id = crate::assets::AssetId::new("PMAPA0");
                    let tex = assets.textures.get(&tex_id);
                    crate::ui::properties::common::paint_thumb_content(ui, rect, tex, None);

                    if tex.is_none() {
                        crate::ui::properties::common::draw_type_placeholder(ui.painter(), rect);
                    }
                });

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(egui::Button::new("X").min_size(egui::vec2(18.0, 18.0)))
                                .on_hover_text("Remove Condition")
                                .clicked()
                            {
                                to_remove = Some(i);
                            }

                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let id = ui.make_persistent_id(format!("ilvl_cond_{}", i));
                                    let current_name = condition_name(cond.condition);

                                    let btn_res = shared::combobox_button(
                                        ui,
                                        current_name,
                                        ui.available_width(),
                                    );
                                    if btn_res.clicked() {
                                        ContextMenu::open(ui, id, btn_res.rect.left_bottom());
                                    }

                                    if let Some(menu) = ContextMenu::get(ui, id) {
                                        ContextMenu::show(ui, menu, btn_res.clicked(), |ui| {
                                            ui.set_min_width(160.0);
                                            for val in 0..=7 {
                                                if crate::ui::properties::common::custom_menu_item(
                                                    ui,
                                                    condition_name(val),
                                                    cond.condition == val,
                                                ) {
                                                    cond.condition = val;
                                                    changed = true;
                                                    ContextMenu::close(ui);
                                                }
                                            }
                                        });
                                    }
                                },
                            );
                        });
                    });

                    shared::draw_separator_line(ui);

                    ui.horizontal(|ui| {
                        if matches!(cond.condition, 1 | 2 | 3) {
                            ui.label("Map Param:");
                            changed |= ui.add(egui::DragValue::new(&mut cond.param)).changed();
                        } else {
                            ui.label(egui::RichText::new("(No Params)").weak().size(11.0));
                        }
                    });
                });
            });
        });

        let tint_color = if is_true {
            egui::Color32::from_rgba_unmultiplied(40, 140, 40, 30)
        } else {
            egui::Color32::from_rgba_unmultiplied(140, 40, 40, 30)
        };

        ui.painter()
            .rect_filled(response.response.rect, 4.0, tint_color);

        ui.add_space(4.0);
    }

    if let Some(idx) = to_remove {
        conditions.remove(idx);
        changed = true;
    }

    changed
}
