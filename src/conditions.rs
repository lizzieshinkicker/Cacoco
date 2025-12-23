use crate::model::{ConditionDef, ConditionType};
use crate::state::PreviewState;

pub fn resolve(conditions: &[ConditionDef], state: &PreviewState) -> bool {
    if conditions.is_empty() { return true; }
    for condition in conditions {
        if !check_single(condition, state) { return false; }
    }
    true
}

fn check_single(condition: &ConditionDef, state: &PreviewState) -> bool {
    use ConditionType::*;
    match condition.condition {
        WeaponOwned | WeaponNotOwned | WeaponSelected | WeaponNotSelected |
        WeaponHasAmmo | SelectedWeaponHasAmmo | AmmoMatch | SlotOwned |
        SlotNotOwned | SlotSelected | SlotNotSelected => {
            check_weapon_condition(condition, state)
        }
        ItemOwned | ItemNotOwned => check_item_condition(condition, state),
        HealthGe | HealthLt | HealthPercentGe | HealthPercentLt |
        ArmorGe | ArmorLt | ArmorPercentGe | ArmorPercentLt |
        SelectedAmmoGe | SelectedAmmoLt | SelectedAmmoPercentGe | SelectedAmmoPercentLt |
        AmmoGe | AmmoLt | AmmoPercentGe | AmmoPercentLt => {
            check_vitals_condition(condition, state)
        }
        GameVersionGe | GameVersionLt | SessionTypeEq | SessionTypeNeq |
        GameModeEq | GameModeNeq | HudModeEq | AutomapModeEq |
        WidgetEnabled | WidgetDisabled | WidescreenModeEq => {
            check_game_state_condition(condition, state)
        }
        EpisodeEq | LevelGe | LevelLt => check_map_condition(condition, state),
    }
}

fn check_weapon_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use ConditionType::*;
    match condition.condition {
        WeaponOwned => match condition.param {
            100 => state.inventory.has_chainsaw,
            101 => state.inventory.has_shotgun,
            102 => state.inventory.has_super_shotgun,
            103 => state.inventory.has_chaingun,
            104 => state.inventory.has_rocket_launcher,
            105 => state.inventory.has_plasma_gun,
            106 => state.inventory.has_bfg,
            _ => false,
        },
        WeaponNotOwned => !check_weapon_condition(&ConditionDef { condition: WeaponOwned, ..*condition }, state),
        SlotOwned => match condition.param {
            1 => state.inventory.has_fist || state.inventory.has_chainsaw,
            2 => state.inventory.has_pistol,
            3 => state.inventory.has_shotgun || state.inventory.has_super_shotgun,
            4 => state.inventory.has_chaingun,
            5 => state.inventory.has_rocket_launcher,
            6 => state.inventory.has_plasma_gun,
            7 => state.inventory.has_bfg,
            _ => false,
        },
        SlotNotOwned => !check_weapon_condition(&ConditionDef { condition: SlotOwned, ..*condition }, state),
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
        WeaponNotSelected => !check_weapon_condition(&ConditionDef { condition: WeaponSelected, ..*condition }, state),
        WeaponHasAmmo => state.get_weapon_ammo_type(condition.param).is_some(),
        SelectedWeaponHasAmmo => state.get_selected_ammo_type() != -1,
        AmmoMatch => state.get_selected_ammo_type() == condition.param,
        _ => true,
    }
}

fn check_item_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use ConditionType::*;
    match condition.condition {
        ItemOwned => match condition.param {
            1 => state.inventory.has_blue_card,
            2 => state.inventory.has_yellow_card,
            3 => state.inventory.has_red_card,
            4 => state.inventory.has_blue_skull,
            5 => state.inventory.has_yellow_skull,
            6 => state.inventory.has_red_skull,
            7 => state.inventory.has_backpack,
            14 => state.player.armor_max == 100,
            15 => state.player.armor_max == 200,
            16 => state.inventory.has_automap,
            17 => state.inventory.has_liteamp,
            18 => state.inventory.has_berserk,
            19 => state.inventory.has_invisibility,
            20 => state.inventory.has_radsuit,
            21 => state.inventory.has_invulnerability,
            _ => false,
        },
        ItemNotOwned => !check_item_condition(&ConditionDef { condition: ItemOwned, ..*condition }, state),
        _ => true,
    }
}

