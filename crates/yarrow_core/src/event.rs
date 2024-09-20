use std::error::Error;

use crate::math::{Point, Vector};

pub mod keyboard;
pub use keyboard::Modifiers;

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
    Keyboard(keyboard::KeyboardEvent),
    TextComposition(keyboard::CompositionEvent),
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
    Points(Vector),
    Lines(Vector),
    Pages(Vector),
}

impl WheelDeltaType {
    pub fn points(&self, points_per_line: f32, points_per_page: f32) -> Vector {
        match self {
            Self::Points(delta) => *delta,
            Self::Lines(delta) => Vector::new(delta.x * points_per_line, delta.y * points_per_line),
            Self::Pages(delta) => Vector::new(delta.x * points_per_page, delta.y * points_per_page),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerEvent {
    Moved {
        position: Point,
        delta: Option<Vector>,
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
        position: Point,
        button: PointerButton,
        pointer_type: PointerType,
        click_count: usize,
        modifiers: Modifiers,
    },
    ButtonJustReleased {
        position: Point,
        button: PointerButton,
        pointer_type: PointerType,
        click_count: usize,
        modifiers: Modifiers,
    },
    ScrollWheel {
        position: Point,
        delta_type: WheelDeltaType,
        pointer_type: PointerType,
        modifiers: Modifiers,
    },
    HoverTimeout {
        position: Point,
    },
    ScrollWheelTimeout,
    PointerLeft,
}

impl PointerEvent {
    pub fn position(&self) -> Point {
        match self {
            Self::Moved { position, .. } => *position,
            Self::ButtonJustPressed { position, .. } => *position,
            Self::ButtonJustReleased { position, .. } => *position,
            Self::ScrollWheel { position, .. } => *position,
            Self::HoverTimeout { position } => *position,
            Self::ScrollWheelTimeout => Point::default(),
            Self::PointerLeft => Point::default(),
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
