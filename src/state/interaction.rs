use eframe::egui;

/// Handles UI-specific interaction state that does not persist in the project file.
#[derive(Debug, Clone, Default)]
pub struct InteractionState {
    pub message_log: Vec<String>,
    pub strobe_timer: f32,
    pub virtual_mouse_pos: egui::Pos2,
    pub hovered_path: Option<Vec<usize>>,
    pub grabbed_path: Option<Vec<usize>>,
}

impl InteractionState {
    pub fn push_message(&mut self, msg: impl Into<String>) {
        let text = msg.into();
        self.message_log.push(text);
        if self.message_log.len() > 10 {
            self.message_log.remove(0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.strobe_timer = (self.strobe_timer - dt).max(0.0);
    }
}
