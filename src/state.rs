use crate::constants::DOOM_TICS_PER_SEC;
use crate::models::sbardef::FeatureLevel;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

fn default_true() -> bool {
    true
}
fn default_one() -> i32 {
    1
}

/// Represents visually whether slots 1 and 3 are overloaded (Vanilla), or extended to 8 and 9.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SlotMapping {
    #[default]
    Vanilla, // 1: Fist/Saw, 3: Shot/SSG
    Extended, // 1: Fist, 8: Saw, 3: Shot, 9: SSG
}

/// Represents the physical state of the player character.
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
    /// Maps powerup IDs to remaining time in seconds.
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
    /// Determines the correct STF patch name based on health and timers.
    pub fn get_face_sprite(
        &self,
        ouch: bool,
        look_dir: u8,
        pain_timer: f32,
        evil_timer: f32,
    ) -> String {
        if self.health <= 0 {
            return "STFDEAD0".to_string();
        }
        if self.is_god_mode && !ouch {
            return "STFGOD0".to_string();
        }

        let damage_level = match self.health {
            80.. => 0,
            60..=79 => 1,
            40..=59 => 2,
            20..=39 => 3,
            _ => 4,
        };

        if pain_timer > 0.0 {
            return format!("STFKILL{}", damage_level);
        }
        if evil_timer > 0.0 {
            return format!("STFEVL{}", damage_level);
        }
        if ouch {
            return format!("STFOUCH{}", damage_level);
        }

        format!("STFST{}{}", damage_level, look_dir)
    }
}

/// Manages ammo counts, weapon ownership, and keycards.
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
    /// Returns the current ammo count for a specific ammo type index.
    pub fn get_ammo(&self, type_idx: i32) -> i32 {
        match type_idx {
            0 => self.ammo_bullets,
            1 => self.ammo_shells,
            2 => self.ammo_cells,
            3 => self.ammo_rockets,
            _ => 0,
        }
    }

    /// Returns the max capacity for a specific ammo type, factoring in the backpack.
    pub fn get_max_ammo(&self, type_idx: i32) -> i32 {
        let (base, pack) = match type_idx {
            0 => (200, 400),
            1 => (50, 100),
            2 => (300, 600),
            3 => (50, 100),
            _ => (0, 0),
        };
        if self.has_backpack { pack } else { base }
    }

    /// Maps the selected weapon slot to its associated ammo type index.
    pub fn get_selected_ammo_type(&self, slot: u8) -> i32 {
        match slot {
            2 | 4 => 0,
            3 => 1,
            5 => 3,
            6 | 7 => 2,
            _ => -1,
        }
    }

    /// Maps a weapon parameter (ID24) to an ammo type.
    pub fn get_weapon_ammo_type(&self, weapon_param: i32) -> Option<i32> {
        match weapon_param {
            0 | 9 => None,
            1 | 3 | 102 | 103 => Some(0),
            2 | 10 | 101 => Some(1),
            4 | 104 => Some(3),
            5 | 6 | 105 | 106 => Some(2),
            _ => None,
        }
    }
}

/// Context regarding the current level and game session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldContext {
    pub session_type: i32,
    pub episode: i32,
    pub level: i32,
    pub game_version: FeatureLevel,
}

/// Rendering and engine-level behavior settings.
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

    #[serde(default = "default_true")]
    pub auto_zoom: bool,
    #[serde(default = "default_one")]
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
            auto_zoom: true,
            zoom_level: 1,
            pan_offset: eframe::egui::Vec2::ZERO,
            slot_mapping: SlotMapping::default(),
        }
    }
}

/// Transient state used only within the Cacoco editor (not serialized).
#[derive(Debug, Clone, Default)]
pub struct EditorContext {
    pub message_log: Vec<String>,
    pub smoothed_fps: f32,
    pub display_fps: f32,
    pub fps_update_timer: f32,
    pub strobe_timer: f32,
    pub evil_timer: f32,
    pub pain_timer: f32,
    pub virtual_mouse_pos: eframe::egui::Pos2,
    pub hovered_path: Option<Vec<usize>>,
    pub grabbed_path: Option<Vec<usize>>,
    pub display_weapon_slot: u8,
    pub display_super_shotgun: bool,
    pub weapon_offset_y: f32,
}

/// The top-level state object representing everything the editor knows about the "Game".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewState {
    pub player: PlayerStats,
    pub inventory: Inventory,
    pub world: WorldContext,
    pub engine: EngineContext,

    pub selected_weapon_slot: u8,
    pub use_super_shotgun: bool,

    /// Editor-only metadata (timers, log, etc)
    #[serde(skip)]
    pub editor: EditorContext,
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
            editor: EditorContext {
                display_weapon_slot: 2,
                display_super_shotgun: true,
                smoothed_fps: 60.0,
                display_fps: 60.0,
                message_log: vec!["You got the Shotgun!".to_string()],
                ..Default::default()
            },
        }
    }
}

impl PreviewState {
    /// Adds a message to the editor console log.
    pub fn push_message(&mut self, msg: impl Into<String>) {
        let text = msg.into();
        self.editor.message_log.push(text);
        if self.editor.message_log.len() > 10 {
            self.editor.message_log.remove(0);
        }
    }

    /// Advances the simulation timers and animations.
    pub fn update(&mut self, dt: f32) {
        if dt > 0.0 {
            let instant_fps = 1.0 / dt;
            self.editor.smoothed_fps = (instant_fps * 0.05) + (self.editor.smoothed_fps * 0.95);
        }

        self.editor.fps_update_timer += dt;
        let tic_duration = 1.0 / (DOOM_TICS_PER_SEC as f32);
        if self.editor.fps_update_timer >= tic_duration {
            self.editor.display_fps = self.editor.smoothed_fps;
            self.editor.fps_update_timer = 0.0;
        }

        self.editor.strobe_timer = (self.editor.strobe_timer - dt).max(0.0);
        self.editor.evil_timer = (self.editor.evil_timer - dt).max(0.0);
        self.editor.pain_timer = (self.editor.pain_timer - dt).max(0.0);

        for (id, duration) in self.player.powerup_durations.iter_mut() {
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

        let distinct_weapons = self.editor.display_weapon_slot != self.selected_weapon_slot;
        let distinct_variants = self.editor.display_weapon_slot == 3
            && self.selected_weapon_slot == 3
            && self.editor.display_super_shotgun != self.use_super_shotgun;

        if distinct_weapons || distinct_variants {
            self.editor.weapon_offset_y += speed;
            if self.editor.weapon_offset_y >= clear_height {
                self.editor.display_weapon_slot = self.selected_weapon_slot;
                self.editor.display_super_shotgun = self.use_super_shotgun;
            }
        } else if self.editor.weapon_offset_y > 0.0 {
            self.editor.weapon_offset_y = (self.editor.weapon_offset_y - speed).max(0.0);
        }
    }

    /// Facade method to calculate a percentage based on game stats.
    pub fn get_stat_percent(&self, current: i32, max: i32) -> i32 {
        if max <= 0 { 0 } else { (current * 100) / max }
    }
}
