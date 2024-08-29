use rootvg::{
    math::{PhysicalPoint, Size},
    text::{glyphon::FontSystem, svg::SvgIconSystem},
};
use rustc_hash::FxHashMap;
use std::{
    error::Error,
    time::{Duration, Instant},
};

use crate::{
    event::{AppWindowEvent, KeyboardEvent},
    prelude::{ActionReceiver, ActionSender},
    style::StyleSystem,
    window::{
        LinuxBackendType, OpenWindowError, PointerLockState, ScaleFactorConfig, WindowBackend,
        WindowCloseRequest, WindowConfig, WindowContext, WindowID, WindowState,
    },
    MAIN_WINDOW,
};

pub trait Application: Sized {
    type Action: Clone + 'static;

    fn init(cx: &mut AppContext<Self::Action>) -> Result<Self, Box<dyn Error>>;

    #[allow(unused)]
    fn on_window_event(
        &mut self,
        event: AppWindowEvent,
        window_id: WindowID,
        cx: &mut AppContext<Self::Action>,
    ) {
    }

    #[allow(unused)]
    fn on_keyboard_event(
        &mut self,
        event: KeyboardEvent,
        window_id: WindowID,
        cx: &mut AppContext<Self::Action>,
    ) {
    }

    #[allow(unused)]
    fn on_action_emitted(&mut self, cx: &mut AppContext<Self::Action>) {}

    #[allow(unused)]
    fn on_tick(&mut self, dt: f64, cx: &mut AppContext<Self::Action>) {}

    #[allow(unused)]
    fn on_request_to_close_window(
        &mut self,
        window_id: WindowID,
        host_will_force_close: bool,
        cx: &mut AppContext<Self::Action>,
    ) -> WindowCloseRequest {
        WindowCloseRequest::CloseImmediately
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimerInterval {
    Fixed(Duration),
    PercentageOfFrameRate(f64),
}

impl Default for TimerInterval {
    fn default() -> Self {
        Self::PercentageOfFrameRate(1.0)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AppConfig {
    pub main_window_config: WindowConfig,
    pub tick_timer_interval: TimerInterval,
    pub pointer_debounce_interval: TimerInterval,
    pub pointer_locking_enabled: bool,
    pub use_dark_theme: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            main_window_config: WindowConfig::default(),
            tick_timer_interval: TimerInterval::PercentageOfFrameRate(1.0),
            pointer_debounce_interval: TimerInterval::PercentageOfFrameRate(2.0),
            pointer_locking_enabled: true,
            use_dark_theme: true,
        }
    }
}

/// A cx for globally-shared resources
pub struct ResourceCtx {
    pub style_system: StyleSystem,
    pub font_system: FontSystem,
    #[cfg(feature = "svg-icons")]
    pub svg_icon_system: SvgIconSystem,
}

impl ResourceCtx {
    pub fn new(use_dark_theme: bool) -> Self {
        Self {
            style_system: StyleSystem::new(use_dark_theme),
            font_system: FontSystem::new(),
            svg_icon_system: SvgIconSystem::default(),
        }
    }
}

pub struct AppContext<A: Clone + 'static> {
    pub(crate) config: AppConfig,
    pub(crate) window_requests: Vec<(WindowID, WindowRequest)>,
    pub(crate) main_window: WindowState<A>,
    pub(crate) window_map: FxHashMap<WindowID, WindowState<A>>,
    pub(crate) linux_backend_type: Option<LinuxBackendType>,
    /// The global resource cx
    pub res: ResourceCtx,

    /// The sending end of the action queue.
    pub action_sender: ActionSender<A>,
    /// The receiving end of the action queue.
    pub action_receiver: ActionReceiver<A>,
}

impl<A: Clone + 'static> AppContext<A> {
    pub(crate) fn new(
        config: AppConfig,
        action_sender: ActionSender<A>,
        action_receiver: ActionReceiver<A>,
        main_window: WindowState<A>,
        res: ResourceCtx,
        linux_backend_type: Option<LinuxBackendType>,
    ) -> Self {
        Self {
            config,
            window_requests: Vec::new(),
            main_window,
            window_map: FxHashMap::default(),
            res,
            linux_backend_type,
            action_sender,
            action_receiver,
        }
    }

