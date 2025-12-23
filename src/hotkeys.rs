use eframe::egui;
use egui::{Key, KeyboardShortcut, Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Undo,
    Redo,
    Save,
    ExportJSON,
    Copy,
    Paste,
    Duplicate,
    Delete,
}

pub struct HotkeyRegistry {
    pub undo: KeyboardShortcut,
    pub redo: KeyboardShortcut,
    pub save: KeyboardShortcut,
    pub export_json: KeyboardShortcut,
    pub copy: KeyboardShortcut,
    pub paste: KeyboardShortcut,
    pub duplicate: KeyboardShortcut,
    pub delete: KeyboardShortcut,
}

impl Default for HotkeyRegistry {
    fn default() -> Self {
        Self {
            undo: KeyboardShortcut::new(Modifiers::COMMAND, Key::Z),
            redo: KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z),
            save: KeyboardShortcut::new(Modifiers::COMMAND, Key::S),
            export_json: KeyboardShortcut::new(Modifiers::COMMAND, Key::E),
            copy: KeyboardShortcut::new(Modifiers::COMMAND, Key::C),
            paste: KeyboardShortcut::new(Modifiers::COMMAND, Key::V),
            duplicate: KeyboardShortcut::new(Modifiers::COMMAND, Key::J),
            delete: KeyboardShortcut::new(Modifiers::NONE, Key::Delete),
        }
    }
}

impl HotkeyRegistry {
    pub fn check(&self, ctx: &egui::Context) -> Option<Action> {
        let redo_y = KeyboardShortcut::new(Modifiers::COMMAND, Key::Y);

        if ctx.input_mut(|i| i.consume_shortcut(&self.redo))
            || ctx.input_mut(|i| i.consume_shortcut(&redo_y))
        {
            return Some(Action::Redo);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.undo)) {
            return Some(Action::Undo);
        }

        let has_copy_event = ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Copy)));
        let has_paste_event = ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Paste(_))));

        if ctx.input_mut(|i| i.consume_shortcut(&self.save)) {
            return Some(Action::Save);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.export_json)) {
            return Some(Action::ExportJSON);
        }

        if has_copy_event || ctx.input_mut(|i| i.consume_shortcut(&self.copy)) {
            return Some(Action::Copy);
        }

        if has_paste_event || ctx.input_mut(|i| i.consume_shortcut(&self.paste)) {
            return Some(Action::Paste);
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