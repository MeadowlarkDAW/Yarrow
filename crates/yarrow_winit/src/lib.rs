use ahash::AHashMap;
use raw_window_handle::HasWindowHandle;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler as WinitApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize as WinitPhysicalSize};
use winit::event::{
    ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, StartCause,
    WindowEvent as WinitWindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window as WinitWindow, WindowId as WinitWindowId};
use yarrow_core::{CursorIcon, ResourceContext};

use yarrow_core::application::{AppConfig, Application, TimerInterval};
use yarrow_core::event::keyboard::{CompositionEvent, CompositionState, Modifiers};
use yarrow_core::event::{AppWindowEvent, EventCaptureStatus, PointerButton, WheelDeltaType};
use yarrow_core::math::{PhysicalPoint, PhysicalSize, Scale, Size, Vector};
use yarrow_core::renderer::RenderBackend;
use yarrow_core::window::{
    CreateWindowResult, LinuxBackendType, PointerLockState, ScaleFactorConfig, WindowBackend,
    WindowConfig, WindowID,
};

mod clipboard;
mod convert;

pub struct WinitWindowBackend<'a> {
    inner: &'a mut WinitAppHandlerInner,
    event_loop: &'a ActiveEventLoop,
}

impl<'a, R: RenderBackend> WindowBackend<R> for WinitWindowBackend<'a> {
    type OpenError = OpenWindowError;

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
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};

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
        scale_factor: f32,
    ) -> Result<(), ()> {
        if let Some(window_handle) = self.inner.windows.get(&window_id) {
            if window_handle
                .request_inner_size(WinitPhysicalSize::new(
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

    fn create_window(
        &mut self,
        window_id: WindowID,
        config: &WindowConfig,
    ) -> Result<CreateWindowResult<R>, OpenWindowError> {
        match create_window(config, self.event_loop) {
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
    res: ResourceContext,
}

struct WinitAppHandlerInner {
    tick_interval: Duration,
    pointer_debounce_interval: Duration,
    prev_cursor_debounce_instant: Instant,
    requested_tick_resume: Instant,
    requested_cursor_debounce_resume: Option<Instant>,

    winit_id_to_window_id_map: AHashMap<WinitWindowId, WindowID>,
    windows: AHashMap<WindowID, Arc<winit::window::Window>>,

    tick_wait_cancelled: bool,
}

struct WinitAppHandler<R: RenderBackend> {
    //app_handler: Option<AppHandler<A>>,
    inner: WinitAppHandlerInner,
    pre_main_window_data: Option<PreMainWindowData>,
    temp_renderer: Option<R>,
    _temp: std::marker::PhantomData<R>,
}

impl<R: RenderBackend> WinitAppHandler<R> {
    fn new(config: AppConfig) -> Result<Self, Box<dyn Error>> {
        let use_dark_theme = config.use_dark_theme;

        Ok(Self {
            //app_handler: None,
            inner: WinitAppHandlerInner {
                tick_interval: Duration::default(),
                pointer_debounce_interval: Duration::default(),
                prev_cursor_debounce_instant: Instant::now(),
                requested_tick_resume: Instant::now(),
                requested_cursor_debounce_resume: None,
                winit_id_to_window_id_map: AHashMap::default(),
                windows: AHashMap::default(),
                tick_wait_cancelled: false,
            },
            pre_main_window_data: Some(PreMainWindowData {
                config,
                res: ResourceContext {
                    //style_system: StyleSystem::new(use_dark_theme),
                    //font_system: FontSystem::new(),
                    #[cfg(feature = "svg-icons")]
                    svg_icon_system: Default::default(),
                },
            }),
            temp_renderer: None,
            _temp: PhantomData::default(),
        })
    }

    fn process_updates(&mut self, event_loop: &ActiveEventLoop) {
        /*
        if let Some(app_handler) = &mut self.app_handler {
            app_handler.process_updates(&mut WinitWindowBackend {
                inner: &mut self.inner,
                event_loop,
            });
        }
        */
    }
}

impl<R: RenderBackend> WinitApplicationHandler for WinitAppHandler<R> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.inner.tick_wait_cancelled = false;

        match cause {
            StartCause::ResumeTimeReached {
                requested_resume, ..
            } => {
                if requested_resume == self.inner.requested_tick_resume {
                    /*
                    if let Some(app_handler) = &mut self.app_handler {
                        app_handler.on_tick();
                    }
                    */

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
            let (window_handle, res) =
                match create_window::<R>(&data.config.main_window_config, event_loop) {
                    Ok(w) => w,
                    Err(e) => {
                        log::error!("Failed to open main window: {}", e);
                        event_loop.exit();
                        return;
                    }
                };

            self.inner
                .windows
                .insert(WindowID::MAIN, Arc::clone(&window_handle));

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
                TimerInterval::FixedSecs(interval) => Duration::from_secs_f64(interval),
                TimerInterval::PercentageOfFrameRate(percentage) => {
                    Duration::from_secs_f64(percentage * 1_000.0 / millihertz as f64)
                }
            };
            self.inner.pointer_debounce_interval = match data.config.pointer_debounce_interval {
                TimerInterval::FixedSecs(interval) => Duration::from_secs_f64(interval),
                TimerInterval::PercentageOfFrameRate(percentage) => {
                    Duration::from_secs_f64(percentage * 1_000.0 / millihertz as f64)
                }
            };

            self.inner
                .winit_id_to_window_id_map
                .insert(window_handle.id(), WindowID::MAIN);

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

            /*
            let app_handler = match AppHandler::new(
                main_window_state,
                &mut WinitWindowBackend {
                    inner: &mut self.inner,
                    event_loop,
                },
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
            */

            self.temp_renderer = Some(res.renderer);

            self.process_updates(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        winit_window_id: WinitWindowId,
        event: WinitWindowEvent,
    ) {
        if let WinitWindowEvent::CloseRequested = event {
            event_loop.exit();
        }

        match event {
            WinitWindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WinitWindowEvent::Resized(physical_size) => {
                if let Some(renderer) = &mut self.temp_renderer {
                    renderer.resize(PhysicalSize::new(
                        physical_size.width as i32,
                        physical_size.height as i32,
                    ));

                    self.inner
                        .windows
                        .get(&WindowID::MAIN)
                        .unwrap()
                        .request_redraw();
                }
            }
            WinitWindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.temp_renderer {
                    renderer.render();
                }
            }
            _ => {}
        }

        /*
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

        let window_state = if window_id == WindowID::MAIN {
            &mut app_handler.cx.main_window
        } else if let Some(window_state) = app_handler.cx.window_map.get_mut(&window_id) {
            window_state
        } else {
            return;
        };

        let mut process_updates = true;

        match event {
            WinitWindowEvent::CloseRequested => {
                if window_id == WindowID::MAIN {
                    match app_handler.on_request_to_close_window(WindowID::MAIN, false) {
                        WindowCloseRequest::CloseImmediately => event_loop.exit(),
                        WindowCloseRequest::DoNotCloseYet => {}
                    }
                } else {
                    match app_handler.on_request_to_close_window(window_id, false) {
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

                app_handler.on_window_event(AppWindowEvent::WindowResized, window_id);
            }
            WinitWindowEvent::ScaleChanged {
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

                app_handler.on_window_event(AppWindowEvent::WindowResized, window_id);
            }
            WinitWindowEvent::Focused(focused) => {
                let event = if focused {
                    window_state.handle_window_focused(
                        &mut app_handler.cx.res,
                        &mut app_handler.action_sender,
                    );
                    AppWindowEvent::WindowFocused
                } else {
                    window_state.handle_window_unfocused(
                        &mut app_handler.cx.res,
                        &mut app_handler.action_sender,
                    );
                    AppWindowEvent::WindowUnfocused
                };

                app_handler.on_window_event(event, window_id);
            }
            WinitWindowEvent::Occluded(hidden) => {
                let event = if hidden {
                    window_state.handle_window_hidden(
                        &mut app_handler.cx.res,
                        &mut app_handler.action_sender,
                    );
                    AppWindowEvent::WindowHidden
                } else {
                    window_state.handle_window_shown(
                        &mut app_handler.cx.res,
                        &mut app_handler.action_sender,
                    );
                    AppWindowEvent::WindowShown
                };

                app_handler.on_window_event(event, window_id);
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
                    &mut app_handler.action_sender,
                );
            }
            WinitWindowEvent::CursorLeft { device_id: _ } => {
                window_state
                    .handle_pointer_left(&mut app_handler.cx.res, &mut app_handler.action_sender);
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

                window_state.handle_mouse_wheel(
                    delta_type,
                    &mut app_handler.cx.res,
                    &mut app_handler.action_sender,
                );
            }
            WinitWindowEvent::Destroyed => {
                app_handler.on_window_event(AppWindowEvent::WindowClosed, window_id);

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

                let mut captured = window_state.handle_keyboard_event(
                    key_event.clone(),
                    &mut app_handler.cx.res,
                    &mut app_handler.action_sender,
                ) == EventCaptureStatus::Captured;

                if !captured {
                    if let Some(text) = &event.text {
                        if !text.is_empty() && event.state == ElementState::Pressed {
                            captured |= window_state.handle_text_composition_event(
                                CompositionEvent {
                                    state: CompositionState::Start,
                                    data: String::new(),
                                },
                                &mut app_handler.cx.res,
                                &mut app_handler.action_sender,
                            ) == EventCaptureStatus::Captured;
                            captured |= window_state.handle_text_composition_event(
                                CompositionEvent {
                                    state: CompositionState::End,
                                    data: text.to_string(),
                                },
                                &mut app_handler.cx.res,
                                &mut app_handler.action_sender,
                            ) == EventCaptureStatus::Captured;
                        }
                    }
                }

                if !captured {
                    app_handler.on_keyboard_event(key_event, window_id);
                }
            }
            _ => (),
        }

        if process_updates {
            self.process_updates(event_loop);
        }
        */
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        /*
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
        */
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        /*
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
        */
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OpenWindowError {
    #[error("{0}")]
    OsError(#[from] winit::error::OsError),

    #[error("Error while initializing rendering backend: {0}")]
    InitRendererError(Box<dyn Error>),

    #[cfg(feature = "gl")]
    #[error("Could not build OpenGL window: {0}")]
    BuildGlError(Box<dyn Error>),

    #[cfg(feature = "gl")]
    #[error("Could not build OpenGL surface attributes: {0}")]
    GlSurfaceAttributes(#[from] raw_window_handle::HandleError),
}

pub fn run_blocking<A: Application, R: RenderBackend>(
    config: AppConfig,
) -> Result<(), Box<dyn Error>>
where
    A::Action: Send,
{
    let event_loop = EventLoop::new()?;
    let mut app_handler = WinitAppHandler::<R>::new(config)?;

    event_loop.run_app(&mut app_handler).map_err(Into::into)
}

fn create_window<R: RenderBackend>(
    config: &WindowConfig,
    event_loop: &ActiveEventLoop,
) -> Result<(Arc<winit::window::Window>, CreateWindowResult<R>), OpenWindowError> {
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
            let size: PhysicalSize = yarrow_core::math::to_physical_size(config.size, scale_factor)
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

    #[cfg(feature = "gl")]
    let (window, renderer) = {
        // TODO: Query the renderer on whether a non-gl backend can be used instead.

        let (window, gl_config) = glutin_winit::DisplayBuilder::new()
            .with_window_attributes(Some(attributes))
            .build(
                event_loop,
                glutin::config::ConfigTemplateBuilder::new(),
                R::gl_config_picker,
            )
            .map_err(|e| OpenWindowError::BuildGlError(e))?;

        let window = Arc::new(window.unwrap());

        let size = window.inner_size();
        let size = PhysicalSize::new(size.width as i32, size.height as i32);

        let renderer = R::new_gl(window.window_handle().unwrap().as_raw(), gl_config, size)
            .map_err(|e| OpenWindowError::InitRendererError(Box::new(e)))?;

        (window, renderer)
    };

    #[cfg(not(feature = "gl"))]
    let window = event_loop.create_window(attributes).map(|w| Arc::new(w))?;

    // Might fix an issue in MacOS with wgpu
    // https://github.com/gfx-rs/wgpu/issues/5722
    window.request_redraw();

    let physical_size = window.inner_size();
    let physical_size = PhysicalSize::new(physical_size.width as i32, physical_size.height as i32);
    let system_scale_factor: f32 = window.scale_factor() as f32;

    let scale_factor = config.scale_factor.scale_factor(system_scale_factor);

    /*
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
        id,
    );
    */

    let clipboard = self::clipboard::ClipboardBackend::create(&window);

    Ok((
        window,
        CreateWindowResult {
            renderer,
            clipboard,
            physical_size,
            scale_factor,
        },
    ))
}
