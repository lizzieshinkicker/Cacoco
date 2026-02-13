use crate::models::sbardef::FeatureLevel;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SlotMapping {
    #[default]
    /// Slot 1 is both Fist and Chainsaw, Slot 3 is both Shotgun and SSG.
    Vanilla,
    /// Woof-style with no slot overloads. Slot 1: Fist, 8: Chainsaw, 3: Shotgun, 9: SSG
    Extended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LookDirection {
    Right = 0,
    Straight = 1,
    Left = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub health: i32,
    pub armor: i32,
    pub armor_max: i32,
    pub is_god_mode: bool,
    pub kills: i32,
    pub items: i32,
    pub secrets: i32,
    pub max_kills: i32,
    pub max_items: i32,
    pub max_secrets: i32,
    pub powerup_durations: HashMap<i32, f32>,
}

impl Default for PlayerStats {
    fn default() -> Self {
        let mut durations = HashMap::new();
        for i in 0..6 {
            durations.insert(i, 0.0);
        }
        Self {
            health: 100,
            armor: 0,
            armor_max: 100,
            is_god_mode: false,
            kills: 0,
            items: 0,
            secrets: 0,
            max_kills: 27,
            max_items: 9,
            max_secrets: 5,
            powerup_durations: durations,
        }
    }
}

impl PlayerStats {
    pub fn get_face_sprite(
        &self,
        ouch: bool,
        look_dir: LookDirection,
        pain_timer: f32,
        evil_timer: f32,
    ) -> String {
        if self.health <= 0 {
            return "STFDEAD0".to_string();
        }
        if self.is_god_mode && !ouch {
            return "STFGOD0".to_string();
        }
        let dmg = match self.health {
            80.. => 0,
            60..=79 => 1,
            40..=59 => 2,
            20..=39 => 3,
            _ => 4,
        };
        if pain_timer > 0.0 {
            return format!("STFKILL{}", dmg);
        }
        if evil_timer > 0.0 {
            return format!("STFEVL{}", dmg);
        }
        if ouch {
            return format!("STFOUCH{}", dmg);
        }
        format!("STFST{}{}", dmg, look_dir as u8)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Inventory {
    pub ammo_bullets: i32,
    pub ammo_shells: i32,
    pub ammo_rockets: i32,
    pub ammo_cells: i32,
    pub has_backpack: bool,
    pub has_fist: bool,
    pub has_chainsaw: bool,
    pub has_pistol: bool,
    pub has_shotgun: bool,
    pub has_super_shotgun: bool,
    pub has_chaingun: bool,
    pub has_rocket_launcher: bool,
    pub has_plasma_gun: bool,
    pub has_bfg: bool,
    pub has_blue_card: bool,
    pub has_yellow_card: bool,
    pub has_red_card: bool,
    pub has_blue_skull: bool,
    pub has_yellow_skull: bool,
    pub has_red_skull: bool,
    pub has_invulnerability: bool,
    pub has_berserk: bool,
    pub has_invisibility: bool,
    pub has_radsuit: bool,
    pub has_automap: bool,
    pub has_liteamp: bool,
}

impl Inventory {
    pub fn get_ammo(&self, idx: i32) -> i32 {
        match idx {
            0 => self.ammo_bullets,
            1 => self.ammo_shells,
            2 => self.ammo_cells,
            3 => self.ammo_rockets,
            _ => 0,
        }
    }
    pub fn get_max_ammo(&self, idx: i32) -> i32 {
        let (base, pack) = match idx {
            0 => (200, 400),
            1 => (50, 100),
            2 => (300, 600),
            3 => (50, 100),
            _ => (0, 0),
        };
        if self.has_backpack { pack } else { base }
    }
    pub fn get_selected_ammo_type(&self, slot: u8) -> i32 {
        match slot {
            2 | 4 => 0,
            3 => 1,
            5 => 3,
            6 | 7 => 2,
            _ => -1,
        }
    }
    pub fn get_weapon_ammo_type(&self, param: i32) -> Option<i32> {
        match param {
            0 | 9 => None,
            1 | 3 | 102 | 103 => Some(0),
            2 | 10 | 101 => Some(1),
            4 | 104 => Some(3),
            5 | 6 | 105 | 106 => Some(2),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldContext {
    pub session_type: i32,
    pub episode: i32,
    pub level: i32,
    pub game_version: FeatureLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineContext {
    pub widescreen_mode: bool,
    pub aspect_correction: bool,
    pub hud_mode: i32,
    pub automap_active: bool,
    pub automap_overlay: bool,
    pub disabled_widgets: HashSet<i32>,
    pub disabled_components: HashSet<String>,
    pub slot_mapping: SlotMapping,
    pub auto_zoom: bool,
    pub zoom_level: i32,
    #[serde(skip)]
    pub pan_offset: eframe::egui::Vec2,
}

impl Default for EngineContext {
    fn default() -> Self {
        Self {
            widescreen_mode: false,
            aspect_correction: true,
            hud_mode: 0,
            automap_active: false,
            automap_overlay: false,
            disabled_widgets: HashSet::new(),
            disabled_components: HashSet::new(),
            slot_mapping: SlotMapping::default(),
            auto_zoom: true,
            zoom_level: 1,
            pan_offset: eframe::egui::Vec2::ZERO,
        }
    }
}

/// The serialized portion of the preview state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub player: PlayerStats,
    pub inventory: Inventory,
    pub world: WorldContext,
    pub engine: EngineContext,
    pub selected_weapon_slot: u8,
    pub use_super_shotgun: bool,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self {
            player: PlayerStats::default(),
            inventory: Inventory {
                has_fist: true,
                has_pistol: true,
                ammo_bullets: 50,
                ..Default::default()
            },
            world: WorldContext {
                episode: 1,
                level: 1,
                game_version: FeatureLevel::ID24,
                session_type: 0,
            },
            engine: EngineContext::default(),
            selected_weapon_slot: 2,
            use_super_shotgun: true,
        }
    }
}
