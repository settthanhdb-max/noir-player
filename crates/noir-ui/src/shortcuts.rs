//! Keyboard input: egui events → `ShortcutRegistry` lookups.

use egui::{Event, Key, Modifiers};
use noir_settings::schema::input::{KeyCombo, Modifiers as NoirModifiers};

/// Convert an egui key press into a registry combo. Returns `None` for keys
/// the registry cannot represent (pure modifier presses, unknown keys).
pub fn combo_from(modifiers: Modifiers, key: Key) -> Option<KeyCombo> {
    let name = key_name(key)?;
    Some(KeyCombo {
        modifiers: noir_modifiers(modifiers),
        key: name.to_string(),
    })
}

fn noir_modifiers(m: Modifiers) -> NoirModifiers {
    NoirModifiers {
        ctrl: m.ctrl,
        shift: m.shift,
        alt: m.alt,
    }
}

fn key_name(key: Key) -> Option<&'static str> {
    Some(match key {
        Key::ArrowLeft => "Left",
        Key::ArrowRight => "Right",
        Key::ArrowUp => "Up",
        Key::ArrowDown => "Down",
        Key::Escape => "Esc",
        Key::Tab => "Tab",
        Key::Backspace => "Backspace",
        Key::Enter => "Enter",
        Key::Space => "Space",
        Key::Insert => "Insert",
        Key::Delete => "Delete",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",
        Key::Minus => "-",
        Key::Plus => "+",
        Key::Equals => "=",
        Key::OpenBracket => "[",
        Key::CloseBracket => "]",
        Key::Backtick => "`",
        Key::Comma => ",",
        Key::Period => ".",
        Key::Slash => "/",
        Key::Backslash => "\\",
        Key::Semicolon => ";",
        Key::Quote => "'",
        Key::A => "a",
        Key::B => "b",
        Key::C => "c",
        Key::D => "d",
        Key::E => "e",
        Key::F => "f",
        Key::G => "g",
        Key::H => "h",
        Key::I => "i",
        Key::J => "j",
        Key::K => "k",
        Key::L => "l",
        Key::M => "m",
        Key::N => "n",
        Key::O => "o",
        Key::P => "p",
        Key::Q => "q",
        Key::R => "r",
        Key::S => "s",
        Key::T => "t",
        Key::U => "u",
        Key::V => "v",
        Key::W => "w",
        Key::X => "x",
        Key::Y => "y",
        Key::Z => "z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        _ => return None,
    })
}

/// Extract all freshly pressed combos from this frame's raw events.
pub fn pressed_combos(events: &[Event]) -> Vec<(NoirModifiers, String)> {
    let mut out = Vec::new();
    for event in events {
        if let Event::Key {
            key,
            pressed: true,
            repeat: false,
            modifiers,
            ..
        } = event
        {
            if let Some(combo) = combo_from(*modifiers, *key) {
                out.push((combo.modifiers, combo.key));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use noir_settings::schema::input::Modifiers as NoirModifiers;

    #[test]
    fn maps_default_player_keys() {
        let none = Modifiers::default();
        assert_eq!(combo_from(none, Key::Space).unwrap().key, "Space");
        assert_eq!(combo_from(none, Key::ArrowLeft).unwrap().key, "Left");
        assert_eq!(combo_from(none, Key::ArrowRight).unwrap().key, "Right");
        assert_eq!(combo_from(none, Key::F).unwrap().key, "f");
        assert_eq!(combo_from(none, Key::M).unwrap().key, "m");
        assert_eq!(combo_from(none, Key::O).unwrap().key, "o");
        assert_eq!(combo_from(none, Key::OpenBracket).unwrap().key, "[");
        assert_eq!(combo_from(none, Key::CloseBracket).unwrap().key, "]");
        assert_eq!(combo_from(none, Key::Num0).unwrap().key, "0");
        assert_eq!(combo_from(none, Key::Escape).unwrap().key, "Esc");
    }

    #[test]
    fn carries_modifiers() {
        let combo = combo_from(Modifiers::CTRL | Modifiers::SHIFT, Key::ArrowUp).unwrap();
        assert_eq!(combo.key, "Up");
        assert!(combo.modifiers.ctrl);
        assert!(combo.modifiers.shift);
        assert!(!combo.modifiers.alt);
    }

    #[test]
    fn modifiers_convert_losslessly() {
        let m = noir_modifiers(Modifiers::CTRL | Modifiers::ALT);
        assert!(m.ctrl && m.alt && !m.shift);
    }

    #[test]
    fn default_keymap_resolves_actions() {
        use noir_settings::schema::input::{ActionBinding, ShortcutRegistry};

        let bindings = noir_settings::schema::input::default_bindings();
        let registry = ShortcutRegistry::new(bindings);
        assert!(!registry.bindings().is_empty());
        assert!(registry.conflicts().is_empty());
        assert_eq!(
            registry.lookup(NoirModifiers::default(), "Space"),
            Some(ActionBinding::PlayPause)
        );
        assert_eq!(
            registry.lookup(NoirModifiers::default(), "]"),
            Some(ActionBinding::SpeedUp)
        );
    }
}
