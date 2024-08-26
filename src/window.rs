use keyboard_types::{CompositionEvent, Modifiers};
use rootvg::surface::{DefaultSurface, DefaultSurfaceConfig};
use std::time::{Duration, Instant};

use crate::action_queue::ActionSender;
use crate::clipboard::Clipboard;
use crate::event::{
    CanvasEvent, EventCaptureStatus, KeyboardEvent, PointerButton, PointerEvent, PointerType,
    WheelDeltaType,
};
use crate::math::{
    to_logical_size_i32, PhysicalPoint, PhysicalSizeI32, Point, ScaleFactor, Size, Vector, ZIndex,
};
use crate::prelude::{ActionReceiver, ResourceCtx};
use crate::style::ClassID;
use crate::{view::ViewConfig, View};
use crate::{CursorIcon, ScissorRectID};

#[cfg(feature = "winit")]
mod winit_backend;
#[cfg(feature = "winit")]
use winit_backend as windowing_backend;

#[cfg(feature = "baseview")]
mod baseview_backend;
#[cfg(feature = "baseview")]
use baseview_backend as windowing_backend;

pub use windowing_backend::{run_blocking, OpenWindowError};

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

pub(crate) trait WindowBackend {
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
        scale_factor: ScaleFactor,
    ) -> Result<(), ()>;
    fn set_minimized(&mut self, window_id: WindowID, minimized: bool);
    fn set_maximized(&mut self, window_id: WindowID, maximized: bool);
    fn focus_window(&mut self, window_id: WindowID);
    fn set_window_title(&mut self, window_id: WindowID, title: String);
    fn create_window<A: Clone + 'static>(
        &mut self,
        window_id: WindowID,
        config: &WindowConfig,
        action_sender: &ActionSender<A>,
        res: &mut ResourceCtx,
    ) -> Result<WindowState<A>, OpenWindowError>;
    fn close_window(&mut self, window_id: WindowID);
}

pub(crate) struct WindowState<A: Clone + 'static> {
    pub queued_pointer_position: Option<PhysicalPoint>,
    pub queued_pointer_delta: Option<(f64, f64)>,
    pub prev_pointer_pos: Option<Point>,

    pub(crate) view: View<A>,
    pub(crate) clipboard: Clipboard,
    pub(crate) scale_factor: ScaleFactor,
    pub(crate) scale_factor_recip: f32,
    pub(crate) pointer_lock_state: PointerLockState,

    renderer: rootvg::Canvas,
    surface: Option<DefaultSurface<'static>>,
    logical_size: Size,
    physical_size: PhysicalSizeI32,
    system_scale_factor: ScaleFactor,
    scale_factor_config: ScaleFactorConfig,
    pointer_btn_states: [PointerBtnState; 5],

    modifiers: Modifiers,
    current_cursor_icon: CursorIcon,
}