    pub fn main_window<'a>(&'a mut self) -> WindowContext<'a, A> {
        self.main_window.context(
            &mut self.res,
            &mut self.action_sender,
            &mut self.action_receiver,
        )
    }

    pub fn window<'a>(&'a mut self, window_id: WindowID) -> Option<WindowContext<'a, A>> {
        self.window_map.get_mut(&window_id).map(|w| {
            w.context(
                &mut self.res,
                &mut self.action_sender,
                &mut self.action_receiver,
            )
        })
    }

    pub fn resize_window(&mut self, window_id: WindowID, logical_size: Size) {
        self.window_requests
            .push((window_id, WindowRequest::Resize(logical_size)));
    }

    pub fn set_minimized(&mut self, window_id: WindowID, minimized: bool) {
        self.window_requests
            .push((window_id, WindowRequest::Minimize(minimized)));
    }

    pub fn set_maximized(&mut self, window_id: WindowID, maximized: bool) {
        self.window_requests
            .push((window_id, WindowRequest::Maximize(maximized)));
    }

    pub fn focus_window(&mut self, window_id: WindowID) {
        self.window_requests.push((window_id, WindowRequest::Focus));
    }

    pub fn close_window(&mut self, window_id: WindowID) {
        self.window_requests.push((window_id, WindowRequest::Close));
    }

    pub fn set_window_title(&mut self, window_id: WindowID, title: String) {
        self.window_requests
            .push((window_id, WindowRequest::SetTitle(title)));
    }

    pub fn set_scale_factor_config(&mut self, window_id: WindowID, config: ScaleFactorConfig) {
        self.window_requests
            .push((window_id, WindowRequest::SetScaleFactor(config)));
    }

    pub fn open_window(&mut self, window_id: WindowID, config: WindowConfig) {
        self.window_requests
            .push((window_id, WindowRequest::Create(config)));
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub fn set_pointer_locking_enabled(&mut self, enabled: bool) {
        self.config.pointer_locking_enabled = enabled;
    }

    pub fn linux_backend_type(&self) -> Option<LinuxBackendType> {
        self.linux_backend_type
    }

    pub fn use_dark_theme(&mut self, use_dark_theme: bool) {
        if self.res.style_system.use_dark_theme != use_dark_theme {
            self.res.style_system.use_dark_theme = use_dark_theme;

            for window_id in self.window_map.keys() {
                self.window_requests
                    .push((*window_id, WindowRequest::NotifyThemeChange));
            }
        }
    }
}

pub(crate) struct AppHandler<A: Application> {
    pub user_app: A,
    pub cx: AppContext<A::Action>,
    pub prev_tick_instant: Instant,
}

impl<A: Application> AppHandler<A> {
    pub fn new(
        main_window: WindowState<A::Action>,
        action_sender: ActionSender<A::Action>,
        action_receiver: ActionReceiver<A::Action>,
        config: AppConfig,
        res: ResourceCtx,
        linux_backend_type: Option<LinuxBackendType>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut cx = AppContext::new(
            config,
            action_sender,
            action_receiver,
            main_window,
            res,
            linux_backend_type,
        );

        let user_app = A::init(&mut cx)?;

        Ok(Self {
            user_app,
            cx,
            prev_tick_instant: Instant::now(),
        })
    }

    pub fn on_tick(&mut self) {
        let now = Instant::now();
        let dt = (now - self.prev_tick_instant).as_secs_f64();
        self.prev_tick_instant = now;

        self.user_app.on_tick(dt, &mut self.cx);

        self.cx.main_window.on_animation_tick(dt, &mut self.cx.res);

        for window_state in self.cx.window_map.values_mut() {
            window_state.on_animation_tick(dt, &mut self.cx.res);
        }
    }

