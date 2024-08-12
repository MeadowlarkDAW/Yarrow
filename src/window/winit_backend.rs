use keyboard_types::{CompositionEvent, CompositionState};
use rootvg::math::{PhysicalPoint, Vector};
use rustc_hash::FxHashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler as WinitApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{
    ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, StartCause,
    WindowEvent as WinitWindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window as WinitWindow, WindowId as WinitWindowId};

use crate::action_queue::ActionSender;
use crate::application::{AppContext, Application, TimerInterval, WindowRequest};
use crate::event::{AppWindowEvent, EventCaptureStatus, PointerButton, WheelDeltaType};
use crate::math::{PhysicalSizeI32, ScaleFactor};
use crate::prelude::ResourceCtx;
use crate::window::{WindowID, MAIN_WINDOW};

use super::{
    LinuxBackendType, PointerLockState, ScaleFactorConfig, WindowCloseRequest, WindowConfig,
    WindowState,
};

mod convert;

struct AppHandler<A: Application> {
    user_app: A,
    context: AppContext<A::Action>,
    action_sender: ActionSender<A::Action>,

    tick_interval: Duration,
    pointer_debounce_interval: Duration,
    prev_tick_instant: Instant,
    prev_cursor_debounce_instant: Instant,
    requested_tick_resume: Instant,
    requested_cursor_debounce_resume: Option<Instant>,

    winit_window_map: FxHashMap<WinitWindowId, (WindowID, Arc<WinitWindow>)>,

    tick_wait_cancelled: bool,
    got_first_resumed_event: bool,
}

impl<A: Application> AppHandler<A> {
    fn new(
        mut user_app: A,
        action_sender: ActionSender<A::Action>,
    ) -> Result<Self, Box<dyn Error>> {
        let config = user_app.init()?;

        let now = Instant::now();

        Ok(Self {
            user_app,
            action_sender,
            context: AppContext::new(config),
            tick_interval: Duration::default(),
            pointer_debounce_interval: Duration::default(),
            prev_tick_instant: now,
            prev_cursor_debounce_instant: now,
            requested_tick_resume: now,
            requested_cursor_debounce_resume: None,
            winit_window_map: FxHashMap::default(),
            tick_wait_cancelled: false,
            got_first_resumed_event: false,
        })
    }

    fn on_tick(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = (now - self.prev_tick_instant).as_secs_f64();
        self.prev_tick_instant = now;

        self.user_app.on_tick(dt, &mut self.context);

        for window_state in self.context.window_map.values_mut() {
            window_state.on_animation_tick(dt, &mut self.context.res);
        }

        self.process_updates(event_loop);
    }

    fn process_updates(&mut self, event_loop: &ActiveEventLoop) {
        self.drain_pointer_moved_events();

        loop {
            let any_actions_processed = self.poll_actions();

            self.drain_window_requests(event_loop);

            let mut any_updates_processed = false;
            for window_state in self.context.window_map.values_mut() {
                if window_state
                    .view
                    .process_updates(&mut self.context.res, &mut window_state.clipboard)
                {
                    any_updates_processed = true;
                }

                if window_state.view.view_needs_repaint() {
                    window_state.winit_window.request_redraw();
                }
            }

            if !any_updates_processed && !any_actions_processed {
                break;
            }
        }

        self.update_pointer_lock_and_cursor();
    }

