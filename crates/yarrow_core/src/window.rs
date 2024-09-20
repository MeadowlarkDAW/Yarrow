use std::error::Error;

use crate::clipboard::Clipboard;
use crate::color::RGBA8;
use crate::math::{PhysicalPoint, PhysicalSize, Size};
use crate::renderer::RenderBackend;
use crate::{CursorIcon, ResourceContext};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The ID of a window
pub struct WindowID(pub u32);

impl WindowID {
    /// The ID of the main window
    pub const MAIN: Self = Self(0);
}

impl Default for WindowID {
    fn default() -> Self {
        Self::MAIN
    }
}

impl From<u32> for WindowID {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<WindowID> for u32 {
    fn from(value: WindowID) -> Self {
        value.0
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowConfig {
    pub title: String,
    pub size: Size,
    pub resizable: bool,
    //pub surface_config: DefaultSurfaceConfig,
    pub focus_on_creation: bool,
    pub scale_factor: ScaleFactorConfig,

    /// The clear color.
    pub clear_color: RGBA8,

    /// An estimate for how many elements are expected to be in this view in a
    /// typical use case. This is used to pre-allocate capacity to improve slightly
    /// improve load-up times.
    ///
    /// By default this is set to `0` (no capacity will be pre-allocated).
    pub preallocate_for_this_many_elements: u32,

    /// The duration between when an element is first hovered and when it receives the
    /// `ElementEvent::Pointer(PointerEvent::HoverTimeout)` event.
    ///
    /// By default this is set to 0.5 seconds.
    pub hover_timeout_secs: f32,

    pub scroll_wheel_timeout_secs: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("Yarrow Window"),
            size: Size::new(400.0, 250.0),
            resizable: true,
            //surface_config: DefaultSurfaceConfig::default(),
            focus_on_creation: true,
            scale_factor: ScaleFactorConfig::default(),
            clear_color: crate::color::BLACK,
            preallocate_for_this_many_elements: 0,
            hover_timeout_secs: 0.5,
            scroll_wheel_timeout_secs: 0.25,
        }
    }
}

pub trait WindowBackend<R: RenderBackend> {
    type OpenError: Error;

    fn set_pointer_position(
        &mut self,
        window_id: WindowID,
        position: PhysicalPoint,
    ) -> Result<(), ()>;

    fn unlock_pointer(&mut self, window_id: WindowID, prev_lock_state: PointerLockState);

    fn request_redraw(&mut self, window_id: WindowID);

    fn has_focus(&mut self, window_id: WindowID) -> bool;

    fn try_lock_pointer(&mut self, window_id: WindowID) -> PointerLockState;

    fn set_cursor_icon(&mut self, window_id: WindowID, icon: CursorIcon);

    fn resize(
        &mut self,
        window_id: WindowID,
        logical_size: Size,
        scale_factor: f32,
    ) -> Result<(), ()>;

    fn set_minimized(&mut self, window_id: WindowID, minimized: bool);

    fn set_maximized(&mut self, window_id: WindowID, maximized: bool);

    fn focus_window(&mut self, window_id: WindowID);

    fn set_window_title(&mut self, window_id: WindowID, title: String);

    fn create_window(
        &mut self,
        window_id: WindowID,
        config: &WindowConfig,
    ) -> Result<CreateWindowResult<R>, Self::OpenError>;

    fn close_window(&mut self, window_id: WindowID);
}

pub struct CreateWindowResult<R: RenderBackend> {
    pub renderer: R,
    pub clipboard: Result<Box<dyn Clipboard>, Box<dyn Error>>,
    pub physical_size: PhysicalSize,
    pub scale_factor: f32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScaleFactorConfig {
    #[default]
    System,
    Custom(f32),
}

impl ScaleFactorConfig {
    pub fn scale_factor(&self, system_scale_factor: f32) -> f32 {
        match self {
            Self::System => system_scale_factor,
            Self::Custom(s) => *s,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerLockState {
    NotLocked,
    LockedUsingOS,
    ManualLock,
}

impl PointerLockState {
    pub fn is_locked(&self) -> bool {
        *self != PointerLockState::NotLocked
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxBackendType {
    Wayland,
    X11,
}
