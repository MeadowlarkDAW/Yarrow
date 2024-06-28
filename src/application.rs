use rootvg::{math::Size, text::glyphon::FontSystem};
use rustc_hash::FxHashMap;
use std::{error::Error, time::Duration};

use crate::{
    event::{AppWindowEvent, KeyboardEvent},
    window::{
        LinuxBackendType, ScaleFactorConfig, WindowCloseRequest, WindowConfig, WindowContext,
        WindowID, WindowState,
    },
};

pub trait Application {
    type Action: Clone + 'static;

    fn init(&mut self) -> Result<AppConfig, Box<dyn Error>> {
        Ok(AppConfig::default())
    }

    fn main_window_config(&self) -> WindowConfig {
        WindowConfig::default()
    }

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
pub enum TimerInterval {
    Fixed(Duration),
    PercentageOfFrameRate(f64),
}

impl Default for TimerInterval {
    fn default() -> Self {
        Self::PercentageOfFrameRate(1.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub tick_timer_interval: TimerInterval,
    pub pointer_debounce_interval: TimerInterval,
    pub pointer_locking_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tick_timer_interval: TimerInterval::PercentageOfFrameRate(1.0),
            pointer_debounce_interval: TimerInterval::PercentageOfFrameRate(2.0),
            pointer_locking_enabled: true,
        }
    }
}

pub struct AppContext<A: Clone + 'static> {
    pub(crate) config: AppConfig,
    pub(crate) window_requests: Vec<(WindowID, WindowRequest)>,
    pub(crate) window_map: FxHashMap<WindowID, WindowState<A>>,
    pub(crate) linux_backend_type: Option<LinuxBackendType>,
    pub font_system: FontSystem,
}

impl<A: Clone + 'static> AppContext<A> {
    pub fn window_context<'a>(&'a mut self, window_id: WindowID) -> Option<WindowContext<'a, A>> {
        self.window_map
            .get_mut(&window_id)
            .map(|w| w.context(&mut self.font_system))
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
}

impl<A: Clone + 'static> AppContext<A> {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            window_requests: Vec::new(),
            window_map: FxHashMap::default(),
            font_system: FontSystem::new(),
            linux_backend_type: None,
        }
    }
}

pub(crate) enum WindowRequest {
    Resize(Size),
    Minimize(bool),
    Maximize(bool),
    Focus,
    Close,
    SetTitle(String),
    SetScaleFactor(ScaleFactorConfig),
    Create(WindowConfig),
}
