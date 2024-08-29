use keyboard_types::{CompositionEvent, CompositionState, Modifiers};
use rootvg::surface::DefaultSurface;
use rootvg::text::FontSystem;
use rustc_hash::FxHashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler as WinitApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, StartCause,
    WindowEvent as WinitWindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window as WinitWindow, WindowId as WinitWindowId};

use crate::action_queue::ActionSender;
use crate::application::{Application, TimerInterval};
use crate::element_system::ElementSystemConfig;
use crate::event::{AppWindowEvent, EventCaptureStatus, PointerButton, WheelDeltaType};
use crate::math::{PhysicalPoint, PhysicalSizeI32, ScaleFactor, Size, Vector};
use crate::prelude::{AppHandler, ResourceCtx};
use crate::style::StyleSystem;
use crate::window::{WindowID, MAIN_WINDOW};
use crate::AppConfig;

use super::{
    Clipboard, CursorIcon, ElementSystem, LinuxBackendType, PointerBtnState, PointerLockState,
    ScaleFactorConfig, WindowBackend, WindowCloseRequest, WindowConfig, WindowState,
};

mod convert;

struct WinitWindowBackend<'a> {
    inner: &'a mut WinitAppHandlerInner,
    event_loop: &'a ActiveEventLoop,
}

impl<'a> WindowBackend for WinitWindowBackend<'a> {
    fn set_pointer_position(
        &mut self,
        window_id: WindowID,
        position: PhysicalPoint,
    ) -> Result<(), ()> {
        if let Some(window_handle) = self.inner.windows.get(&window_id) {
            if let Err(e) =
                window_handle.set_cursor_position(PhysicalPosition::new(position.x, position.y))
            {
                log::debug!("Could not set cursor position: {}", e);
                Err(())
            } else {
                Ok(())
            }
        } else {
            Err(())
        }
    }

    fn unlock_pointer(&mut self, window_id: WindowID, prev_lock_state: PointerLockState) {
        let Some(window_handle) = self.inner.windows.get(&window_id) else {
            return;
        };

        match prev_lock_state {
            PointerLockState::LockedUsingOS => {
                if let Err(e) = window_handle.set_cursor_grab(CursorGrabMode::None) {
                    log::error!("Error while unlocking pointer: {}", e);
                }
                window_handle.set_cursor_visible(true);
            }
            PointerLockState::ManualLock { .. } => {
                window_handle.set_cursor_visible(true);
            }
            _ => {}
        }
    }

    fn request_redraw(&mut self, window_id: WindowID) {
        if let Some(window_handle) = self.inner.windows.get(&window_id) {
            window_handle.request_redraw();
        }
    }

    fn has_focus(&mut self, window_id: WindowID) -> bool {
        self.inner
            .windows
            .get(&window_id)
            .map(|w| w.has_focus())
            .unwrap_or(false)
    }