    fn drain_pointer_moved_events(&mut self) {
        for window_state in self.context.window_map.values_mut() {
            if let Some(delta) = window_state.queued_pointer_delta.take() {
                if window_state.pointer_lock_state().is_locked() {
                    let delta = crate::math::to_logical_point_from_recip(
                        PhysicalPoint::new(delta.0 as f32, delta.1 as f32),
                        window_state.scale_factor_recip,
                    )
                    .to_vector();

                    window_state.handle_locked_pointer_delta(delta, &mut self.context.res);
                }
            }

            if let Some(pos) = window_state.queued_pointer_position.take() {
                match window_state.pointer_lock_state() {
                    PointerLockState::NotLocked => {
                        window_state.handle_pointer_moved(pos, &mut self.context.res);
                    }
                    PointerLockState::LockedUsingOS => {
                        // Only send events from the raw device input when locked.
                    }
                    PointerLockState::ManualLock => {
                        if let Some(prev_pos) = window_state.prev_pointer_pos {
                            let new_pos =
                                crate::math::to_physical_point(prev_pos, window_state.scale_factor);

                            if let Err(e) = window_state
                                .winit_window
                                .set_cursor_position(PhysicalPosition::new(new_pos.x, new_pos.y))
                            {
                                log::debug!("Could not set cursor position: {}", e);

                                unlock_pointer(
                                    &window_state.winit_window,
                                    window_state.pointer_lock_state(),
                                );
                                window_state.set_pointer_locked(PointerLockState::NotLocked);

                                window_state.handle_pointer_moved(pos, &mut self.context.res);
                            }
                        }
                    }
                }
            }
        }

        self.requested_cursor_debounce_resume = None;
    }

    fn poll_actions(&mut self) -> bool {
        let any_actions_processed = self.action_sender.any_action_sent();
        if any_actions_processed {
            self.user_app.on_action_emitted(&mut self.context);
        }
        return any_actions_processed;
    }

    fn drain_window_requests(&mut self, event_loop: &ActiveEventLoop) {
        let mut windows_to_close: Vec<WindowID> = Vec::new();
        let mut successful_open_requests: Vec<WindowID> = Vec::new();
        let mut failed_open_requests: Vec<(WindowID, OpenWindowError)> = Vec::new();

        for (window_id, request) in self.context.window_requests.drain(..) {
            match request {
                WindowRequest::Resize(new_size) => {
                    if let Some(window_state) = self.context.window_map.get_mut(&window_id) {
                        match window_state
                            .winit_window
                            .request_inner_size(LogicalSize::new(new_size.width, new_size.height))
                        {
                            Some(_) => {}
                            None => {
                                log::warn!(
                                    "Failed to set inner size {:?} for window {}",
                                    LogicalSize::new(new_size.width, new_size.height),
                                    window_id
                                );
                            }
                        }
                    } else {
                        log::warn!("Ignored request to set window size for window {}, window does not exist", window_id);
                    }
                }
                WindowRequest::Minimize(minimized) => {
                    if let Some(window_state) = self.context.window_map.get_mut(&window_id) {
                        window_state.winit_window.set_minimized(minimized);
                    } else {
                        log::warn!(
                            "Ignored request to minimize window {}, window does not exist",
                            window_id
                        );
                    }
                }
                WindowRequest::Maximize(maximized) => {
                    if let Some(window_state) = self.context.window_map.get_mut(&window_id) {
                        window_state.winit_window.set_maximized(maximized);
                    } else {
                        log::warn!(
                            "Ignored request to maximize window {}, window does not exist",
                            window_id
                        );
                    }
                }
                WindowRequest::Focus => {
                    if let Some(window_state) = self.context.window_map.get_mut(&window_id) {
                        window_state.winit_window.focus_window();
                    } else {
                        log::warn!(
                            "Ignored request to focus window {}, window does not exist",
                            window_id
                        );
                    }
                }
                WindowRequest::Close => {
                    if self.context.window_map.contains_key(&window_id) {
                        windows_to_close.push(window_id);
                    } else {
                        log::warn!(
                            "Ignored request to focus window {}, window does not exist",
                            window_id
                        );
                    }
                }
                WindowRequest::SetTitle(title) => {
                    if let Some(window_state) = self.context.window_map.get_mut(&window_id) {
                        window_state.winit_window.set_title(&title);
                    } else {
                        log::warn!(
                            "Ignored request to set title for window {}, window does not exist",
                            window_id
                        );
                    }
                }
                WindowRequest::SetScaleFactor(config) => {
                    if let Some(window_state) = self.context.window_map.get_mut(&window_id) {
                        if let Some(_requested_physical_size) =
                            window_state.set_scale_factor_config(config)
                        {
                            // TODO: Find a way to resize winit window from code.
                        }
                    } else {
                        log::warn!(
                            "Ignored request to set scale factor for window {}, window does not exist",
                            window_id
                        );
                    }
                }
                WindowRequest::Create(config) => {
                    if self.context.window_map.contains_key(&window_id) {
                        log::warn!(
                            "Ignored request to create window {}, window already exists",
                            window_id
                        );
                        continue;
                    }

                    match create_window(
                        window_id,
                        config,
                        event_loop,
                        &self.action_sender,
                        &mut self.context.res,
                    ) {
                        Ok((window, window_state)) => {
                            self.winit_window_map
                                .insert(window.id(), (window_id, window));
                            self.context.window_map.insert(window_id, window_state);

                            successful_open_requests.push(window_id);
                        }
                        Err(e) => failed_open_requests.push((window_id, e)),
                    }
                }
                WindowRequest::NotifyThemeChange => {
                    if let Some(window_state) = self.context.window_map.get_mut(&window_id) {
                        window_state.on_theme_changed(&mut self.context.res);
                    }
                }
            }
        }

        for window_id in windows_to_close {
            let winit_window_id = self
                .context
                .window_map
                .get(&window_id)
                .unwrap()
                .winit_window
                .id();

            self.winit_window_map.remove(&winit_window_id);
            self.context.window_map.remove(&window_id);
        }

        for window_id in successful_open_requests.drain(..) {
            self.user_app.on_window_event(
                AppWindowEvent::WindowOpened,
                window_id,
                &mut self.context,
            );
        }

        for (window_id, error) in failed_open_requests.drain(..) {
            log::error!("Failed to open window {}: {}", window_id, &error);
            self.user_app.on_window_event(
                AppWindowEvent::OpenWindowFailed(error),
                window_id,
                &mut self.context,
            );
        }
    }

