use crate::constants::DOOM_TICS_PER_SEC;
use crate::model::FeatureLevel;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    pub powerup_durations: HashMap<i32, f32>, // param -> seconds remaining
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
            max_kills: 100,
            max_items: 50,
            max_secrets: 10,
            powerup_durations: durations,
        }
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewState {
    pub player: PlayerStats,
    pub inventory: Inventory,
    pub world: WorldContext,
    pub engine: EngineContext,

    pub selected_weapon_slot: u8,
    pub use_super_shotgun: bool,

    #[serde(skip)]
    pub message_log: Vec<String>,
    #[serde(skip)]
    pub display_weapon_slot: u8,
    #[serde(skip)]
    pub display_super_shotgun: bool,
    #[serde(skip)]
    pub weapon_offset_y: f32,

    #[serde(skip)]
    pub smoothed_fps: f32,
    #[serde(skip)]
    pub display_fps: f32,
    #[serde(skip)]
    pub fps_update_timer: f32,
    #[serde(skip)]
    pub strobe_timer: f32,
    #[serde(skip)]
    pub evil_timer: f32,
    #[serde(skip)]
    pub pain_timer: f32,
    #[serde(skip)]
    pub virtual_mouse_pos: eframe::egui::Pos2,
}

impl Default for PreviewState {
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
            display_weapon_slot: 2,
            display_super_shotgun: true,
            weapon_offset_y: 0.0,
            smoothed_fps: 60.0,
            display_fps: 60.0,
            fps_update_timer: 0.0,
            strobe_timer: 0.0,
            evil_timer: 0.0,
            pain_timer: 0.0,
            virtual_mouse_pos: eframe::egui::pos2(0.0, 0.0),
            message_log: vec!["You got the Shotgun!".to_string()],
        }
    }
}

impl PreviewState {
    pub fn push_message(&mut self, msg: impl Into<String>) {
        let text = msg.into();
        println!("HUD: {}", text);
        self.message_log.push(text);
        if self.message_log.len() > 10 {
            self.message_log.remove(0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        if dt > 0.0 {
            let instant_fps = 1.0 / dt;
            self.smoothed_fps = (instant_fps * 0.05) + (self.smoothed_fps * 0.95);
        }

        self.fps_update_timer += dt;

        let tic_duration = 1.0 / (DOOM_TICS_PER_SEC as f32);
        if self.fps_update_timer >= tic_duration {
            self.display_fps = self.smoothed_fps;
            self.fps_update_timer = 0.0;
        }

        if self.strobe_timer > 0.0 {
            self.strobe_timer = (self.strobe_timer - dt).max(0.0);
        }
        if self.evil_timer > 0.0 {
            self.evil_timer = (self.evil_timer - dt).max(0.0);
        }
        if self.pain_timer > 0.0 {
            self.pain_timer = (self.pain_timer - dt).max(0.0);
        }

        for (id, duration) in self.player.powerup_durations.iter_mut() {
            // ID 1 (Berserk) and ID 4 (Map) are toggles/infinite
            if *id != 1 && *id != 4 {
                *duration = (*duration - dt).max(0.0);
            } else if *duration > 0.0 {
                *duration = 1.0;
            }
        }

        self.sync_inventory_with_durations();
        self.update_weapon_animation(dt);
    }

    fn sync_inventory_with_durations(&mut self) {
        let d = &self.player.powerup_durations;
        self.inventory.has_invulnerability = d.get(&0).map_or(false, |v| *v > 0.0);
        self.inventory.has_berserk = d.get(&1).map_or(false, |v| *v > 0.0);
        self.inventory.has_invisibility = d.get(&2).map_or(false, |v| *v > 0.0);
        self.inventory.has_radsuit = d.get(&3).map_or(false, |v| *v > 0.0);
        self.inventory.has_automap = d.get(&4).map_or(false, |v| *v > 0.0);
        self.inventory.has_liteamp = d.get(&5).map_or(false, |v| *v > 0.0);
    }

    fn update_weapon_animation(&mut self, dt: f32) {
        let speed = 600.0 * dt;
        let clear_height = 150.0;

        let distinct_weapons = self.display_weapon_slot != self.selected_weapon_slot;
        let distinct_variants = self.display_weapon_slot == 3
            && self.selected_weapon_slot == 3
            && self.display_super_shotgun != self.use_super_shotgun;

        if distinct_weapons || distinct_variants {
            self.weapon_offset_y += speed;
            if self.weapon_offset_y >= clear_height {
                self.display_weapon_slot = self.selected_weapon_slot;
                self.display_super_shotgun = self.use_super_shotgun;
            }
        } else if self.weapon_offset_y > 0.0 {
            self.weapon_offset_y = (self.weapon_offset_y - speed).max(0.0);
        }
    }

    pub fn get_ammo(&self, type_idx: i32) -> i32 {
        match type_idx {
            0 => self.inventory.ammo_bullets,
            1 => self.inventory.ammo_shells,
            2 => self.inventory.ammo_cells,
            3 => self.inventory.ammo_rockets,
            _ => 0,
        }
    }

    pub fn get_max_ammo(&self, type_idx: i32) -> i32 {
        let (base, pack) = match type_idx {
            0 => (200, 400),
            1 => (50, 100),
            2 => (300, 600),
            3 => (50, 100),
            _ => (0, 0),
        };
        if self.inventory.has_backpack {
            pack
        } else {
            base
        }
    }

    pub fn get_selected_ammo_type(&self) -> i32 {
        match self.selected_weapon_slot {
            2 | 4 => 0,
            3 => 1,
            5 => 3,
            6 | 7 => 2,
            _ => -1,
        }
    }

    pub fn get_weapon_ammo_type(&self, weapon_param: i32) -> Option<i32> {
        match weapon_param {
            101 | 3 => Some(1),
            103 | 2 | 4 => Some(0),
            104 | 5 => Some(3),
            105 | 106 | 6 | 7 => Some(2),
            _ => None,
        }
    }

    pub fn get_stat_percent(&self, current: i32, max: i32) -> i32 {
        if max <= 0 { 0 } else { (current * 100) / max }
    }

    pub fn get_face_sprite(&self, ouch: bool, look_dir: u8) -> String {
        if self.player.health <= 0 {
            return "STFDEAD0".to_string();
        }

        if self.player.is_god_mode && !ouch {
            return "STFGOD0".to_string();
        }

        let damage_level = if self.player.health >= 80 {
            0
        } else if self.player.health >= 60 {
            1
        } else if self.player.health >= 40 {
            2
        } else if self.player.health >= 20 {
            3
        } else {
            4
        };

        if self.pain_timer > 0.0 {
            return format!("STFKILL{}", damage_level);
        }
        if self.evil_timer > 0.0 {
            return format!("STFEVL{}", damage_level);
        }
        if ouch {
            return format!("STFOUCH{}", damage_level);
        }

        format!("STFST{}{}", damage_level, look_dir)
    }
}
