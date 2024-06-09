use std::f32::consts::PI;

use rootvg::{
    color::RGBA8,
    math::{Angle, Rect, Size, Vector},
    PrimitiveGroup,
};

use crate::{
    layout::SizeType,
    style::{Background, BorderStyle, QuadStyle},
    view::element::RenderContext,
};

use super::virtual_slider::{
    NormalsState, UpdateResult, VirtualSlider, VirtualSliderRenderer, VirtualSliderState,
};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct KnobStyle {
    pub back: KnobBackStyle,
    pub notch: KnobNotchStyle,
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
            bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
            border: BorderStyle {
                radius: 10000.0.into(),
                color: RGBA8::new(105, 105, 105, 255),
                width: 1.0,
                ..Default::default()
            },
        };

        let hovered_style = QuadStyle {
            border: BorderStyle {
                color: RGBA8::new(150, 150, 150, 255),
                ..idle_style.border
            },
            ..idle_style.clone()
        };

        Self {
            idle_style: idle_style.clone(),
            hovered_style: hovered_style.clone(),
            gesturing_style: hovered_style,
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
}

impl KnobNotchStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        match self {
            Self::Quad(s) => s.states_differ(a, b),
        }
    }
}

impl Default for KnobNotchStyle {
    fn default() -> Self {
        Self::Quad(KnobNotchStyleQuad::default())
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
            bg: Background::Solid(RGBA8::new(200, 200, 200, 255)),
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
            size: SizeType::FixedPoints(5.5),
            edge_offset: SizeType::FixedPoints(6.0),
        }
    }
}

/// The range between the minimum and maximum angle (in radians) a knob
/// will rotate.
///
/// `0.0` radians points straight down at the bottom of the knob, with the
/// angles rotating clockwise towards `2*PI.
///
/// Values < `0.0` and >= `2*PI` are not allowed.
///
/// The default minimum (converted to degrees) is `30` degrees, and the default
/// maximum is `330` degrees, giving a span of `300` degrees, and a halfway
/// point pointing strait up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnobAngleRange {
    min: Angle,
    max: Angle,
}

impl std::default::Default for KnobAngleRange {
    fn default() -> Self {
        Self {
            min: Self::DEFAULT_MIN,
            max: Self::DEFAULT_MAX,
        }
    }
}

impl KnobAngleRange {
    /// The default minimum angle of a rotating element such as a Knob
    pub const DEFAULT_MIN: Angle = Angle {
        radians: 30.0 * PI / 180.0,
    };
    /// The default maximum angle of a rotating element such as a Knob
    pub const DEFAULT_MAX: Angle = Angle {
        radians: (360.0 - 30.0) * PI / 180.0,
    };

    /// The range between the `min` and `max` angle (in degrees) a knob
    /// will rotate.
    ///
    /// `0.0` degrees points straight down at the bottom of the knob, with the
    /// angles rotating clockwise towards `360` degrees.
    ///
    /// Values < `0.0` and >= `360.0` will be set to `0.0`.
    ///
    /// The default minimum is `30` degrees, and the default maximum is `330`
    /// degrees, giving a span of `300` degrees, and a halfway point pointing
    /// strait up.
    ///
    /// # Panics
    ///
    /// This will panic if `min` > `max`.
    pub fn from_degrees(min: f32, max: f32) -> Self {
        let min_rad = min * PI / 180.0;
        let max_rad = max * PI / 180.0;

        Self::from_radians(min_rad, max_rad)
    }

    /// The span between the `min` and `max` angle (in radians) the knob
    /// will rotate.
    ///
    /// `0.0` radians points straight down at the bottom of the knob, with the
    /// angles rotating clockwise towards `2*PI` radians.
    ///
    /// Values < `0.0` and >= `2*PI` will be set to `0.0`.
    ///
    /// The default minimum (converted to degrees) is `30` degrees, and the
    /// default maximum is `330` degrees, giving a span of `300` degrees, and
    /// a halfway point pointing strait up.
    ///
    /// # Panics
    ///
    /// This will panic if `min` > `max`.
    pub fn from_radians(mut min: f32, mut max: f32) -> Self {
        assert!(min <= max);

        if min < 0.0 || min >= 2.0 * PI {
            log::warn!("KnobAngleRange min value {min} is out of range of [0.0, 2.0*PI), using 0.0 instead");
            min = 0.0;
        }
        if max < 0.0 || max >= 2.0 * PI {
            log::warn!("KnobAngleRange max value {max} is out of range of [0.0, 2.0*PI), using 0.0 instead");
            max = 0.0;
        }

        Self {
            min: Angle { radians: min },
            max: Angle { radians: max },
        }
    }

    /// Returns the minimum angle (between `0.0` and `2*PI`)
    pub fn min(&self) -> Angle {
        self.min
    }

    /// Returns the maximum angle (between `0.0` and `2*PI`)
    pub fn max(&self) -> Angle {
        self.max
    }

    /// Returns `self.max() - self.min()` in radians
    pub fn span(&self) -> Angle {
        self.max - self.min
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
        style: &Self::Style,
    ) -> UpdateResult {
        // Only repaint if the appearance is different.
        UpdateResult {
            repaint: style.states_differ(prev_state, new_state),
            animating: false,
        }
    }

    fn render_primitives(
        &mut self,
        style: &Self::Style,
        normals: NormalsState,
        state: VirtualSliderState,
        disabled: bool,
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
                let quad_style = s.quad_style(state, disabled);
                if !quad_style.is_transparent() {
                    primitives.add(quad_style.create_primitive(back_bounds));
                }
            }
        }

        match &style.notch {
            KnobNotchStyle::Quad(s) => {
                let quad_style = s.quad_style(state, disabled);
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

                    let normal_val = normals
                        .automation_info
                        .current_normal
                        .unwrap_or(normals.normal_value);

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
        }
    }
}

pub type Knob = VirtualSlider<KnobRenderer>;
