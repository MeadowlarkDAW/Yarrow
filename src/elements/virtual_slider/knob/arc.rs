use std::f32::consts::PI;

use rootvg::{
    math::{Angle, Point, Rect},
    mesh::MeshPrimitive,
    tessellation::{
        fill::FillStyle,
        path::{ArcPath, PathBuilder},
        stroke::{LineCap, LineDash, LineJoin, Stroke},
        Tessellator,
    },
};

#[cfg(feature = "gradient")]
use rootvg::gradient::PackedGradient;

use crate::{
    elements::virtual_slider::VirtualSliderState,
    layout::SizeType,
    style::{Background, ClassID, DisabledBackground},
    theme::DEFAULT_ACCENT_COLOR,
};

use super::{KnobAngleRange, KnobMarkersStyle, KnobStyle};

#[derive(Debug, Clone)]
pub struct KnobMarkersArcStyle {
    pub width: SizeType,
    pub back_width: SizeType,
    pub edge_offset: SizeType,
    pub line_cap: LineCap,
    pub fill_bg: Background,
    pub fill_bg_hover: Option<Background>,
    pub fill_bg_gesturing: Option<Background>,
    pub fill_bg_disabled: DisabledBackground,
    pub back_bg: Background,
    pub back_bg_disabled: DisabledBackground,
    pub hide_threshold_normal: f32,
}

impl Default for KnobMarkersArcStyle {
    fn default() -> Self {
        Self {
            width: SizeType::Scale(0.15),
            back_width: SizeType::Scale(0.15),
            edge_offset: SizeType::Scale(0.15),
            line_cap: LineCap::Round,
            fill_bg: Background::Solid(DEFAULT_ACCENT_COLOR),
            fill_bg_hover: None,
            fill_bg_gesturing: None,
            fill_bg_disabled: Default::default(),
            back_bg: Background::TRANSPARENT,
            back_bg_disabled: Default::default(),
            hide_threshold_normal: 0.005,
        }
    }
}

impl KnobMarkersArcStyle {
    pub fn create_back_primitive(
        &self,
        back_size: f32,
        angle_range: KnobAngleRange,
        disabled: bool,
    ) -> MeshPrimitive {
        let width = self.back_width.points(back_size);
        let edge_offset = self.edge_offset.points(back_size);
        let half_back_size = back_size * 0.5;
        let half_width = width * 0.5;

        let radius = half_back_size + half_width + edge_offset;

        let arc_path = PathBuilder::new()
            .arc(ArcPath {
                center: Point::new(back_size * 0.5, back_size * 0.5),
                radius,
                start_angle: angle_range.min() + Angle { radians: PI * 0.5 },
                end_angle: angle_range.max() + Angle { radians: PI * 0.5 },
            })
            .build();

        let back_bg = if disabled {
            self.back_bg_disabled.get(self.back_bg)
        } else {
            self.back_bg
        };

        let fill_style = match &back_bg {
            Background::Solid(c) => FillStyle::Solid((*c).into()),
            #[cfg(feature = "gradient")]
            Background::Gradient(g) => {
                let full_radius = radius + half_width;

                FillStyle::Gradient(PackedGradient::new(
                    g,
                    Rect::new(
                        Point::new(half_back_size - full_radius, half_back_size - full_radius),
                        crate::math::Size::new(full_radius * 2.0, full_radius * 2.0),
                    ),
                ))
            }
        };

        let stroke = Stroke {
            style: fill_style,
            width,
            line_cap: self.line_cap,
            line_join: LineJoin::default(),
            line_dash: LineDash::default(),
        };

        Tessellator::new()
            .stroke(&arc_path, stroke)
            .into_primitive()
            .unwrap()
    }

    pub fn create_front_primitive(
        &self,
        back_bounds: Rect,
        normal_val: f32,
        angle_range: KnobAngleRange,
        state: VirtualSliderState,
        bipolar: bool,
    ) -> Option<MeshPrimitive> {
        if bipolar {
            if normal_val > 0.5 - self.hide_threshold_normal
                && normal_val < 0.5 + self.hide_threshold_normal
            {
                return None;
            }
        } else if normal_val < self.hide_threshold_normal {
            return None;
        };

        let value_angle =
            angle_range.min() + (angle_range.span() * normal_val) + Angle { radians: PI * 0.5 };

        let (start_angle, end_angle) = if bipolar {
            let center_angle =
                angle_range.min() + (angle_range.span() * 0.5) + Angle { radians: PI * 0.5 };

            if normal_val < 0.5 {
                (value_angle, center_angle)
            } else {
                (center_angle, value_angle)
            }
        } else {
            (angle_range.min() + Angle { radians: PI * 0.5 }, value_angle)
        };

        let width = self.width.points(back_bounds.width());
        let edge_offset = self.edge_offset.points(back_bounds.width());
        let half_back_size = back_bounds.width() * 0.5;
        let half_width = width * 0.5;

        let radius = half_back_size + half_width + edge_offset;

        let arc_path = PathBuilder::new()
            .arc(ArcPath {
                center: back_bounds.center(),
                radius,
                start_angle,
                end_angle,
            })
            .build();

        let bg = match state {
            VirtualSliderState::Idle => self.fill_bg,
            VirtualSliderState::Hovered => self.fill_bg_hover.unwrap_or(self.fill_bg),
            VirtualSliderState::Gesturing => self
                .fill_bg_gesturing
                .unwrap_or(self.fill_bg_hover.unwrap_or(self.fill_bg)),
            VirtualSliderState::Disabled => self.fill_bg_disabled.get(self.fill_bg),
        };

        let fill_style = match &bg {
            Background::Solid(c) => FillStyle::Solid((*c).into()),
            #[cfg(feature = "gradient")]
            Background::Gradient(g) => {
                let full_radius = radius + half_width;

                FillStyle::Gradient(PackedGradient::new(
                    g,
                    Rect::new(
                        Point::new(half_back_size - full_radius, half_back_size - full_radius),
                        crate::math::Size::new(full_radius * 2.0, full_radius * 2.0),
                    ),
                ))
            }
        };

        let stroke = Stroke {
            style: fill_style,
            width,
            line_cap: self.line_cap,
            line_join: LineJoin::default(),
            line_dash: LineDash::default(),
        };

        Tessellator::new()
            .stroke(&arc_path, stroke)
            .into_primitive()
    }
}

#[derive(Default)]
pub(super) struct CachedKnobMarkerArcFrontMesh {
    mesh: Option<MeshPrimitive>,
    class: ClassID,
    back_bounds: Rect,
    normal_val: f32,
    state: VirtualSliderState,
    bipolar: bool,
}

impl CachedKnobMarkerArcFrontMesh {
    pub fn create_primitive(
        &mut self,
        class: ClassID,
        style: &KnobStyle,
        back_bounds: Rect,
        normal_val: f32,
        state: VirtualSliderState,
        bipolar: bool,
    ) -> Option<MeshPrimitive> {
        let KnobMarkersStyle::Arc(arc_style) = &style.markers else {
            return None;
        };

        // Since these are the two most likely to change, check these first.
        let mut changed =
            self.normal_val != normal_val || self.state != state || self.mesh.is_none();

        if !changed {
            changed =
                self.class != class || self.back_bounds != back_bounds || self.bipolar != bipolar;
        }

        if changed {
            self.mesh = arc_style.create_front_primitive(
                back_bounds,
                normal_val,
                style.angle_range,
                state,
                bipolar,
            );

            self.class = class;
            self.back_bounds = back_bounds;
            self.normal_val = normal_val;
            self.state = state;
            self.bipolar = bipolar;
        }

        self.mesh.clone()
    }
}
