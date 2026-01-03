use eframe::egui;
use egui::{Event, Key, KeyboardShortcut, Modifiers};

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
    pub copy: KeyboardShortcut,
    pub paste: KeyboardShortcut,
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
            copy: KeyboardShortcut::new(cmd, Key::C),
            paste: KeyboardShortcut::new(cmd, Key::V),
        }
    }
}

impl HotkeyRegistry {
    pub fn check(&self, ctx: &egui::Context) -> Option<Action> {
        let mut event_action = None;
        ctx.input(|i| {
            for event in &i.events {
                match event {
                    Event::Copy => event_action = Some(Action::Copy),
                    Event::Cut => event_action = Some(Action::Copy),
                    Event::Paste(_) => event_action = Some(Action::Paste),
                    _ => {}
                }
            }
        });

        if let Some(action) = event_action {
            return Some(action);
        }

        if ctx.wants_keyboard_input() && !ctx.input(|i| i.modifiers.any()) {
            return None;
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.save)) {
            return Some(Action::Save);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.open)) {
            return Some(Action::Open);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.export_json)) {
            return Some(Action::ExportJSON);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.redo) || i.consume_shortcut(&self.redo_alt)) {
            return Some(Action::Redo);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.undo)) {
            return Some(Action::Undo);
        }

        if ctx.input_mut(|i| i.consume_shortcut(&self.copy)) {
            return Some(Action::Copy);
        }
        if ctx.input_mut(|i| i.consume_shortcut(&self.paste)) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Context, Event, Key, Modifiers, RawInput};

    /// Simulates the OS triggering a "Copy" event (which egui intercepts).
    /// This tests that we are reading `ctx.input().events`.
    #[test]
    fn test_hotkey_copy_event() {
        let registry = HotkeyRegistry::default();
        let ctx = Context::default();
        let mut input = RawInput::default();

        input.events.push(Event::Copy);

        let mut action = None;
        let _ = ctx.run(input, |ctx| {
            action = registry.check(ctx);
        });

        assert_eq!(action, Some(Action::Copy));
    }

    /// Simulates the OS triggering a "Paste" event.
    #[test]
    fn test_hotkey_paste_event() {
        let registry = HotkeyRegistry::default();
        let ctx = Context::default();
        let mut input = RawInput::default();

        input.events.push(Event::Paste("some content".to_owned()));

        let mut action = None;
        let _ = ctx.run(input, |ctx| {
            action = registry.check(ctx);
        });

        assert_eq!(action, Some(Action::Paste));
    }

    /// Simulates a raw key press (Ctrl+J) that the OS does NOT treat as a standard command.
    /// This tests that `consume_shortcut` is still working for custom hotkeys.
    #[test]
    fn test_hotkey_duplicate_shortcut() {
        let registry = HotkeyRegistry::default();
        let ctx = Context::default();
        let mut input = RawInput::default();

        input.modifiers = Modifiers::COMMAND;
        input.events.push(Event::Key {
            key: Key::J,
            physical_key: Some(Key::J),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        });

        let mut action = None;
        let _ = ctx.run(input, |ctx| {
            action = registry.check(ctx);
        });

        assert_eq!(action, Some(Action::Duplicate));
    }

    /// Tests that the fallback raw key check for Copy still works if no Event::Copy is present.
    #[test]
    fn test_hotkey_copy_fallback_raw() {
        let registry = HotkeyRegistry::default();
        let ctx = Context::default();
        let mut input = RawInput::default();

        input.modifiers = Modifiers::COMMAND;
        input.events.push(Event::Key {
            key: Key::C,
            physical_key: Some(Key::C),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        });

        let mut action = None;
        let _ = ctx.run(input, |ctx| {
            action = registry.check(ctx);
        });

        assert_eq!(action, Some(Action::Copy));
    }

    #[test]
    fn test_hotkey_copy_with_persistent_modifiers() {
        let registry = HotkeyRegistry::default();
        let ctx = Context::default();
        let mut input = RawInput::default();

        input.modifiers = Modifiers::COMMAND;
        input.events.push(Event::Key {
            key: Key::C,
            physical_key: Some(Key::C),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        });

        let mut action = None;
        let _ = ctx.run(input, |ctx| {
            action = registry.check(ctx);
        });

        assert_eq!(action, Some(Action::Copy));
        ctx.input(|i| assert!(i.key_pressed(Key::C) == false || action.is_some()));
    }
}
