use std::{f32::consts::PI, rc::Rc};

use cache::{KnobRenderCache, KnobRenderCacheInner};
use rootvg::{
    color::RGBA8,
    math::{Angle, Point, Rect, Size, Transform, Vector},
    PrimitiveGroup,
};

use crate::{
    layout::SizeType,
    style::{Background, BorderStyle, QuadStyle},
    view::element::{ElementRenderCache, RenderContext},
};

use super::virtual_slider::{
    ParamerMarkerType, UpdateResult, VirtualSlider, VirtualSliderRenderInfo, VirtualSliderRenderer,
    VirtualSliderState,
};

mod angle_range;
mod cache;
mod notch_line;

pub use angle_range::KnobAngleRange;
pub use notch_line::{KnobNotchLinePrimitives, KnobNotchStyleLine, KnobNotchStyleLineBg};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct KnobStyle {
    pub back: KnobBackStyle,
    pub notch: KnobNotchStyle,
    pub markers: KnobMarkersStyle,
    pub angle_range: KnobAngleRange,
}

impl KnobStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        self.back.states_differ(a, b) || self.notch.states_differ(a, b)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KnobBackStyle {
    Quad(KnobBackStyleQuad),
}

impl KnobBackStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        match self {
            Self::Quad(s) => s.states_differ(a, b),
        }
    }

    pub fn size(&self) -> SizeType {
        match self {
            Self::Quad(s) => s.size,
        }
    }
}

