use crate::assets::{AssetId, AssetStore};
use crate::model::ConditionDef;
use crate::state::PreviewState;

/// Resolves a set of SBARDEF conditions against the current simulated game state.
///
/// Returns `true` if all conditions are met (logical AND), or if the list is empty.
pub fn resolve(conditions: &[ConditionDef], state: &PreviewState, assets: &AssetStore) -> bool {
    if conditions.is_empty() {
        return true;
    }
    for condition in conditions {
        if !check_single(condition, state, assets) {
            return false;
        }
    }
    true
}

/// Evaluates a single SBARDEF condition logic block.
fn check_single(condition: &ConditionDef, state: &PreviewState, assets: &AssetStore) -> bool {
    use crate::model::ConditionType::*;
    match condition.condition {
        WeaponOwned
        | WeaponNotOwned
        | WeaponSelected
        | WeaponNotSelected
        | WeaponHasAmmo
        | SelectedWeaponHasAmmo
        | AmmoMatch
        | SlotOwned
        | SlotNotOwned
        | SlotSelected
        | SlotNotSelected => check_weapon_condition(condition, state),

        ItemOwned | ItemNotOwned => check_item_condition(condition, state),

        HealthGe
        | HealthLt
        | HealthPercentGe
        | HealthPercentLt
        | ArmorGe
        | ArmorLt
        | ArmorPercentGe
        | ArmorPercentLt
        | SelectedAmmoGe
        | SelectedAmmoLt
        | SelectedAmmoPercentGe
        | SelectedAmmoPercentLt
        | AmmoGe
        | AmmoLt
        | AmmoPercentGe
        | AmmoPercentLt => check_vitals_condition(condition, state),

        GameVersionGe | GameVersionLt | SessionTypeEq | SessionTypeNeq | GameModeEq
        | GameModeNeq | HudModeEq | AutomapModeEq | WidgetEnabled | WidgetDisabled
        | WidescreenModeEq => check_game_state_condition(condition, state),

        EpisodeEq | LevelGe | LevelLt => check_map_condition(condition, state),

        PatchEmpty | PatchNotEmpty => {
            let patch_name = condition.param_string.as_deref().unwrap_or("");

            let id = AssetId::new(patch_name);
            let exists = assets.textures.contains_key(&id);

            if condition.condition == PatchEmpty {
                !exists
            } else {
                exists
            }
        }

        KillsLt | KillsGe | ItemsLt | ItemsGe | SecretsLt | SecretsGe | KillsPercentLt
        | KillsPercentGe | ItemsPercentLt | ItemsPercentGe | SecretsPercentLt
        | SecretsPercentGe => check_stats_condition(condition, state),

        PowerupTimeLt | PowerupTimeGe | PowerupTimePercentLt | PowerupTimePercentGe => {
            check_powerup_condition(condition, state)
        }
    }
}

/// Evaluation for conditions involving weapons, slots, and ownership.
fn check_weapon_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use crate::model::ConditionType::*;
    let inv = &state.inventory;

    match condition.condition {
        WeaponOwned => match condition.param {
            100 => inv.has_chainsaw,
            101 => inv.has_shotgun,
            102 => inv.has_super_shotgun,
            103 => inv.has_chaingun,
            104 => inv.has_rocket_launcher,
            105 => inv.has_plasma_gun,
            106 => inv.has_bfg,
            _ => false,
        },
        WeaponNotOwned => !check_weapon_condition(
            &ConditionDef {
                condition: WeaponOwned,
                ..condition.clone()
            },
            state,
        ),
        SlotOwned => match condition.param {
            1 => inv.has_fist || inv.has_chainsaw,
            2 => inv.has_pistol,
            3 => inv.has_shotgun || inv.has_super_shotgun,
            4 => inv.has_chaingun,
            5 => inv.has_rocket_launcher,
            6 => inv.has_plasma_gun,
            7 => inv.has_bfg,
            _ => false,
        },
        SlotNotOwned => !check_weapon_condition(
            &ConditionDef {
                condition: SlotOwned,
                ..condition.clone()
            },
            state,
        ),
        SlotSelected => state.selected_weapon_slot == condition.param as u8,
        SlotNotSelected => state.selected_weapon_slot != condition.param as u8,
        WeaponSelected => match condition.param {
            100 => state.selected_weapon_slot == 1,
            101 => state.selected_weapon_slot == 3 && !state.use_super_shotgun,
            102 => state.selected_weapon_slot == 3 && state.use_super_shotgun,
            103 => state.selected_weapon_slot == 4,
            104 => state.selected_weapon_slot == 5,
            105 => state.selected_weapon_slot == 6,
            106 => state.selected_weapon_slot == 7,
            _ => false,
        },
        WeaponNotSelected => !check_weapon_condition(
            &ConditionDef {
                condition: WeaponSelected,
                ..condition.clone()
            },
            state,
        ),
        WeaponHasAmmo => inv.get_weapon_ammo_type(condition.param).is_some(),
        SelectedWeaponHasAmmo => inv.get_selected_ammo_type(state.selected_weapon_slot) != -1,
        AmmoMatch => inv.get_selected_ammo_type(state.selected_weapon_slot) == condition.param,
        _ => true,
    }
}

