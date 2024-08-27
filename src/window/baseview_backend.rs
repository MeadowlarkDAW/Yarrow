use baseview::{
    MouseCursor, Window as BaseviewWindow, WindowHandler as BaseviewWindowHandler,
    WindowOpenOptions, WindowScalePolicy,
};
use keyboard_types::{CompositionEvent, KeyState, Modifiers};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use raw_window_handle_06::{
    AppKitDisplayHandle, AppKitWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
    XcbDisplayHandle, XcbWindowHandle, XlibDisplayHandle, XlibWindowHandle,
};
use rootvg::math::Vector;
use rootvg::surface::{DefaultSurface, NewSurfaceError};
use std::error::Error;
use std::num::{NonZeroIsize, NonZeroU32};
use std::ptr::NonNull;

mod convert;

use super::{ScaleFactorConfig, WindowBackend, WindowConfig, WindowID, WindowState, MAIN_WINDOW};
use crate::action_queue::ActionSender;
use crate::application::Application;
use crate::clipboard::Clipboard;
use crate::event::{EventCaptureStatus, PointerButton, WheelDeltaType};
use crate::math::{PhysicalPoint, PhysicalSizeI32, ScaleFactor, Size};
use crate::prelude::{ActionReceiver, AppHandler, ResourceCtx};
use crate::window::{PointerBtnState, PointerLockState};
use crate::{CursorIcon, View};

struct BaseviewWindowBackend<'a, 'b> {
    main_window: &'a mut BaseviewWindow<'b>,
}

impl<'a, 'b> WindowBackend for BaseviewWindowBackend<'a, 'b> {
    fn set_pointer_position(
        &mut self,
        _window_id: WindowID,
        _position: PhysicalPoint,
    ) -> Result<(), ()> {
        // Baseview does not support setting the pointer position yet.
        Err(())
    }

    fn unlock_pointer(&mut self, _window_id: WindowID, _prev_lock_state: PointerLockState) {
        // Baseview does not support pointer locking yet.
    }

    fn request_redraw(&mut self, _window_id: WindowID) {
        // Not relevant for baseview.
    }

    fn has_focus(&mut self, window_id: WindowID) -> bool {
        if window_id == MAIN_WINDOW {
            // Baseview does not implement this yet (it just panics with "not implemented")
            //self.main_window.has_focus()
            true
        } else {
            false
        }
    }

    fn try_lock_pointer(&mut self, _window_id: WindowID) -> PointerLockState {
        // Baseview does not support pointer locking yet.
        PointerLockState::NotLocked
    }

    fn set_cursor_icon(&mut self, window_id: WindowID, icon: CursorIcon) {
        if window_id == MAIN_WINDOW {
            self.main_window.set_mouse_cursor(match icon {
                CursorIcon::Default => MouseCursor::Default,
                CursorIcon::ContextMenu => todo!("ContextMenu"),
                CursorIcon::Help => MouseCursor::Help,
                CursorIcon::Pointer => MouseCursor::Hand,
                CursorIcon::Progress => todo!("Progress"),
                CursorIcon::Wait => todo!("Wait"),
                CursorIcon::Cell => todo!("Cell"),
                CursorIcon::Crosshair => MouseCursor::Crosshair,
                CursorIcon::Text => MouseCursor::Text,
                CursorIcon::VerticalText => todo!("VerticalText"),
                CursorIcon::Alias => MouseCursor::Alias,
                CursorIcon::Copy => MouseCursor::Copy,
                CursorIcon::Move => MouseCursor::Move,
                CursorIcon::NoDrop => todo!("NoDrop"),
                CursorIcon::NotAllowed => MouseCursor::NotAllowed,
                CursorIcon::Grab => MouseCursor::Hand,
                CursorIcon::Grabbing => MouseCursor::HandGrabbing,
                CursorIcon::EResize => MouseCursor::EResize,
                CursorIcon::NResize => MouseCursor::NResize,
                CursorIcon::NeResize => MouseCursor::NeResize,
                CursorIcon::NwResize => MouseCursor::NwResize,
                CursorIcon::SResize => MouseCursor::SResize,
                CursorIcon::SeResize => MouseCursor::SeResize,
                CursorIcon::SwResize => MouseCursor::SwResize,
                CursorIcon::WResize => MouseCursor::WResize,
                CursorIcon::EwResize => MouseCursor::EwResize,
                CursorIcon::NsResize => MouseCursor::NsResize,
                CursorIcon::NeswResize => MouseCursor::NeswResize,
                CursorIcon::NwseResize => MouseCursor::NwseResize,
                CursorIcon::ColResize => MouseCursor::ColResize,
                CursorIcon::RowResize => MouseCursor::RowResize,
                CursorIcon::AllScroll => MouseCursor::AllScroll,
                CursorIcon::ZoomIn => MouseCursor::ZoomIn,
                CursorIcon::ZoomOut => MouseCursor::ZoomOut,
            })
        }
    }

