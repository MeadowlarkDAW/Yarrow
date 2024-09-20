use std::{error::Error, sync::Arc};

pub(super) struct ClipboardBackend {
    clipboard: window_clipboard::Clipboard,
}

impl ClipboardBackend {
    pub fn create(
        window_handle: &Arc<winit::window::Window>,
    ) -> Result<Box<dyn yarrow_core::clipboard::Clipboard>, Box<dyn Error>> {
        // SAFETY:
        // A reference-counted handle to the window is stored in `WindowState`,
        // ensuring that the window will not be dropped before the clipboard
        // is dropped.
        let clipboard = unsafe { window_clipboard::Clipboard::connect(window_handle) }?;

        Ok(Box::new(Self { clipboard }))
    }
}

impl yarrow_core::clipboard::Clipboard for ClipboardBackend {
    fn read(&self, kind: yarrow_core::clipboard::ClipboardKind) -> Option<String> {
        let res = match kind {
            yarrow_core::clipboard::ClipboardKind::Primary => self.clipboard.read_primary()?,
            yarrow_core::clipboard::ClipboardKind::Standard => self.clipboard.read(),
        };

        match res {
            Ok(s) => Some(s),
            Err(e) => {
                log::error!("Could not read system clipboard: {}", e);
                None
            }
        }
    }

    fn write(
        &mut self,
        kind: yarrow_core::clipboard::ClipboardKind,
        contents: String,
    ) -> Result<(), Box<dyn Error>> {
        match kind {
            yarrow_core::clipboard::ClipboardKind::Primary => {
                if let Some(res) = self.clipboard.write_primary(contents) {
                    res
                } else {
                    Err(String::from("Primary clipboard does not exist").into())
                }
            }
            yarrow_core::clipboard::ClipboardKind::Standard => self.clipboard.write(contents),
        }
    }
}
