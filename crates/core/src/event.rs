use std::error::Error;

pub use keyboard_types::{Code, CompositionEvent, KeyState, Location, Modifiers};

use crate::math::{Pos2, Vec2};

#[derive(Debug)]
pub enum AppWindowEvent {
    WindowOpened,
    WindowClosed,
    WindowResized,
    WindowShown,
    WindowHidden,
    WindowFocused,
    WindowUnfocused,
    OpenWindowFailed(Box<dyn Error>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementEvent {
    Animation { delta_seconds: f64 },
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    TextComposition(CompositionEvent),
    PositionChanged,
    SizeChanged,
    ZIndexChanged,
    StyleChanged,
    Hidden,
    Shown,
    KeyboardFocus(bool),
    PointerFocus(bool),
    ClickedOff,
    Init,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyboardEvent {
    /// Whether the key is pressed or released.
    pub state: KeyState,
    /// Physical key code.
    pub code: Code,
    /// The native key code if the physical code could not be determined.
    pub native_code: NativeKey,
    /// Location for keys with multiple instances on common keyboards.
    pub location: Location,
    /// Flags for pressed modifier keys.
    pub modifiers: Modifiers,
    /// True if the key is currently auto-repeated.
    pub repeat: bool,
    /// Events with this flag should be ignored in a text editor
    /// and instead composition events should be used.
    pub is_composing: bool,
}

/// Contains the platform-native logical key identifier
///
/// Exactly what that means differs from platform to platform, but the values are to some degree
/// tied to the currently active keyboard layout. The same key on the same keyboard may also report
/// different values on different platforms, which is one of the reasons this is a per-platform
/// enum.
///
/// This enum is primarily used to store raw keysym when Winit doesn't map a given native logical
/// key identifier to a meaningful [`Key`] variant. This lets you use [`Key`], and let the user
/// define keybinds which work in the presence of identifiers we haven't mapped for you yet.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NativeKey {
    #[default]
    Unidentified,
    /// An Android "keycode", which is similar to a "virtual-key code" on Windows.
    Android(u32),
    /// A macOS "scancode". There does not appear to be any direct analogue to either keysyms or
    /// "virtual-key" codes in macOS, so we report the scancode instead.
    MacOS(u16),
    /// A Windows "virtual-key code".
    Windows(u16),
    /// An XKB "keysym".
    Xkb(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButtonState {
    StayedUnpressed,
    StayedPressed,
    JustPressed,
    JustUnpressed,
}

impl PointerButtonState {
    pub fn just_pressed(&self) -> bool {
        *self == PointerButtonState::JustPressed
    }

    pub fn just_unpressed(&self) -> bool {
        *self == PointerButtonState::JustUnpressed
    }

    pub fn is_down(&self) -> bool {
        *self == PointerButtonState::JustPressed || *self == PointerButtonState::StayedPressed
    }
}

impl Default for PointerButtonState {
    fn default() -> Self {
        PointerButtonState::StayedUnpressed
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
    #[default]
    Unknown,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    #[default]
    Primary = 0,
    Secondary,
    Auxiliary,
    Fourth,
    Fifth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WheelDeltaType {
    Points(Vec2),
    Lines(Vec2),
    Pages(Vec2),
}

impl WheelDeltaType {
    pub fn points(&self, points_per_line: f32, points_per_page: f32) -> Vec2 {
        match self {
            Self::Points(delta) => *delta,
            Self::Lines(delta) => Vec2::new(delta.x * points_per_line, delta.y * points_per_line),
            Self::Pages(delta) => Vec2::new(delta.x * points_per_page, delta.y * points_per_page),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent {
    Moved {
        position: Pos2,
        delta: Option<Vec2>,
        /// Whether or not the backend has locked the pointer in place.
        ///
        /// This will only be `true` if all the following conditions are true:
        /// * This element has requested to steal focus and lock the pointer.
        /// * This element has exclusive focus.
        /// * The backend supports locking the pointer.
        ///
        /// Note if this is `false`, then you will generally want to use
        /// `position` instead of `delta` for better accuracy.
        is_locked: bool,
        pointer_type: PointerType,
        modifiers: Modifiers,
        just_entered: bool,
    },
    ButtonJustPressed {
        position: Pos2,
        button: PointerButton,
        pointer_type: PointerType,
        click_count: usize,
        modifiers: Modifiers,
    },
    ButtonJustReleased {
        position: Pos2,
        button: PointerButton,
        pointer_type: PointerType,
        click_count: usize,
        modifiers: Modifiers,
    },
    ScrollWheel {
        position: Pos2,
        delta_type: WheelDeltaType,
        pointer_type: PointerType,
        modifiers: Modifiers,
    },
    HoverTimeout {
        position: Pos2,
    },
    ScrollWheelTimeout,
    PointerLeft,
}

impl PointerEvent {
    pub fn position(&self) -> Pos2 {
        match self {
            Self::Moved { position, .. } => *position,
            Self::ButtonJustPressed { position, .. } => *position,
            Self::ButtonJustReleased { position, .. } => *position,
            Self::ScrollWheel { position, .. } => *position,
            Self::HoverTimeout { position } => *position,
            Self::ScrollWheelTimeout => Pos2::default(),
            Self::PointerLeft => Pos2::default(),
        }
    }
}

/// Whether or not the event was captured by this element.
///
/// Note, this is only relevant for `Event::Pointer`, `Event::Keyboard`,
/// and `Event::TextComposition`.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCaptureStatus {
    #[default]
    NotCaptured,
    Captured,
}
