use crate::model::{ConditionDef, ConditionType};

#[derive(Clone, Copy)]
pub struct LookupItem {
    pub id: i32,
    pub name: &'static str,
    pub icon: Option<&'static str>,
}

macro_rules! item {
    ($id:expr, $name:expr) => {
        LookupItem {
            id: $id,
            name: $name,
            icon: None,
        }
    };
    ($id:expr, $name:expr, $icon:expr) => {
        LookupItem {
            id: $id,
            name: $name,
            icon: Some($icon),
        }
    };
}

pub const ITEMS: &[LookupItem] = &[
    item!(1, "Blue Card", "BKEYA0"),
    item!(2, "Yellow Card", "YKEYA0"),
    item!(3, "Red Card", "RKEYA0"),
    item!(4, "Blue Skull", "BSKUB0"),
    item!(5, "Yellow Skull", "YSKUB0"),
    item!(6, "Red Skull", "RSKUB0"),
    item!(7, "Backpack", "BPAKA0"),
    item!(14, "Green Armor", "ARM1A0"),
    item!(15, "Megaarmor", "ARM2A0"),
    item!(16, "Comp. Map", "PMAPA0"),
    item!(17, "Lite-Amp", "PVISA0"),
    item!(18, "Berserk", "PSTRA0"),
    item!(19, "Invisibility", "PINSA0"),
    item!(20, "Rad Suit", "SUITA0"),
    item!(21, "Invulnerability", "PINVA0"),
];

pub const WEAPONS: &[LookupItem] = &[
    item!(100, "Chainsaw", "SAWGA0"),
    item!(101, "Shotgun", "SHTGA0"),
    item!(102, "Super Shotgun", "SHT2A0"),
    item!(103, "Chaingun", "CHGGA0"),
    item!(104, "Rocket Launcher", "MISGA0"),
    item!(105, "Plasma Rifle", "PLSGA0"),
    item!(106, "BFG 9000", "BFGGA0"),
];

pub const AMMO_TYPES: &[LookupItem] = &[
    item!(0, "Bullets", "AMMOA0"),
    item!(1, "Shells", "SHELA0"),
    item!(2, "Cells", "CELLA0"),
    item!(3, "Rockets", "ROCKA0"),
];

pub const POWERUPS: &[LookupItem] = &[
    item!(0, "Invulnerability", "PINVA0"),
    item!(1, "Berserk", "PSTRA0"),
    item!(2, "Invisibility", "PINSA0"),
    item!(3, "Rad Suit", "SUITA0"),
    item!(4, "Area Map", "PMAPA0"),
    item!(5, "Lite-Amp", "PVISA0"),
];

pub const STRING_TYPES: &[LookupItem] = &[
    item!(0, "Custom String"),
    item!(1, "Level Title"),
    item!(2, "Level Label"),
    item!(3, "Level Author"),
];

pub const SESSION_TYPES: &[LookupItem] = &[
    item!(0, "Single Player", "STFST01"),
    item!(1, "Cooperative", "STFST01"),
    item!(2, "Deathmatch", "STFDEAD0"),
];

pub const HUD_MODES: &[LookupItem] = &[item!(0, "Standard"), item!(1, "Compact")];

pub const WIDESCREEN_MODES: &[LookupItem] = &[item!(0, "Disabled"), item!(1, "Enabled")];

pub const AUTOMAP_FLAGS: &[LookupItem] = &[
    item!(1, "Enabled"),
    item!(2, "Overlay"),
    item!(4, "Disabled"),
];