    fn try_lock_pointer(&mut self, window_id: WindowID) -> PointerLockState {
        let Some(window_handle) = self.inner.windows.get(&window_id) else {
            return PointerLockState::NotLocked;
        };

        #[allow(unused_mut, unused_assignments)]
        let mut try_os_lock = false;
        #[allow(unused_mut, unused_assignments)]
        let mut try_manual_lock = false;

        #[cfg(target_family = "wasm")]
        {
            try_os_lock = true;
        }

        #[cfg(not(target_family = "wasm"))]
        {
            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
            ))]
            {
                use raw_window_handle_06::{HasWindowHandle, RawWindowHandle};

                if let Ok(window_handle) = window_handle.window_handle() {
                    match window_handle.as_raw() {
                        RawWindowHandle::Wayland(_) => try_os_lock = true,
                        RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_) => {
                            try_manual_lock = true
                        }
                        _ => {}
                    };
                }
            }

            #[cfg(target_os = "macos")]
            {
                try_os_lock = true;
                try_manual_lock = true;
            }

            #[cfg(target_os = "windows")]
            {
                try_manual_lock = true;
            }
        }

        let state = if try_os_lock {
            match window_handle.set_cursor_grab(CursorGrabMode::Locked) {
                Ok(_) => PointerLockState::LockedUsingOS,
                Err(e) => {
                    log::debug!("Could not lock pointer: {}", e);
                    PointerLockState::NotLocked
                }
            }
        } else {
            PointerLockState::NotLocked
        };

        if state.is_locked() {
            window_handle.set_cursor_visible(false);
            state
        } else if try_manual_lock {
            window_handle.set_cursor_visible(false);
            PointerLockState::ManualLock
        } else {
            PointerLockState::NotLocked
        }
    }

    fn set_cursor_icon(&mut self, window_id: WindowID, icon: CursorIcon) {
        if let Some(window_handle) = self.inner.windows.get(&window_id) {
            let winit_icon = self::convert::convert_cursor_icon_to_winit(icon);
            window_handle.set_cursor(winit_icon);
        }
    }

    fn resize(
        &mut self,
        window_id: WindowID,
        logical_size: Size,
        scale_factor: ScaleFactor,
    ) -> Result<(), ()> {
        if let Some(window_handle) = self.inner.windows.get(&window_id) {
            if window_handle
                .request_inner_size(PhysicalSize::new(
                    logical_size.width * scale_factor,
                    logical_size.height * scale_factor,
                ))
                .is_some()
            {
                Ok(())
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    fn set_minimized(&mut self, window_id: WindowID, minimized: bool) {
        if let Some(window_handle) = self.inner.windows.get(&window_id) {
            window_handle.set_minimized(minimized);
        }
    }

    fn set_maximized(&mut self, window_id: WindowID, maximized: bool) {
        if let Some(window_handle) = self.inner.windows.get(&window_id) {
            window_handle.set_maximized(maximized);
        }
    }

    fn focus_window(&mut self, window_id: WindowID) {
        if let Some(window_handle) = self.inner.windows.get(&window_id) {
            window_handle.focus_window()
        }
    }

    fn set_window_title(&mut self, window_id: WindowID, title: String) {
        if let Some(window_handle) = self.inner.windows.get(&window_id) {
            window_handle.set_title(&title)
        }
    }

    fn create_window<A: Clone + 'static>(
        &mut self,
        window_id: WindowID,
        config: &WindowConfig,
        action_sender: &ActionSender<A>,
        res: &mut ResourceCtx,
    ) -> Result<WindowState<A>, OpenWindowError> {
        match create_window(window_id, config, self.event_loop, action_sender, res) {
            Ok((window_handle, window_state)) => {
                self.inner
                    .winit_id_to_window_id_map
                    .insert(window_handle.id(), window_id);
                self.inner.windows.insert(window_id, window_handle);

                Ok(window_state)
            }
            Err(e) => Err(e),
        }
    }

    fn close_window(&mut self, window_id: WindowID) {
        if let Some(window_handle) = self.inner.windows.remove(&window_id) {
            self.inner
                .winit_id_to_window_id_map
                .remove(&window_handle.id());

            // Window handle is dropped here.
        }
    }
}

struct PreMainWindowData {
    config: AppConfig,
    res: ResourceCtx,
}

struct WinitAppHandlerInner {
    tick_interval: Duration,
    pointer_debounce_interval: Duration,
    prev_cursor_debounce_instant: Instant,
    requested_tick_resume: Instant,
    requested_cursor_debounce_resume: Option<Instant>,

    winit_id_to_window_id_map: FxHashMap<WinitWindowId, WindowID>,
    windows: FxHashMap<WindowID, Arc<winit::window::Window>>,

    tick_wait_cancelled: bool,
}

struct WinitAppHandler<A: Application> {
    app_handler: Option<AppHandler<A>>,
    inner: WinitAppHandlerInner,
    pre_main_window_data: Option<PreMainWindowData>,
}

impl<A: Application> WinitAppHandler<A> {
    fn new(config: AppConfig) -> Result<Self, Box<dyn Error>> {
        let use_dark_theme = config.use_dark_theme;

        Ok(Self {
            app_handler: None,
            inner: WinitAppHandlerInner {
                tick_interval: Duration::default(),
                pointer_debounce_interval: Duration::default(),
                prev_cursor_debounce_instant: Instant::now(),
                requested_tick_resume: Instant::now(),
                requested_cursor_debounce_resume: None,
                winit_id_to_window_id_map: FxHashMap::default(),
                windows: FxHashMap::default(),
                tick_wait_cancelled: false,
            },
            pre_main_window_data: Some(PreMainWindowData {
                config,
                res: ResourceCtx {
                    style_system: StyleSystem::new(use_dark_theme),
                    font_system: FontSystem::new(),
                    #[cfg(feature = "svg-icons")]
                    svg_icon_system: Default::default(),
                },
            }),
        })
    }

    fn process_updates(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(app_handler) = &mut self.app_handler {
            app_handler.process_updates(&mut WinitWindowBackend {
                inner: &mut self.inner,
                event_loop,
            });
        }
    }
}

impl<A: Application> WinitApplicationHandler for WinitAppHandler<A> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.inner.tick_wait_cancelled = false;

        match cause {
            StartCause::ResumeTimeReached {
                requested_resume, ..
            } => {
                if requested_resume == self.inner.requested_tick_resume {
                    if let Some(app_handler) = &mut self.app_handler {
                        app_handler.on_tick();
                    }

                    self.process_updates(event_loop);
                } else if let Some(pointer_resume_instant) =
                    self.inner.requested_cursor_debounce_resume
                {
                    if pointer_resume_instant == requested_resume {
                        self.process_updates(event_loop);
                    }
                }
            }
            StartCause::WaitCancelled { .. } => self.inner.tick_wait_cancelled = true,
            _ => {}
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(mut data) = self.pre_main_window_data.take() {
            let (action_sender, action_receiver) = crate::action_channel::<A::Action>();

            let (window_handle, main_window_state) = match create_window(
                MAIN_WINDOW,
                &data.config.main_window_config,
                event_loop,
                &action_sender,
                &mut data.res,
            ) {
                Ok(w) => w,
                Err(e) => {
                    log::error!("Failed to open main window: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            self.inner
                .windows
                .insert(MAIN_WINDOW, Arc::clone(&window_handle));

            let find_millihertz =
                if let TimerInterval::PercentageOfFrameRate(_) = data.config.tick_timer_interval {
                    true
                } else if let TimerInterval::PercentageOfFrameRate(_) =
                    data.config.pointer_debounce_interval
                {
                    true
                } else {
                    false
                };
            let millihertz = if find_millihertz {
                // Attempt to get the refresh rate of the current monitor. If that's
                // not possible, try other methods.
                let mut millihertz = None;
                if let Some(monitor) = window_handle.current_monitor() {
                    millihertz = monitor.refresh_rate_millihertz();
                }
                if millihertz.is_none() {
                    if let Some(monitor) = window_handle.primary_monitor() {
                        millihertz = monitor.refresh_rate_millihertz();
                    }
                }
                if millihertz.is_none() {
                    if let Some(monitor) = event_loop.primary_monitor() {
                        millihertz = monitor.refresh_rate_millihertz();
                    }
                }
                if millihertz.is_none() {
                    for monitor in event_loop.available_monitors() {
                        if let Some(m) = monitor.refresh_rate_millihertz() {
                            millihertz = Some(m);
                            break;
                        }
                    }
                }
                millihertz.unwrap_or(60_000)
            } else {
                60_000
            };

            self.inner.tick_interval = match data.config.tick_timer_interval {
                TimerInterval::Fixed(interval) => interval,
                TimerInterval::PercentageOfFrameRate(percentage) => {
                    Duration::from_secs_f64(percentage * 1_000.0 / millihertz as f64)
                }
            };
            self.inner.pointer_debounce_interval = match data.config.pointer_debounce_interval {
                TimerInterval::Fixed(interval) => interval,
                TimerInterval::PercentageOfFrameRate(percentage) => {
                    Duration::from_secs_f64(percentage * 1_000.0 / millihertz as f64)
                }
            };

            self.inner
                .winit_id_to_window_id_map
                .insert(window_handle.id(), MAIN_WINDOW);

            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
            ))]
            let linux_backend_type = {
                use winit::platform::wayland::ActiveEventLoopExtWayland;
                use winit::platform::x11::ActiveEventLoopExtX11;

                if event_loop.is_x11() {
                    Some(LinuxBackendType::X11)
                } else if event_loop.is_wayland() {
                    Some(LinuxBackendType::Wayland)
                } else {
                    log::warn!("Could not parse whether windowing backend is X11 or Wayland");
                    None
                }
            };

            #[cfg(not(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
            )))]
            let linux_backend_type = None;

            let app_handler = match AppHandler::new(
                main_window_state,
                action_sender,
                action_receiver,
                data.config,
                data.res,
                linux_backend_type,
            ) {
                Ok(a) => a,
                Err(e) => {
                    log::error!("Application returned error on init: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            self.app_handler = Some(app_handler);

            self.process_updates(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        winit_window_id: WinitWindowId,
        event: WinitWindowEvent,
    ) {
        let Some(app_handler) = &mut self.app_handler else {
            return;
        };

        let Some(window_id) = self
            .inner
            .winit_id_to_window_id_map
            .get(&winit_window_id)
            .copied()
        else {
            return;
        };

        let window_state = if window_id == MAIN_WINDOW {
            &mut app_handler.cx.main_window
        } else if let Some(window_state) = app_handler.cx.window_map.get_mut(&window_id) {
            window_state
        } else {
            return;
        };

        let mut process_updates = true;

        match event {
            WinitWindowEvent::CloseRequested => {
                if window_id == MAIN_WINDOW {
                    match app_handler.user_app.on_request_to_close_window(
                        MAIN_WINDOW,
                        false,
                        &mut app_handler.cx,
                    ) {
                        WindowCloseRequest::CloseImmediately => event_loop.exit(),
                        WindowCloseRequest::DoNotCloseYet => {}
                    }
                } else {
                    match app_handler.user_app.on_request_to_close_window(
                        window_id,
                        false,
                        &mut app_handler.cx,
                    ) {
                        WindowCloseRequest::CloseImmediately => {
                            app_handler.cx.window_map.remove(&window_id);
                            self.inner
                                .winit_id_to_window_id_map
                                .remove(&winit_window_id);
                            self.inner.windows.remove(&window_id);
                        }
                        WindowCloseRequest::DoNotCloseYet => {}
                    }
                }
            }
            WinitWindowEvent::RedrawRequested => {
                process_updates = false;

                let window_handle = self.inner.windows.get(&window_id).unwrap();

                match window_state.render(
                    || window_handle.pre_present_notify(),
                    &mut app_handler.cx.res,
                ) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        let inner_size = window_handle.inner_size();
                        let new_size =
                            PhysicalSizeI32::new(inner_size.width as i32, inner_size.height as i32);
                        let new_scale_factor = window_handle.scale_factor().into();

                        window_state.set_size(new_size, new_scale_factor);
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Yarrow: Out of GPU memory");
                        event_loop.exit();
                    }
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => log::debug!("{:?}", e),
                }
            }
            WinitWindowEvent::Resized(new_size) => {
                let new_size = PhysicalSizeI32::new(new_size.width as i32, new_size.height as i32);

                let window_handle = self.inner.windows.get(&window_id).unwrap();

                let scale_factor = window_handle.scale_factor().into();
                window_state.set_size(new_size, scale_factor);
                window_handle.request_redraw();

                app_handler.user_app.on_window_event(
                    AppWindowEvent::WindowResized,
                    window_id,
                    &mut app_handler.cx,
                );
            }
            WinitWindowEvent::ScaleFactorChanged {
                scale_factor,
                mut inner_size_writer,
            } => {
                let new_size: PhysicalSizeI32 =
                    crate::math::to_physical_size(window_state.logical_size(), scale_factor.into())
                        .round()
                        .cast();
                let new_inner_size = winit::dpi::PhysicalSize {
                    width: new_size.width as u32,
                    height: new_size.height as u32,
                };
                if let Err(e) = inner_size_writer.request_inner_size(new_inner_size) {
                    log::error!("{}", e);
                }

                window_state.set_size(new_size, scale_factor.into());

                app_handler.user_app.on_window_event(
                    AppWindowEvent::WindowResized,
                    window_id,
                    &mut app_handler.cx,
                );
            }
            WinitWindowEvent::Focused(focused) => {
                let event = if focused {
                    window_state.handle_window_focused(&mut app_handler.cx.res);
                    AppWindowEvent::WindowFocused
                } else {
                    window_state.handle_window_unfocused(&mut app_handler.cx.res);
                    AppWindowEvent::WindowUnfocused
                };

                app_handler
                    .user_app
                    .on_window_event(event, window_id, &mut app_handler.cx);
            }
            WinitWindowEvent::Occluded(hidden) => {
                let event = if hidden {
                    window_state.handle_window_hidden(&mut app_handler.cx.res);
                    AppWindowEvent::WindowHidden
                } else {
                    window_state.handle_window_shown(&mut app_handler.cx.res);
                    AppWindowEvent::WindowShown
                };

                app_handler
                    .user_app
                    .on_window_event(event, window_id, &mut app_handler.cx);
            }
            WinitWindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                let pos = PhysicalPoint::new(position.x as f32, position.y as f32);

                window_state.queued_pointer_position = Some(pos);

                let now = Instant::now();
                if now - self.inner.prev_cursor_debounce_instant
                    < self.inner.pointer_debounce_interval
                {
                    process_updates = false;

                    // Make sure that the latest cursor move event is always sent.
                    let mut resume_instant = now + self.inner.pointer_debounce_interval;
                    if resume_instant == self.inner.requested_tick_resume {
                        // Make sure we don't clash with the tick timer.
                        resume_instant += Duration::from_micros(1);
                    }
                    self.inner.requested_cursor_debounce_resume = Some(resume_instant);

                    event_loop.set_control_flow(ControlFlow::WaitUntil(resume_instant));
                }
            }
            WinitWindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let button = match button {
                    WinitMouseButton::Left => PointerButton::Primary,
                    WinitMouseButton::Right => PointerButton::Secondary,
                    WinitMouseButton::Middle => PointerButton::Auxiliary,
                    WinitMouseButton::Back => PointerButton::Fourth,
                    WinitMouseButton::Forward => PointerButton::Fifth,
                    _ => return,
                };

                window_state.handle_mouse_button(
                    button,
                    state.is_pressed(),
                    &mut app_handler.cx.res,
                );
            }
            WinitWindowEvent::CursorLeft { device_id: _ } => {
                window_state.handle_pointer_left(&mut app_handler.cx.res);
            }
            WinitWindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let delta_type = match delta {
                    MouseScrollDelta::LineDelta(x, y) => WheelDeltaType::Lines(Vector::new(x, -y)),
                    MouseScrollDelta::PixelDelta(pos) => WheelDeltaType::Points(Vector::new(
                        pos.x as f32 * window_state.scale_factor_recip,
                        -pos.y as f32 * window_state.scale_factor_recip,
                    )),
                };

                window_state.handle_mouse_wheel(delta_type, &mut app_handler.cx.res);
            }
            WinitWindowEvent::Destroyed => {
                app_handler.user_app.on_window_event(
                    AppWindowEvent::WindowClosed,
                    window_id,
                    &mut app_handler.cx,
                );

                app_handler.cx.window_map.remove(&window_id);
                self.inner
                    .winit_id_to_window_id_map
                    .remove(&winit_window_id);
                self.inner.windows.remove(&window_id);
            }
            WinitWindowEvent::ModifiersChanged(winit_modifiers) => {
                let modifiers = self::convert::convert_modifiers(winit_modifiers);

                window_state.set_modifiers(modifiers);
            }
            WinitWindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let key_event =
                    self::convert::convert_keyboard_event(&event, window_state.modifiers);

                let mut captured = window_state
                    .handle_keyboard_event(key_event.clone(), &mut app_handler.cx.res)
                    == EventCaptureStatus::Captured;

                if !captured {
                    if let Some(text) = &event.text {
                        if !text.is_empty() && event.state == ElementState::Pressed {
                            captured |= window_state.handle_text_composition_event(
                                CompositionEvent {
                                    state: CompositionState::Start,
                                    data: String::new(),
                                },
                                &mut app_handler.cx.res,
                            ) == EventCaptureStatus::Captured;
                            captured |= window_state.handle_text_composition_event(
                                CompositionEvent {
                                    state: CompositionState::End,
                                    data: text.to_string(),
                                },
                                &mut app_handler.cx.res,
                            ) == EventCaptureStatus::Captured;
                        }
                    }
                }

                if !captured {
                    app_handler.user_app.on_keyboard_event(
                        key_event,
                        window_id,
                        &mut app_handler.cx,
                    );
                }
            }
            _ => (),
        }

        if process_updates {
            self.process_updates(event_loop);
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let Some(app_handler) = &mut self.app_handler else {
            return;
        };

        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            for window in app_handler
                .cx
                .window_map
                .values_mut()
                .chain([&mut app_handler.cx.main_window])
            {
                if window.pointer_lock_state().is_locked() {
                    if let Some(prev_delta) = &mut window.queued_pointer_delta {
                        prev_delta.0 += delta.0;
                        prev_delta.1 += delta.1;
                    } else {
                        window.queued_pointer_delta = Some(delta);
                    }

                    let now = Instant::now();
                    if now - self.inner.prev_cursor_debounce_instant
                        < self.inner.pointer_debounce_interval
                    {
                        // Make sure that the latest cursor move event is always sent.
                        let mut resume_instant = now + self.inner.pointer_debounce_interval;
                        if resume_instant == self.inner.requested_tick_resume {
                            // Make sure we don't clash with the tick timer.
                            resume_instant += Duration::from_micros(1);
                        }
                        self.inner.requested_cursor_debounce_resume = Some(resume_instant);

                        event_loop.set_control_flow(ControlFlow::WaitUntil(resume_instant));
                    }
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.inner.tick_wait_cancelled {
            let now = Instant::now();

            let Some(app_handler) = &mut self.app_handler else {
                return;
            };

            let mut next_instant = if app_handler.prev_tick_instant + self.inner.tick_interval > now
            {
                app_handler.prev_tick_instant + self.inner.tick_interval
            } else {
                app_handler.on_tick();

                now + self.inner.tick_interval
            };

            if let Some(pointer_resume_instant) = self.inner.requested_cursor_debounce_resume {
                if next_instant == pointer_resume_instant {
                    // Make sure we don't clash with the pointer debounce timer.
                    next_instant += Duration::from_micros(1);
                }
            }

            self.inner.requested_tick_resume = next_instant;

            event_loop.set_control_flow(ControlFlow::WaitUntil(next_instant));
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OpenWindowError {
    #[error("{0}")]
    OsError(#[from] winit::error::OsError),
    #[error("{0}")]
    SurfaceError(#[from] rootvg::surface::NewSurfaceError),
}

pub fn run_blocking<A: Application>(config: AppConfig) -> Result<(), Box<dyn Error>>
where
    A::Action: Send,
{
    let event_loop = EventLoop::new()?;
    let mut app_handler = WinitAppHandler::<A>::new(config)?;

    event_loop.run_app(&mut app_handler).map_err(Into::into)
}

fn create_window<A: Clone + 'static>(
    id: WindowID,
    config: &WindowConfig,
    event_loop: &ActiveEventLoop,
    action_sender: &ActionSender<A>,
    res: &mut ResourceCtx,
) -> Result<(Arc<winit::window::Window>, WindowState<A>), OpenWindowError> {
    #[allow(unused_mut)]
    let mut attributes = WinitWindow::default_attributes()
        .with_title(config.title.clone())
        .with_resizable(config.resizable)
        .with_active(config.focus_on_creation);

    match config.scale_factor {
        ScaleFactorConfig::System => {
            attributes = attributes.with_inner_size(winit::dpi::LogicalSize::new(
                config.size.width,
                config.size.height,
            ));
        }
        ScaleFactorConfig::Custom(scale_factor) => {
            let size: PhysicalSizeI32 = crate::math::to_physical_size(config.size, scale_factor)
                .round()
                .cast();

            attributes = attributes.with_inner_size(winit::dpi::PhysicalSize {
                width: size.width as u32,
                height: size.height as u32,
            });
        }
    }

    #[cfg(all(
        any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "dragonfly"
        ),
        not(target_family = "wasm")
    ))]
    {
        use winit::platform::startup_notify::EventLoopExtStartupNotify;
        use winit::platform::startup_notify::WindowAttributesExtStartupNotify;

        if config.focus_on_creation {
            if let Some(token) = event_loop.read_token_from_env() {
                winit::platform::startup_notify::reset_activation_token_env();
                log::info!("Using token {:?} to activate a window", token);
                attributes = attributes.with_activation_token(token);
            }
        }
    }

    let window = event_loop.create_window(attributes).map(|w| Arc::new(w))?;

    // Might fix an issue in MacOS with wgpu
    // https://github.com/gfx-rs/wgpu/issues/5722
    window.request_redraw();

    let physical_size = window.inner_size();
    let physical_size =
        PhysicalSizeI32::new(physical_size.width as i32, physical_size.height as i32);
    let system_scale_factor: ScaleFactor = window.scale_factor().into();

    let scale_factor = config.scale_factor.scale_factor(system_scale_factor);

    let surface = DefaultSurface::new(
        physical_size,
        scale_factor,
        Arc::clone(&window),
        config.surface_config.clone(),
    )?;

    let canvas_config = surface.canvas_config();

    let renderer = rootvg::Canvas::new(
        &surface.device,
        &surface.queue,
        surface.format(),
        canvas_config,
        &mut res.font_system,
    );

    let element_system = ElementSystem::new(
        physical_size,
        scale_factor,
        ElementSystemConfig {
            clear_color: config.clear_color,
            preallocate_for_this_many_elements: config.preallocate_for_this_many_elements,
            hover_timeout_duration: config.hover_timeout_duration,
            scroll_wheel_timeout_duration: config.scroll_wheel_timeout_duration,
        },
        action_sender.clone(),
        id,
    );

    let clipboard = new_clipboard(&window);

    Ok((
        window,
        WindowState {
            element_system,
            renderer,
            surface: Some(surface),
            multisample: canvas_config.multisample,
            logical_size: config.size,
            physical_size,
            scale_factor,
            scale_factor_recip: scale_factor.recip(),
            system_scale_factor,
            scale_factor_config: config.scale_factor,
            queued_pointer_position: None,
            queued_pointer_delta: None,
            prev_pointer_pos: None,
            pointer_btn_states: [PointerBtnState::default(); 5],
            modifiers: Modifiers::empty(),
            current_cursor_icon: CursorIcon::Default,
            pointer_lock_state: PointerLockState::NotLocked,
            clipboard,
        },
    ))
}

fn new_clipboard(window_handle: &Arc<WinitWindow>) -> Clipboard {
    // SAFETY:
    // A reference-counted handle to the window is stored in `WindowState`,
    // ensuring that the window will not be dropped before the clipboard
    // is dropped.
    let state = unsafe { window_clipboard::Clipboard::connect(window_handle) }
        .ok()
        .map(crate::clipboard::State::Connected)
        .unwrap_or(crate::clipboard::State::Unavailable);

    Clipboard { state }
}
