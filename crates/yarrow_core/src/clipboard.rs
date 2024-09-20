use std::error::Error;

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

pub trait Clipboard {
    /// Reads the current content of the [`Clipboard`] as text.
    fn read(&self, kind: ClipboardKind) -> Option<String>;

    /// Writes the given text contents to the [`Clipboard`].
    fn write(&mut self, kind: ClipboardKind, contents: String) -> Result<(), Box<dyn Error>>;
}
