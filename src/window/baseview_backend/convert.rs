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