pub const FEATURE_LEVELS: &[LookupItem] = &[
    item!(0, "Doom 1.9"),
    item!(1, "Limit Removing"),
    item!(2, "Boom 2.02"),
    item!(3, "Complevel 9"),
    item!(4, "MBF"),
    item!(5, "MBF21"),
    item!(6, "ID24"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GroupStyle {
    Standard,
    Natural,
    AmmoComplex,
}

pub struct GroupVariant {
    pub label: &'static str,
    pub condition: ConditionType,
}

pub struct ConditionGroup {
    pub name: &'static str,
    pub style: GroupStyle,
    pub variants: &'static [GroupVariant],
    pub icon: Option<&'static str>,
    pub default_param: i32,
}

macro_rules! v {
    ($lbl:expr, $typ:ident) => {
        GroupVariant {
            label: $lbl,
            condition: ConditionType::$typ,
        }
    };
}

pub const GROUPS: &[ConditionGroup] = &[
    ConditionGroup {
        name: "Weapon",
        icon: Some("PISGA0"),
        style: GroupStyle::Natural,
        default_param: 101,
        variants: &[
            v!("is owned", WeaponOwned),
            v!("NOT owned", WeaponNotOwned),
            v!("is selected", WeaponSelected),
            v!("NOT selected", WeaponNotSelected),
            v!("has ammo", WeaponHasAmmo),
        ],
    },
    ConditionGroup {
        name: "Item",
        icon: Some("PSTRA0"),
        style: GroupStyle::Natural,
        default_param: 1,
        variants: &[v!("is owned", ItemOwned), v!("NOT owned", ItemNotOwned)],
    },
    ConditionGroup {
        name: "Weapon Slot",
        icon: Some("STGNUM2"),
        style: GroupStyle::Natural,
        default_param: 2,
        variants: &[
            v!("is owned", SlotOwned),
            v!("NOT owned", SlotNotOwned),
            v!("is selected", SlotSelected),
            v!("NOT selected", SlotNotSelected),
        ],
    },
    ConditionGroup {
        name: "Health",
        icon: Some("MEDIA0"),
        style: GroupStyle::Standard,
        default_param: 100,
        variants: &[
            v!(">=", HealthGe),
            v!("<", HealthLt),
            v!("% >=", HealthPercentGe),
            v!("% <", HealthPercentLt),
        ],
    },
    ConditionGroup {
        name: "Armor",
        icon: Some("ARM1A0"),
        style: GroupStyle::Standard,
        default_param: 100,
        variants: &[
            v!(">=", ArmorGe),
            v!("<", ArmorLt),
            v!("% >=", ArmorPercentGe),
            v!("% <", ArmorPercentLt),
        ],
    },
    ConditionGroup {
        name: "Level Stats",
        icon: Some("STKILLS"),
        style: GroupStyle::Standard,
        default_param: 10,
        variants: &[
            v!("Kills >=", KillsGe),
            v!("Kills <", KillsLt),
            v!("Items >=", ItemsGe),
            v!("Items <", ItemsLt),
            v!("Secrets >=", SecretsGe),
            v!("Secrets <", SecretsLt),
            v!("Kills % >=", KillsPercentGe),
            v!("Kills % <", KillsPercentLt),
            v!("Items % >=", ItemsPercentGe),
            v!("Items % <", ItemsPercentLt),
            v!("Secrets % >=", SecretsPercentGe),
            v!("Secrets % <", SecretsPercentLt),
        ],
    },
    ConditionGroup {
        name: "Powerups",
        icon: Some("PINVA0"),
        style: GroupStyle::AmmoComplex,
        default_param: 10,
        variants: &[
            v!("Time >=", PowerupTimeGe),
            v!("Time <", PowerupTimeLt),
            v!("Time % >=", PowerupTimePercentGe),
            v!("Time % <", PowerupTimePercentLt),
        ],
    },
    ConditionGroup {
        name: "Selected Ammo",
        icon: Some("SHELA0"),
        style: GroupStyle::Standard,
        default_param: 10,
        variants: &[
            v!(">=", SelectedAmmoGe),
            v!("<", SelectedAmmoLt),
            v!("% >=", SelectedAmmoPercentGe),
            v!("% <", SelectedAmmoPercentLt),
            v!("matches", AmmoMatch),
            v!("uses ammo", SelectedWeaponHasAmmo),
        ],
    },
    ConditionGroup {
        name: "Specific Ammo",
        icon: Some("BPAK0"),
        style: GroupStyle::AmmoComplex,
        default_param: 10,
        variants: &[
            v!(">=", AmmoGe),
            v!("<", AmmoLt),
            v!("% >=", AmmoPercentGe),
            v!("% <", AmmoPercentLt),
        ],
    },
    ConditionGroup {
        name: "Game State",
        icon: Some("STFST01"),
        style: GroupStyle::Standard,
        default_param: 0,
        variants: &[
            v!("Session", SessionTypeEq),
            v!("Session NOT", SessionTypeNeq),
            v!("HUD Mode", HudModeEq),
            v!("Widescreen", WidescreenModeEq),
            v!("Automap", AutomapModeEq),
            v!("Version >=", GameVersionGe),
            v!("Version <", GameVersionLt),
        ],
    },
    ConditionGroup {
        name: "Map Info",
        icon: Some("PMAPA0"),
        style: GroupStyle::Standard,
        default_param: 1,
        variants: &[
            v!("Episode", EpisodeEq),
            v!("Level >=", LevelGe),
            v!("Level <", LevelLt),
        ],
    },
    ConditionGroup {
        name: "Patch Check",
        icon: Some("HICACOCO"),
        style: GroupStyle::Standard,
        default_param: 0,
        variants: &[
            v!("is empty", PatchEmpty),
            v!("is NOT empty", PatchNotEmpty),
        ],
    },
    ConditionGroup {
        name: "Widgets",
        icon: Some("M_OPTION"),
        style: GroupStyle::Standard,
        default_param: 0,
        variants: &[v!("Enabled", WidgetEnabled), v!("Disabled", WidgetDisabled)],
    },
];

pub fn find_group_for_type(t: ConditionType) -> (usize, usize) {
    for (g_idx, group) in GROUPS.iter().enumerate() {
        for (v_idx, variant) in group.variants.iter().enumerate() {
            if variant.condition == t {
                return (g_idx, v_idx);
            }
        }
    }
    (0, 0)
}

pub enum ParamUsage {
    None,
    Param1,
    Both,
    String,
}

pub fn get_param_usage(condition: ConditionType) -> ParamUsage {
    use ConditionType::*;
    match condition {
        SelectedWeaponHasAmmo | GameModeEq | GameModeNeq => ParamUsage::None,
        AmmoGe | AmmoLt | AmmoPercentGe | AmmoPercentLt => ParamUsage::Both,
        PowerupTimeGe | PowerupTimeLt | PowerupTimePercentGe | PowerupTimePercentLt => {
            ParamUsage::Both
        }
        PatchEmpty | PatchNotEmpty | WidgetEnabled | WidgetDisabled => ParamUsage::String,
        _ => ParamUsage::Param1,
    }
}

fn find_icon(list: &[LookupItem], id: i32) -> Option<&'static str> {
    list.iter().find(|i| i.id == id).and_then(|i| i.icon)
}

pub fn resolve_condition_icon(
    cond: &ConditionDef,
    state: &crate::state::PreviewState,
) -> Option<String> {
    use ConditionType::*;

    match cond.condition {
        SlotOwned | SlotNotOwned | SlotSelected | SlotNotSelected => {
            return Some(format!("STGNUM{}", cond.param));
        }
        ArmorGe | ArmorLt | ArmorPercentGe | ArmorPercentLt => {
            return Some(if state.player.armor_max >= 200 {
                "ARM2A0".to_string()
            } else {
                "ARM1A0".to_string()
            });
        }
        PowerupTimeGe | PowerupTimeLt | PowerupTimePercentGe | PowerupTimePercentLt => {
            return find_icon(POWERUPS, cond.param2).map(|s| s.to_string());
        }
        SelectedAmmoGe
        | SelectedAmmoLt
        | SelectedAmmoPercentGe
        | SelectedAmmoPercentLt
        | SelectedWeaponHasAmmo => {
            return match state.get_selected_ammo_type() {
                0 => Some("AMMOA0".to_string()),
                1 => Some("SHELA0".to_string()),
                2 => Some("CELLA0".to_string()),
                3 => Some("ROCKA0".to_string()),
                _ => Some("BPAK0".to_string()),
            };
        }
        _ => {}
    }

    let specific_icon = match cond.condition {
        WidgetEnabled => Some("M_SKULL1"),
        WidgetDisabled => Some("M_SKULL2"),
        WeaponOwned | WeaponNotOwned | WeaponSelected | WeaponNotSelected | WeaponHasAmmo => {
            find_icon(WEAPONS, cond.param)
        }
        ItemOwned | ItemNotOwned => find_icon(ITEMS, cond.param),
        AmmoMatch => find_icon(AMMO_TYPES, cond.param),
        AmmoGe | AmmoLt | AmmoPercentGe | AmmoPercentLt => find_icon(AMMO_TYPES, cond.param2),
        SessionTypeEq | SessionTypeNeq => find_icon(SESSION_TYPES, cond.param),
        _ => None,
    };

    if let Some(icon) = specific_icon {
        return Some(icon.to_string());
    }

    let (g_idx, _) = find_group_for_type(cond.condition);
    if g_idx < GROUPS.len() {
        return GROUPS[g_idx].icon.map(|s| s.to_string());
    }

    None
}
