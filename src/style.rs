use rootvg::color;
use rootvg::math::Rect;
use rootvg::quad::{GradientQuad, QuadPrimitive, SolidQuad};

use crate::prelude::ElementStyle;
use crate::vg::color::RGBA8;
use crate::vg::gradient::Gradient;
use crate::vg::quad::{Border, Radius};

mod style_system;

pub use style_system::StyleSystem;

pub const DEFAULT_ACCENT_COLOR: RGBA8 = RGBA8::new(179, 123, 95, 255);
pub const DEFAULT_ACCENT_HOVER_COLOR: RGBA8 = RGBA8::new(200, 137, 106, 255);
pub const DEFAULT_DISABLED_ALPHA_MULTIPLIER: f32 = 0.5;
pub const DEFAULT_ICON_SIZE: f32 = 20.0;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BorderStyle {
    /// The color of the border.
    pub color: RGBA8,

    /// The width of the border in logical points.
    pub width: f32,

    /// The radius of the border in logical points.
    pub radius: Radius,
}

impl BorderStyle {
    pub fn is_transparent(&self) -> bool {
        self.width == 0.0 || self.color == rootvg::color::TRANSPARENT
    }

    pub const TRANSPARENT: Self = Self {
        color: rootvg::color::TRANSPARENT,
        width: 0.0,
        radius: Radius::zero(),
    };
}

/*
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ShadowStyle {
    /// The color of the shadow.
    pub color: RGBA8,

    /// The offset of the shadow in logical points.
    pub offset: Point,

    /// The blur radius of the shadow in logical points.
    pub blur_radius: f32,
}
*/

#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuadStyle {
    /// The background of the quad
    pub bg: Background,

    /// The [`BorderStyle`] of the quad
    pub border: BorderStyle,
    /*
    /// The [`ShadowStyle`] of the quad
    ///
    /// (Note this currently has no effect if the background is a
    /// gradient.)
    pub shadow: ShadowStyle,
    */
}

impl QuadStyle {
    pub const TRANSPARENT: Self = Self {
        bg: Background::Solid(rootvg::color::TRANSPARENT),
        border: BorderStyle::TRANSPARENT,
    };

    pub fn is_transparent(&self) -> bool {
        self.bg.is_transparent() && self.border.is_transparent()
    }

    pub fn create_primitive(&self, bounds: Rect) -> QuadPrimitive {
        match &self.bg {
            Background::Solid(bg_color) => QuadPrimitive::Solid(
                SolidQuad {
                    bounds,
                    bg_color: (*bg_color).into(),
                    border: self.border.into(),
                    //shadow: self.shadow.into(),
                }
                .into(),
            ),
            Background::Gradient(bg_gradient) => QuadPrimitive::Gradient(
                GradientQuad {
                    bounds,
                    bg_gradient: *bg_gradient,
                    border: self.border.into(),
                }
                .into(),
            ),
        }
    }

    pub fn multiply_alpha(&mut self, multiplier: f32) {
        match &mut self.bg {
            Background::Solid(c) => *c = color::multiply_alpha(*c, multiplier),
            Background::Gradient(g) => g.multiply_alpha(multiplier),
        }

        self.border.color = color::multiply_alpha(self.border.color, multiplier);
    }
}

impl ElementStyle for QuadStyle {
    const ID: &'static str = "qd";
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Background {
    Solid(RGBA8),
    Gradient(Gradient),
}

impl Background {
    pub const TRANSPARENT: Self = Self::Solid(rootvg::color::TRANSPARENT);

    pub fn is_transparent(&self) -> bool {
        if let Self::Solid(color) = self {
            *color == rootvg::color::TRANSPARENT
        } else {
            false
        }
    }

    pub fn multiply_alpha(&mut self, multiplier: f32) {
        match self {
            Self::Solid(c) => *c = color::multiply_alpha(*c, multiplier),
            Self::Gradient(g) => g.multiply_alpha(multiplier),
        }
    }
}

impl Default for Background {
    fn default() -> Self {
        Self::Solid(rootvg::color::BLACK)
    }
}

impl Into<Border> for BorderStyle {
    fn into(self) -> Border {
        Border {
            color: self.color.into(),
            width: self.width,
            radius: self.radius,
        }
    }
}

/*
impl Into<Shadow> for ShadowStyle {
    fn into(self) -> Shadow {
        Shadow {
            color: self.color.into(),
            offset: self.offset,
            blur_radius: self.blur_radius,
        }
    }
}
*/
