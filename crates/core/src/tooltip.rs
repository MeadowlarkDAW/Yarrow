use crate::math::Align2;

/// Tooltip data assigned to an element
#[derive(Debug, Clone, PartialEq)]
pub struct TooltipData {
    /// The tooltip text
    pub text: String,
    /// Where to align the tooltip relative to this element
    pub align: Align2,
}

impl TooltipData {
    /// Construct tooltip data for an element
    ///
    /// * `text` - The tooltip text
    /// * `align` - Where to align the tooltip relative to this element
    pub fn new(text: impl Into<String>, align: Align2) -> Self {
        Self {
            text: text.into(),
            align,
        }
    }
}
