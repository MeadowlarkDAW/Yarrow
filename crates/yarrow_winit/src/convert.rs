use winit::event::ElementState;
use winit::{
    keyboard::{KeyCode, KeyLocation, NativeKeyCode as WinitNativeKeyCode, PhysicalKey},
    window::CursorIcon as WinitCursorIcon,
};
use yarrow_core::event::keyboard::{Code, KeyState, KeyboardEvent, Location, Modifiers, NativeKey};
use yarrow_core::CursorIcon;

/*
pub fn convert_cursor_icon_from_winit(icon: WinitCursorIcon) -> CursorIcon {
    match icon {
        WinitCursorIcon::Default => CursorIcon::Default,
        WinitCursorIcon::ContextMenu => CursorIcon::ContextMenu,
        WinitCursorIcon::Help => CursorIcon::Help,
        WinitCursorIcon::Pointer => CursorIcon::Pointer,
        WinitCursorIcon::Progress => CursorIcon::Progress,
        WinitCursorIcon::Wait => CursorIcon::Wait,
        WinitCursorIcon::Cell => CursorIcon::Cell,
        WinitCursorIcon::Crosshair => CursorIcon::Crosshair,
        WinitCursorIcon::Text => CursorIcon::Text,
        WinitCursorIcon::VerticalText => CursorIcon::VerticalText,
        WinitCursorIcon::Alias => CursorIcon::Alias,
        WinitCursorIcon::Copy => CursorIcon::Copy,
        WinitCursorIcon::Move => CursorIcon::Move,
        WinitCursorIcon::NoDrop => CursorIcon::NoDrop,
        WinitCursorIcon::NotAllowed => CursorIcon::NotAllowed,
        WinitCursorIcon::Grab => CursorIcon::Grab,
        WinitCursorIcon::Grabbing => CursorIcon::Grabbing,
        WinitCursorIcon::EResize => CursorIcon::EResize,
        WinitCursorIcon::NResize => CursorIcon::NResize,
        WinitCursorIcon::NeResize => CursorIcon::NeResize,
        WinitCursorIcon::NwResize => CursorIcon::NwResize,
        WinitCursorIcon::SResize => CursorIcon::SResize,
        WinitCursorIcon::SeResize => CursorIcon::SeResize,
        WinitCursorIcon::SwResize => CursorIcon::SwResize,
        WinitCursorIcon::WResize => CursorIcon::WResize,
        WinitCursorIcon::EwResize => CursorIcon::EwResize,
        WinitCursorIcon::NsResize => CursorIcon::NsResize,
        WinitCursorIcon::NeswResize => CursorIcon::NeswResize,
        WinitCursorIcon::NwseResize => CursorIcon::NwseResize,
        WinitCursorIcon::ColResize => CursorIcon::ColResize,
        WinitCursorIcon::RowResize => CursorIcon::RowResize,
        WinitCursorIcon::AllScroll => CursorIcon::AllScroll,
        WinitCursorIcon::ZoomIn => CursorIcon::ZoomIn,
        WinitCursorIcon::ZoomOut => CursorIcon::ZoomOut,
        _ => CursorIcon::Default,
    }
}
*/

pub fn convert_cursor_icon_to_winit(icon: CursorIcon) -> WinitCursorIcon {
    match icon {
        CursorIcon::Default => WinitCursorIcon::Default,
        CursorIcon::ContextMenu => WinitCursorIcon::ContextMenu,
        CursorIcon::Help => WinitCursorIcon::Help,
        CursorIcon::Pointer => WinitCursorIcon::Pointer,
        CursorIcon::Progress => WinitCursorIcon::Progress,
        CursorIcon::Wait => WinitCursorIcon::Wait,
        CursorIcon::Cell => WinitCursorIcon::Cell,
        CursorIcon::Crosshair => WinitCursorIcon::Crosshair,
        CursorIcon::Text => WinitCursorIcon::Text,
        CursorIcon::VerticalText => WinitCursorIcon::VerticalText,
        CursorIcon::Alias => WinitCursorIcon::Alias,
        CursorIcon::Copy => WinitCursorIcon::Copy,
        CursorIcon::Move => WinitCursorIcon::Move,
        CursorIcon::NoDrop => WinitCursorIcon::NoDrop,
        CursorIcon::NotAllowed => WinitCursorIcon::NotAllowed,
        CursorIcon::Grab => WinitCursorIcon::Grab,
        CursorIcon::Grabbing => WinitCursorIcon::Grabbing,
        CursorIcon::EResize => WinitCursorIcon::EResize,
        CursorIcon::NResize => WinitCursorIcon::NResize,
        CursorIcon::NeResize => WinitCursorIcon::NeResize,
        CursorIcon::NwResize => WinitCursorIcon::NwResize,
        CursorIcon::SResize => WinitCursorIcon::SResize,
        CursorIcon::SeResize => WinitCursorIcon::SeResize,
        CursorIcon::SwResize => WinitCursorIcon::SwResize,
        CursorIcon::WResize => WinitCursorIcon::WResize,
        CursorIcon::EwResize => WinitCursorIcon::EwResize,
        CursorIcon::NsResize => WinitCursorIcon::NsResize,
        CursorIcon::NeswResize => WinitCursorIcon::NeswResize,
        CursorIcon::NwseResize => WinitCursorIcon::NwseResize,
        CursorIcon::ColResize => WinitCursorIcon::ColResize,
        CursorIcon::RowResize => WinitCursorIcon::RowResize,
        CursorIcon::AllScroll => WinitCursorIcon::AllScroll,
        CursorIcon::ZoomIn => WinitCursorIcon::ZoomIn,
        CursorIcon::ZoomOut => WinitCursorIcon::ZoomOut,
        _ => WinitCursorIcon::Default,
    }
}

