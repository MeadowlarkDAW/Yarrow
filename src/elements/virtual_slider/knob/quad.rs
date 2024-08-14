use std::f32::consts::PI;

use rootvg::{
    color::{self, RGBA8},
    math::{Angle, Rect, Size, Vector},
    quad::{QuadFlags, QuadPrimitive, Radius},
};

use crate::{
    elements::virtual_slider::VirtualSliderState,
    layout::SizeType,
    style::{Background, BorderStyle, DisabledBackground, DisabledColor, QuadStyle},
};

use super::KnobAngleRange;

#[derive(Debug, Clone, PartialEq)]
pub struct KnobBackStyleQuad {
    pub bg: Background,
    pub bg_hover: Option<Background>,
    pub bg_gesturing: Option<Background>,
    pub bg_disabled: DisabledBackground,

    pub border_color: RGBA8,
    pub border_color_hover: Option<RGBA8>,
    pub border_color_gesturing: Option<RGBA8>,
    pub border_color_disabled: DisabledColor,

    pub border_width: f32,
    pub border_width_hover: Option<f32>,

    pub size: SizeType,

    /// Additional flags for the quad primitives.
    ///
    /// By default this is set to `QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL`.
    pub quad_flags: QuadFlags,
}

impl KnobBackStyleQuad {
    pub fn quad_style(&self, state: VirtualSliderState) -> QuadStyle {
        match state {
            VirtualSliderState::Idle => QuadStyle {
                bg: self.bg,
                border: BorderStyle {
                    color: self.border_color,
                    width: self.border_width,
                    radius: Radius::CIRCLE,
                },
                flags: self.quad_flags,
            },
            VirtualSliderState::Hovered => QuadStyle {
                bg: self.bg_hover.unwrap_or(self.bg),
                border: BorderStyle {
                    color: self.border_color_hover.unwrap_or(self.border_color),
                    width: self.border_width_hover.unwrap_or(self.border_width),
                    radius: Radius::CIRCLE,
                },
                flags: self.quad_flags,
            },
            VirtualSliderState::Gesturing => QuadStyle {
                bg: self
                    .bg_gesturing
                    .unwrap_or(self.bg_hover.unwrap_or(self.bg)),
                border: BorderStyle {
                    color: self
                        .border_color_gesturing
                        .unwrap_or(self.border_color_hover.unwrap_or(self.border_color)),
                    width: self.border_width_hover.unwrap_or(self.border_width),
                    radius: Radius::CIRCLE,
                },
                flags: self.quad_flags,
            },
            VirtualSliderState::Disabled => QuadStyle {
                bg: self.bg_disabled.get(self.bg),
                border: BorderStyle {
                    color: self.border_color_disabled.get(self.border_color),
                    width: self.border_width,
                    radius: Radius::CIRCLE,
                },
                flags: self.quad_flags,
            },
        }
    }

    pub fn create_primitive(&self, state: VirtualSliderState, bounds: Rect) -> QuadPrimitive {
        let quad_style = self.quad_style(state);
        quad_style.create_primitive(bounds)
    }

    pub fn back_bounds(&self, element_size: Size) -> Rect {
        let bounds_rect = Rect::from_size(element_size);

        match self.size {
            SizeType::FixedPoints(points) => {
                crate::layout::centered_rect(bounds_rect.center(), Size::new(points, points))
            }
            SizeType::Scale(scale) => {
                let min_side_length = bounds_rect.width().min(bounds_rect.height());
                let side_length = min_side_length * scale;

                Rect::new(
                    bounds_rect.center() - Vector::new(side_length * 0.5, side_length * 0.5),
                    Size::new(side_length, side_length),
                )
            }
        }
    }
}