    pub fn process_updates<B: WindowBackend>(&mut self, backend: &mut B) {
        self.drain_pointer_moved_events(backend);

        loop {
            let any_actions_processed = self.poll_actions();

            self.drain_window_requests(backend);

            let mut any_updates_processed = false;
            for (window_id, window_state) in self
                .cx
                .window_map
                .iter_mut()
                .chain([(&MAIN_WINDOW, &mut self.cx.main_window)])
            {
                if window_state.process_updates(&mut self.cx.res) {
                    any_updates_processed = true;
                }

                if window_state.needs_repaint() {
                    backend.request_redraw(*window_id);
                }
            }

            if !any_updates_processed && !any_actions_processed {
                break;
            }
        }

        self.update_pointer_lock_and_cursor(backend);
    }

    fn drain_pointer_moved_events<B: WindowBackend>(&mut self, backend: &mut B) {
        for (window_id, window_state) in self
            .cx
            .window_map
            .iter_mut()
            .chain([(&MAIN_WINDOW, &mut self.cx.main_window)])
        {
            if let Some(delta) = window_state.queued_pointer_delta.take() {
                if window_state.pointer_lock_state().is_locked() {
                    let delta = crate::math::to_logical_point_from_recip(
                        PhysicalPoint::new(delta.0 as f32, delta.1 as f32),
                        window_state.scale_factor_recip,
                    )
                    .to_vector();

                    window_state.handle_locked_pointer_delta(delta, &mut self.cx.res);
                }
            }

            if let Some(pos) = window_state.queued_pointer_position.take() {
                match window_state.pointer_lock_state() {
                    PointerLockState::NotLocked => {
                        window_state.handle_pointer_moved(pos, &mut self.cx.res);
                    }
                    PointerLockState::LockedUsingOS => {
                        // Only send events from the raw device input when locked.
                    }
                    PointerLockState::ManualLock => {
                        if let Some(prev_pos) = window_state.prev_pointer_pos {
                            let new_pos =
                                crate::math::to_physical_point(prev_pos, window_state.scale_factor);

                            #[allow(unused)]
                            if let Err(_) = backend.set_pointer_position(*window_id, new_pos) {
                                backend
                                    .unlock_pointer(*window_id, window_state.pointer_lock_state());
                                window_state.set_pointer_locked(PointerLockState::NotLocked);

                                window_state.handle_pointer_moved(pos, &mut self.cx.res);
                            }
                        }
                    }
                }
            }
        }
    }

