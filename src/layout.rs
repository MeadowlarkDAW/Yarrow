use crate::math::{Point, Rect, SideOffsets, Size};

pub type Padding = SideOffsets;
pub type Margin = SideOffsets;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartEndAlign {
    #[default]
    Start,
    End,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Align2 {
    pub horizontal: Align,
    pub vertical: Align,
}

impl Align2 {
    pub const TOP_LEFT: Self = Self {
        horizontal: Align::Start,
        vertical: Align::Start,
    };
    pub const TOP_CENTER: Self = Self {
        horizontal: Align::Center,
        vertical: Align::Start,
    };
    pub const TOP_RIGHT: Self = Self {
        horizontal: Align::End,
        vertical: Align::Start,
    };

    pub const CENTER_LEFT: Self = Self {
        horizontal: Align::Start,
        vertical: Align::Center,
    };
    pub const CENTER: Self = Self {
        horizontal: Align::Center,
        vertical: Align::Center,
    };
    pub const CENTER_RIGHT: Self = Self {
        horizontal: Align::End,
        vertical: Align::Center,
    };

    pub const BOTTOM_LEFT: Self = Self {
        horizontal: Align::Start,
        vertical: Align::End,
    };
    pub const BOTTOM_CENTER: Self = Self {
        horizontal: Align::Center,
        vertical: Align::End,
    };
    pub const BOTTOM_RIGHT: Self = Self {
        horizontal: Align::End,
        vertical: Align::End,
    };
}

impl Align2 {
    pub fn align_rect_to_point(&self, point: Point, size: Size) -> Rect {
        let x = match self.horizontal {
            Align::Start => point.x,
            Align::Center => point.x - (size.width * 0.5),
            Align::End => point.x - size.width,
        };
        let y = match self.vertical {
            Align::Start => point.y,
            Align::Center => point.y - (size.height * 0.5),
            Align::End => point.y - size.height,
        };

        Rect::new(Point::new(x, y), size)
    }

    pub fn align_floating_element(
        &self,
        bounds: Rect,
        floating_size: Size,
        padding: Padding,
    ) -> Point {
        match *self {
            Align2::TOP_LEFT => Point::new(
                bounds.min_x(),
                bounds.min_y() - floating_size.height - padding.top,
            ),
            Align2::TOP_CENTER => Point::new(
                bounds.min_x() + ((bounds.width() - floating_size.width) * 0.5),
                bounds.min_y() - floating_size.height - padding.top,
            ),
            Align2::TOP_RIGHT => Point::new(
                bounds.max_x() - floating_size.width,
                bounds.min_y() - floating_size.height - padding.top,
            ),
            Align2::CENTER_LEFT => Point::new(
                bounds.min_x() - floating_size.width - padding.left,
                bounds.min_y() + ((bounds.height() - floating_size.height) * 0.5),
            ),
            Align2::CENTER => Point::new(
                bounds.min_x() + ((bounds.width() - floating_size.width) * 0.5),
                bounds.min_y() + ((bounds.height() - floating_size.height) * 0.5),
            ),
            Align2::CENTER_RIGHT => Point::new(
                bounds.max_x() + padding.right,
                bounds.min_y() + ((bounds.height() - floating_size.height) * 0.5),
            ),
            Align2::BOTTOM_LEFT => Point::new(bounds.min_x(), bounds.max_y() + padding.bottom),
            Align2::BOTTOM_CENTER => Point::new(
                bounds.min_x() + ((bounds.width() - floating_size.width) * 0.5),
                bounds.max_y() + padding.bottom,
            ),
            Align2::BOTTOM_RIGHT => Point::new(
                bounds.max_x() - floating_size.width,
                bounds.max_y() + padding.bottom,
            ),
        }
    }
}

/// Describes how to lay out some content within a given bounding
/// rectangle.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct BoundsLayout {
    /// The padding from the edges of the content rectangle to the
    /// edges of the padded background rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub padding: Padding,

    /// The margin from the edges of the padded background rectangle
    /// to the edges of a bounding rectangle.
    ///
    /// This is only relevant if `stretch` is not `Stretch::None`.
    ///
    /// By default this has all values set to `0.0`.
    pub margin: Margin,

    /// How to stretch the content rectangle to fill a bounding
    /// rectangle.
    ///
    /// By default this is set to `Stretch::None`.
    pub stretch: Stretch,

    /// Where to place the content within a bounding rectangle.
    ///
    /// By default this is set to `BoundsPlacement::TopLeft`.
    pub placement: BoundsPlacement,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeType {
    FixedPoints(f32),
    Scale(f32),
}

impl SizeType {
    pub fn points(&self, bounds_points: f32) -> f32 {
        match self {
            Self::FixedPoints(p) => *p,
            Self::Scale(s) => *s * bounds_points,
        }
    }
}

impl Default for SizeType {
    fn default() -> Self {
        Self::Scale(1.0)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stretch {
    #[default]
    None,
    Horizontal,
    Vertical,
    All,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundsPlacement {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    CenterMarginRect,
    CenterPaddedRect,
    CenterContentRect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutMarginPaddingResult {
    pub padded_rect: Rect,
    pub margin_rect: Rect,
}

pub fn centered_rect(center: Point, size: Size) -> Rect {
    Rect::new(
        Point::new(
            center.x - (size.width * 0.5),
            center.y - (size.height * 0.5),
        ),
        size,
    )
}

/// Returns a rectangle outside of the given content rectangle with the
/// padding applied.
pub fn layout_padded_rect(padding: SideOffsets, content_rect: Rect) -> Rect {
    Rect {
        origin: Point::new(
            content_rect.min_x() - padding.left,
            content_rect.min_y() - padding.top,
        ),
        size: Size::new(
            content_rect.width() + padding.left + padding.right,
            content_rect.height() + padding.top + padding.bottom,
        ),
    }
}

/// Returns a rectangle inside of the given outer rectangle with the
/// padding applied.
///
/// If `outer_rect` is too small to fit a rectangle with the padding, this
/// will return `None`.
pub fn layout_inner_rect(padding: SideOffsets, outer_rect: Rect) -> Option<Rect> {
    let inner_size = Size::new(
        outer_rect.width() - padding.left - padding.right,
        outer_rect.height() - padding.top - padding.bottom,
    );

    if inner_size.width > 0.0 && inner_size.height > 0.0 {
        Some(Rect {
            origin: Point::new(
                outer_rect.min_x() + padding.left,
                outer_rect.min_y() + padding.top,
            ),
            size: inner_size,
        })
    } else {
        None
    }
}

/// Returns a rectangle inside of the given outer rectangle with the padding
/// applied.
///
/// If `outer_rect` is too small to fit a rectangle with the minimum size with
/// the padding, then this will return a rectangle with a size of
/// `min_inner_size` and sharing the same center as `outer_rect`.
pub fn layout_inner_rect_with_min_size(
    padding: SideOffsets,
    outer_rect: Rect,
    min_inner_size: Size,
) -> Rect {
    let inner_size = Size::new(
        outer_rect.width() - padding.left - padding.right,
        outer_rect.height() - padding.top - padding.bottom,
    );

    let (inner_x, inner_width) = if inner_size.width >= min_inner_size.width {
        (outer_rect.min_x() + padding.left, inner_size.width)
    } else {
        (
            (outer_rect.width() - min_inner_size.width) / 2.0,
            min_inner_size.width,
        )
    };

    let (inner_y, inner_height) = if inner_size.height >= min_inner_size.height {
        (outer_rect.min_y() + padding.top, inner_size.height)
    } else {
        (
            (outer_rect.height() - min_inner_size.height) / 2.0,
            min_inner_size.height,
        )
    };

    Rect {
        origin: Point::new(inner_x, inner_y),
        size: Size::new(inner_width, inner_height),
    }
}

pub fn layout_margin_padding(
    content_rect: Rect,
    margin: Margin,
    padding: Padding,
) -> LayoutMarginPaddingResult {
    let padded_rect = layout_padded_rect(padding, content_rect);
    let margin_rect = layout_padded_rect(margin, padded_rect);

    LayoutMarginPaddingResult {
        padded_rect,
        margin_rect,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutMarginPaddingBoundedResult {
    pub content_rect: Rect,
    pub padded_rect: Rect,
    pub margin_rect: Rect,
    pub content_width_shrunk: bool,
    pub content_height_shrunk: bool,
    pub could_not_fit_width: bool,
    pub could_not_fit_height: bool,
}

pub fn layout_margin_padding_bounded(
    bounds: Rect,
    desired_content_size: Size,
    min_content_size: Size,
    margin: Margin,
    padding: Padding,
    placement: BoundsPlacement,
    stretch: Stretch,
) -> LayoutMarginPaddingBoundedResult {
    struct HResult {
        bounds_x: f32,
        bounds_width: f32,
        content_width: f32,
        content_width_shrunk: bool,
        could_not_fit_width: bool,
    }
    struct VResult {
        bounds_y: f32,
        bounds_height: f32,
        content_height: f32,
        content_height_shrunk: bool,
        could_not_fit_height: bool,
    }

    let margin_plus_padding_width = margin.left + margin.right + padding.left + padding.right;
    let margin_plus_padding_height = margin.top + margin.bottom + padding.top + padding.bottom;

    let h_result: HResult =
        if desired_content_size.width + margin_plus_padding_width > bounds.width() {
            // The desired size doesn't fit the within the bounds, so try shrinking
            // the content to make it fit.

            if min_content_size.width + margin_plus_padding_width >= bounds.width() {
                // If the minimum size doesn't fit either, just use the minimum and grow
                // the bounds to fit.
                let content_width = min_content_size.width + margin_plus_padding_width;
                HResult {
                    bounds_x: bounds.min_x() + ((bounds.width() - content_width) / 2.0),
                    bounds_width: content_width,
                    content_width,
                    content_width_shrunk: true,
                    could_not_fit_width: true,
                }
            } else {
                // Else shrink the content to fit.
                let content_width = bounds.width() - margin_plus_padding_width;
                HResult {
                    bounds_x: bounds.min_x(),
                    bounds_width: bounds.width(),
                    content_width,
                    content_width_shrunk: true,
                    could_not_fit_width: false,
                }
            }
        } else if stretch == Stretch::Horizontal || stretch == Stretch::All {
            // Use the desired sizes and stretch the content rect to fill the bounds.
            HResult {
                bounds_x: bounds.min_x(),
                bounds_width: bounds.width(),
                content_width: bounds.width() - margin_plus_padding_width,
                content_width_shrunk: false,
                could_not_fit_width: false,
            }
        } else {
            // Use the desired sizes and don't stretch.
            HResult {
                bounds_x: bounds.min_x(),
                bounds_width: bounds.width(),
                content_width: desired_content_size.width,
                content_width_shrunk: false,
                could_not_fit_width: false,
            }
        };

    let v_result: VResult =
        if desired_content_size.height + margin_plus_padding_height > bounds.height() {
            // The desired size doesn't fit the within the bounds, so try shrinking
            // the content to make it fit.

            if min_content_size.height + margin_plus_padding_height >= bounds.height() {
                // If the minimum size doesn't fit either, just use the minimum and grow
                // the bounds to fit.
                let content_height = min_content_size.height + margin_plus_padding_height;
                VResult {
                    bounds_y: bounds.min_y() + ((bounds.height() - content_height) / 2.0),
                    bounds_height: content_height,
                    content_height,
                    content_height_shrunk: true,
                    could_not_fit_height: true,
                }
            } else {
                // Else shrink the content to fit.
                let content_height = bounds.height() - margin_plus_padding_height;
                VResult {
                    bounds_y: bounds.min_y(),
                    bounds_height: bounds.height(),
                    content_height,
                    content_height_shrunk: true,
                    could_not_fit_height: false,
                }
            }
        } else if stretch == Stretch::Vertical || stretch == Stretch::All {
            // Use the desired sizes and stretch the content rect to fill the bounds.
            VResult {
                bounds_y: bounds.min_y(),
                bounds_height: bounds.height(),
                content_height: bounds.height() - margin_plus_padding_height,
                content_height_shrunk: false,
                could_not_fit_height: false,
            }
        } else {
            // Use the desired sizes and don't stretch.
            VResult {
                bounds_y: bounds.min_y(),
                bounds_height: bounds.height(),
                content_height: desired_content_size.height,
                content_height_shrunk: false,
                could_not_fit_height: false,
            }
        };

    let content_rect_x = match placement {
        BoundsPlacement::TopLeft | BoundsPlacement::BottomLeft => {
            h_result.bounds_x + margin.left + padding.left
        }
        BoundsPlacement::TopRight | BoundsPlacement::BottomRight => {
            h_result.bounds_x + h_result.bounds_width
                - h_result.content_width
                - margin.right
                - padding.right
        }
        BoundsPlacement::CenterMarginRect => {
            let margin_rect_width = h_result.content_width + margin_plus_padding_width;
            h_result.bounds_x
                + ((h_result.bounds_width - margin_rect_width) / 2.0)
                + margin.left
                + padding.left
        }
        BoundsPlacement::CenterPaddedRect => {
            let padded_rect_width = h_result.content_width + padding.left + padding.right;
            h_result.bounds_x + ((h_result.bounds_width - padded_rect_width) / 2.0) + padding.left
        }
        BoundsPlacement::CenterContentRect => {
            h_result.bounds_x + ((h_result.bounds_width - h_result.content_width) / 2.0)
        }
    };

    let content_rect_y = match placement {
        BoundsPlacement::TopLeft | BoundsPlacement::TopRight => {
            v_result.bounds_y + margin.top + padding.top
        }
        BoundsPlacement::BottomLeft | BoundsPlacement::BottomRight => {
            v_result.bounds_y + v_result.bounds_height
                - v_result.content_height
                - margin.bottom
                - padding.bottom
        }
        BoundsPlacement::CenterMarginRect => {
            let margin_rect_height = v_result.content_height + margin_plus_padding_height;
            v_result.bounds_y
                + ((v_result.bounds_height - margin_rect_height) / 2.0)
                + margin.top
                + padding.top
        }
        BoundsPlacement::CenterPaddedRect => {
            let padded_rect_height = v_result.content_height + padding.top + padding.bottom;
            v_result.bounds_y + ((v_result.bounds_height - padded_rect_height) / 2.0) + padding.top
        }
        BoundsPlacement::CenterContentRect => {
            v_result.bounds_y + ((v_result.bounds_height - v_result.content_height) / 2.0)
        }
    };

    let content_rect = Rect::new(
        Point::new(content_rect_x, content_rect_y),
        Size::new(h_result.content_width, v_result.content_height),
    );
    let result = layout_margin_padding(content_rect, margin, padding);

    LayoutMarginPaddingBoundedResult {
        content_rect,
        padded_rect: result.padded_rect,
        margin_rect: result.padded_rect,
        content_width_shrunk: h_result.content_width_shrunk,
        content_height_shrunk: v_result.content_height_shrunk,
        could_not_fit_width: h_result.could_not_fit_width,
        could_not_fit_height: v_result.could_not_fit_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_margin_padding() {
        let content_rect = Rect::new(Point::new(20.0, 30.0), Size::new(200.0, 100.0));
        let margin = Margin::new(1.0, 2.0, 3.0, 4.0);
        let padding = Padding::new(5.0, 6.0, 7.0, 8.0);

        assert_eq!(
            layout_margin_padding(content_rect, margin, padding),
            LayoutMarginPaddingResult {
                padded_rect: Rect::new(Point::new(12.0, 25.0), Size::new(214.0, 112.0)),
                margin_rect: Rect::new(Point::new(8.0, 24.0), Size::new(220.0, 116.0)),
            }
        );
    }

    // TODO: write tests for the layout_margin_padding_bounded function
}
