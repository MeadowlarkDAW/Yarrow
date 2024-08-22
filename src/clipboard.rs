use baseview::Window as BaseviewWindow;
use raw_window_handle::{DisplayHandle, HasDisplayHandle, HasRawDisplayHandle, RawDisplayHandle};
use std::{ptr::NonNull, sync::Arc};

/// The kind of [`Clipboard`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardKind {
    /// The standard clipboard.
    #[default]
    Standard,
    /// The primary clipboard.
    ///
    /// Normally only present in X11 and Wayland.
    Primary,
}

/// A buffer for short-term storage and transfer within and between
/// applications.
pub struct Clipboard {
    state: Option<State>,
    // TODO: seems like this isnt being used?
    // #[cfg(feature = "winit")]
    // _window: Arc<winit::window::Window>,
}

impl Clipboard {
    pub(crate) fn new(
        // TODO:
        #[cfg(feature = "winit")] window: &Arc<winit::window::Window>,
        #[cfg(feature = "baseview")] window: &mut BaseviewWindow,
    ) -> Clipboard {
        // SAFETY:
        // A reference-counted handle to the window is stored in this struct,
        // ensuring that the window will not be dropped before the clipboard
        // is dropped.

        struct BaseviewHandle(RawDisplayHandle);

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
                        _ => todo!(),
                    })
                })
            }
        }

        let state = unsafe {
            window_clipboard::Clipboard::connect(&BaseviewHandle(window.raw_display_handle()))
        }
        .ok()
        .map(State::Connected)
        .unwrap_or(State::Unavailable);

        Clipboard {
            state: Some(state),
            // _window: Arc::clone(window),
        }
    }

    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read(&self, kind: ClipboardKind) -> Option<String> {
        let res = match self.state.as_ref().unwrap() {
            State::Connected(clipboard) => match kind {
                ClipboardKind::Standard => clipboard.read().ok(),
                ClipboardKind::Primary => clipboard.read_primary().and_then(Result::ok),
            },
            State::Unavailable => None,
        };

        if let Some(res) = res {
            if res.is_empty() {
                None
            } else {
                Some(res)
            }
        } else {
            None
        }
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write(&mut self, kind: ClipboardKind, contents: String) {
        match self.state.as_mut().unwrap() {
            State::Connected(clipboard) => {
                let result = match kind {
                    ClipboardKind::Standard => clipboard.write(contents),
                    ClipboardKind::Primary => clipboard.write_primary(contents).unwrap_or(Ok(())),
                };

                match result {
                    Ok(()) => {}
                    Err(error) => {
                        log::warn!("error writing to clipboard: {error}");
                    }
                }
            }
            State::Unavailable => {}
        }
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        // Make sure that the clipboard is dropped before the window.
        let _ = self.state.take();
    }
}

enum State {
    Connected(window_clipboard::Clipboard),
    Unavailable,
}