    fn update_pointer_lock_and_cursor(&mut self) {
        for window_state in self.context.window_map.values_mut() {
            let mut do_unlock_pointer = false;

            if let Some(lock) = window_state.new_pointer_lock_request() {
                if lock
                    && self.context.config.pointer_locking_enabled
                    && !window_state.pointer_lock_state().is_locked()
                    && window_state.winit_window.has_focus()
                {
                    let new_state = try_lock_pointer(
                        &window_state.winit_window,
                        #[cfg(any(
                            target_os = "linux",
                            target_os = "freebsd",
                            target_os = "dragonfly",
                            target_os = "openbsd",
                            target_os = "netbsd",
                        ))]
                        self.context.linux_backend_type,
                    );
                    if new_state.is_locked() {
                        window_state.set_pointer_locked(new_state);
                    }
                } else if (!lock && window_state.pointer_lock_state().is_locked())
                    || (window_state.pointer_lock_state().is_locked()
                        && !window_state.winit_window.has_focus())
                {
                    do_unlock_pointer = true;
                }
            } else if window_state.pointer_lock_state().is_locked()
                && !window_state.winit_window.has_focus()
            {
                do_unlock_pointer = true;
            }

            if do_unlock_pointer {
                unlock_pointer(
                    &window_state.winit_window,
                    window_state.pointer_lock_state(),
                );

                window_state.set_pointer_locked(PointerLockState::NotLocked);
            }

            if !window_state.pointer_lock_state.is_locked() {
                if let Some(new_icon) = window_state.new_cursor_icon() {
                    let winit_icon = self::convert::convert_cursor_icon_to_winit(new_icon);
                    window_state.winit_window.set_cursor(winit_icon);
                }
            }
        }
    }
}

impl<A: Application> WinitApplicationHandler for AppHandler<A> {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.tick_wait_cancelled = false;

