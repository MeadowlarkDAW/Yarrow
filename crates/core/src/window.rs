#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The ID of a window
pub struct WindowID(pub u32);

impl WindowID {
    /// The ID of the main window
    pub const MAIN: Self = Self(0);
}

impl Default for WindowID {
    fn default() -> Self {
        Self::MAIN
    }
}

impl From<u32> for WindowID {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<WindowID> for u32 {
    fn from(value: WindowID) -> Self {
        value.0
    }
}
