use crate::constants::DOOM_TICS_PER_SEC;

/// Handles transient visual state for the viewport preview.
#[derive(Debug, Clone)]
pub struct ViewerState {
    pub smoothed_fps: f32,
    pub display_fps: f32,
    pub fps_update_timer: f32,
    pub evil_timer: f32,
    pub pain_timer: f32,
    pub display_weapon_slot: u8,
    pub display_super_shotgun: bool,
    pub weapon_offset_y: f32,
    pub sky_yaw: i32,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            smoothed_fps: 60.0,
            display_fps: 60.0,
            fps_update_timer: 0.0,
            evil_timer: 0.0,
            pain_timer: 0.0,
            display_weapon_slot: 2,
            display_super_shotgun: true,
            weapon_offset_y: 0.0,
            sky_yaw: 0,
        }
    }
}

impl ViewerState {
    pub fn update(&mut self, dt: f32, target_slot: u8, use_ssg: bool) {
        // FPS Smoothing logic
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

        // Face timers
        self.evil_timer = (self.evil_timer - dt).max(0.0);
        self.pain_timer = (self.pain_timer - dt).max(0.0);

        // Weapon animation logic
        let speed = 600.0 * dt;
        let clear_height = 150.0;

        let distinct_weapons = self.display_weapon_slot != target_slot;
        let distinct_variants = self.display_weapon_slot == 3
            && target_slot == 3
            && self.display_super_shotgun != use_ssg;

        if distinct_weapons || distinct_variants {
            self.weapon_offset_y += speed;
            if self.weapon_offset_y >= clear_height {
                self.display_weapon_slot = target_slot;
                self.display_super_shotgun = use_ssg;
            }
        } else if self.weapon_offset_y > 0.0 {
            self.weapon_offset_y = (self.weapon_offset_y - speed).max(0.0);
        }
    }
}
