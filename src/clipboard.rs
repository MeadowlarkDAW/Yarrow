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
    pub(crate) state: State,
}

impl Clipboard {
    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read(&self, kind: ClipboardKind) -> Option<String> {
        let res = match &self.state {
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
        match &mut self.state {
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

pub(crate) enum State {
    Connected(window_clipboard::Clipboard),
    Unavailable,
}
