use keyboard_types::{Code, Key};

use crate::event::{KeyboardEvent, NativeKey};

pub fn convert_keyboard_event(event: &keyboard_types::KeyboardEvent) -> KeyboardEvent {
    KeyboardEvent {
        state: event.state,
        code: event.code,
        //TODO:
        native_code: NativeKey::Unidentified,
        location: event.location,
        modifiers: event.modifiers,
        repeat: event.repeat,
        is_composing: event.is_composing,
    }
}

pub fn key_to_composition(key: Key, code: Code) -> Option<String> {
    match key {
        Key::Character(char) => Some(char),
        _ => match code {
            Code::Numpad0 => Some("0".to_string()),
            Code::Numpad1 => Some("1".to_string()),
            Code::Numpad2 => Some("2".to_string()),
            Code::Numpad3 => Some("3".to_string()),
            Code::Numpad4 => Some("4".to_string()),
            Code::Numpad5 => Some("5".to_string()),
            Code::Numpad6 => Some("6".to_string()),
            Code::Numpad7 => Some("7".to_string()),
            Code::Numpad8 => Some("8".to_string()),
            Code::Numpad9 => Some("9".to_string()),
            Code::NumpadDecimal => Some(".".to_string()),
            _ => None,
        },
    }
}
