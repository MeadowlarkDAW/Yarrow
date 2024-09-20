use raw_window_handle::RawWindowHandle;
use std::error::Error;

use crate::math::PhysicalSize;

pub trait RenderBackend: Sized {
    type InitError: Error + 'static;

    #[allow(unused)]
    fn new_non_gl(window: RawWindowHandle) -> Option<Result<Self, Self::InitError>> {
        None
    }

    fn resize(&mut self, window_size: PhysicalSize);

    fn render(&mut self);

    #[cfg(feature = "gl")]
    fn gl_config_picker(
        configs: Box<dyn Iterator<Item = glutin::config::Config> + '_>,
    ) -> glutin::config::Config;

    #[cfg(feature = "gl")]
    fn new_gl(
        window: RawWindowHandle,
        gl_config: glutin::config::Config,
        size: PhysicalSize,
    ) -> Result<Self, Self::InitError>;
}