/// Evaluation for conditions involving general inventory items and keycards.
fn check_item_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use crate::model::ConditionType::*;
    let inv = &state.inventory;

    match condition.condition {
        ItemOwned => match condition.param {
            1 => inv.has_blue_card,
            2 => inv.has_yellow_card,
            3 => inv.has_red_card,
            4 => inv.has_blue_skull,
            5 => inv.has_yellow_skull,
            6 => inv.has_red_skull,
            7 => inv.has_backpack,
            14 => state.player.armor_max == 100,
            15 => state.player.armor_max == 200,
            16 => inv.has_automap,
            17 => inv.has_liteamp,
            18 => inv.has_berserk,
            19 => inv.has_invisibility,
            20 => inv.has_radsuit,
            21 => inv.has_invulnerability,
            _ => false,
        },
        ItemNotOwned => !check_item_condition(
            &ConditionDef {
                condition: ItemOwned,
                ..condition.clone()
            },
            state,
        ),
        _ => true,
    }
}

/// Evaluation for conditions involving Player vitals and precise ammo counts.
fn check_vitals_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use crate::model::ConditionType::*;
    let param = condition.param;
    let param2 = condition.param2;
    let p = &state.player;
    let inv = &state.inventory;

    match condition.condition {
        HealthGe => p.health >= param,
        HealthLt => p.health < param,
        HealthPercentGe => p.health >= param,
        HealthPercentLt => p.health < param,
        ArmorGe => p.armor >= param,
        ArmorLt => p.armor < param,
        ArmorPercentGe => state.get_stat_percent(p.armor, p.armor_max) >= param,
        ArmorPercentLt => state.get_stat_percent(p.armor, p.armor_max) < param,
        SelectedAmmoGe => {
            let idx = inv.get_selected_ammo_type(state.selected_weapon_slot);
            if idx == -1 {
                false
            } else {
                inv.get_ammo(idx) >= param
            }
        }
        SelectedAmmoLt => {
            let idx = inv.get_selected_ammo_type(state.selected_weapon_slot);
            if idx == -1 {
                false
            } else {
                inv.get_ammo(idx) < param
            }
        }
        SelectedAmmoPercentGe => {
            let idx = inv.get_selected_ammo_type(state.selected_weapon_slot);
            if idx == -1 {
                false
            } else {
                state.get_stat_percent(inv.get_ammo(idx), inv.get_max_ammo(idx)) >= param
            }
        }
        SelectedAmmoPercentLt => {
            let idx = inv.get_selected_ammo_type(state.selected_weapon_slot);
            if idx == -1 {
                false
            } else {
                state.get_stat_percent(inv.get_ammo(idx), inv.get_max_ammo(idx)) < param
            }
        }
        AmmoGe => inv.get_ammo(param2) >= param,
        AmmoLt => inv.get_ammo(param2) < param,
        AmmoPercentGe => {
            state.get_stat_percent(inv.get_ammo(param2), inv.get_max_ammo(param2)) >= param
        }
        AmmoPercentLt => {
            state.get_stat_percent(inv.get_ammo(param2), inv.get_max_ammo(param2)) < param
        }
        _ => true,
    }
}