        match cause {
            StartCause::ResumeTimeReached {
                requested_resume, ..
            } => {
                if requested_resume == self.requested_tick_resume {
                    self.on_tick(event_loop);
                } else if let Some(pointer_resume_instant) = self.requested_cursor_debounce_resume {
                    if pointer_resume_instant == requested_resume {
                        self.process_updates(event_loop);
                    }
                }
            }
            StartCause::WaitCancelled { .. } => self.tick_wait_cancelled = true,
            _ => {}
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.got_first_resumed_event {
            self.got_first_resumed_event = true;

            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd",
            ))]
            {
                use winit::platform::wayland::ActiveEventLoopExtWayland;
                use winit::platform::x11::ActiveEventLoopExtX11;

                self.context.linux_backend_type = if event_loop.is_x11() {
                    Some(LinuxBackendType::X11)
                } else if event_loop.is_wayland() {
                    Some(LinuxBackendType::Wayland)
                } else {
                    log::warn!("Could not parse whether windowing backend is X11 or Wayland");
                    None
                };
            }

            let (main_window, main_window_state) = match create_window(
                MAIN_WINDOW,
                self.user_app.main_window_config(),
                event_loop,
                &self.action_sender,
                &mut self.context.res,
            ) {
                Ok(w) => w,
                Err(e) => {
                    log::error!("Failed to open main window: {}", e);
                    event_loop.exit();
                    return;
                }
            };

            let find_millihertz = if let TimerInterval::PercentageOfFrameRate(_) =
                self.context.config.tick_timer_interval
            {
                true
            } else if let TimerInterval::PercentageOfFrameRate(_) =
                self.context.config.pointer_debounce_interval
            {
                true
            } else {
                false
            };
            let millihertz = if find_millihertz {
                // Attempt to get the refresh rate of the current monitor. If that's
                // not possible, try other methods.
                let mut millihertz = None;
                if let Some(monitor) = main_window.current_monitor() {
                    millihertz = monitor.refresh_rate_millihertz();
                }
                if millihertz.is_none() {
                    if let Some(monitor) = main_window.primary_monitor() {
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

            self.tick_interval = match self.context.config.tick_timer_interval {
                TimerInterval::Fixed(interval) => interval,
                TimerInterval::PercentageOfFrameRate(percentage) => {
                    Duration::from_secs_f64(percentage * 1_000.0 / millihertz as f64)
                }
            };
            self.pointer_debounce_interval = match self.context.config.pointer_debounce_interval {
                TimerInterval::Fixed(interval) => interval,
                TimerInterval::PercentageOfFrameRate(percentage) => {
                    Duration::from_secs_f64(percentage * 1_000.0 / millihertz as f64)
                }
            };

            self.context
                .window_map
                .insert(MAIN_WINDOW, main_window_state);
            self.winit_window_map
                .insert(main_window.id(), (MAIN_WINDOW, main_window));

            self.user_app.on_window_event(
                AppWindowEvent::WindowOpened,
                MAIN_WINDOW,
                &mut self.context,
            );

            self.process_updates(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        winit_window_id: WinitWindowId,
        event: WinitWindowEvent,
    ) {
        let Some((window_id, window)) = self.winit_window_map.get(&winit_window_id) else {
            return;
        };
        let window_id = *window_id;

        let mut process_updates = true;

        match event {
            WinitWindowEvent::CloseRequested => {
                if window_id == MAIN_WINDOW {
                    match self.user_app.on_request_to_close_window(
                        MAIN_WINDOW,
                        false,
                        &mut self.context,
                    ) {
                        WindowCloseRequest::CloseImmediately => event_loop.exit(),
                        WindowCloseRequest::DoNotCloseYet => {}
                    }
                } else {
                    match self.user_app.on_request_to_close_window(
                        window_id,
                        false,
                        &mut self.context,
                    ) {
                        WindowCloseRequest::CloseImmediately => {
                            self.winit_window_map.remove(&winit_window_id);
                            self.context.window_map.remove(&window_id);
                        }
                        WindowCloseRequest::DoNotCloseYet => {}
                    }
                }
            }
            WinitWindowEvent::RedrawRequested => {
                process_updates = false;

                let window_state = self.context.window_map.get_mut(&window_id).unwrap();

                match window_state.render(|| window.pre_present_notify(), &mut self.context.res) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {
                        let inner_size = window.inner_size();
                        let new_size =
                            PhysicalSizeI32::new(inner_size.width as i32, inner_size.height as i32);
                        let new_scale_factor = window.scale_factor().into();

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

                let window_state = self.context.window_map.get_mut(&window_id).unwrap();

                let scale_factor = window.scale_factor().into();
                window_state.set_size(new_size, scale_factor);
                window.request_redraw();

                self.user_app.on_window_event(
                    AppWindowEvent::WindowResized,
                    window_id,
                    &mut self.context,
                );
            }
            WinitWindowEvent::ScaleFactorChanged {
                scale_factor,
                mut inner_size_writer,
            } => {
                let window_state = self.context.window_map.get_mut(&window_id).unwrap();

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
                window.request_redraw();

                self.user_app.on_window_event(
                    AppWindowEvent::WindowResized,
                    window_id,
                    &mut self.context,
                );
            }
            WinitWindowEvent::Focused(focused) => {
                let window_state = self.context.window_map.get_mut(&window_id).unwrap();

                let event = if focused {
                    window_state.handle_window_focused(&mut self.context.res);
                    AppWindowEvent::WindowFocused
                } else {
                    window_state.handle_window_unfocused(&mut self.context.res);
                    AppWindowEvent::WindowUnfocused
                };

                self.user_app
                    .on_window_event(event, window_id, &mut self.context);
            }
            WinitWindowEvent::Occluded(hidden) => {
                let window_state = self.context.window_map.get_mut(&window_id).unwrap();

                let event = if hidden {
                    window_state.handle_window_hidden(&mut self.context.res);
                    AppWindowEvent::WindowHidden
                } else {
                    window_state.handle_window_shown(&mut self.context.res);
                    AppWindowEvent::WindowShown
                };

                self.user_app
                    .on_window_event(event, window_id, &mut self.context);
            }
            WinitWindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                let pos = PhysicalPoint::new(position.x as f32, position.y as f32);

                self.context
                    .window_map
                    .get_mut(&window_id)
                    .unwrap()
                    .queued_pointer_position = Some(pos);

                let now = Instant::now();
                if now - self.prev_cursor_debounce_instant < self.pointer_debounce_interval {
                    process_updates = false;

                    // Make sure that the latest cursor move event is always sent.
                    let mut resume_instant = now + self.pointer_debounce_interval;
                    if resume_instant == self.requested_tick_resume {
                        // Make sure we don't clash with the tick timer.
                        resume_instant += Duration::from_micros(1);
                    }
                    self.requested_cursor_debounce_resume = Some(resume_instant);

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

                self.context
                    .window_map
                    .get_mut(&window_id)
                    .unwrap()
                    .handle_mouse_button(button, state.is_pressed(), &mut self.context.res);

                self.process_updates(event_loop);
            }
            WinitWindowEvent::CursorLeft { device_id: _ } => {
                self.context
                    .window_map
                    .get_mut(&window_id)
                    .unwrap()
                    .handle_pointer_left(&mut self.context.res);
            }
            WinitWindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _,
            } => {
                let delta_type = match delta {
                    MouseScrollDelta::LineDelta(x, y) => WheelDeltaType::Lines(Vector::new(x, -y)),
                    MouseScrollDelta::PixelDelta(pos) => WheelDeltaType::Points(Vector::new(
                        (pos.x / window.scale_factor()) as f32,
                        (-pos.y / window.scale_factor()) as f32,
                    )),
                };

                self.context
                    .window_map
                    .get_mut(&window_id)
                    .unwrap()
                    .handle_mouse_wheel(delta_type, &mut self.context.res);
            }
            WinitWindowEvent::Destroyed => {
                self.user_app.on_window_event(
                    AppWindowEvent::WindowClosed,
                    window_id,
                    &mut self.context,
                );

                self.context.window_map.remove(&window_id);
                self.winit_window_map.remove(&winit_window_id);
            }
            WinitWindowEvent::ModifiersChanged(winit_modifiers) => {
                let modifiers = self::convert::convert_modifiers(winit_modifiers);

                self.context
                    .window_map
                    .get_mut(&window_id)
                    .unwrap()
                    .set_modifiers(modifiers);
            }
            WinitWindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let window_state = self.context.window_map.get_mut(&window_id).unwrap();

                let key_event =
                    self::convert::convert_keyboard_event(&event, window_state.modifiers);

                let mut captured = window_state
                    .handle_keyboard_event(key_event.clone(), &mut self.context.res)
                    == EventCaptureStatus::Captured;

                if !captured {
                    if let Some(text) = &event.text {
                        if !text.is_empty() && event.state == ElementState::Pressed {
                            captured |= window_state.handle_text_composition_event(
                                CompositionEvent {
                                    state: CompositionState::Start,
                                    data: String::new(),
                                },
                                &mut self.context.res,
                            ) == EventCaptureStatus::Captured;
                            captured |= window_state.handle_text_composition_event(
                                CompositionEvent {
                                    state: CompositionState::End,
                                    data: text.to_string(),
                                },
                                &mut self.context.res,
                            ) == EventCaptureStatus::Captured;
                        }
                    }
                }

                if !captured {
                    self.user_app
                        .on_keyboard_event(key_event, window_id, &mut self.context);
                }

                process_updates = true;
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
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            for window in self.context.window_map.values_mut() {
                if window.pointer_lock_state().is_locked() {
                    if let Some(prev_delta) = &mut window.queued_pointer_delta {
                        prev_delta.0 += delta.0;
                        prev_delta.1 += delta.1;
                    } else {
                        window.queued_pointer_delta = Some(delta);
                    }

                    let now = Instant::now();
                    if now - self.prev_cursor_debounce_instant < self.pointer_debounce_interval {
                        // Make sure that the latest cursor move event is always sent.
                        let mut resume_instant = now + self.pointer_debounce_interval;
                        if resume_instant == self.requested_tick_resume {
                            // Make sure we don't clash with the tick timer.
                            resume_instant += Duration::from_micros(1);
                        }
                        self.requested_cursor_debounce_resume = Some(resume_instant);

                        event_loop.set_control_flow(ControlFlow::WaitUntil(resume_instant));
                    }
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.tick_wait_cancelled {
            let now = Instant::now();

            let mut next_instant = if self.prev_tick_instant + self.tick_interval > now {
                self.prev_tick_instant + self.tick_interval
            } else {
                self.on_tick(event_loop);

                now + self.tick_interval
            };

            if let Some(pointer_resume_instant) = self.requested_cursor_debounce_resume {
                if next_instant == pointer_resume_instant {
                    // Make sure we don't clash with the pointer debounce timer.
                    next_instant += Duration::from_micros(1);
                }
            }

            self.requested_tick_resume = next_instant;

            event_loop.set_control_flow(ControlFlow::WaitUntil(next_instant));
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {}
}

#[derive(thiserror::Error, Debug)]
pub enum OpenWindowError {
    #[error("{0}")]
    OsError(#[from] winit::error::OsError),
    #[error("{0}")]
    SurfaceError(#[from] rootvg::surface::NewSurfaceError),
}

pub fn run_blocking<A: Application>(
    app: A,
    action_sender: ActionSender<A::Action>,
) -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app_handler = AppHandler::new(app, action_sender)?;

    event_loop.run_app(&mut app_handler).map_err(Into::into)
}

fn create_window<A: Clone + 'static>(
    id: WindowID,
    config: WindowConfig,
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

    let window_state = WindowState::new(
        &window,
        config.size,
        physical_size,
        system_scale_factor,
        config.scale_factor,
        config.view_config,
        config.surface_config,
        action_sender.clone(),
        id,
        res,
    )?;

    Ok((window, window_state))
}

fn try_lock_pointer(
    window: &WinitWindow,
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd",
    ))]
    linux_backend_type: Option<LinuxBackendType>,
) -> PointerLockState {
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
            match linux_backend_type {
                Some(LinuxBackendType::Wayland) => try_os_lock = true,
                Some(LinuxBackendType::X11) => try_manual_lock = true,
                _ => {}
            };
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
        match window.set_cursor_grab(CursorGrabMode::Locked) {
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
        window.set_cursor_visible(false);
        state
    } else if try_manual_lock {
        window.set_cursor_visible(false);
        PointerLockState::ManualLock
    } else {
        PointerLockState::NotLocked
    }
}

fn unlock_pointer(window: &WinitWindow, prev_state: PointerLockState) {
    match prev_state {
        PointerLockState::LockedUsingOS => {
            if let Err(e) = window.set_cursor_grab(CursorGrabMode::None) {
                log::error!("Error while unlocking pointer: {}", e);
            }
            window.set_cursor_visible(true);
        }
        PointerLockState::ManualLock { .. } => {
            window.set_cursor_visible(true);
        }
        _ => {}
    }
}