impl<A: Clone + 'static> WindowState<A> {
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

    pub fn set_scale_factor_config(&mut self, config: ScaleFactorConfig) -> Option<Size> {
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

        self.scale_factor = scale_factor;
        self.scale_factor_recip = scale_factor.recip();

        self.view.resize(self.physical_size, scale_factor);
        self.surface
            .as_mut()
            .unwrap()
            .resize(self.physical_size, scale_factor);

        Some(logical_size)
    }

    pub fn set_pointer_locked(&mut self, state: PointerLockState) {
        self.pointer_lock_state = state;
        self.view.on_pointer_locked(state.is_locked());
    }

    pub fn pointer_lock_state(&self) -> PointerLockState {
        self.pointer_lock_state
    }

    pub fn on_animation_tick(&mut self, dt: f64, res: &mut ResourceCtx) {
        self.view.handle_event(
            &CanvasEvent::Animation {
                delta_seconds: dt,
                pointer_position: self.prev_pointer_pos,
            },
            res,
            &mut self.clipboard,
        );
    }

    pub fn handle_window_unfocused(&mut self, res: &mut ResourceCtx) {
        self.view
            .handle_event(&CanvasEvent::WindowUnfocused, res, &mut self.clipboard);
    }

    pub fn handle_window_focused(&mut self, res: &mut ResourceCtx) {
        self.view
            .handle_event(&CanvasEvent::WindowFocused, res, &mut self.clipboard);
    }

    pub fn handle_window_hidden(&mut self, res: &mut ResourceCtx) {
        self.handle_window_unfocused(res);
        self.view
            .handle_event(&CanvasEvent::WindowHidden, res, &mut self.clipboard);
    }

    pub fn handle_window_shown(&mut self, res: &mut ResourceCtx) {
        self.view
            .handle_event(&CanvasEvent::WindowShown, res, &mut self.clipboard);
    }

    pub fn set_modifiers(&mut self, modifiers: Modifiers) {
        self.modifiers = modifiers;
    }

    pub fn handle_keyboard_event(
        &mut self,
        event: KeyboardEvent,
        res: &mut ResourceCtx,
    ) -> EventCaptureStatus {
        self.modifiers = event.modifiers;

        self.view
            .handle_event(&CanvasEvent::Keyboard(event), res, &mut self.clipboard)
    }

    pub fn handle_text_composition_event(
        &mut self,
        event: CompositionEvent,
        res: &mut ResourceCtx,
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
            res,
            &mut self.clipboard,
        )
    }

    pub fn handle_pointer_left(&mut self, res: &mut ResourceCtx) {
        self.view.handle_event(
            &CanvasEvent::Pointer(PointerEvent::PointerLeft),
            res,
            &mut self.clipboard,
        );
    }

    pub fn handle_pointer_moved(&mut self, new_pos: PhysicalPoint, res: &mut ResourceCtx) {
        let new_pos = crate::math::to_logical_point_from_recip(new_pos, self.scale_factor_recip);

        let delta = if self.pointer_lock_state == PointerLockState::LockedUsingOS {
            // The delta will already be sent in `handle_locked_pointer_delta()`, so
            // avoid sending a duplicate.
            None
        } else if let Some(prev_pos) = self.prev_pointer_pos {
            Some(new_pos.to_vector() - prev_pos.to_vector())
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
            res,
            &mut self.clipboard,
        );
    }

    pub fn handle_locked_pointer_delta(&mut self, delta: Vector, res: &mut ResourceCtx) {
        self.view.handle_event(
            &CanvasEvent::Pointer(PointerEvent::Moved {
                position: self.prev_pointer_pos.unwrap_or_default(),
                delta: Some(delta),
                is_locked: false,
                pointer_type: PointerType::default(),
                modifiers: self.modifiers,
                just_entered: false,
            }),
            res,
            &mut self.clipboard,
        );
    }

    pub fn handle_mouse_button(
        &mut self,
        button: PointerButton,
        is_down: bool,
        res: &mut ResourceCtx,
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
                    res,
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
                    res,
                    &mut self.clipboard,
                );
            }
            _ => {}
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta_type: WheelDeltaType, res: &mut ResourceCtx) {
        let position = self.prev_pointer_pos.unwrap_or(Point::zero());

        self.view.handle_event(
            &CanvasEvent::Pointer(PointerEvent::ScrollWheel {
                position,
                delta_type,
                pointer_type: PointerType::default(),
                modifiers: self.modifiers,
            }),
            res,
            &mut self.clipboard,
        );
    }

    pub fn render<P: FnOnce()>(
        &mut self,
        pre_present_notify: P,
        res: &mut ResourceCtx,
    ) -> Result<(), wgpu::SurfaceError> {
        let surface = self.surface.as_ref().unwrap();

        self.view.render(
            &surface.surface,
            &surface.device,
            &surface.queue,
            &mut self.renderer,
            pre_present_notify,
            res,
        )
    }

    pub fn logical_size(&self) -> Size {
        self.logical_size
    }

    pub fn context<'b>(
        &'b mut self,
        res: &'b mut ResourceCtx,
        action_sender: &'b mut ActionSender<A>,
        action_receiver: &'b mut ActionReceiver<A>,
    ) -> WindowContext<'b, A> {
        WindowContext {
            view: &mut self.view,
            res,
            clipboard: &mut self.clipboard,
            action_sender,
            action_receiver,
            z_index_stack: Vec::new(),
            scissor_rect_stack: Vec::new(),
            class_stack: Vec::new(),
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

    pub fn on_theme_changed(&mut self, res: &mut ResourceCtx) {
        self.view.on_theme_changed(res, &mut self.clipboard);
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
    pub res: &'a mut ResourceCtx,
    pub clipboard: &'a mut Clipboard,
    /// The sending end of the action queue.
    pub action_sender: &'a mut ActionSender<A>,
    /// The receiving end of the action queue.
    pub action_receiver: &'a mut ActionReceiver<A>,
    z_index_stack: Vec<ZIndex>,
    scissor_rect_stack: Vec<ScissorRectID>,
    class_stack: Vec<ClassID>,
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

    /// Get the current z index from the stack (peek)
    pub fn z_index(&self) -> ZIndex {
        self.z_index_stack.last().copied().unwrap_or_default()
    }

    /// Get the current scissor rect ID from the stack (peek)
    pub fn scissor_rect(&self) -> ScissorRectID {
        self.scissor_rect_stack.last().copied().unwrap_or_default()
    }

    /// Get the current style class ID from the stack (peek)
    pub fn class(&self) -> ClassID {
        self.class_stack.last().map(|s| *s).unwrap_or_default()
    }

    /// Push a z index onto the stack
    pub fn push_z_index(&mut self, z_index: ZIndex) {
        self.z_index_stack.push(z_index)
    }

    /// Push a z index onto the stack that is equal to `WindowContext::z_index() + 1`
    pub fn push_next_z_index(&mut self) {
        self.z_index_stack.push(self.z_index() + 1)
    }

    /// Push a scissor rect ID onto the stack
    pub fn push_scissor_rect(&mut self, scissor_rect: ScissorRectID) {
        self.scissor_rect_stack.push(scissor_rect);
    }

    /// Push a style class ID onto the stack
    pub fn push_class(&mut self, class: ClassID) {
        self.class_stack.push(class);
    }

    /// Pop a z index from the stack
    pub fn pop_z_index(&mut self) -> Option<ZIndex> {
        self.z_index_stack.pop()
    }

    /// Pop a scissor rect ID from the stack
    pub fn pop_scissor_rect(&mut self) -> Option<ScissorRectID> {
        self.scissor_rect_stack.pop()
    }

    /// Pop a style class ID from the stack
    pub fn pop_class(&mut self) -> Option<ClassID> {
        self.class_stack.pop()
    }

    /// Reset the z index stack.
    pub fn reset_z_index(&mut self) {
        self.z_index_stack.clear();
    }

    /// Reset the scissor rect ID stack
    pub fn reset_scissor_rect(&mut self) {
        self.scissor_rect_stack.clear();
    }

    /// Returns the z index, scissor rect ID, and class ID from the given builder values.
    pub fn builder_values(
        &self,
        z_index: Option<ZIndex>,
        scissor_rect: Option<ScissorRectID>,
        class: Option<ClassID>,
    ) -> (ZIndex, ScissorRectID, ClassID) {
        (
            z_index.unwrap_or_else(|| self.z_index()),
            scissor_rect.unwrap_or_else(|| self.scissor_rect()),
            class.unwrap_or_else(|| self.class()),
        )
    }

    pub fn with_z_index<T, F: FnOnce(&mut Self) -> T>(&mut self, z_index: ZIndex, f: F) -> T {
        self.push_z_index(z_index);
        let r = (f)(self);
        self.pop_z_index();
        r
    }

    pub fn with_scissor_rect<T, F: FnOnce(&mut Self) -> T>(
        &mut self,
        scissor_rect: ScissorRectID,
        f: F,
    ) -> T {
        self.push_scissor_rect(scissor_rect);
        let r = (f)(self);
        self.pop_scissor_rect();
        r
    }

    pub fn with_z_index_and_scissor_rect<T, F: FnOnce(&mut Self) -> T>(
        &mut self,
        z_index: ZIndex,
        scissor_rect: ScissorRectID,
        f: F,
    ) -> T {
        self.push_z_index(z_index);
        self.push_scissor_rect(scissor_rect);
        let r = (f)(self);
        self.pop_z_index();
        self.pop_scissor_rect();
        r
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxBackendType {
    Wayland,
    X11,
}