/// Evaluation for conditions involving global engine status and HUD configuration.
fn check_game_state_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use crate::model::ConditionType::*;
    let param = condition.param;
    let world = &state.world;
    let engine = &state.engine;

    match condition.condition {
        GameVersionGe => (world.game_version as i32) >= param,
        GameVersionLt => (world.game_version as i32) < param,
        SessionTypeEq => world.session_type == param,
        SessionTypeNeq => world.session_type != param,

        GameModeEq => (world.game_version as i32) == param,
        GameModeNeq => (world.game_version as i32) != param,

        HudModeEq => engine.hud_mode == param,
        AutomapModeEq => {
            let enabled = engine.automap_active;
            let overlay = engine.automap_active && engine.automap_overlay;
            let req_enabled = (param & 1) != 0;
            let req_overlay = (param & 2) != 0;
            let req_disabled = (param & 4) != 0;
            if req_disabled && enabled {
                return false;
            }
            if req_enabled && !enabled {
                return false;
            }
            if req_overlay && !overlay {
                return false;
            }
            true
        }
        WidgetEnabled => {
            if let Some(name) = &condition.param_string {
                !engine.disabled_components.contains(name)
            } else {
                !engine.disabled_widgets.contains(&param)
            }
        }
        WidgetDisabled => {
            if let Some(name) = &condition.param_string {
                engine.disabled_components.contains(name)
            } else {
                engine.disabled_widgets.contains(&param)
            }
        }
        WidescreenModeEq => engine.widescreen_mode == (param != 0),
        _ => true,
    }
}

/// Evaluation for conditions involving map indices.
fn check_map_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use crate::model::ConditionType::*;
    match condition.condition {
        EpisodeEq => state.world.episode == condition.param,
        LevelGe => state.world.level >= condition.param,
        LevelLt => state.world.level < condition.param,
        _ => true,
    }
}

/// Evaluation for conditions involving cumulative level statistics.
fn check_stats_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use crate::model::ConditionType::*;
    let p = &state.player;
    let param = condition.param;
    match condition.condition {
        KillsLt => p.kills < param,
        KillsGe => p.kills >= param,
        ItemsLt => p.items < param,
        ItemsGe => p.items >= param,
        SecretsLt => p.secrets < param,
        SecretsGe => p.secrets >= param,
        KillsPercentLt => state.get_stat_percent(p.kills, p.max_kills) < param,
        KillsPercentGe => state.get_stat_percent(p.kills, p.max_kills) >= param,
        ItemsPercentLt => state.get_stat_percent(p.items, p.max_items) < param,
        ItemsPercentGe => state.get_stat_percent(p.items, p.max_items) >= param,
        SecretsPercentLt => state.get_stat_percent(p.secrets, p.max_secrets) < param,
        SecretsPercentGe => state.get_stat_percent(p.secrets, p.max_secrets) >= param,
        _ => true,
    }
}

/// Evaluation for conditions involving powerup duration countdowns.
fn check_powerup_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use crate::model::ConditionType::*;
    let duration = state
        .player
        .powerup_durations
        .get(&condition.param2)
        .cloned()
        .unwrap_or(0.0);
    let id = condition.param2;
    let p = condition.param as f32;

    match condition.condition {
        PowerupTimeLt => duration < p,
        PowerupTimeGe => duration >= p,
        PowerupTimePercentLt | PowerupTimePercentGe => {
            let percent = if (id == 1 || id == 4) && duration > 0.0 {
                100.0
            } else if duration <= 0.0 {
                0.0
            } else {
                let max_dur = match id {
                    0 => 30.0,
                    2 => 60.0,
                    3 => 60.0,
                    5 => 120.0,
                    _ => 30.0,
                };
                duration * 100.0 / max_dur
            };

            if condition.condition == PowerupTimePercentLt {
                percent < p
            } else {
                percent >= p
            }
        }
        _ => true,
    }
}