pub fn convert_modifiers(winit_modifiers: winit::event::Modifiers) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    if winit_modifiers.state().shift_key() {
        modifiers.insert(Modifiers::SHIFT);
    }
    if winit_modifiers.state().control_key() {
        modifiers.insert(Modifiers::CONTROL);
    }
    if winit_modifiers.state().alt_key() {
        modifiers.insert(Modifiers::ALT);
    }
    if winit_modifiers.state().super_key() {
        modifiers.insert(Modifiers::SUPER);
    }
    modifiers
}

pub fn convert_keyboard_event(
    event: &winit::event::KeyEvent,
    modifiers: Modifiers,
) -> KeyboardEvent {
    let (code, native_code) = convert_physical_key(event.physical_key, event.location);

    let state = match event.state {
        ElementState::Pressed => KeyState::Down,
        ElementState::Released => KeyState::Up,
    };

    let location = match event.location {
        KeyLocation::Left => Location::Left,
        KeyLocation::Right => Location::Right,
        KeyLocation::Numpad => Location::Numpad,
        KeyLocation::Standard => Location::Standard,
    };

    KeyboardEvent {
        state,
        code,
        native_code,
        location,
        modifiers,
        repeat: event.repeat,
        is_composing: event.text.is_some(),
    }
}

fn convert_physical_key(key: PhysicalKey, location: KeyLocation) -> (Code, NativeKey) {
    match key {
        PhysicalKey::Code(code) => {
            let code = match code {
                KeyCode::Backquote => Code::Backquote,
                KeyCode::Backslash => Code::Backslash,
                KeyCode::BracketLeft => Code::BracketLeft,
                KeyCode::BracketRight => Code::BracketRight,
                KeyCode::Comma => Code::Comma,
                KeyCode::Digit0 => Code::Digit0,
                KeyCode::Digit1 => Code::Digit1,
                KeyCode::Digit2 => Code::Digit2,
                KeyCode::Digit3 => Code::Digit3,
                KeyCode::Digit4 => Code::Digit4,
                KeyCode::Digit5 => Code::Digit5,
                KeyCode::Digit6 => Code::Digit6,
                KeyCode::Digit7 => Code::Digit7,
                KeyCode::Digit8 => Code::Digit8,
                KeyCode::Equal => Code::Equal,
                KeyCode::IntlBackslash => Code::IntlBackslash,
                KeyCode::IntlRo => Code::IntlRo,
                KeyCode::IntlYen => Code::IntlYen,
                KeyCode::KeyA => Code::KeyA,
                KeyCode::KeyB => Code::KeyB,
                KeyCode::KeyC => Code::KeyC,
                KeyCode::KeyD => Code::KeyD,
                KeyCode::KeyE => Code::KeyE,
                KeyCode::KeyF => Code::KeyF,
                KeyCode::KeyG => Code::KeyG,
                KeyCode::KeyH => Code::KeyH,
                KeyCode::KeyI => Code::KeyI,
                KeyCode::KeyJ => Code::KeyJ,
                KeyCode::KeyK => Code::KeyK,
                KeyCode::KeyL => Code::KeyL,
                KeyCode::KeyM => Code::KeyM,
                KeyCode::KeyN => Code::KeyN,
                KeyCode::KeyO => Code::KeyO,
                KeyCode::KeyP => Code::KeyP,
                KeyCode::KeyQ => Code::KeyQ,
                KeyCode::KeyR => Code::KeyR,
                KeyCode::KeyS => Code::KeyS,
                KeyCode::KeyT => Code::KeyT,
                KeyCode::KeyU => Code::KeyU,
                KeyCode::KeyV => Code::KeyV,
                KeyCode::KeyW => Code::KeyW,
                KeyCode::KeyX => Code::KeyX,
                KeyCode::KeyY => Code::KeyY,
                KeyCode::KeyZ => Code::KeyZ,
                KeyCode::Minus => Code::Minus,
                KeyCode::Period => Code::Period,
                KeyCode::Quote => Code::Quote,
                KeyCode::Semicolon => Code::Semicolon,
                KeyCode::Slash => Code::Slash,
                KeyCode::AltLeft => Code::AltLeft,
                KeyCode::AltRight => Code::AltRight,
                KeyCode::Backspace => Code::Backspace,
                KeyCode::CapsLock => Code::CapsLock,
                KeyCode::ContextMenu => Code::ContextMenu,
                KeyCode::ControlLeft => Code::ControlLeft,
                KeyCode::ControlRight => Code::ControlRight,
                KeyCode::Enter => Code::Enter,
                KeyCode::SuperLeft => Code::Unidentified,
                KeyCode::SuperRight => Code::Unidentified,
                KeyCode::ShiftLeft => Code::ShiftLeft,
                KeyCode::ShiftRight => Code::ShiftRight,
                KeyCode::Space => Code::Space,
                KeyCode::Tab => Code::Tab,
                KeyCode::Convert => Code::Convert,
                KeyCode::KanaMode => Code::KanaMode,
                KeyCode::Lang1 => Code::Lang1,
                KeyCode::Lang2 => Code::Lang2,
                KeyCode::Lang3 => Code::Lang3,
                KeyCode::Lang4 => Code::Lang4,
                KeyCode::Lang5 => Code::Lang5,
                KeyCode::NonConvert => Code::NonConvert,
                KeyCode::Delete => Code::Delete,
                KeyCode::End => Code::End,
                KeyCode::Help => Code::Help,
                KeyCode::Home => Code::Home,
                KeyCode::Insert => Code::Insert,
                KeyCode::PageDown => Code::PageDown,
                KeyCode::PageUp => Code::PageUp,
                KeyCode::ArrowDown => Code::ArrowDown,
                KeyCode::ArrowLeft => Code::ArrowLeft,
                KeyCode::ArrowRight => Code::ArrowRight,
                KeyCode::ArrowUp => Code::ArrowUp,
                KeyCode::NumLock => Code::NumLock,
                KeyCode::Numpad0 => Code::Numpad0,
                KeyCode::Numpad1 => Code::Numpad1,
                KeyCode::Numpad2 => Code::Numpad2,
                KeyCode::Numpad3 => Code::Numpad3,
                KeyCode::Numpad4 => Code::Numpad4,
                KeyCode::Numpad5 => Code::Numpad5,
                KeyCode::Numpad6 => Code::Numpad6,
                KeyCode::Numpad7 => Code::Numpad7,
                KeyCode::Numpad8 => Code::Numpad8,
                KeyCode::Numpad9 => Code::Numpad9,
                KeyCode::NumpadAdd => Code::NumpadAdd,
                KeyCode::NumpadBackspace => Code::NumpadBackspace,
                KeyCode::NumpadClear => Code::NumpadClear,
                KeyCode::NumpadClearEntry => Code::NumpadClearEntry,
                KeyCode::NumpadComma => Code::NumpadComma,
                KeyCode::NumpadDecimal => Code::NumpadDecimal,
                KeyCode::NumpadDivide => Code::NumpadDivide,
                KeyCode::NumpadEnter => Code::NumpadEnter,
                KeyCode::NumpadEqual => Code::NumpadEqual,
                KeyCode::NumpadHash => Code::NumpadHash,
                KeyCode::NumpadMemoryAdd => Code::NumpadMemoryAdd,
                KeyCode::NumpadMemoryClear => Code::NumpadMemoryClear,
                KeyCode::NumpadMemoryRecall => Code::NumpadMemoryRecall,
                KeyCode::NumpadMemoryStore => Code::NumpadMemoryStore,
                KeyCode::NumpadMemorySubtract => Code::NumpadMemorySubtract,
                KeyCode::NumpadMultiply => Code::NumpadMultiply,
                KeyCode::NumpadParenLeft => Code::NumpadParenLeft,
                KeyCode::NumpadParenRight => Code::NumpadParenRight,
                KeyCode::NumpadStar => Code::NumpadStar,
                KeyCode::NumpadSubtract => Code::NumpadSubtract,
                KeyCode::Escape => Code::Escape,
                KeyCode::Fn => Code::Fn,
                KeyCode::FnLock => Code::FnLock,
                KeyCode::PrintScreen => Code::PrintScreen,
                KeyCode::ScrollLock => Code::ScrollLock,
                KeyCode::Pause => Code::Pause,
                KeyCode::BrowserBack => Code::BrowserBack,
                KeyCode::BrowserFavorites => Code::BrowserFavorites,
                KeyCode::BrowserForward => Code::BrowserForward,
                KeyCode::BrowserHome => Code::BrowserHome,
                KeyCode::BrowserRefresh => Code::BrowserRefresh,
                KeyCode::BrowserSearch => Code::BrowserSearch,
                KeyCode::BrowserStop => Code::BrowserStop,
                KeyCode::Eject => Code::Eject,
                KeyCode::LaunchApp1 => Code::LaunchApp1,
                KeyCode::LaunchApp2 => Code::LaunchApp2,
                KeyCode::LaunchMail => Code::LaunchMail,
                KeyCode::MediaPlayPause => Code::MediaPlayPause,
                KeyCode::MediaSelect => Code::MediaSelect,
                KeyCode::MediaStop => Code::MediaStop,
                KeyCode::MediaTrackNext => Code::MediaTrackNext,
                KeyCode::MediaTrackPrevious => Code::MediaTrackPrevious,
                KeyCode::Power => Code::Power,
                KeyCode::Sleep => Code::Sleep,
                KeyCode::AudioVolumeDown => Code::AudioVolumeDown,
                KeyCode::AudioVolumeMute => Code::AudioVolumeMute,
                KeyCode::AudioVolumeUp => Code::AudioVolumeUp,
                KeyCode::WakeUp => Code::WakeUp,
                KeyCode::Meta => match location {
                    KeyLocation::Right => Code::MetaRight,
                    _ => Code::MetaLeft,
                },
                KeyCode::Hyper => Code::Hyper,
                KeyCode::Turbo => Code::Turbo,
                KeyCode::Abort => Code::Abort,
                KeyCode::Resume => Code::Resume,
                KeyCode::Suspend => Code::Suspend,
                KeyCode::Again => Code::Again,
                KeyCode::Copy => Code::Copy,
                KeyCode::Cut => Code::Cut,
                KeyCode::Find => Code::Find,
                KeyCode::Open => Code::Open,
                KeyCode::Paste => Code::Paste,
                KeyCode::Props => Code::Props,
                KeyCode::Select => Code::Select,
                KeyCode::Undo => Code::Undo,
                KeyCode::Hiragana => Code::Hiragana,
                KeyCode::Katakana => Code::Katakana,
                KeyCode::F1 => Code::F1,
                KeyCode::F2 => Code::F2,
                KeyCode::F3 => Code::F3,
                KeyCode::F4 => Code::F4,
                KeyCode::F5 => Code::F5,
                KeyCode::F6 => Code::F6,
                KeyCode::F7 => Code::F7,
                KeyCode::F8 => Code::F8,
                KeyCode::F9 => Code::F9,
                KeyCode::F10 => Code::F10,
                KeyCode::F11 => Code::F11,
                KeyCode::F12 => Code::F12,
                KeyCode::F13 => Code::F13,
                KeyCode::F14 => Code::F14,
                KeyCode::F15 => Code::F15,
                KeyCode::F16 => Code::F16,
                KeyCode::F17 => Code::F17,
                KeyCode::F18 => Code::F18,
                KeyCode::F19 => Code::F19,
                KeyCode::F20 => Code::F20,
                KeyCode::F21 => Code::F21,
                KeyCode::F22 => Code::F22,
                KeyCode::F23 => Code::F23,
                KeyCode::F24 => Code::F24,
                //KeyCode::F25 => Code::F25,
                //KeyCode::F26 => Code::F26,
                //KeyCode::F27 => Code::F27,
                //KeyCode::F28 => Code::F28,
                //KeyCode::F29 => Code::F29,
                //KeyCode::F30 => Code::F30,
                //KeyCode::F31 => Code::F31,
                //KeyCode::F32 => Code::F32,
                //KeyCode::F33 => Code::F33,
                //KeyCode::F34 => Code::F34,
                //KeyCode::F35 => Code::F35,
                _ => Code::Unidentified,
            };

            (code, NativeKey::Unidentified)
        }
        PhysicalKey::Unidentified(code) => {
            let native_code = match code {
                WinitNativeKeyCode::Android(c) => NativeKey::Android(c),
                WinitNativeKeyCode::MacOS(c) => NativeKey::MacOS(c),
                WinitNativeKeyCode::Windows(c) => NativeKey::Windows(c),
                WinitNativeKeyCode::Xkb(c) => NativeKey::Xkb(c),
                _ => NativeKey::Unidentified,
            };

            (Code::Unidentified, native_code)
        }
    }
}