impl Default for KnobBackStyle {
    fn default() -> Self {
        Self::Quad(KnobBackStyleQuad::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnobBackStyleQuad {
    pub idle_style: QuadStyle,
    pub hovered_style: QuadStyle,
    pub gesturing_style: QuadStyle,
    pub disabled_style: QuadStyle,
    pub size: SizeType,
}

impl KnobBackStyleQuad {
    pub fn quad_style(&self, state: VirtualSliderState, disabled: bool) -> &QuadStyle {
        if disabled {
            &self.disabled_style
        } else {
            match state {
                VirtualSliderState::Idle => &self.idle_style,
                VirtualSliderState::Hovered => &self.hovered_style,
                VirtualSliderState::Gesturing => &self.gesturing_style,
            }
        }
    }

    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        self.quad_style(a, false) != self.quad_style(b, false)
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
            size: SizeType::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KnobNotchStyle {
    Quad(KnobNotchStyleQuad),
    Line(KnobNotchStyleLine),
}

impl KnobNotchStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        match self {
            Self::Quad(s) => s.states_differ(a, b),
            Self::Line(s) => s.states_differ(a, b),
        }
    }
}

impl Default for KnobNotchStyle {
    fn default() -> Self {
        Self::Line(KnobNotchStyleLine::default())
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
    pub fn quad_style(&self, state: VirtualSliderState, disabled: bool) -> &QuadStyle {
        if disabled {
            &self.disabled_style
        } else {
            match state {
                VirtualSliderState::Idle => &self.idle_style,
                VirtualSliderState::Hovered => &self.hovered_style,
                VirtualSliderState::Gesturing => &self.gesturing_style,
            }
        }
    }

    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        self.quad_style(a, false) != self.quad_style(b, false)
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
            size: SizeType::FixedPoints(4.5),
            edge_offset: SizeType::FixedPoints(5.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KnobMarkersStyle {
    Dots(KnobMarkersDotStyle),
}

impl Default for KnobMarkersStyle {
    fn default() -> Self {
        Self::Dots(KnobMarkersDotStyle::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnobMarkersDotStyle {
    primary_quad_style: QuadStyle,
    secondary_quad_style: QuadStyle,
    third_quad_style: QuadStyle,
    primary_size: f32,
    secondary_size: f32,
    third_size: f32,
    primary_padding: f32,
    secondary_padding: f32,
    third_padding: f32,
}

impl Default for KnobMarkersDotStyle {
    fn default() -> Self {
        Self {
            primary_quad_style: QuadStyle {
                bg: Background::Solid(RGBA8::new(150, 150, 150, 150)),
                border: BorderStyle {
                    radius: 10000.0.into(),
                    ..Default::default()
                },
            },
            secondary_quad_style: QuadStyle {
                bg: Background::Solid(RGBA8::new(150, 150, 150, 100)),
                border: BorderStyle {
                    radius: 10000.0.into(),
                    ..Default::default()
                },
            },
            third_quad_style: QuadStyle {
                bg: Background::Solid(RGBA8::new(150, 150, 150, 50)),
                border: BorderStyle {
                    radius: 10000.0.into(),
                    ..Default::default()
                },
            },
            primary_size: 2.0,
            secondary_size: 1.0,
            third_size: 1.0,
            primary_padding: 3.0,
            secondary_padding: 3.0,
            third_padding: 3.0,
        }
    }
}

#[derive(Default)]
pub struct KnobRenderer {}

impl VirtualSliderRenderer for KnobRenderer {
    type Style = KnobStyle;

    fn on_state_changed(
        &mut self,
        prev_state: VirtualSliderState,
        new_state: VirtualSliderState,
        style: &Rc<Self::Style>,
    ) -> UpdateResult {
        // Only repaint if the appearance is different.
        UpdateResult {
            repaint: style.states_differ(prev_state, new_state),
            animating: false,
        }
    }

    fn render_primitives(
        &mut self,
        style: &Rc<Self::Style>,
        info: VirtualSliderRenderInfo<'_>,
        cx: RenderContext<'_>,
        primitives: &mut PrimitiveGroup,
    ) {
        let back_size = style.back.size();

        let back_bounds = match back_size {
            SizeType::FixedPoints(points) => crate::layout::centered_rect(
                Rect::from_size(cx.bounds_size).center(),
                Size::new(points, points),
            ),
            SizeType::Scale(scale) => {
                let bounds_rect = Rect::from_size(cx.bounds_size);

                let min_side_length = bounds_rect.width().min(bounds_rect.height());
                let side_length = min_side_length * scale;

                Rect::new(
                    bounds_rect.center() - Vector::new(side_length * 0.5, side_length * 0.5),
                    Size::new(side_length, side_length),
                )
            }
        };

        match &style.back {
            KnobBackStyle::Quad(s) => {
                let quad_style = s.quad_style(info.state, info.disabled);
                if !quad_style.is_transparent() {
                    primitives.add(quad_style.create_primitive(back_bounds));
                }
            }
        }

        match &style.markers {
            KnobMarkersStyle::Dots(s) => {
                let primary_center_offset =
                    ((back_bounds.width() + s.primary_size) * 0.5) + s.primary_padding;
                let secondary_center_offset =
                    ((back_bounds.width() + s.secondary_size) * 0.5) + s.secondary_padding;
                let third_center_offset =
                    ((back_bounds.width() + s.third_size) * 0.5) + s.third_padding;

                info.with_markers(|marker| {
                    let angle = style.angle_range.min()
                        + (style.angle_range.span() * marker.normal_val)
                        - Angle { radians: PI / 2.0 };

                    let (mut y_offset, mut x_offset) = angle.sin_cos();

                    let (center_offset, size, quad_style) = match marker.type_ {
                        ParamerMarkerType::Primary => {
                            (primary_center_offset, s.primary_size, &s.primary_quad_style)
                        }
                        ParamerMarkerType::Secondary => (
                            secondary_center_offset,
                            s.secondary_size,
                            &s.secondary_quad_style,
                        ),
                        ParamerMarkerType::Third => {
                            (third_center_offset, s.third_size, &s.third_quad_style)
                        }
                    };

                    x_offset *= center_offset;
                    y_offset *= center_offset;

                    let bounds = crate::layout::centered_rect(
                        back_bounds.center() - Vector::new(x_offset, y_offset),
                        Size::new(size, size),
                    );

                    primitives.add(quad_style.create_primitive(bounds));
                });
            }
        }

        match &style.notch {
            KnobNotchStyle::Quad(s) => {
                let quad_style = s.quad_style(info.state, info.disabled);
                if !quad_style.is_transparent() {
                    let notch_size = match s.size {
                        SizeType::FixedPoints(points) => points,
                        SizeType::Scale(scale) => back_bounds.width() * scale,
                    };

                    let center_offset = match s.edge_offset {
                        SizeType::FixedPoints(points) => (back_bounds.width() * 0.5) - points,
                        SizeType::Scale(scale) => {
                            (back_bounds.width() - (back_bounds.width() * scale)) * 0.5
                        }
                    };

                    let normal_val = info
                        .automation_info
                        .current_normal
                        .unwrap_or(info.normal_value);

                    let notch_angle = style.angle_range.min()
                        + (style.angle_range.span() * normal_val as f32)
                        - Angle { radians: PI / 2.0 };

                    let (mut y_offset, mut x_offset) = notch_angle.sin_cos();
                    x_offset *= center_offset;
                    y_offset *= center_offset;

                    let notch_bounds = crate::layout::centered_rect(
                        back_bounds.center() - Vector::new(x_offset, y_offset),
                        Size::new(notch_size, notch_size),
                    );

                    primitives.set_z_index(1);
                    primitives.add(quad_style.create_primitive(notch_bounds));
                }
            }
            KnobNotchStyle::Line(_) => {
                let render_cache = cx
                    .render_cache
                    .unwrap()
                    .get_mut()
                    .downcast_mut::<KnobRenderCacheInner>()
                    .unwrap();

                let meshes = render_cache
                    .get_notch_line_mesh(style, back_bounds.width())
                    .unwrap();

                let mut mesh = meshes.mesh(info.state, info.disabled).clone();

                mesh.set_offset(Point::new(
                    cx.bounds_size.width * 0.5,
                    cx.bounds_size.height * 0.5,
                ));

                let normal_val = info
                    .automation_info
                    .current_normal
                    .unwrap_or(info.normal_value);

                let notch_angle =
                    style.angle_range.min() + (style.angle_range.span() * normal_val as f32);
                mesh.set_transform(Transform::identity().then_rotate(notch_angle));

                primitives.set_z_index(1);
                primitives.add(mesh);
            }
        }
    }

    /// A unique identifier for the optional global render cache.
    ///
    /// All instances of this element type must return the same value.
    fn global_render_cache_id(&self) -> Option<u32> {
        Some(KnobRenderCache::ID)
    }

    /// An optional struct that is shared across all instances of this element type
    /// which can be used to cache rendering primitives.
    ///
    /// This will only be called once at the creation of the first instance of this
    /// element type.
    fn global_render_cache(&self) -> Option<Box<dyn ElementRenderCache>> {
        Some(Box::new(KnobRenderCache::new()))
    }
}

pub type Knob = VirtualSlider<KnobRenderer>;
