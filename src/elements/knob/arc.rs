use std::{f32::consts::PI, rc::Rc};

use rootvg::{
    color::RGBA8,
    gradient::{Gradient, PackedGradient},
    math::{Angle, Point, Rect, Size},
    mesh::MeshPrimitive,
    tessellation::{
        fill::FillStyle,
        path::{ArcPath, PathBuilder},
        stroke::{LineCap, LineDash, LineJoin, Stroke},
        Tessellator,
    },
};

use crate::{
    elements::{knob::KnobMarkersStyle, virtual_slider::VirtualSliderState},
    layout::SizeType,
    style::{Background, DEFAULT_ACCENT_COLOR, DEFAULT_ACCENT_HOVER_COLOR},
};

use super::{KnobAngleRange, KnobStyle};

#[derive(Debug, Clone)]
pub enum KnobMarkersArcStyle {
    Solid {
        width: SizeType,
        back_width: SizeType,
        edge_offset: SizeType,
        line_cap: LineCap,
        idle_color: RGBA8,
        hovered_color: RGBA8,
        gesturing_color: RGBA8,
        disabled_color: RGBA8,
        back: Background,
        hide_threshold_normal: f32,
    },
    Gradient {
        width: SizeType,
        back_width: SizeType,
        edge_offset: SizeType,
        line_cap: LineCap,
        idle_gradient: Box<Gradient>,
        hovered_gradient: Box<Gradient>,
        gesturing_gradient: Box<Gradient>,
        disabled_gradient: Box<Gradient>,
        back: Background,
        hide_threshold_normal: f32,
    },
}

impl Default for KnobMarkersArcStyle {
    fn default() -> Self {
        Self::Solid {
            width: SizeType::FixedPoints(4.0),
            back_width: SizeType::FixedPoints(4.0),
            edge_offset: SizeType::FixedPoints(4.0),
            line_cap: LineCap::Round,
            idle_color: DEFAULT_ACCENT_COLOR,
            hovered_color: DEFAULT_ACCENT_HOVER_COLOR,
            gesturing_color: DEFAULT_ACCENT_HOVER_COLOR,
            disabled_color: RGBA8::new(150, 150, 150, 150),
            back: Background::Solid(RGBA8::new(50, 50, 50, 255)),
            hide_threshold_normal: 0.005,
        }
    }
}

