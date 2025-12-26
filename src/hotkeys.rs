use eframe::egui;
use egui::{Key, KeyboardShortcut, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Undo,
    Redo,
    Save,
    Open,
    ExportJSON,
    Copy,
    Paste,
    Duplicate,
    Delete,
}

pub struct HotkeyRegistry {
    pub undo: KeyboardShortcut,
    pub redo: KeyboardShortcut,
    pub redo_alt: KeyboardShortcut,
    pub save: KeyboardShortcut,
    pub open: KeyboardShortcut,
    pub export_json: KeyboardShortcut,
    pub copy: KeyboardShortcut,
    pub paste: KeyboardShortcut,
    pub duplicate: KeyboardShortcut,
    pub delete: KeyboardShortcut,
}

impl Default for HotkeyRegistry {
    fn default() -> Self {
        let cmd = Modifiers::COMMAND;

        Self {
            undo: KeyboardShortcut::new(cmd, Key::Z),
            redo: KeyboardShortcut::new(cmd | Modifiers::SHIFT, Key::Z),
            redo_alt: KeyboardShortcut::new(cmd, Key::Y), // Standard alternative for Redo
            save: KeyboardShortcut::new(cmd, Key::S),
            open: KeyboardShortcut::new(cmd, Key::O),
            export_json: KeyboardShortcut::new(cmd, Key::E),
            copy: KeyboardShortcut::new(cmd, Key::C),
            paste: KeyboardShortcut::new(cmd, Key::V),
            duplicate: KeyboardShortcut::new(cmd, Key::J),
            delete: KeyboardShortcut::new(Modifiers::NONE, Key::Delete),
        }
    }
}

impl HotkeyRegistry {
    pub fn check(&self, ctx: &egui::Context) -> Option<Action> {
        if ctx.input_mut(|i| i.consume_shortcut(&self.save)) {
            return Some(Action::Save);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.open)) {
            return Some(Action::Open);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.export_json)) {
            return Some(Action::ExportJSON);
        }

        if ctx.wants_keyboard_input() {
            return None;
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.redo) || i.consume_shortcut(&self.redo_alt)) {
            return Some(Action::Redo);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.undo)) {
            return Some(Action::Undo);
        }

        let has_paste_event =
            ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Paste(_))));
        if has_paste_event || ctx.input_mut(|i| i.consume_shortcut(&self.paste)) {
            return Some(Action::Paste);
        }

        let has_copy_event = ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Copy)));
        if has_copy_event || ctx.input_mut(|i| i.consume_shortcut(&self.copy)) {
            return Some(Action::Copy);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.duplicate)) {
            return Some(Action::Duplicate);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.delete)) {
            return Some(Action::Delete);
        }

        None
    }
}
