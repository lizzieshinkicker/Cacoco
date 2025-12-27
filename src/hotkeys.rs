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
    pub duplicate: KeyboardShortcut,
    pub delete: KeyboardShortcut,
}

impl Default for HotkeyRegistry {
    fn default() -> Self {
        let cmd = Modifiers::COMMAND;

        Self {
            undo: KeyboardShortcut::new(cmd, Key::Z),
            redo: KeyboardShortcut::new(cmd | Modifiers::SHIFT, Key::Z),
            redo_alt: KeyboardShortcut::new(cmd, Key::Y),
            save: KeyboardShortcut::new(cmd, Key::S),
            open: KeyboardShortcut::new(cmd, Key::O),
            export_json: KeyboardShortcut::new(cmd, Key::E),
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

        let is_paste = ctx.input(|i| i.key_pressed(Key::V) && i.modifiers.command);
        if is_paste {
            ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::V));
            return Some(Action::Paste);
        }

        let is_copy = ctx.input(|i| i.key_pressed(Key::C) && i.modifiers.command);
        if is_copy {
            ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::C));
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
