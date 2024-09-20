use raw_window_handle::RawWindowHandle;
use yarrow_core::math::PhysicalSize;

mod gl_renderer;

pub struct SkiaRenderer {
    #[cfg(feature = "gl")]
    gl_state: Option<gl_renderer::GlRendererState>,
}

impl yarrow_core::renderer::RenderBackend for SkiaRenderer {
    type InitError = InitError;

    fn resize(&mut self, window_size: PhysicalSize) {
        if let Some(gl_state) = &mut self.gl_state {
            gl_state.resize(window_size);
        }
    }

    fn render(&mut self) {
        if let Some(gl_state) = &mut self.gl_state {
            gl_state.render();
        }
    }

    #[cfg(feature = "gl")]
    fn new_gl(
        window: RawWindowHandle,
        gl_config: glutin::config::Config,
        size: PhysicalSize,
    ) -> Result<Self, Self::InitError> {
        let gl_state = Some(gl_renderer::GlRendererState::new(window, gl_config, size)?);

        Ok(Self { gl_state })
    }

    #[cfg(feature = "gl")]
    fn gl_config_picker(
        configs: Box<dyn Iterator<Item = glutin::config::Config> + '_>,
    ) -> glutin::config::Config {
        use glutin::config::GlConfig;

        // Find the config with the minimum number of samples. Usually Skia takes care of
        // anti-aliasing and may not be able to create appropriate Surfaces for samples > 0.
        // See https://github.com/rust-skia/rust-skia/issues/782
        // And https://github.com/rust-skia/rust-skia/issues/764
        configs
            .reduce(|accum, config| {
                if config.num_samples() < accum.num_samples() {
                    config
                } else {
                    accum
                }
            })
            .expect("No compatible GLConfig found.")
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InitError {
    #[error("Failed to create Skia interface")]
    FailedToCreateSkiaInterface,
    #[error("Failed to create Skia direct context")]
    FailedToCreateSkiaDirectContext,
    #[error("Failed to create Skia surface")]
    FailedToCreateSkiaSurface,

    #[cfg(feature = "gl")]
    #[error("Failed to create OpenGL context: {0}")]
    FailedToCreateGlContext(glutin::error::Error),

    #[cfg(feature = "gl")]
    #[error("Failed to create OpenGL surface: {0}")]
    FailedToCreateGlSurface(glutin::error::Error),
}