impl KnobMarkersArcStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        match self {
            Self::Solid {
                idle_color,
                hovered_color,
                gesturing_color,
                disabled_color,
                ..
            } => {
                let color_a = match a {
                    VirtualSliderState::Idle => idle_color,
                    VirtualSliderState::Hovered => hovered_color,
                    VirtualSliderState::Gesturing => gesturing_color,
                    VirtualSliderState::Disabled => disabled_color,
                };
                let color_b = match b {
                    VirtualSliderState::Idle => idle_color,
                    VirtualSliderState::Hovered => hovered_color,
                    VirtualSliderState::Gesturing => gesturing_color,
                    VirtualSliderState::Disabled => disabled_color,
                };

                color_a != color_b
            }
            Self::Gradient {
                idle_gradient,
                hovered_gradient,
                gesturing_gradient,
                disabled_gradient,
                ..
            } => {
                let gradient_a = match a {
                    VirtualSliderState::Idle => idle_gradient,
                    VirtualSliderState::Hovered => hovered_gradient,
                    VirtualSliderState::Gesturing => gesturing_gradient,
                    VirtualSliderState::Disabled => disabled_gradient,
                };
                let gradient_b = match b {
                    VirtualSliderState::Idle => idle_gradient,
                    VirtualSliderState::Hovered => hovered_gradient,
                    VirtualSliderState::Gesturing => gesturing_gradient,
                    VirtualSliderState::Disabled => disabled_gradient,
                };

                gradient_a != gradient_b
            }
        }
    }

    pub fn create_back_primitive(
        &self,
        back_size: f32,
        angle_range: KnobAngleRange,
    ) -> MeshPrimitive {
        let (width, edge_offset, line_cap, back) = match self {
            Self::Solid {
                back_width,
                edge_offset,
                line_cap,
                back,
                ..
            } => (back_width, edge_offset, line_cap, back),
            Self::Gradient {
                back_width,
                edge_offset,
                line_cap,
                back,
                ..
            } => (back_width, edge_offset, line_cap, back),
        };

        let width = width.points(back_size);
        let edge_offset = edge_offset.points(back_size);
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

        let stroke_style = match back {
            Background::Solid(c) => FillStyle::Solid((*c).into()),
            Background::Gradient(g) => {
                let full_radius = radius + half_width;

                FillStyle::Gradient(PackedGradient::new(
                    g,
                    Rect::new(
                        Point::new(half_back_size - full_radius, half_back_size - full_radius),
                        Size::new(full_radius * 2.0, full_radius * 2.0),
                    ),
                ))
            }
        };

        let stroke = Stroke {
            style: stroke_style,
            width,
            line_cap: *line_cap,
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

        match self {
            Self::Solid {
                width,
                edge_offset,
                line_cap,
                idle_color,
                hovered_color,
                gesturing_color,
                disabled_color,
                hide_threshold_normal,
                ..
            } => {
                if bipolar {
                    if normal_val > 0.5 - *hide_threshold_normal
                        && normal_val < 0.5 + *hide_threshold_normal
                    {
                        return None;
                    }
                } else if normal_val < *hide_threshold_normal {
                    return None;
                };

                let width = width.points(back_bounds.width());
                let edge_offset = edge_offset.points(back_bounds.width());
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

                let color = match state {
                    VirtualSliderState::Idle => idle_color,
                    VirtualSliderState::Hovered => hovered_color,
                    VirtualSliderState::Gesturing => gesturing_color,
                    VirtualSliderState::Disabled => disabled_color,
                };

                let stroke = Stroke {
                    style: FillStyle::Solid((*color).into()),
                    width,
                    line_cap: *line_cap,
                    line_join: LineJoin::default(),
                    line_dash: LineDash::default(),
                };

                Tessellator::new()
                    .stroke(&arc_path, stroke)
                    .into_primitive()
            }
            Self::Gradient {
                width,
                edge_offset,
                line_cap,
                idle_gradient,
                hovered_gradient,
                gesturing_gradient,
                disabled_gradient,
                hide_threshold_normal,
                ..
            } => {
                if bipolar {
                    if normal_val > 0.5 - *hide_threshold_normal
                        && normal_val < 0.5 + *hide_threshold_normal
                    {
                        return None;
                    }
                } else if normal_val < *hide_threshold_normal {
                    return None;
                };

                let width = width.points(back_bounds.width());
                let edge_offset = edge_offset.points(back_bounds.width());
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

                let gradient = match state {
                    VirtualSliderState::Idle => idle_gradient,
                    VirtualSliderState::Hovered => hovered_gradient,
                    VirtualSliderState::Gesturing => gesturing_gradient,
                    VirtualSliderState::Disabled => disabled_gradient,
                };

                let full_radius = radius + half_width;

                let stroke = Stroke {
                    style: FillStyle::Gradient(PackedGradient::new(
                        gradient,
                        Rect::new(
                            Point::new(half_back_size - full_radius, half_back_size - full_radius),
                            Size::new(full_radius * 2.0, full_radius * 2.0),
                        ),
                    )),
                    width,
                    line_cap: *line_cap,
                    line_join: LineJoin::default(),
                    line_dash: LineDash::default(),
                };

                Tessellator::new()
                    .stroke(&arc_path, stroke)
                    .into_primitive()
            }
        }
    }
}

#[derive(Default)]
pub(super) struct CachedKnobMarkerArcFrontMesh {
    mesh: Option<MeshPrimitive>,
    style: Option<Rc<KnobStyle>>,
    back_bounds: Rect,
    normal_val: f32,
    state: VirtualSliderState,
    bipolar: bool,
}

impl CachedKnobMarkerArcFrontMesh {
    pub fn create_primitive(
        &mut self,
        style: &Rc<KnobStyle>,
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
            changed = !Rc::ptr_eq(self.style.as_ref().unwrap(), style)
                || self.back_bounds != back_bounds
                || self.bipolar != bipolar;
        }

        if changed {
            self.mesh = arc_style.create_front_primitive(
                back_bounds,
                normal_val,
                style.angle_range,
                state,
                bipolar,
            );

            self.style = Some(Rc::clone(style));
            self.back_bounds = back_bounds;
            self.normal_val = normal_val;
            self.state = state;
            self.bipolar = bipolar;
        }

        self.mesh.clone()
    }
}
