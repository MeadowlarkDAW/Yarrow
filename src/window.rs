use keyboard_types::{CompositionEvent, Modifiers};
use rootvg::math::{to_logical_size_i32, PhysicalPoint, Point};
use rootvg::surface::{DefaultSurface, DefaultSurfaceConfig, NewSurfaceError};
use rootvg::text::glyphon::FontSystem;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::action_queue::ActionSender;
use crate::clipboard::Clipboard;
use crate::event::{
    CanvasEvent, EventCaptureStatus, KeyboardEvent, PointerButton, PointerEvent, PointerType,
    WheelDeltaType,
};
use crate::math::{PhysicalSizeI32, ScaleFactor, Size};
use crate::CursorIcon;
use crate::{view::ViewConfig, View};

#[cfg(feature = "winit")]
mod winit_backend;
#[cfg(feature = "winit")]
pub use winit_backend::{run_blocking, OpenWindowError};

pub type WindowID = u32;

pub const MAIN_WINDOW: WindowID = 0;

// TODO: Get click intervals from OS.
const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(300);

#[derive(Clone, Copy)]
struct PointerBtnState {
    is_down: bool,
    prev_down_instant: Option<Instant>,
    click_count: usize,
}

impl Default for PointerBtnState {
    fn default() -> Self {
        Self {
            is_down: false,
            prev_down_instant: None,
            click_count: 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointerLockState {
    NotLocked,
    LockedUsingOS,
    ManualLock,
}

impl PointerLockState {
    pub fn is_locked(&self) -> bool {
        *self != PointerLockState::NotLocked
    }
}

pub(crate) struct WindowState<A: Clone + 'static> {
    view: View<A>,
    renderer: rootvg::Canvas,
    surface: Option<DefaultSurface>,
    logical_size: Size,
    physical_size: PhysicalSizeI32,
    scale_factor: ScaleFactor,
    scale_factor_recip: f32,
    system_scale_factor: ScaleFactor,
    scale_factor_config: ScaleFactorConfig,
    pub queued_pointer_position: Option<PhysicalPoint>,
    pub queued_pointer_delta: Option<(f64, f64)>,
    #[cfg(feature = "winit")]
    pub winit_window: Arc<winit::window::Window>,
    clipboard: Clipboard,

    pub prev_pointer_pos: Option<Point>,
    pointer_btn_states: [PointerBtnState; 5],
    pointer_lock_state: PointerLockState,

    modifiers: Modifiers,
    current_cursor_icon: CursorIcon,
}

impl<A: Clone + 'static> WindowState<A> {
    pub fn new(
        #[cfg(feature = "winit")] winit_window: &Arc<winit::window::Window>,
        logical_size: Size,
        physical_size: PhysicalSizeI32,
        system_scale_factor: ScaleFactor,
        scale_factor_config: ScaleFactorConfig,
        view_config: ViewConfig,
        surface_config: DefaultSurfaceConfig,
        action_sender: ActionSender<A>,
        id: WindowID,
    ) -> Result<Self, NewSurfaceError> {
        let scale_factor = scale_factor_config.scale_factor(system_scale_factor);

        let surface = DefaultSurface::new(
            physical_size,
            scale_factor,
            Arc::clone(winit_window),
            surface_config,
        )?;
        let renderer = rootvg::Canvas::new(
            &surface.device,
            &surface.queue,
            surface.format(),
            surface.canvas_config(),
        );

        let view = View::new(physical_size, scale_factor, view_config, action_sender, id);

        let clipboard = Clipboard::new(winit_window);

        Ok(Self {
            view,
            renderer,
            surface: Some(surface),
            logical_size,
            physical_size,
            scale_factor,
            scale_factor_recip: scale_factor.recip(),
            system_scale_factor,
            scale_factor_config,
            queued_pointer_position: None,
            queued_pointer_delta: None,
            winit_window: Arc::clone(winit_window),
            prev_pointer_pos: None,
            pointer_btn_states: [PointerBtnState::default(); 5],
            modifiers: Modifiers::empty(),
            current_cursor_icon: CursorIcon::Default,
            pointer_lock_state: PointerLockState::NotLocked,
            clipboard,
        })
    }

    pub fn set_size(&mut self, new_size: PhysicalSizeI32, new_system_scale_factor: ScaleFactor) {
        if self.physical_size == new_size && self.system_scale_factor == new_system_scale_factor {
            return;
        }

        let scale_factor = self
            .scale_factor_config
            .scale_factor(new_system_scale_factor);

        self.physical_size = new_size;
        self.logical_size = to_logical_size_i32(new_size, scale_factor);
        self.scale_factor = scale_factor;
        self.scale_factor_recip = scale_factor.recip();

        self.view.resize(new_size, scale_factor);
        self.surface
            .as_mut()
            .unwrap()
            .resize(new_size, scale_factor);
    }

    pub fn set_scale_factor_config(
        &mut self,
        config: ScaleFactorConfig,
    ) -> Option<PhysicalSizeI32> {
        if self.scale_factor_config == config {
            return None;
        }
        self.scale_factor_config = config;

        let scale_factor = self
            .scale_factor_config
            .scale_factor(self.system_scale_factor);

        if self.scale_factor == scale_factor {
            return None;
        }

        let logical_size = crate::math::to_logical_size_i32(self.physical_size, self.scale_factor);
        let requested_physical_size: PhysicalSizeI32 =
            crate::math::to_physical_size(logical_size, scale_factor)
                .round()
                .cast();

        self.scale_factor = scale_factor;
        self.scale_factor_recip = scale_factor.recip();

        self.view.resize(self.physical_size, scale_factor);
        self.surface
            .as_mut()
            .unwrap()
            .resize(self.physical_size, scale_factor);

        Some(requested_physical_size)
    }

    pub fn set_pointer_locked(&mut self, state: PointerLockState) {
        self.pointer_lock_state = state;
        self.view.on_pointer_locked(state.is_locked());
    }

    pub fn pointer_lock_state(&self) -> PointerLockState {
        self.pointer_lock_state
    }

    pub fn on_animation_tick(&mut self, dt: f64, font_system: &mut FontSystem) {
        self.view.handle_event(
            &CanvasEvent::Animation {
                delta_seconds: dt,
                pointer_position: self.prev_pointer_pos,
            },
            font_system,
            &mut self.clipboard,
        );
    }

    pub fn handle_window_unfocused(&mut self, font_system: &mut FontSystem) {
        self.view.handle_event(
            &CanvasEvent::WindowUnfocused,
            font_system,
            &mut self.clipboard,
        );
    }

    pub fn handle_window_focused(&mut self, font_system: &mut FontSystem) {
        self.view.handle_event(
            &CanvasEvent::WindowFocused,
            font_system,
            &mut self.clipboard,
        );
    }

    pub fn handle_window_hidden(&mut self, font_system: &mut FontSystem) {
        self.handle_window_unfocused(font_system);
        self.view
            .handle_event(&CanvasEvent::WindowHidden, font_system, &mut self.clipboard);
    }

    pub fn handle_window_shown(&mut self, font_system: &mut FontSystem) {
        self.view
            .handle_event(&CanvasEvent::WindowShown, font_system, &mut self.clipboard);
    }

    pub fn set_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    pub fn handle_keyboard_event(
        &mut self,
        event: KeyboardEvent,
        font_system: &mut FontSystem,
    ) -> EventCaptureStatus {
        self.view.handle_event(
            &CanvasEvent::Keyboard(event),
            font_system,
            &mut self.clipboard,
        )
    }

    pub fn handle_text_composition_event(
        &mut self,
        event: CompositionEvent,
        font_system: &mut FontSystem,
    ) -> EventCaptureStatus {
        // Don't send the event if the input might be a keyboard shortcut.
        if self.modifiers.intersects(
            Modifiers::ALT
                | Modifiers::CONTROL
                | Modifiers::META
                | Modifiers::HYPER
                | Modifiers::SUPER,
        ) {
            return EventCaptureStatus::NotCaptured;
        }

        self.view.handle_event(
            &CanvasEvent::TextComposition(event),
            font_system,
            &mut self.clipboard,
        )
    }

    pub fn handle_pointer_left(&mut self, font_system: &mut FontSystem) {
        self.view.handle_event(
            &CanvasEvent::Pointer(PointerEvent::PointerLeft),
            font_system,
            &mut self.clipboard,
        );
    }

    pub fn handle_pointer_moved(&mut self, new_pos: PhysicalPoint, font_system: &mut FontSystem) {
        let new_pos = crate::math::to_logical_point_from_recip(new_pos, self.scale_factor_recip);

        let delta = if self.pointer_lock_state == PointerLockState::LockedUsingOS {
            // The delta will already be sent in `handle_locked_pointer_delta()`, so
            // avoid sending a duplicate.
            None
        } else if let Some(prev_pos) = self.prev_pointer_pos {
            Some(new_pos - prev_pos.to_vector())
        } else {
            None
        };
        self.prev_pointer_pos = Some(new_pos);

        self.view.handle_event(
            &CanvasEvent::Pointer(PointerEvent::Moved {
                position: new_pos,
                delta,
                is_locked: false,
                pointer_type: PointerType::default(),
                modifiers: self.modifiers,
                just_entered: false,
            }),
            font_system,
            &mut self.clipboard,
        );
    }

    pub fn handle_locked_pointer_delta(&mut self, delta: Point, font_system: &mut FontSystem) {
        self.view.handle_event(
            &CanvasEvent::Pointer(PointerEvent::Moved {
                position: self.prev_pointer_pos.unwrap_or_default(),
                delta: Some(delta),
                is_locked: false,
                pointer_type: PointerType::default(),
                modifiers: self.modifiers,
                just_entered: false,
            }),
            font_system,
            &mut self.clipboard,
        );
    }

    pub fn handle_mouse_button(
        &mut self,
        button: PointerButton,
        is_down: bool,
        font_system: &mut FontSystem,
    ) {
        enum State {
            Unchanged,
            JustPressed,
            JustUnpressed,
        }

        let (state, click_count) = {
            let btn_state = &mut self.pointer_btn_states[button as usize];

            let s = if !btn_state.is_down && is_down {
                if let Some(prev_down_instant) = btn_state.prev_down_instant.take() {
                    if prev_down_instant.elapsed() < DOUBLE_CLICK_INTERVAL {
                        btn_state.click_count += 1;
                    } else {
                        btn_state.click_count = 1;
                    }
                }

                btn_state.prev_down_instant = Some(Instant::now());

                State::JustPressed
            } else if btn_state.is_down && !is_down {
                State::JustUnpressed
            } else {
                State::Unchanged
            };

            btn_state.is_down = is_down;

            (s, btn_state.click_count)
        };

        let position = self.prev_pointer_pos.unwrap_or(Point::zero());

        match state {
            State::JustPressed => {
                self.view.handle_event(
                    &CanvasEvent::Pointer(PointerEvent::ButtonJustPressed {
                        position,
                        button,
                        pointer_type: PointerType::default(),
                        click_count,
                        modifiers: self.modifiers,
                    }),
                    font_system,
                    &mut self.clipboard,
                );
            }
            State::JustUnpressed => {
                self.view.handle_event(
                    &CanvasEvent::Pointer(PointerEvent::ButtonJustReleased {
                        position,
                        button,
                        pointer_type: PointerType::default(),
                        click_count,
                        modifiers: self.modifiers,
                    }),
                    font_system,
                    &mut self.clipboard,
                );
            }
            _ => {}
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta_type: WheelDeltaType, font_system: &mut FontSystem) {
        let position = self.prev_pointer_pos.unwrap_or(Point::zero());

        self.view.handle_event(
            &CanvasEvent::Pointer(PointerEvent::ScrollWheel {
                position,
                delta_type,
                pointer_type: PointerType::default(),
                modifiers: self.modifiers,
            }),
            font_system,
            &mut self.clipboard,
        );
    }

    pub fn render<P: FnOnce()>(
        &mut self,
        pre_present_notify: P,
        font_system: &mut FontSystem,
    ) -> Result<(), wgpu::SurfaceError> {
        let surface = self.surface.as_ref().unwrap();

        self.view.render(
            &surface.surface,
            &surface.device,
            &surface.queue,
            &mut self.renderer,
            pre_present_notify,
            font_system,
        )
    }

    pub fn logical_size(&self) -> Size {
        self.logical_size
    }

    pub fn context<'a>(&'a mut self, font_system: &'a mut FontSystem) -> WindowContext<'a, A> {
        WindowContext {
            view: &mut self.view,
            font_system,
            clipboard: &mut self.clipboard,
            logical_size: self.logical_size,
            physical_size: self.physical_size,
            scale_factor: self.scale_factor,
            system_scale_factor: self.system_scale_factor,
            scale_factor_config: self.scale_factor_config,
        }
    }

    pub fn new_cursor_icon(&mut self) -> Option<CursorIcon> {
        if self.current_cursor_icon != self.view.cursor_icon() {
            self.current_cursor_icon = self.view.cursor_icon();
            Some(self.current_cursor_icon)
        } else {
            None
        }
    }

    pub fn new_pointer_lock_request(&mut self) -> Option<bool> {
        self.view.pointer_lock_request()
    }
}

impl<A: Clone + 'static> Drop for WindowState<A> {
    fn drop(&mut self) {
        // For some reason if the surface isn't dropped before the other
        // structs it causes a segfault. This is probably a bug in wgpu
        // or winit.
        self.surface = None;
    }
}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: Size,
    pub resizable: bool,
    pub view_config: ViewConfig,
    pub surface_config: DefaultSurfaceConfig,
    pub focus_on_creation: bool,
    pub scale_factor: ScaleFactorConfig,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("Window"),
            size: Size::new(400.0, 250.0),
            resizable: true,
            view_config: ViewConfig::default(),
            surface_config: DefaultSurfaceConfig::default(),
            focus_on_creation: true,
            scale_factor: ScaleFactorConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowCloseRequest {
    DoNotCloseYet,
    CloseImmediately,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScaleFactorConfig {
    #[default]
    System,
    Custom(ScaleFactor),
}

impl ScaleFactorConfig {
    pub fn scale_factor(&self, system_scale_factor: ScaleFactor) -> ScaleFactor {
        match self {
            Self::System => system_scale_factor,
            Self::Custom(s) => *s,
        }
    }
}

pub struct WindowContext<'a, A: Clone + 'static> {
    pub view: &'a mut View<A>,
    pub font_system: &'a mut FontSystem,
    pub clipboard: &'a mut Clipboard,
    logical_size: Size,
    physical_size: PhysicalSizeI32,
    scale_factor: ScaleFactor,
    scale_factor_config: ScaleFactorConfig,
    system_scale_factor: ScaleFactor,
}

impl<'a, A: Clone + 'static> WindowContext<'a, A> {
    pub fn logical_size(&self) -> Size {
        self.logical_size
    }

    pub fn physical_size(&self) -> PhysicalSizeI32 {
        self.physical_size
    }

    pub fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    pub fn system_scale_factor(&self) -> ScaleFactor {
        self.system_scale_factor
    }

    pub fn scale_factor_config(&self) -> ScaleFactorConfig {
        self.scale_factor_config
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxBackendType {
    Wayland,
    X11,
}