    fn resize(
        &mut self,
        window_id: WindowID,
        logical_size: Size,
        _scale_factor: ScaleFactor,
    ) -> Result<(), ()> {
        if window_id == MAIN_WINDOW {
            self.main_window.resize(baseview::Size {
                width: logical_size.width as f64,
                height: logical_size.height as f64,
            });
            Ok(())
        } else {
            Err(())
        }
    }

    fn set_minimized(&mut self, _window_id: WindowID, _minimized: bool) {
        // Baseview does not support minimizing the window yet.
    }

    fn set_maximized(&mut self, _window_id: WindowID, _maximized: bool) {
        // Baseview does not support maximizing the window yet.
    }

    fn focus_window(&mut self, window_id: WindowID) {
        if window_id == MAIN_WINDOW {
            self.main_window.focus();
        }
    }

    fn set_window_title(&mut self, _window_id: WindowID, _title: String) {
        // Baseview does not support setting the window title yet.
    }

    fn create_window<A: Clone + 'static>(
        &mut self,
        _window_id: WindowID,
        _config: &WindowConfig,
        _action_sender: &ActionSender<A>,
        _res: &mut ResourceCtx,
    ) -> Result<WindowState<A>, OpenWindowError> {
        // Baseview does not support multiple windows yet.
        Err(OpenWindowError::MultiWindowNotSupported)
    }

    fn close_window(&mut self, window_id: WindowID) {
        if window_id == MAIN_WINDOW {
            self.main_window.close();
        }
    }
}

struct BaseviewAppHandlerInner {
    first_resize: bool,
}

struct BaseviewAppHandler<A: Application> {
    app_handler: AppHandler<A>,
    inner: BaseviewAppHandlerInner,
}

impl<A: Application> BaseviewAppHandler<A> {
    fn new(
        user_app: A,
        action_sender: ActionSender<A::Action>,
        action_receiver: ActionReceiver<A::Action>,
        main_window_config: WindowConfig,
        window: &mut BaseviewWindow,
    ) -> Result<Self, Box<dyn Error>> {
        let mut app_handler = AppHandler::new(user_app, action_sender, action_receiver)?;

        let window_state = new_window::<A>(main_window_config, &mut app_handler, window)?;

        app_handler
            .context
            .window_map
            .insert(MAIN_WINDOW, window_state);

        Ok(Self {
            app_handler,
            inner: BaseviewAppHandlerInner { first_resize: true },
        })
    }

    fn process_updates(&mut self, window: &mut BaseviewWindow) {
        self.app_handler
            .process_updates(&mut BaseviewWindowBackend {
                main_window: window,
            });
    }
}

impl<A: Application> BaseviewWindowHandler for BaseviewAppHandler<A> {
    fn on_frame(&mut self, window: &mut BaseviewWindow) {
        self.app_handler.on_tick();
        self.process_updates(window);

        if let Err(e) = self
            .app_handler
            .context
            .window_map
            .get_mut(&MAIN_WINDOW)
            .unwrap()
            .render(|| {}, &mut self.app_handler.context.res)
        {
            log::error!("render error: {}", e);
        }
    }

