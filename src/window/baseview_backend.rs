use std::error::Error;
use std::sync::Arc;

use baseview::{
    Window as BaseviewWindow, WindowHandler as BaseviewWindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use rootvg::math::{PhysicalSizeI32, ScaleFactor, Size};

use super::{ScaleFactorConfig, WindowState, MAIN_WINDOW};
use crate::action_queue::ActionSender;
use crate::application::{AppContext, Application};

struct AppHandler<A: Application> {
    user_app: A,
    context: AppContext<A::Action>,
    action_sender: ActionSender<A::Action>,
}

impl<A: Application> AppHandler<A> {
    fn new(
        mut user_app: A,
        action_sender: ActionSender<A::Action>,
    ) -> Result<Self, Box<dyn Error>> {
        let config = user_app.init()?;

        Ok(Self {
            user_app,
            context: AppContext::new(config),
            action_sender,
        })
    }
}

impl<A: Application> BaseviewWindowHandler for AppHandler<A> {
    fn on_frame(&mut self, window: &mut BaseviewWindow) {
        // TODO: unwrap
        let window_state = self.context.window_map.get_mut(&MAIN_WINDOW).unwrap();
        window_state.render(|| {}, &mut self.context.res).unwrap();
    }
    fn on_event(
        &mut self,
        window: &mut BaseviewWindow,
        event: baseview::Event,
    ) -> baseview::EventStatus {
        // TODO:

        baseview::EventStatus::Ignored
    }
}

pub struct BaseviewWindowWrapper<'bv_window_wrapper> {
    bv_window: &'bv_window_wrapper BaseviewWindow<'bv_window_wrapper>,
}

impl HasWindowHandle for BaseviewWindowWrapper<'_> {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        todo!()
    }
}

impl HasDisplayHandle for BaseviewWindowWrapper<'_> {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        todo!()
    }
}

// TODO: ok figure out if this is really a good idea
unsafe impl Sync for BaseviewWindowWrapper<'_> {}
unsafe impl Send for BaseviewWindowWrapper<'_> {}

pub fn run_blocking<A: Application + 'static + Send>(
    user_app: A,
    action_sender: ActionSender<A::Action>,
) -> Result<(), Box<dyn Error>>
where
    <A as Application>::Action: Send,
{
    let config = user_app.main_window_config();
    let title = config.title.clone();
    let scale = match config.scale_factor {
        ScaleFactorConfig::System => WindowScalePolicy::SystemScaleFactor,
        ScaleFactorConfig::Custom(c) => WindowScalePolicy::ScaleFactor(c.into()),
    };
    let size = baseview::Size::new(config.size.width as f64, config.size.height as f64);
    let options = WindowOpenOptions { title, scale, size };

    BaseviewWindow::open_blocking(options, move |window: &mut BaseviewWindow| {
        // TODO: get rid of unwrap
        let mut app_handler = AppHandler::new(user_app, action_sender.clone()).unwrap();
        let window_state = WindowState::new(
            &Arc::new(BaseviewWindowWrapper { bv_window: window }),
            config.size,
            // TODO:
            PhysicalSizeI32::new(config.size.width as i32, config.size.height as i32),
            // TODO:
            ScaleFactor::new(0.0),
            config.scale_factor,
            config.view_config,
            config.surface_config,
            action_sender.clone(),
            MAIN_WINDOW,
            &mut app_handler.context.res,
        )
        .unwrap();

        app_handler
            .context
            .window_map
            .insert(MAIN_WINDOW, window_state);

        app_handler
    });

    Ok(())
}
