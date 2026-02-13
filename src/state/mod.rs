pub mod interaction;
pub mod simulation;
pub mod viewer;

pub use interaction::InteractionState;
pub use simulation::{EngineContext, SimulationState, SlotMapping};
pub use viewer::ViewerState;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreviewState {
    /// The serialized Doom simulation.
    pub sim: SimulationState,

    /// Transient interaction state (Editor logic).
    #[serde(skip)]
    pub interaction: InteractionState,

    /// Transient visual state (Viewport logic).
    #[serde(skip)]
    pub viewer: ViewerState,
}

impl PreviewState {
    pub fn update(&mut self, dt: f32) {
        self.interaction.update(dt);
        self.viewer.update(
            dt,
            self.sim.selected_weapon_slot,
            self.sim.use_super_shotgun,
        );
        for (id, duration) in self.sim.player.powerup_durations.iter_mut() {
            if *id != 1 && *id != 4 {
                *duration = (*duration - dt).max(0.0);
            } else if *duration > 0.0 {
                *duration = 1.0;
            }
        }

        self.sync_inventory_with_durations();
    }

    fn sync_inventory_with_durations(&mut self) {
        let d = &self.sim.player.powerup_durations;
        let inv = &mut self.sim.inventory;
        inv.has_invulnerability = d.get(&0).map_or(false, |v| *v > 0.0);
        inv.has_berserk = d.get(&1).map_or(false, |v| *v > 0.0);
        inv.has_invisibility = d.get(&2).map_or(false, |v| *v > 0.0);
        inv.has_radsuit = d.get(&3).map_or(false, |v| *v > 0.0);
        inv.has_automap = d.get(&4).map_or(false, |v| *v > 0.0);
        inv.has_liteamp = d.get(&5).map_or(false, |v| *v > 0.0);
    }

    pub fn get_stat_percent(&self, current: i32, max: i32) -> i32 {
        if max <= 0 { 0 } else { (current * 100) / max }
    }
}