    fn on_event(
        &mut self,
        window: &mut BaseviewWindow,
        event: baseview::Event,
    ) -> baseview::EventStatus {
        let mut process_updates = true;

        // TODO:
        match event {
            baseview::Event::Mouse(mouse_event) => match mouse_event {
                baseview::MouseEvent::CursorMoved {
                    position,
                    modifiers,
                } => {
                    self.app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap()
                        .set_modifiers(modifiers);

                    let pos = PhysicalPoint::new(position.x as f32, position.y as f32);

                    self.app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap()
                        .queued_pointer_position = Some(pos);

                    // Debounce mouse move events by queing them to be processed in `on_frame()`
                    process_updates = false;
                }
                baseview::MouseEvent::ButtonPressed { button, modifiers } => {
                    self.app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap()
                        .set_modifiers(modifiers);

                    let button = match button {
                        baseview::MouseButton::Left => PointerButton::Primary,
                        baseview::MouseButton::Middle => PointerButton::Auxiliary,
                        baseview::MouseButton::Right => PointerButton::Secondary,
                        baseview::MouseButton::Back => PointerButton::Fourth,
                        baseview::MouseButton::Forward => PointerButton::Fifth,
                        _ => return baseview::EventStatus::Ignored,
                    };

                    self.app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap()
                        .handle_mouse_button(button, true, &mut self.app_handler.context.res);
                }
                baseview::MouseEvent::ButtonReleased { button, modifiers } => {
                    self.app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap()
                        .set_modifiers(modifiers);

                    let button = match button {
                        baseview::MouseButton::Left => PointerButton::Primary,
                        baseview::MouseButton::Middle => PointerButton::Auxiliary,
                        baseview::MouseButton::Right => PointerButton::Secondary,
                        baseview::MouseButton::Back => PointerButton::Fourth,
                        baseview::MouseButton::Forward => PointerButton::Fifth,
                        _ => return baseview::EventStatus::Ignored,
                    };

                    self.app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap()
                        .handle_mouse_button(button, false, &mut self.app_handler.context.res);
                }
                baseview::MouseEvent::WheelScrolled { delta, modifiers } => {
                    let window_state = self
                        .app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap();

                    window_state.set_modifiers(modifiers);

                    let delta_type = match delta {
                        baseview::ScrollDelta::Lines { x, y } => {
                            WheelDeltaType::Lines(Vector::new(x, -y))
                        }
                        baseview::ScrollDelta::Pixels { x, y } => {
                            WheelDeltaType::Points(Vector::new(
                                x * window_state.scale_factor_recip,
                                -y * window_state.scale_factor_recip,
                            ))
                        }
                    };

                    window_state.handle_mouse_wheel(delta_type, &mut self.app_handler.context.res)
                }
                baseview::MouseEvent::CursorEntered => (),
                baseview::MouseEvent::CursorLeft => self
                    .app_handler
                    .context
                    .window_map
                    .get_mut(&MAIN_WINDOW)
                    .unwrap()
                    .handle_pointer_left(&mut self.app_handler.context.res),
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
                let window_state = self
                    .app_handler
                    .context
                    .window_map
                    .get_mut(&MAIN_WINDOW)
                    .unwrap();

                let key_event = self::convert::convert_keyboard_event(&keyboard_event);

                let mut captured = window_state
                    .handle_keyboard_event(key_event.clone(), &mut self.app_handler.context.res)
                    == EventCaptureStatus::Captured;

                if !captured && keyboard_event.state == KeyState::Down {
                    if let Some(text) =
                        self::convert::key_to_composition(keyboard_event.key, keyboard_event.code)
                    {
                        captured |= window_state.handle_text_composition_event(
                            CompositionEvent {
                                state: keyboard_types::CompositionState::Start,
                                data: String::new(),
                            },
                            &mut self.app_handler.context.res,
                        ) == EventCaptureStatus::Captured;
                        captured |= window_state.handle_text_composition_event(
                            CompositionEvent {
                                state: keyboard_types::CompositionState::End,
                                data: text,
                            },
                            &mut self.app_handler.context.res,
                        ) == EventCaptureStatus::Captured
                    }
                }

                if !captured {
                    self.app_handler.user_app.on_keyboard_event(
                        key_event,
                        MAIN_WINDOW,
                        &mut self.app_handler.context,
                    );
                }
            }
            baseview::Event::Window(window_event) => match window_event {
                baseview::WindowEvent::Resized(info) => {
                    let physical_size = info.physical_size();
                    let new_size = PhysicalSizeI32::new(
                        physical_size.width as i32,
                        physical_size.height as i32,
                    );

                    let window_state = self
                        .app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap();

                    let scale_factor = info.scale();

                    window_state.set_size(new_size, scale_factor.into());

                    if self.inner.first_resize {
                        self.inner.first_resize = false;

                        self.app_handler.user_app.on_window_event(
                            crate::event::AppWindowEvent::WindowOpened,
                            MAIN_WINDOW,
                            &mut self.app_handler.context,
                        );
                    } else {
                        self.app_handler.user_app.on_window_event(
                            crate::event::AppWindowEvent::WindowResized,
                            MAIN_WINDOW,
                            &mut self.app_handler.context,
                        );
                    }
                }
                baseview::WindowEvent::Focused => {
                    let window_state = self
                        .app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap();

                    window_state.handle_window_focused(&mut self.app_handler.context.res);
                    self.app_handler.user_app.on_window_event(
                        crate::event::AppWindowEvent::WindowFocused,
                        MAIN_WINDOW,
                        &mut self.app_handler.context,
                    );
                }
                baseview::WindowEvent::Unfocused => {
                    let window_state = self
                        .app_handler
                        .context
                        .window_map
                        .get_mut(&MAIN_WINDOW)
                        .unwrap();

                    window_state.handle_window_unfocused(&mut self.app_handler.context.res);
                    self.app_handler.user_app.on_window_event(
                        crate::event::AppWindowEvent::WindowUnfocused,
                        MAIN_WINDOW,
                        &mut self.app_handler.context,
                    );
                }
                baseview::WindowEvent::WillClose => {
                    self.app_handler.user_app.on_request_to_close_window(
                        MAIN_WINDOW,
                        true,
                        &mut self.app_handler.context,
                    );
                }
            },
        }

        if process_updates {
            self.process_updates(window);
        }

        baseview::EventStatus::Ignored
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OpenWindowError {
    #[error("Baseview does not yet support multiple windows")]
    MultiWindowNotSupported,
}

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

        // TODO: get rid of unwrap once baseview supports erros on build closures.
        BaseviewAppHandler::new(
            user_app,
            action_sender,
            action_receiver,
            main_window_config,
            window,
        )
        .unwrap()
    });

    Ok(())
}

pub fn run_parented<P: HasRawWindowHandle, A: Application + 'static, B>(
    parent: &P,
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

    BaseviewWindow::open_parented(parent, options, move |window: &mut BaseviewWindow| {
        let user_app = (build_app)();

        // TODO: get rid of unwrap once baseview supports erros on build closures.
        BaseviewAppHandler::new(
            user_app,
            action_sender,
            action_receiver,
            main_window_config,
            window,
        )
        .unwrap()
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
