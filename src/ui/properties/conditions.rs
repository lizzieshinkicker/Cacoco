use super::common::{self, paint_thumb_content};
use super::lookups;
use crate::assets::AssetStore;
use crate::model::{ConditionDef, ConditionType, Element, ElementWrapper, NumberType};
use eframe::egui;

pub fn draw_conditions_editor(
    ui: &mut egui::Ui,
    element: &mut ElementWrapper,
    assets: &AssetStore,
    state: &crate::state::PreviewState,
) -> bool {
    let mut changed = false;
    let is_ammo_selected = is_ammo_selected_type(element);

    let common = element.get_common_mut();
    ui.add_space(12.0);

    if is_ammo_selected {
        let has_safety = common
            .conditions
            .iter()
            .any(|c| c.condition == ConditionType::SelectedWeaponHasAmmo);
        if !has_safety {
            common.conditions.insert(
                0,
                ConditionDef {
                    condition: ConditionType::SelectedWeaponHasAmmo,
                    param: 0,
                    param2: 0,
                },
            );
            changed = true;
        }
    }

    ui.horizontal(|ui| {
        ui.heading(format!("Conditions ({})", common.conditions.len()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !common.conditions.is_empty() {
                if ui
                    .add_enabled(!is_ammo_selected, egui::Button::new("Clear"))
                    .clicked()
                {
                    common.conditions.clear();
                    changed = true;
                }
            }
            if ui.button("Add").clicked() {
                common.conditions.push(ConditionDef {
                    condition: ConditionType::WeaponOwned,
                    param: 101,
                    param2: 0,
                });
                changed = true;
            }
        });
    });
    ui.separator();

    let mut remove_idx = None;
    for (i, cond) in common.conditions.iter_mut().enumerate() {
        let id = ui.make_persistent_id(format!("cond_card_{}", i));
        ui.push_id(id, |ui| {
            let can_remove =
                !(is_ammo_selected && cond.condition == ConditionType::SelectedWeaponHasAmmo);
            changed |= draw_condition_card(ui, cond, assets, &mut remove_idx, i, state, can_remove);
        });
        ui.add_space(4.0);
    }

    if let Some(i) = remove_idx {
        common.conditions.remove(i);
        changed = true;
    }

    changed
}

fn is_ammo_selected_type(el: &ElementWrapper) -> bool {
    match &el.data {
        Element::Number(n) | Element::Percent(n) => n.type_ == NumberType::AmmoSelected,
        _ => false,
    }
}

