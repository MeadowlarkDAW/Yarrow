use std::f32::consts::PI;

use rootvg::{
    color::RGBA8,
    math::{Angle, Rect, Size, Vector},
    quad::QuadPrimitive,
};

use crate::{
    elements::virtual_slider::VirtualSliderState,
    layout::SizeType,
    style::{Background, BorderStyle, QuadStyle},
};

use super::KnobAngleRange;

#[derive(Debug, Clone, PartialEq)]
pub struct KnobBackStyleQuad {
    pub idle_style: QuadStyle,
    pub hovered_style: QuadStyle,
    pub gesturing_style: QuadStyle,
    pub disabled_style: QuadStyle,
    pub size: SizeType,
}

impl KnobBackStyleQuad {
    pub fn quad_style(&self, state: VirtualSliderState) -> &QuadStyle {
        match state {
            VirtualSliderState::Idle => &self.idle_style,
            VirtualSliderState::Hovered => &self.hovered_style,
            VirtualSliderState::Gesturing => &self.gesturing_style,
            VirtualSliderState::Disabled => &self.disabled_style,
        }
    }

    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        self.quad_style(a) != self.quad_style(b)
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
        let idle_style = QuadStyle {
            bg: Background::Solid(RGBA8::new(70, 70, 70, 255)),
            border: BorderStyle {
                radius: 10000.0.into(),
                color: RGBA8::new(105, 105, 105, 255),
                width: 1.0,
                ..Default::default()
            },
        };

        Self {
            idle_style: idle_style.clone(),
            hovered_style: QuadStyle {
                border: BorderStyle {
                    color: RGBA8::new(135, 135, 135, 255),
                    ..idle_style.border
                },
                ..idle_style.clone()
            },
            gesturing_style: QuadStyle {
                border: BorderStyle {
                    color: RGBA8::new(150, 150, 150, 255),
                    ..idle_style.border
                },
                ..idle_style.clone()
            },
            disabled_style: QuadStyle {
                border: BorderStyle {
                    color: RGBA8::new(65, 65, 65, 255),
                    ..idle_style.border
                },
                ..idle_style
            },
            size: SizeType::Scale(0.8),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnobNotchStyleQuad {
    pub idle_style: QuadStyle,
    pub hovered_style: QuadStyle,
    pub gesturing_style: QuadStyle,
    pub disabled_style: QuadStyle,
    pub size: SizeType,
    /// * When `SizeType::Fixed(value)`, the value is the distance from the
    /// edge of the knob to the center of the notch in points.
    /// * When `SizeType::Scale(value)`, a value of `0.0` is on the edge of
    /// the knob and a value of `1.0` is in the center of the knob.
    pub edge_offset: SizeType,
}

impl KnobNotchStyleQuad {
    pub fn quad_style(&self, state: VirtualSliderState) -> &QuadStyle {
        match state {
            VirtualSliderState::Idle => &self.idle_style,
            VirtualSliderState::Hovered => &self.hovered_style,
            VirtualSliderState::Gesturing => &self.gesturing_style,
            VirtualSliderState::Disabled => &self.disabled_style,
        }
    }

    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        self.quad_style(a) != self.quad_style(b)
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
            SizeType::Scale(scale) => (back_bounds.width() - (back_bounds.width() * scale)) * 0.5,
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
        let idle_style = QuadStyle {
            bg: Background::Solid(RGBA8::new(255, 255, 255, 255)),
            border: BorderStyle {
                radius: 10000.0.into(),
                ..Default::default()
            },
        };

        Self {
            idle_style: idle_style.clone(),
            hovered_style: idle_style.clone(),
            gesturing_style: idle_style.clone(),
            disabled_style: QuadStyle {
                bg: Background::Solid(RGBA8::new(105, 105, 105, 255)),
                ..idle_style
            },
            size: SizeType::FixedPoints(5.0),
            edge_offset: SizeType::FixedPoints(5.5),
        }
    }
}
