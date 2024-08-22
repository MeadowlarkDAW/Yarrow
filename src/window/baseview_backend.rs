use baseview::{
    Window as BaseviewWindow, WindowHandler as BaseviewWindowHandler, WindowOpenOptions,
    WindowScalePolicy,
};
use keyboard_types::Modifiers;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use raw_window_handle_06::{
    AppKitDisplayHandle, AppKitWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
    XcbDisplayHandle, XcbWindowHandle, XlibDisplayHandle, XlibWindowHandle,
};
use rootvg::math::{PhysicalPoint, PhysicalSizeI32};
use rootvg::surface::{DefaultSurface, NewSurfaceError};
use std::error::Error;
use std::num::{NonZeroIsize, NonZeroU32};
use std::ptr::NonNull;

mod convert;

use super::{ScaleFactorConfig, WindowConfig, WindowState, MAIN_WINDOW};
use crate::action_queue::ActionSender;
use crate::application::{AppContext, Application};
use crate::clipboard::Clipboard;
use crate::event::{EventCaptureStatus, PointerButton};
use crate::prelude::ActionReceiver;
use crate::window::{PointerBtnState, PointerLockState};
use crate::{CursorIcon, View};

pub(crate) type WindowHandle = ();

struct AppHandler<A: Application> {
    user_app: A,
    context: AppContext<A::Action>,
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
                baseview::MouseEvent::WheelScrolled { delta, modifiers } => (),
                baseview::MouseEvent::CursorEntered => (),
                baseview::MouseEvent::CursorLeft => (),
                baseview::MouseEvent::DragEntered {
                    position,
                    modifiers,
                    data,
                } => (),
                baseview::MouseEvent::DragMoved {
                    position,
                    modifiers,
                    data,
                } => (),
                baseview::MouseEvent::DragLeft => (),
                baseview::MouseEvent::DragDropped {
                    position,
                    modifiers,
                    data,
                } => (),
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
                baseview::WindowEvent::Resized(_) => (),
                baseview::WindowEvent::Focused => (),
                baseview::WindowEvent::Unfocused => (),
                baseview::WindowEvent::WillClose => (),
            },
        }

        baseview::EventStatus::Ignored
    }
}

#[derive(thiserror::Error, Debug)]
#[error("oopsie")]
pub struct OpenWindowError;

pub fn run_blocking<A: Application + 'static, B>(
    main_window_config: WindowConfig,
    action_sender: ActionSender<A::Action>,
    action_receiver: ActionReceiver<A::Action>,
    mut build_app: B,
) -> Result<(), Box<dyn Error>>
where
    A::Action: Send,
    B: FnMut() -> A,
    B: 'static + Send,
{
    let options = WindowOpenOptions {
        title: main_window_config.title.clone(),
        scale: match main_window_config.scale_factor {
            ScaleFactorConfig::System => WindowScalePolicy::SystemScaleFactor,
            ScaleFactorConfig::Custom(c) => WindowScalePolicy::ScaleFactor(c.into()),
        },
        size: baseview::Size::new(
            main_window_config.size.width as f64,
            main_window_config.size.height as f64,
        ),
    };

    BaseviewWindow::open_blocking(options, move |window: &mut BaseviewWindow| {
        let user_app = (build_app)();

        // TODO: get rid of unwrap
        let mut app_handler =
            AppHandler::new(user_app, action_sender.clone(), action_receiver).unwrap();

        let window_state = new_window::<A>(main_window_config, &mut app_handler, window).unwrap();

        app_handler
            .context
            .window_map
            .insert(MAIN_WINDOW, window_state);

        app_handler
    });

    Ok(())
}

