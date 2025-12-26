use crate::state::PreviewState;
use eframe::egui;

struct Cheat {
    code: &'static str,
    action: fn(&mut PreviewState),
}

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
            s.push_message(msg);
        },
    },
    Cheat {
        code: "idkfa",
        action: |s| {
            give_all(s, true);
            s.push_message("Very Happy Ammo Added.");
        },
    },
    Cheat {
        code: "idfa",
        action: |s| {
            give_all(s, false);
            s.push_message("Happy Ammo Added.");
        },
    },
    Cheat {
        code: "idchoppers",
        action: |s| {
            s.inventory.has_chainsaw = true;
            s.selected_weapon_slot = 1;
            s.push_message("...something small for the children, sir?");
        },
    },
    Cheat {
        code: "idbeholdv",
        action: |s| {
            s.inventory.has_invulnerability = !s.inventory.has_invulnerability;
            s.push_message("Invulnerability On/Off");
        },
    },
    Cheat {
        code: "idbeholds",
        action: |s| {
            s.inventory.has_berserk = !s.inventory.has_berserk;
            s.player.health = 100;
            s.push_message("Berserk On/Off");
        },
    },
    Cheat {
        code: "idbeholdi",
        action: |s| {
            s.inventory.has_invisibility = !s.inventory.has_invisibility;
            s.push_message("Invisibility On/Off");
        },
    },
    Cheat {
        code: "idbeholdr",
        action: |s| {
            s.inventory.has_radsuit = !s.inventory.has_radsuit;
            s.push_message("Radiation Suit On/Off");
        },
    },
    Cheat {
        code: "idbeholda",
        action: |s| {
            s.inventory.has_automap = !s.inventory.has_automap;
            s.push_message("Computer Map Added");
        },
    },
    Cheat {
        code: "idbeholdl",
        action: |s| {
            s.inventory.has_liteamp = !s.inventory.has_liteamp;
            s.push_message("Light Amplification On/Off");
        },
    },
];

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
    pub fn process_input(&mut self, ctx: &egui::Context, state: &mut PreviewState) {
        if ctx.wants_keyboard_input() {
            return;
        }

        ctx.input(|i| {
            if i.key_pressed(egui::Key::Num1) {
                state.selected_weapon_slot = 1;
                state.inventory.has_chainsaw = true;
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

    fn check_cheats(&mut self, state: &mut PreviewState) {
        if self.buffer.ends_with("idbehold") {
            return;
        }

        for cheat in CHEATS {
            if self.buffer.ends_with(cheat.code) {
                println!("CHEAT: {}", cheat.code.to_uppercase());
                (cheat.action)(state);
                self.buffer.clear();
                return;
            }
        }
    }
}

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
