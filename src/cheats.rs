use crate::state::PreviewState;
use crate::ui::messages::{self, EditorEvent};
use eframe::egui;

/// Represents a secret console command that modifies the preview state.
struct Cheat {
    code: &'static str,
    action: fn(&mut PreviewState),
}

/// The registry of supported cheat codes.
static CHEATS: &[Cheat] = &[
    Cheat {
        code: "iddqd",
        action: |s| {
            s.player.is_god_mode = !s.player.is_god_mode;
            s.player.health = 100;
            let msg = if s.player.is_god_mode {
                "Degreelessness Mode On"
            } else {
                "Degreelessness Mode Off"
            };
            messages::log_event(s, EditorEvent::Cheat(msg.to_string()));
        },
    },
    Cheat {
        code: "idkfa",
        action: |s| {
            give_all(s, true);
            messages::log_event(s, EditorEvent::Cheat("Very Happy Ammo Added.".to_string()));
        },
    },
    Cheat {
        code: "idfa",
        action: |s| {
            give_all(s, false);
            messages::log_event(s, EditorEvent::Cheat("Happy Ammo Added.".to_string()));
        },
    },
    Cheat {
        code: "idchoppers",
        action: |s| {
            s.inventory.has_chainsaw = true;
            s.selected_weapon_slot = 1;
            messages::log_event(
                s,
                EditorEvent::Cheat("...something small for the children, sir?".to_string()),
            );
        },
    },
    Cheat {
        code: "idbeholdv",
        action: |s| {
            let dur = if s.inventory.has_invulnerability {
                0.0
            } else {
                30.0
            };
            s.player.powerup_durations.insert(0, dur);
            messages::log_event(s, EditorEvent::Cheat("Invulnerability On/Off".to_string()));
        },
    },
    Cheat {
        code: "idbeholds",
        action: |s| {
            let dur = if s.inventory.has_berserk { 0.0 } else { 1.0 };
            s.player.powerup_durations.insert(1, dur);
            s.player.health = 100;
            messages::log_event(s, EditorEvent::Cheat("Berserk On/Off".to_string()));
        },
    },
    Cheat {
        code: "idbeholdi",
        action: |s| {
            let dur = if s.inventory.has_invisibility {
                0.0
            } else {
                60.0
            };
            s.player.powerup_durations.insert(2, dur);
            messages::log_event(s, EditorEvent::Cheat("Invisibility On/Off".to_string()));
        },
    },
    Cheat {
        code: "idbeholdr",
        action: |s| {
            let dur = if s.inventory.has_radsuit { 0.0 } else { 60.0 };
            s.player.powerup_durations.insert(3, dur);
            messages::log_event(s, EditorEvent::Cheat("Radiation Suit On/Off".to_string()));
        },
    },
    Cheat {
        code: "idbeholda",
        action: |s| {
            let dur = if s.inventory.has_automap { 0.0 } else { 1.0 };
            s.player.powerup_durations.insert(4, dur);
            messages::log_event(s, EditorEvent::Cheat("Computer Map Added".to_string()));
        },
    },
    Cheat {
        code: "idbeholdl",
        action: |s| {
            let dur = if s.inventory.has_liteamp { 0.0 } else { 120.0 };
            s.player.powerup_durations.insert(5, dur);
            messages::log_event(
                s,
                EditorEvent::Cheat("Light Amplification On/Off".to_string()),
            );
        },
    },
];

/// Monitors and processes keyboard input for cheat codes and weapon hotkeys.
pub struct CheatEngine {
    buffer: String,
}

impl Default for CheatEngine {
    fn default() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl CheatEngine {
    /// Processes current frame input to update cheats and weapon slots.
    pub fn process_input(&mut self, ctx: &egui::Context, state: &mut PreviewState) {
        if ctx.wants_keyboard_input() {
            return;
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Num1) {
                state.selected_weapon_slot = 1;
                state.inventory.has_fist = true;
                if state.engine.slot_mapping == crate::state::SlotMapping::Vanilla {
                    state.inventory.has_chainsaw = true;
                }
            }
            if i.key_pressed(egui::Key::Num2) {
                state.selected_weapon_slot = 2;
                state.inventory.has_pistol = true;
            }
            if i.key_pressed(egui::Key::Num3) {
                state.selected_weapon_slot = 3;
                state.use_super_shotgun = true;
                state.inventory.has_shotgun = true;
                state.inventory.has_super_shotgun = true;
            }
            if i.key_pressed(egui::Key::Num4) {
                state.selected_weapon_slot = 4;
                state.inventory.has_chaingun = true;
            }
            if i.key_pressed(egui::Key::Num5) {
                state.selected_weapon_slot = 5;
                state.inventory.has_rocket_launcher = true;
            }
            if i.key_pressed(egui::Key::Num6) {
                state.selected_weapon_slot = 6;
                state.inventory.has_plasma_gun = true;
            }
            if i.key_pressed(egui::Key::Num7) {
                state.selected_weapon_slot = 7;
                state.inventory.has_bfg = true;
            }
            if i.key_pressed(egui::Key::Num8) {
                state.selected_weapon_slot = 8;
                state.inventory.has_chainsaw = true;
            }
            if i.key_pressed(egui::Key::Num9) {
                state.selected_weapon_slot = 9;
                state.inventory.has_super_shotgun = true;
            }
        });

        let events = ctx.input(|i| i.events.clone());
        for event in events {
            if let egui::Event::Text(text) = event {
                self.buffer.push_str(&text.to_lowercase());
                if self.buffer.len() > 20 {
                    let len = self.buffer.len();
                    self.buffer = self.buffer[len - 20..].to_string();
                }
                self.check_cheats(state);
            }
        }
    }

    /// Checks if the current buffer ends with a valid cheat code.
    fn check_cheats(&mut self, state: &mut PreviewState) {
        if self.buffer.ends_with("idbehold") {
            return;
        }

        for cheat in CHEATS {
            if self.buffer.ends_with(cheat.code) {
                (cheat.action)(state);
                self.buffer.clear();
                return;
            }
        }
    }
}

/// Helper used by IDFA and IDKFA to refill the player's stock.
fn give_all(s: &mut PreviewState, give_keys: bool) {
    s.player.armor = 200;
    s.player.armor_max = 200;

    s.inventory.has_fist = true;
    s.inventory.has_chainsaw = true;
    s.inventory.has_pistol = true;
    s.inventory.has_shotgun = true;
    s.inventory.has_super_shotgun = true;
    s.inventory.has_chaingun = true;
    s.inventory.has_rocket_launcher = true;
    s.inventory.has_plasma_gun = true;
    s.inventory.has_bfg = true;

    s.inventory.has_backpack = true;
    s.inventory.ammo_bullets = 400;
    s.inventory.ammo_shells = 100;
    s.inventory.ammo_rockets = 100;
    s.inventory.ammo_cells = 600;

    if give_keys {
        s.inventory.has_blue_card = true;
        s.inventory.has_yellow_card = true;
        s.inventory.has_red_card = true;
        s.inventory.has_blue_skull = true;
        s.inventory.has_yellow_skull = true;
        s.inventory.has_red_skull = true;
    }
}