    fn drain_window_requests<B: WindowBackend>(&mut self, backend: &mut B) {
        let mut windows_to_close: Vec<WindowID> = Vec::new();
        let mut successful_open_requests: Vec<WindowID> = Vec::new();
        let mut failed_open_requests: Vec<(WindowID, OpenWindowError)> = Vec::new();

        for (window_id, request) in self.cx.window_requests.drain(..) {
            if let WindowRequest::Create(config) = &request {
                if window_id == MAIN_WINDOW || self.cx.window_map.contains_key(&window_id) {
                    log::warn!("Ignored request to create window with ID {window_id}. Window with that ID already exists.")
                }

                match backend.create_window(
                    window_id,
                    config,
                    &self.cx.action_sender,
                    &mut self.cx.res,
                ) {
                    Ok(window_state) => {
                        self.cx.window_map.insert(window_id, window_state);
                        successful_open_requests.push(window_id);
                    }
                    Err(e) => failed_open_requests.push((window_id, e)),
                }

                continue;
            }

            let window_state = if window_id == MAIN_WINDOW {
                &mut self.cx.main_window
            } else if let Some(window_state) = self.cx.window_map.get_mut(&window_id) {
                window_state
            } else {
                log::warn!(
                    "Ignored request {:?} for window {}, window does not exist",
                    request,
                    window_id
                );
                continue;
            };

            match request {
                WindowRequest::Resize(new_size) => {
                    match backend.resize(window_id, new_size, window_state.scale_factor) {
                        Ok(_) => {}
                        Err(_) => {
                            log::warn!(
                                "Failed to set inner size {:?} for window {}",
                                new_size,
                                window_id
                            );
                        }
                    }
                }
                WindowRequest::Minimize(minimized) => {
                    backend.set_minimized(window_id, minimized);
                }
                WindowRequest::Maximize(maximized) => {
                    backend.set_maximized(window_id, maximized);
                }
                WindowRequest::Focus => {
                    backend.focus_window(window_id);
                }
                WindowRequest::Close => {
                    windows_to_close.push(window_id);
                }
                WindowRequest::SetTitle(title) => {
                    backend.set_window_title(window_id, title);
                }
                WindowRequest::SetScaleFactor(config) => {
                    if let Some(new_size) = window_state.set_scale_factor_config(config) {
                        match backend.resize(window_id, new_size, window_state.scale_factor) {
                            Ok(_) => {}
                            Err(_) => {
                                log::warn!(
                                    "Failed to set inner size {:?} for window {}",
                                    new_size,
                                    window_id
                                );
                            }
                        }
                    }
                }
                WindowRequest::NotifyThemeChange => {
                    window_state.on_theme_changed(&mut self.cx.res);
                }
                _ => {}
            }
        }

        for window_id in windows_to_close {
            self.cx.window_map.remove(&window_id);

            backend.close_window(window_id);
        }

        for window_id in successful_open_requests.drain(..) {
            self.user_app
                .on_window_event(AppWindowEvent::WindowOpened, window_id, &mut self.cx);
        }

        for (window_id, error) in failed_open_requests.drain(..) {
            log::error!("Failed to open window {}: {}", window_id, &error);
            self.user_app.on_window_event(
                AppWindowEvent::OpenWindowFailed(error),
                window_id,
                &mut self.cx,
            );
        }
    }

    fn poll_actions(&mut self) -> bool {
        let any_actions_processed = self.cx.action_sender.any_action_sent();
        if any_actions_processed {
            self.user_app.on_action_emitted(&mut self.cx);
        }
        return any_actions_processed;
    }

    fn update_pointer_lock_and_cursor<B: WindowBackend>(&mut self, backend: &mut B) {
        for (window_id, window_state) in self
            .cx
            .window_map
            .iter_mut()
            .chain([(&MAIN_WINDOW, &mut self.cx.main_window)])
        {
            let mut do_unlock_pointer = false;

            let has_focus = backend.has_focus(*window_id);

            if let Some(lock) = window_state.new_pointer_lock_request() {
                if lock
                    && self.cx.config.pointer_locking_enabled
                    && !window_state.pointer_lock_state().is_locked()
                    && has_focus
                {
                    let new_state = backend.try_lock_pointer(*window_id);
                    if new_state.is_locked() {
                        window_state.set_pointer_locked(new_state);
                    }
                } else if (!lock && window_state.pointer_lock_state().is_locked())
                    || (window_state.pointer_lock_state().is_locked() && !has_focus)
                {
                    do_unlock_pointer = true;
                }
            } else if window_state.pointer_lock_state().is_locked() && !has_focus {
                do_unlock_pointer = true;
            }

            if do_unlock_pointer {
                backend.unlock_pointer(*window_id, window_state.pointer_lock_state());

                window_state.set_pointer_locked(PointerLockState::NotLocked);
            }

            if !window_state.pointer_lock_state.is_locked() {
                if let Some(new_icon) = window_state.new_cursor_icon() {
                    backend.set_cursor_icon(*window_id, new_icon);
                }
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum WindowRequest {
    Resize(Size),
    Minimize(bool),
    Maximize(bool),
    Focus,
    Close,
    SetTitle(String),
    SetScaleFactor(ScaleFactorConfig),
    Create(WindowConfig),
    NotifyThemeChange,
}
