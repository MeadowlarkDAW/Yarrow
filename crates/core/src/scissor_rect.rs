/// The ID of a scissoring rectangle in a given window
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScissorRectID(pub u32);

impl ScissorRectID {
    /// `ScissorRectID` of `0` means to use use the main ElementSystem itself as the
    /// scissoring rectangle.
    pub const DEFAULT: Self = Self(0);
}

impl Default for ScissorRectID {
    fn default() -> Self {
        Self::DEFAULT
    }
}
