use std::ffi::CString;
use std::num::NonZeroU32;
use std::time::Instant;

use gl_rs as gl;
use glutin::config::Config;
use glutin::context::{NotCurrentGlContext, PossiblyCurrentContext};
use glutin::surface::{GlSurface, Surface as GlutinSurface, WindowSurface};
use raw_window_handle::RawWindowHandle;
use skia_safe::gpu::gl::FramebufferInfo;
use skia_safe::ColorType;
use yarrow_core::math::PhysicalSize;

use crate::InitError;

pub struct GlRendererState {
    pub surface: skia_safe::Surface,
    pub gr_context: skia_safe::gpu::DirectContext,
    pub gl_context: PossiblyCurrentContext,
    pub gl_surface: GlutinSurface<WindowSurface>,
    pub fb_info: FramebufferInfo,
    pub num_samples: usize,
    pub stencil_size: usize,
}

impl GlRendererState {
    pub fn new(
        window: RawWindowHandle,
        gl_config: Config,
        window_size: PhysicalSize,
    ) -> Result<Self, InitError> {
        use glutin::config::GlConfig;
        use glutin::context::{ContextApi, ContextAttributesBuilder};
        use glutin::display::{GetGlDisplay, GlDisplay};
        use glutin::surface::SurfaceAttributesBuilder;

        assert!(!window_size.is_empty());

        // The context creation part. It can be created before surface and that's how
        // it's expected in multithreaded + multiwindow operation mode, since you
        // can send NotCurrentContext, but not Surface.
        let context_attributes = ContextAttributesBuilder::new().build(Some(window));

        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try gles.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(window));

        let not_current_gl_context = unsafe {
            let gl_display = gl_config.display();

            if let Ok(cx) = gl_display.create_context(&gl_config, &context_attributes) {
                cx
            } else {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .map_err(|e| InitError::FailedToCreateGlContext(e))?
            }
        };

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window,
            NonZeroU32::new(window_size.width as u32).unwrap(),
            NonZeroU32::new(window_size.height as u32).unwrap(),
        );

        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .map_err(|e| InitError::FailedToCreateGlSurface(e))?
        };

        let gl_context = not_current_gl_context
            .make_current(&gl_surface)
            .expect("Could not make GL context current when setting up skia renderer");

        gl::load_with(|s| {
            gl_config
                .display()
                .get_proc_address(CString::new(s).unwrap().as_c_str())
        });

        let interface = skia_safe::gpu::gl::Interface::new_load_with(|name| {
            if name == "eglGetCurrentDisplay" {
                return std::ptr::null();
            }
            gl_config
                .display()
                .get_proc_address(CString::new(name).unwrap().as_c_str())
        })
        .ok_or(InitError::FailedToCreateSkiaInterface)?;

        let mut gr_context = skia_safe::gpu::direct_contexts::make_gl(interface, None)
            .ok_or(InitError::FailedToCreateSkiaDirectContext)?;

        let fb_info = {
            let mut fboid: gl::types::GLint = 0;
            unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

            skia_safe::gpu::gl::FramebufferInfo {
                fboid: fboid.try_into().unwrap(),
                format: skia_safe::gpu::gl::Format::RGBA8.into(),
                ..Default::default()
            }
        };

        let num_samples = gl_config.num_samples() as usize;
        let stencil_size = gl_config.stencil_size() as usize;

        let surface = create_surface(
            &fb_info,
            &mut gr_context,
            num_samples,
            stencil_size,
            window_size,
        )?;

        Ok(Self {
            surface,
            gl_context,
            gl_surface,
            gr_context,
            fb_info,
            num_samples,
            stencil_size,
        })
    }

    pub fn resize(&mut self, window_size: PhysicalSize) {
        self.surface = create_surface(
            &self.fb_info,
            &mut self.gr_context,
            self.num_samples,
            self.stencil_size,
            window_size,
        )
        .unwrap();

        self.gl_surface.resize(
            &mut self.gl_context,
            NonZeroU32::new(window_size.width.max(1) as u32).unwrap(),
            NonZeroU32::new(window_size.height.max(1) as u32).unwrap(),
        );
    }

    pub fn render(&mut self) {
        let canvas = self.surface.canvas();

        canvas.clear(skia_safe::Color::BLUE);

        self.gr_context.flush_and_submit();
        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
    }
}

fn create_surface(
    fb_info: &FramebufferInfo,
    gr_context: &mut skia_safe::gpu::DirectContext,
    num_samples: usize,
    stencil_size: usize,
    window_size: PhysicalSize,
) -> Result<skia_safe::Surface, InitError> {
    let backend_render_target = skia_safe::gpu::backend_render_targets::make_gl(
        (window_size.width, window_size.height),
        num_samples,
        stencil_size,
        *fb_info,
    );

    skia_safe::gpu::surfaces::wrap_backend_render_target(
        gr_context,
        &backend_render_target,
        skia_safe::gpu::SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .ok_or(InitError::FailedToCreateSkiaSurface)
}
