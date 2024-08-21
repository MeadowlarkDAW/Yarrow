use baseview::Window as BaseviewWindow;
use std::sync::Arc;

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
        // #[cfg(feature = "winit")] window: &Arc<winit::window::Window>
        window: &mut BaseviewWindow,
    ) -> Clipboard {
        // SAFETY:
        // A reference-counted handle to the window is stored in this struct,
        // ensuring that the window will not be dropped before the clipboard
        // is dropped.
        let state = unsafe { window_clipboard::Clipboard::connect(window) }
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