impl Default for KnobBackStyleQuad {
    fn default() -> Self {
        Self {
            bg: Background::TRANSPARENT,
            bg_hover: None,
            bg_gesturing: None,
            bg_disabled: Default::default(),
            border_color: color::TRANSPARENT,
            border_color_hover: None,
            border_color_gesturing: None,
            border_color_disabled: Default::default(),
            border_width: 0.0,
            border_width_hover: None,
            size: SizeType::default(),
            quad_flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnobNotchStyleQuad {
    pub bg: Background,
    pub bg_hover: Option<Background>,
    pub bg_gesturing: Option<Background>,
    pub bg_disabled: DisabledBackground,

    pub border_color: RGBA8,
    pub border_color_hover: Option<RGBA8>,
    pub border_color_gesturing: Option<RGBA8>,
    pub border_color_disabled: DisabledColor,

    pub border_width: f32,
    pub border_width_hover: Option<f32>,

    pub size: SizeType,
    /// * When `SizeType::Fixed(value)`, the value is the distance from the
    /// edge of the knob to the center of the notch in points.
    /// * When `SizeType::Scale(value)`, a value of `0.0` is on the edge of
    /// the knob and a value of `1.0` is in the center of the knob.
    pub edge_offset: SizeType,

    /// Additional flags for the quad primitives.
    ///
    /// By default this is set to `QuadFlags::empty()`.
    pub quad_flags: QuadFlags,
}

impl KnobNotchStyleQuad {
    pub fn quad_style(&self, state: VirtualSliderState) -> QuadStyle {
        match state {
            VirtualSliderState::Idle => QuadStyle {
                bg: self.bg,
                border: BorderStyle {
                    color: self.border_color,
                    width: self.border_width,
                    radius: Radius::CIRCLE,
                },
                flags: self.quad_flags,
            },
            VirtualSliderState::Hovered => QuadStyle {
                bg: self.bg_hover.unwrap_or(self.bg),
                border: BorderStyle {
                    color: self.border_color_hover.unwrap_or(self.border_color),
                    width: self.border_width_hover.unwrap_or(self.border_width),
                    radius: Radius::CIRCLE,
                },
                flags: self.quad_flags,
            },
            VirtualSliderState::Gesturing => QuadStyle {
                bg: self
                    .bg_gesturing
                    .unwrap_or(self.bg_hover.unwrap_or(self.bg)),
                border: BorderStyle {
                    color: self
                        .border_color_gesturing
                        .unwrap_or(self.border_color_hover.unwrap_or(self.border_color)),
                    width: self.border_width_hover.unwrap_or(self.border_width),
                    radius: Radius::CIRCLE,
                },
                flags: self.quad_flags,
            },
            VirtualSliderState::Disabled => QuadStyle {
                bg: self.bg_disabled.get(self.bg),
                border: BorderStyle {
                    color: self.border_color_disabled.get(self.border_color),
                    width: self.border_width,
                    radius: Radius::CIRCLE,
                },
                flags: self.quad_flags,
            },
        }
    }

    pub fn create_primitive(
        &self,
        normal_val: f32,
        angle_range: KnobAngleRange,
        state: VirtualSliderState,
        back_bounds: Rect,
    ) -> QuadPrimitive {
        let quad_style = self.quad_style(state);

        let notch_size = match self.size {
            SizeType::FixedPoints(points) => points,
            SizeType::Scale(scale) => back_bounds.width() * scale,
        };

        let center_offset = match self.edge_offset {
            SizeType::FixedPoints(points) => (back_bounds.width() * 0.5) - points,
            SizeType::Scale(scale) => (back_bounds.width() * 0.5) - (back_bounds.width() * scale),
        };

        let notch_angle = angle_range.min() + (angle_range.span() * normal_val as f32)
            - Angle { radians: PI / 2.0 };

        let (mut y_offset, mut x_offset) = notch_angle.sin_cos();
        x_offset *= center_offset;
        y_offset *= center_offset;

        let notch_bounds = crate::layout::centered_rect(
            back_bounds.center() - Vector::new(x_offset, y_offset),
            Size::new(notch_size, notch_size),
        );

        quad_style.create_primitive(notch_bounds)
    }
}

impl Default for KnobNotchStyleQuad {
    fn default() -> Self {
        Self {
            bg: Background::TRANSPARENT,
            bg_hover: None,
            bg_gesturing: None,
            bg_disabled: Default::default(),
            border_color: color::TRANSPARENT,
            border_color_hover: None,
            border_color_gesturing: None,
            border_color_disabled: Default::default(),
            border_width: 0.0,
            border_width_hover: None,
            size: SizeType::Scale(0.2),
            edge_offset: SizeType::Scale(0.18),
            quad_flags: QuadFlags::empty(),
        }
    }
}
