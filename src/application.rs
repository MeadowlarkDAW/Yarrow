use rootvg::{math::Size, text::glyphon::FontSystem};
use rustc_hash::FxHashMap;
use std::{error::Error, time::Duration};

use crate::{
    event::{AppWindowEvent, KeyboardEvent},
    window::{WindowCloseRequest, WindowConfig, WindowContext, WindowID, WindowState},
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
    pub cursor_debounce_interval: TimerInterval,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tick_timer_interval: TimerInterval::PercentageOfFrameRate(1.0),
            cursor_debounce_interval: TimerInterval::PercentageOfFrameRate(2.0),
        }
    }
}

pub struct AppContext<A: Clone + 'static> {
    pub(crate) window_requests: Vec<(WindowID, WindowRequest)>,
    pub(crate) window_map: FxHashMap<WindowID, WindowState<A>>,
    pub(crate) tick_interval: Duration,
    pub(crate) cursor_debounce_interval: Duration,
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

    pub fn open_window(&mut self, window_id: WindowID, config: WindowConfig) {
        self.window_requests
            .push((window_id, WindowRequest::Create(config)));
    }

    pub fn tick_interval(&self) -> Duration {
        self.tick_interval
    }

    pub fn cursor_debounce_interval(&self) -> Duration {
        self.cursor_debounce_interval
    }
}

impl<A: Clone + 'static> Default for AppContext<A> {
    fn default() -> Self {
        Self {
            window_requests: Vec::new(),
            window_map: FxHashMap::default(),
            tick_interval: Duration::default(),
            cursor_debounce_interval: Duration::default(),
            font_system: FontSystem::new(),
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
    Create(WindowConfig),
}
