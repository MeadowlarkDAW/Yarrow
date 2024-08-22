use std::error::Error;

use baseview::{
    Window as BaseviewWindow, WindowHandler as BaseviewWindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};

use rootvg::math::{PhysicalPoint, PhysicalSizeI32, ScaleFactor};

mod convert;

use super::{ScaleFactorConfig, WindowState, MAIN_WINDOW};
use crate::action_queue::ActionSender;
use crate::application::{AppContext, Application};
use crate::event::{EventCaptureStatus, PointerButton};
use crate::prelude::ActionReceiver;

struct AppHandler<A: Application> {
    user_app: A,
    context: AppContext<A::Action>,
    action_sender: ActionSender<A::Action>,
}

impl<A: Application> AppHandler<A> {
    fn new(
        mut user_app: A,
        action_sender: ActionSender<A::Action>,
        action_reciever: ActionReceiver<A::Action>,
    ) -> Result<Self, Box<dyn Error>> {
        let config = user_app.init()?;

        Ok(Self {
            user_app,
            context: AppContext::new(config, action_sender.clone(), action_reciever),
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
        match event {
            baseview::Event::Mouse(mouse_event) => match mouse_event {
                baseview::MouseEvent::CursorMoved {
                    position,
                    modifiers,
                } => {
                    let pos = PhysicalPoint::new(position.x as f32, position.y as f32);

                    self.context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap()
                        .queued_pointer_position = Some(pos);
                }
                baseview::MouseEvent::ButtonPressed { button, modifiers } => {
                    let button = match button {
                        baseview::MouseButton::Left => PointerButton::Primary,
                        baseview::MouseButton::Middle => PointerButton::Auxiliary,
                        baseview::MouseButton::Right => PointerButton::Secondary,
                        baseview::MouseButton::Back => PointerButton::Fourth,
                        baseview::MouseButton::Forward => PointerButton::Fifth,
                        _ => return baseview::EventStatus::Ignored,
                    };

                    self.context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap()
                        .handle_mouse_button(button, true, &mut self.context.res);
                }
                baseview::MouseEvent::ButtonReleased { button, modifiers } => {
                    let button = match button {
                        baseview::MouseButton::Left => PointerButton::Primary,
                        baseview::MouseButton::Middle => PointerButton::Auxiliary,
                        baseview::MouseButton::Right => PointerButton::Secondary,
                        baseview::MouseButton::Back => PointerButton::Fourth,
                        baseview::MouseButton::Forward => PointerButton::Fifth,
                        _ => return baseview::EventStatus::Ignored,
                    };

                    self.context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap()
                        .handle_mouse_button(button, true, &mut self.context.res);
                }
                baseview::MouseEvent::WheelScrolled { delta, modifiers } => todo!(),
                baseview::MouseEvent::CursorEntered => todo!(),
                baseview::MouseEvent::CursorLeft => todo!(),
                baseview::MouseEvent::DragEntered {
                    position,
                    modifiers,
                    data,
                } => todo!(),
                baseview::MouseEvent::DragMoved {
                    position,
                    modifiers,
                    data,
                } => todo!(),
                baseview::MouseEvent::DragLeft => todo!(),
                baseview::MouseEvent::DragDropped {
                    position,
                    modifiers,
                    data,
                } => todo!(),
            },
            baseview::Event::Keyboard(keyboard_event) => {
                let window_state = self.context.window_map.get_mut(&MAIN_WINDOW).unwrap();

                let key_event = self::convert::convert_keyboard_event(&keyboard_event);

                let mut captured = window_state
                    .handle_keyboard_event(key_event.clone(), &mut self.context.res)
                    == EventCaptureStatus::Captured;

                if !captured {
                    // TODO: composition?

                    self.user_app
                        .on_keyboard_event(key_event, MAIN_WINDOW, &mut self.context);
                }
            }
            baseview::Event::Window(window_event) => match window_event {
                baseview::WindowEvent::Resized(_) => todo!(),
                baseview::WindowEvent::Focused => todo!(),
                baseview::WindowEvent::Unfocused => todo!(),
                baseview::WindowEvent::WillClose => todo!(),
            },
        }

        baseview::EventStatus::Ignored
    }
}

#[derive(thiserror::Error, Debug)]
#[error("oopsie")]
pub struct OpenWindowError;

pub fn run_blocking<A: Application + 'static + Send>(
    user_app: A,
    action_sender: ActionSender<A::Action>,
    action_reciever: ActionReceiver<A::Action>,
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

    // TODO: get rid of unwrap
    let mut app_handler =
        AppHandler::new(user_app, action_sender.clone(), action_reciever).unwrap();

    BaseviewWindow::open_blocking(options, |window: &mut BaseviewWindow| {
        let window_state = WindowState::new(
            window,
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
