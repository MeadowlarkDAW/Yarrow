use rootvg::color;
use rootvg::math::Rect;
use rootvg::quad::{GradientQuad, QuadPrimitive, SolidQuad};

use crate::prelude::ElementStyle;
use crate::theme::DEFAULT_DISABLED_ALPHA_MULTIPLIER;
use crate::vg::color::RGBA8;
use crate::vg::gradient::Gradient;
use crate::vg::quad::{Border, Radius};

mod style_system;

pub use style_system::StyleSystem;

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
    pub const TRANSPARENT: Self = Self {
        color: rootvg::color::TRANSPARENT,
        width: 0.0,
        radius: Radius::ZERO,
    };

    pub const fn new(color: RGBA8, width: f32, radius: Radius) -> Self {
        Self {
            color,
            width,
            radius,
        }
    }

    pub const fn from_radius(radius: Radius) -> Self {
        Self {
            color: rootvg::color::TRANSPARENT,
            width: 0.0,
            radius,
        }
    }

    pub fn is_transparent(&self) -> bool {
        self.width == 0.0 || self.color == rootvg::color::TRANSPARENT
    }
}

/// An alias for `BorderStyle::new(color, width, radius)`
pub const fn border(color: RGBA8, width: f32, radius: Radius) -> BorderStyle {
    BorderStyle::new(color, width, radius)
}

/// An alias for `BorderStyle::from_radius(color, width, radius)`
pub const fn border_radius_only(radius: Radius) -> BorderStyle {
    BorderStyle::from_radius(radius)
}

/*
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ShadowStyle {
    /// The color of the shadow.
    pub color: RGBA8,

    /// The offset of the shadow in logical points.
    pub offset: Vector,

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

    pub const fn new(bg: Background, border: BorderStyle) -> Self {
        Self { bg, border }
    }

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

/// An alias for `QuadStyle::new(color, width, radius)`
pub const fn quad_style(bg: Background, border: BorderStyle) -> QuadStyle {
    QuadStyle::new(bg, border)
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuadStyleDisabled {
    /// Use a multipler on the alpha channel for all colors.
    AlphaMultiplier(f32),
    /// Use a custom-defined style.
    Custom { bg: Background, border_color: RGBA8 },
}

impl Default for QuadStyleDisabled {
    fn default() -> Self {
        QuadStyleDisabled::AlphaMultiplier(DEFAULT_DISABLED_ALPHA_MULTIPLIER)
    }
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

/// An alias for `Background::Solid(color)`
pub const fn background(color: RGBA8) -> Background {
    Background::Solid(color)
}

/// An alias for `Background::Solid(RGBA8::new(r, g, b, a))`
pub const fn background_rgba(r: u8, g: u8, b: u8, a: u8) -> Background {
    Background::Solid(RGBA8::new(r, g, b, a))
}

/// An alias for `Background::Solid(RGBA8::new(r, g, b, 255))`
pub const fn background_rgb(r: u8, g: u8, b: u8) -> Background {
    Background::Solid(RGBA8::new(r, g, b, 255))
}

/// An alias for `Background::Solid(RGBA8::new(v, v, v, a))`
pub const fn background_gray_a(v: u8, a: u8) -> Background {
    Background::Solid(RGBA8::new(v, v, v, a))
}

/// An alias for `Background::Solid(RGBA8::new(v, v, v, 255))`
pub const fn background_gray(v: u8) -> Background {
    Background::Solid(RGBA8::new(v, v, v, 255))
}

/// How to style a color property when an element is disabled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisabledColor {
    /// Use a multiplier on the alpha channel of the property color.
    AlphaMultiplier(f32),
    /// Override the poperty color with a custom color.
    Custom(RGBA8),
}

impl DisabledColor {
    pub fn get(&self, property_color: RGBA8) -> RGBA8 {
        match self {
            DisabledColor::AlphaMultiplier(multiplier) => {
                color::multiply_alpha(property_color, *multiplier)
            }
            DisabledColor::Custom(color) => *color,
        }
    }
}

impl Default for DisabledColor {
    fn default() -> Self {
        Self::AlphaMultiplier(DEFAULT_DISABLED_ALPHA_MULTIPLIER)
    }
}

/// How to style a gradient property when an element is disabled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisabledGradient {
    /// Use a multiplier on the alpha channels of the property gradient.
    AlphaMultiplier(f32),
    /// Override the poperty gradient with a custom gradient.
    Custom(Gradient),
}

impl DisabledGradient {
    pub fn get(&self, property_gradient: Gradient) -> Gradient {
        match self {
            DisabledGradient::AlphaMultiplier(multiplier) => {
                let mut g = property_gradient;
                g.multiply_alpha(*multiplier);
                g
            }
            DisabledGradient::Custom(g) => *g,
        }
    }
}

impl Default for DisabledGradient {
    fn default() -> Self {
        Self::AlphaMultiplier(DEFAULT_DISABLED_ALPHA_MULTIPLIER)
    }
}

/// How to style a background property when an element is disabled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisabledBackground {
    /// Use a multiplier on the alpha channels of the property background.
    AlphaMultiplier(f32),
    /// Override the poperty background with a custom background.
    Custom(Background),
}

impl DisabledBackground {
    pub fn get(&self, property_bg: Background) -> Background {
        match self {
            DisabledBackground::AlphaMultiplier(multiplier) => {
                let mut bg = property_bg;
                bg.multiply_alpha(*multiplier);
                bg
            }
            DisabledBackground::Custom(bg) => *bg,
        }
    }
}

impl Default for DisabledBackground {
    fn default() -> Self {
        Self::AlphaMultiplier(DEFAULT_DISABLED_ALPHA_MULTIPLIER)
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