fn new_window<A: Application>(
    config: WindowConfig,
    app_handler: &mut AppHandler<A>,
    window: &mut BaseviewWindow,
) -> Result<WindowState<A::Action>, NewSurfaceError> {
    let scale_factor = config.scale_factor.scale_factor(1.0.into());

    let raw_display_handle = window.raw_display_handle();
    let raw_window_handle = window.raw_window_handle();

    let target = wgpu::SurfaceTargetUnsafe::RawHandle {
        raw_display_handle: match raw_display_handle {
            raw_window_handle::RawDisplayHandle::AppKit(_) => {
                raw_window_handle_06::RawDisplayHandle::AppKit(AppKitDisplayHandle::new())
            }
            raw_window_handle::RawDisplayHandle::Xlib(handle) => {
                raw_window_handle_06::RawDisplayHandle::Xlib(XlibDisplayHandle::new(
                    NonNull::new(handle.display),
                    handle.screen,
                ))
            }
            raw_window_handle::RawDisplayHandle::Xcb(handle) => {
                raw_window_handle_06::RawDisplayHandle::Xcb(XcbDisplayHandle::new(
                    NonNull::new(handle.connection),
                    handle.screen,
                ))
            }
            raw_window_handle::RawDisplayHandle::Windows(_) => {
                raw_window_handle_06::RawDisplayHandle::Windows(WindowsDisplayHandle::new())
            }
            _ => panic!("unsupported display handle"),
        },
        raw_window_handle: match raw_window_handle {
            raw_window_handle::RawWindowHandle::AppKit(handle) => {
                raw_window_handle_06::RawWindowHandle::AppKit(AppKitWindowHandle::new(
                    NonNull::new(handle.ns_view).unwrap(),
                ))
            }
            raw_window_handle::RawWindowHandle::Xlib(handle) => {
                raw_window_handle_06::RawWindowHandle::Xlib(XlibWindowHandle::new(handle.window))
            }
            raw_window_handle::RawWindowHandle::Xcb(handle) => {
                raw_window_handle_06::RawWindowHandle::Xcb(XcbWindowHandle::new(
                    NonZeroU32::new(handle.window).unwrap(),
                ))
            }
            raw_window_handle::RawWindowHandle::Win32(handle) => {
                // will this work? i have no idea!
                let mut raw_handle =
                    Win32WindowHandle::new(NonZeroIsize::new(handle.hwnd as isize).unwrap());

                raw_handle.hinstance = handle
                    .hinstance
                    .is_null()
                    .then(|| NonZeroIsize::new(handle.hinstance as isize).unwrap());

                raw_window_handle_06::RawWindowHandle::Win32(raw_handle)
            }
            _ => panic!("unsupported window handle"),
        },
    };

    let physical_size = PhysicalSizeI32::new(config.size.width as i32, config.size.height as i32);

    let surface = unsafe {
        DefaultSurface::new_unsafe(physical_size, scale_factor, target, config.surface_config)?
    };
    let renderer = rootvg::Canvas::new(
        &surface.device,
        &surface.queue,
        surface.format(),
        surface.canvas_config(),
        &mut app_handler.context.res.font_system,
    );

    let view = View::new(
        physical_size,
        scale_factor,
        config.view_config,
        app_handler.context.action_sender.clone(),
        MAIN_WINDOW,
    );

    let clipboard = new_clipboard(window);

    Ok(WindowState {
        view,
        renderer,
        surface: Some(surface),
        logical_size: config.size,
        physical_size,
        scale_factor,
        scale_factor_recip: scale_factor.recip(),
        system_scale_factor: 1.0.into(),
        scale_factor_config: config.scale_factor,
        queued_pointer_position: None,
        queued_pointer_delta: None,
        window_handle: (),
        prev_pointer_pos: None,
        pointer_btn_states: [PointerBtnState::default(); 5],
        modifiers: Modifiers::empty(),
        current_cursor_icon: CursorIcon::Default,
        pointer_lock_state: PointerLockState::NotLocked,
        clipboard,
    })
}

fn new_clipboard(window: &BaseviewWindow) -> Clipboard {
    struct BaseviewHandle(raw_window_handle::RawDisplayHandle);

    impl raw_window_handle_06::HasDisplayHandle for BaseviewHandle {
        fn display_handle(
            &self,
        ) -> Result<raw_window_handle_06::DisplayHandle<'_>, raw_window_handle_06::HandleError>
        {
            Ok(unsafe {
                raw_window_handle_06::DisplayHandle::borrow_raw(match self.0 {
                    raw_window_handle::RawDisplayHandle::AppKit(_) => {
                        raw_window_handle_06::RawDisplayHandle::AppKit(
                            raw_window_handle_06::AppKitDisplayHandle::new(),
                        )
                    }
                    raw_window_handle::RawDisplayHandle::Xlib(handle) => {
                        raw_window_handle_06::RawDisplayHandle::Xlib(
                            raw_window_handle_06::XlibDisplayHandle::new(
                                NonNull::new(handle.display),
                                handle.screen,
                            ),
                        )
                    }
                    raw_window_handle::RawDisplayHandle::Xcb(handle) => {
                        raw_window_handle_06::RawDisplayHandle::Xcb(
                            raw_window_handle_06::XcbDisplayHandle::new(
                                NonNull::new(handle.connection),
                                handle.screen,
                            ),
                        )
                    }
                    raw_window_handle::RawDisplayHandle::Windows(_) => {
                        raw_window_handle_06::RawDisplayHandle::Windows(
                            raw_window_handle_06::WindowsDisplayHandle::new(),
                        )
                    }
                    _ => panic!("unsupported display handle"),
                })
            })
        }
    }

    let state = unsafe {
        window_clipboard::Clipboard::connect(&BaseviewHandle(window.raw_display_handle()))
    }
    .ok()
    .map(crate::clipboard::State::Connected)
    .unwrap_or(crate::clipboard::State::Unavailable);

    Clipboard { state }
}
