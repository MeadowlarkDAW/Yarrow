use std::f32::consts::PI;

use rootvg::{
    math::{Angle, Rect, Size, Vector},
    quad::QuadPrimitive,
    PrimitiveGroup,
};

use crate::{
    prelude::{ParamMarkersConfig, ParamerMarkerType},
    style::QuadStyle,
};

use super::KnobAngleRange;

#[derive(Debug, Clone, PartialEq)]
pub struct KnobMarkersDotStyle {
    pub primary_quad_style: QuadStyle,
    pub secondary_quad_style: Option<QuadStyle>,
    pub third_quad_style: Option<QuadStyle>,
    pub primary_size: f32,
    pub secondary_size: f32,
    pub third_size: f32,
    pub primary_padding: f32,
    pub secondary_padding: f32,
    pub third_padding: f32,
}

impl Default for KnobMarkersDotStyle {
    fn default() -> Self {
        Self {
            primary_quad_style: QuadStyle::TRANSPARENT,
            secondary_quad_style: None,
            third_quad_style: None,
            primary_size: 2.0,
            secondary_size: 1.0,
            third_size: 1.0,
            primary_padding: 4.0,
            secondary_padding: 4.0,
            third_padding: 4.0,
        }
    }
}

impl KnobMarkersDotStyle {
    pub fn add_primitives(
        &self,
        markers: &ParamMarkersConfig,
        back_bounds: Rect,
        bipolar: bool,
        num_quantized_steps: Option<u32>,
        angle_range: KnobAngleRange,
        primitives: &mut PrimitiveGroup,
    ) {
        let primary_center_offset =
            ((back_bounds.width() + self.primary_size) * 0.5) + self.primary_padding;
        let secondary_center_offset =
            ((back_bounds.width() + self.secondary_size) * 0.5) + self.secondary_padding;
        let third_center_offset =
            ((back_bounds.width() + self.third_size) * 0.5) + self.third_padding;

        markers.with_markers(bipolar, num_quantized_steps, |marker| {
            let angle = angle_range.min() + (angle_range.span() * marker.normal_val)
                - Angle { radians: PI / 2.0 };

            let (mut y_offset, mut x_offset) = angle.sin_cos();

            let (center_offset, size, quad_style) = match marker.type_ {
                ParamerMarkerType::Primary => (
                    primary_center_offset,
                    self.primary_size,
                    &self.primary_quad_style,
                ),
                ParamerMarkerType::Secondary => (
                    secondary_center_offset,
                    self.secondary_size,
                    self.secondary_quad_style
                        .as_ref()
                        .unwrap_or(&self.primary_quad_style),
                ),
                ParamerMarkerType::Third => (
                    third_center_offset,
                    self.third_size,
                    self.third_quad_style.as_ref().unwrap_or(
                        self.secondary_quad_style
                            .as_ref()
                            .unwrap_or(&self.primary_quad_style),
                    ),
                ),
            };

            x_offset *= center_offset;
            y_offset *= center_offset;

            let bounds = crate::layout::centered_rect(
                back_bounds.center() - Vector::new(x_offset, y_offset),
                Size::new(size, size),
            );

            match quad_style.create_primitive(bounds) {
                QuadPrimitive::Solid(s) => primitives.add_solid_quad(s),
                QuadPrimitive::Gradient(s) => primitives.add_gradient_quad(s),
            }
        });
    }
}
