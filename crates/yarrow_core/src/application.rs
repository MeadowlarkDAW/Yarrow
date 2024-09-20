use crate::window::WindowConfig;

/// The core Application trait in Yarrow.
pub trait Application: Sized {
    /// The type to use as the application's `Action`. This will typically be an `enum`.
    type Action: Clone + 'static;
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimerInterval {
    FixedSecs(f64),
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