fn check_vitals_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use ConditionType::*;
    let param = condition.param;
    let param2 = condition.param2;
    match condition.condition {
        HealthGe => state.player.health >= param,
        HealthLt => state.player.health < param,
        HealthPercentGe => state.player.health >= param,
        HealthPercentLt => state.player.health < param,
        ArmorGe => state.player.armor >= param,
        ArmorLt => state.player.armor < param,
        ArmorPercentGe => {
            if state.player.armor_max == 0 { false }
            else { (state.player.armor * 100 / state.player.armor_max) >= param }
        },
        ArmorPercentLt => {
            if state.player.armor_max == 0 { false }
            else { (state.player.armor * 100 / state.player.armor_max) < param }
        },
        SelectedAmmoGe => {
            let idx = state.get_selected_ammo_type();
            if idx == -1 { false } else { state.get_ammo(idx) >= param }
        },
        SelectedAmmoLt => {
            let idx = state.get_selected_ammo_type();
            if idx == -1 { false } else { state.get_ammo(idx) < param }
        },
        SelectedAmmoPercentGe => {
            let idx = state.get_selected_ammo_type();
            if idx == -1 { false } else {
                let max = state.get_max_ammo(idx);
                if max == 0 { false } else { (state.get_ammo(idx) * 100 / max) >= param }
            }
        },
        SelectedAmmoPercentLt => {
            let idx = state.get_selected_ammo_type();
            if idx == -1 { false } else {
                let max = state.get_max_ammo(idx);
                if max == 0 { false } else { (state.get_ammo(idx) * 100 / max) < param }
            }
        },
        AmmoGe => state.get_ammo(param2) >= param,
        AmmoLt => state.get_ammo(param2) < param,
        AmmoPercentGe => {
            let max = state.get_max_ammo(param2);
            if max == 0 { false } else { (state.get_ammo(param2) * 100 / max) >= param }
        },
        AmmoPercentLt => {
            let max = state.get_max_ammo(param2);
            if max == 0 { false } else { (state.get_ammo(param2) * 100 / max) < param }
        },
        _ => true,
    }
}

fn check_game_state_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use ConditionType::*;
    let param = condition.param;
    match condition.condition {
        GameVersionGe => state.world.game_version >= param,
        GameVersionLt => state.world.game_version < param,
        SessionTypeEq => state.world.session_type == param,
        SessionTypeNeq => state.world.session_type != param,
        GameModeEq => true,
        GameModeNeq => false,
        HudModeEq => state.engine.hud_mode == param,
        AutomapModeEq => {
            let enabled = state.engine.automap_active;
            let overlay = state.engine.automap_active && state.engine.automap_overlay;
            let req_enabled = (param & 1) != 0;
            let req_overlay = (param & 2) != 0;
            let req_disabled = (param & 4) != 0;
            if req_disabled && enabled { return false; }
            if req_enabled && !enabled { return false; }
            if req_overlay && !overlay { return false; }
            true
        },
        WidgetEnabled => !state.engine.disabled_widgets.contains(&param),
        WidgetDisabled => state.engine.disabled_widgets.contains(&param),
        WidescreenModeEq => state.engine.widescreen_mode == (param != 0),
        _ => true,
    }
}

fn check_map_condition(condition: &ConditionDef, state: &PreviewState) -> bool {
    use ConditionType::*;
    match condition.condition {
        EpisodeEq => state.world.episode == condition.param,
        LevelGe => state.world.level >= condition.param,
        LevelLt => state.world.level < condition.param,
        _ => true,
    }
}