fn draw_condition_card(
    ui: &mut egui::Ui,
    cond: &mut ConditionDef,
    assets: &AssetStore,
    remove_idx: &mut Option<usize>,
    my_idx: usize,
    state: &crate::state::PreviewState,
    can_remove: bool,
) -> bool {
    let mut changed = false;
    let is_true = crate::conditions::resolve(&[cond.clone()], state);

    let frame = egui::Frame::new()
        .inner_margin(4.0)
        .corner_radius(4.0)
        .fill(egui::Color32::from_white_alpha(5))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(15)));

    let response = frame.show(ui, |ui| {
        let master_icon_name = lookups::resolve_condition_icon(cond, state);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let box_size = 44.0;
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(box_size, box_size), egui::Sense::hover());
                ui.painter()
                    .rect_filled(rect, 4.0, egui::Color32::from_gray(45));
                let tex = master_icon_name
                    .as_ref()
                    .and_then(|n| assets.textures.get(n));
                paint_thumb_content(ui, rect, tex, None);
                if tex.is_none() {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "?",
                        egui::FontId::proportional(18.0),
                        egui::Color32::from_gray(100),
                    );
                }
            });

            ui.vertical(|ui| {
                let (g_idx, _v_idx) = lookups::find_group_for_type(cond.condition);

                ui.horizontal(|ui| {
                    let group = &lookups::GROUPS[g_idx];

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(can_remove, egui::Button::new("X").small())
                            .on_hover_text("Remove Condition")
                            .clicked()
                        {
                            *remove_idx = Some(my_idx);
                        }

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            egui::ComboBox::from_id_salt(format!("group_dd_{}_{}", g_idx, my_idx))
                                .selected_text(group.name)
                                .width(ui.available_width())
                                .height(1000.0)
                                .show_ui(ui, |ui| {
                                    ui.set_min_width(120.0);
                                    for (idx, g) in lookups::GROUPS.iter().enumerate() {
                                        if common::custom_menu_item(ui, g.name, g_idx == idx) {
                                            if idx != g_idx {
                                                let new_group = &lookups::GROUPS[idx];
                                                cond.condition = new_group.variants[0].condition;
                                                cond.param = new_group.default_param;
                                                cond.param2 = 0;
                                                changed = true;
                                            }
                                        }
                                    }
                                });
                        });
                    });
                });

                let stroke_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
                let (div_rect, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 2.0),
                    egui::Sense::hover(),
                );
                ui.painter().line_segment(
                    [
                        egui::pos2(div_rect.min.x, div_rect.center().y),
                        egui::pos2(div_rect.max.x, div_rect.center().y),
                    ],
                    egui::Stroke::new(1.0, stroke_color.gamma_multiply(0.5)),
                );

                ui.horizontal(|ui| {
                    let (g_idx, _) = lookups::find_group_for_type(cond.condition);
                    let group = &lookups::GROUPS[g_idx];
                    changed |= draw_condition_predicate(ui, group, cond, assets, my_idx);
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

    changed
}

fn draw_condition_predicate(
    ui: &mut egui::Ui,
    group: &lookups::ConditionGroup,
    cond: &mut ConditionDef,
    assets: &AssetStore,
    my_idx: usize,
) -> bool {
    let mut changed = false;
    match group.style {
        lookups::GroupStyle::Standard => {
            changed |= draw_operator_selector(ui, group, cond, assets, my_idx);
            changed |= draw_params_for_type(ui, cond, assets, my_idx);
        }
        lookups::GroupStyle::Natural => {
            changed |= draw_params_for_type(ui, cond, assets, my_idx);
            changed |= draw_operator_selector(ui, group, cond, assets, my_idx);
        }
        lookups::GroupStyle::AmmoComplex => {
            changed |= draw_operator_selector(ui, group, cond, assets, my_idx);
            changed |= ui
                .add(
                    egui::DragValue::new(&mut cond.param)
                        .speed(1)
                        .range(0..=999),
                )
                .changed();
            changed |= common::draw_lookup_param_dd(
                ui,
                &format!("param2_{}", my_idx),
                &mut cond.param2,
                lookups::AMMO_TYPES,
                assets,
            );
        }
    }
    changed
}

fn draw_operator_selector(
    ui: &mut egui::Ui,
    group: &lookups::ConditionGroup,
    cond: &mut ConditionDef,
    _assets: &AssetStore,
    my_idx: usize,
) -> bool {
    let mut changed = false;
    let current_variant = group
        .variants
        .iter()
        .find(|v| v.condition == cond.condition)
        .unwrap_or(&group.variants[0]);

    let width = match group.style {
        lookups::GroupStyle::Standard | lookups::GroupStyle::AmmoComplex => 55.0,
        lookups::GroupStyle::Natural => 50.0,
    };

    egui::ComboBox::from_id_salt(format!("op_dd_{:?}_{}", cond.condition, my_idx))
        .selected_text(current_variant.label)
        .width(width)
        .height(1000.0)
        .show_ui(ui, |ui| {
            ui.set_min_width(100.0);
            for v in group.variants {
                if common::custom_menu_item(ui, v.label, cond.condition == v.condition) {
                    cond.condition = v.condition;
                    changed = true;
                }
            }
        });
    changed
}

fn draw_params_for_type(
    ui: &mut egui::Ui,
    cond: &mut ConditionDef,
    assets: &AssetStore,
    my_idx: usize,
) -> bool {
    use crate::model::ConditionType::*;
    use lookups::*;

    let mut changed = false;
    match cond.condition {
        WeaponOwned | WeaponNotOwned | WeaponSelected | WeaponNotSelected | WeaponHasAmmo => {
            changed |= common::draw_lookup_param_dd(
                ui,
                &format!("p1_wpn_{:?}_{}", cond.condition, my_idx),
                &mut cond.param,
                WEAPONS,
                assets,
            );
        }
        ItemOwned | ItemNotOwned => {
            changed |= common::draw_lookup_param_dd(
                ui,
                &format!("p1_item_{:?}_{}", cond.condition, my_idx),
                &mut cond.param,
                ITEMS,
                assets,
            );
        }
        AmmoMatch => {
            changed |= common::draw_lookup_param_dd(
                ui,
                &format!("p1_ammo_{:?}_{}", cond.condition, my_idx),
                &mut cond.param,
                AMMO_TYPES,
                assets,
            );
        }
        SessionTypeEq | SessionTypeNeq => {
            changed |= common::draw_lookup_param_dd(
                ui,
                &format!("p1_sess_{:?}_{}", cond.condition, my_idx),
                &mut cond.param,
                SESSION_TYPES,
                assets,
            );
        }
        HudModeEq => {
            changed |= common::draw_lookup_param_dd(
                ui,
                &format!("p1_hud_{:?}_{}", cond.condition, my_idx),
                &mut cond.param,
                HUD_MODES,
                assets,
            );
        }
        WidescreenModeEq => {
            changed |= common::draw_lookup_param_dd(
                ui,
                &format!("p1_wide_{:?}_{}", cond.condition, my_idx),
                &mut cond.param,
                WIDESCREEN_MODES,
                assets,
            );
        }
        GameVersionGe | GameVersionLt => {
            changed |= common::draw_lookup_param_dd(
                ui,
                &format!("p1_ver_{:?}_{}", cond.condition, my_idx),
                &mut cond.param,
                FEATURE_LEVELS,
                assets,
            );
        }
        AutomapModeEq => {
            changed |= draw_automap_param(ui, &mut cond.param);
        }
        SlotOwned | SlotNotOwned | SlotSelected | SlotNotSelected => {
            changed |= ui
                .add(egui::DragValue::new(&mut cond.param).range(1..=9))
                .changed();
        }
        _ => {
            if matches!(
                get_param_usage(cond.condition),
                ParamUsage::Param1 | ParamUsage::Both
            ) {
                changed |= ui
                    .add(
                        egui::DragValue::new(&mut cond.param)
                            .speed(1)
                            .range(0..=999),
                    )
                    .changed();
            }
        }
    }
    changed
}

pub fn draw_automap_param(ui: &mut egui::Ui, param: &mut i32) -> bool {
    let mut changed = false;
    ui.menu_button("Flags...", |ui| {
        ui.set_min_width(120.0);
        for flag in lookups::AUTOMAP_FLAGS {
            let mut is_set = (*param & flag.id) != 0;
            if ui.checkbox(&mut is_set, flag.name).changed() {
                if is_set {
                    *param |= flag.id;
                } else {
                    *param &= !flag.id;
                }
                changed = true;
            }
        }
    });
    changed
}
