use std::f32::consts::PI;

pub use euclid::UnknownUnit as Logical;
pub use euclid::{approxeq, approxord, num, Trig};

/// A unit of physical pixels
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Physical {}

/// A 2D point in units of logical points
///
/// Alias for `euclid::Point2D<f32, Logical>`.
pub type Point = euclid::default::Point2D<f32>;

/// A 2D point in units of physical pixels
///
/// Alias for `euclid::Point2D<i32, Physical>`.
pub type PhysicalPoint = euclid::Point2D<i32, Physical>;

/// A 2D rectangle in units of logical points
///
/// Alias for `euclid::Rect<f32, Logical>`.
pub type Rect = euclid::default::Rect<f32>;

/// A 2D rectangle in units of integer logical points
///
/// Alias for `euclid::Rect<f32, Logical>`.
pub type RectI32 = euclid::default::Rect<i32>;

/// A 2D rectangle in units of physical pixels
///
/// Alias for `euclid::Rect<i32, Physical>`.
pub type PhysicalRect = euclid::Rect<i32, Physical>;

/// A 2D rectangle represented by its minimum and maximum coordinates,
/// in units of logical points
///
/// Alias for `euclid::Box2D<f32, Logical>`.
pub type Box = euclid::default::Box2D<f32>;

/// A 2D rectangle represented by its minimum and maximum coordinates,
/// in integer units of logical points
///
/// Alias for `euclid::Box2D<f32, Logical>`.
pub type BoxI32 = euclid::default::Box2D<i32>;

/// A 2D rectangle represented by its minimum and maximum coordinates,
/// in units of physical pixels
///
/// Alias for `euclid::Box<i32, Physical>`.
pub type PhysicalBox = euclid::Box2D<i32, Physical>;

/// A scaling factor
///
/// Alias for `euclid::Scale<f32, Logical>`.
pub type Scale = euclid::default::Scale<f32>;

/// Margin/padding in units of logical points
///
/// Alias for `euclid::SideOffsets2D<f32, Logical>`.
pub type Margin = euclid::default::SideOffsets2D<f32>;

/// 2D transformation matrix in units of logical points
///
/// Alias for `euclid::Transform2D<f32, Logical, UnknownUnit>`.
pub type Transform = euclid::default::Transform2D<f32>;

/// A 2D size in units of logical points
///
/// Alias for `euclid::Size2D<f32, Logical>`.
pub type Size = euclid::default::Size2D<f32>;

/// A 2D size in units of physical pixels
///
/// Alias for `euclid::Size2D<i32, Physical>`.
pub type PhysicalSize = euclid::Size2D<i32, Physical>;

/// A 2D vector in units of logical points
///
/// Alias for `euclid::Vector2D<f32, Logical>`.
pub type Vector = euclid::default::Vector2D<f32>;

/// A 2D vector in units of integer logical points
///
/// Alias for `euclid::Vector2D<f32, Logical>`.
pub type VectorI32 = euclid::default::Vector2D<i32>;

/// An angle
///
/// Alias for `euclid::Angle<f32>`.
pub type Angle = euclid::Angle<f32>;

/// Construct a point in units of logical points.
///
/// Shorthand for `Point::new(x, y)`
#[inline]
pub const fn point(x: f32, y: f32) -> Point {
    Point::new(x, y)
}

/// Construct a rectangle in units of logical points.
///
/// Shorthand for `Rect::new(Point::new(x, y), Size::new(width, height))`
#[inline]
pub const fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}

/// Construct a margin in units of logical points.
///
/// Shorthand for `Margin::new(top, right, bottom, left)`
#[inline]
pub const fn margin(top: f32, right: f32, bottom: f32, left: f32) -> Margin {
    Margin::new(top, right, bottom, left)
}

/// Construct a margin in units of logical points with all fields set
/// to the same value.
///
/// Shorthand for `Margin::new(all, all, all, all)`
#[inline]
pub const fn margin_all_same(all: f32) -> Margin {
    Margin::new(all, all, all, all)
}

/// Construct a size in units of logical points.
///
/// Shorthand for `Size::new(width, height)`
#[inline]
pub const fn size(width: f32, height: f32) -> Size {
    Size::new(width, height)
}

/// Construct a physical size in units of physical pixels.
///
/// Shorthand for `PhysicalSize::new(width, height)`
#[inline]
pub const fn physical_size(width: i32, height: i32) -> PhysicalSize {
    PhysicalSize::new(width, height)
}

/// Construct a vector in units of logical points.
///
/// Shorthand for `Vector::new(x, y)`
#[inline]
pub const fn vector(x: f32, y: f32) -> Vector {
    Vector::new(x, y)
}

/// Construct an angle from radians.
///
/// Shorthand for `Angle { radians }`
#[inline]
pub const fn radians(radians: f32) -> Angle {
    Angle { radians }
}

/// Construct an angle from degrees.
///
/// Shorthand for `Angle { radians: degrees * (180.0 / PI) }`
#[inline]
pub fn degrees(degrees: f32) -> Angle {
    Angle {
        radians: degrees * (180.0 / PI),
    }
}

/// Convert a point in units of physical pixels to units of logical points.
#[inline]
pub fn to_logical_point(point: PhysicalPoint, scale_factor: f32) -> Point {
    point.cast::<f32>().cast_unit::<Logical>() / scale_factor
}

/// Convert a point in units of logical points to units of logical pixels.
#[inline]
pub fn to_physical_point(point: Point, scale_factor: f32) -> PhysicalPoint {
    (point * scale_factor).round().cast::<i32>().cast_unit()
}

/// Convert a size in units of physical pixels to units of logical points.
#[inline]
pub fn to_logical_size(size: PhysicalSize, scale_factor: f32) -> Size {
    size.cast::<f32>().cast_unit() / scale_factor
}

/// Convert a size in units of logical points to units of logical pixels.
#[inline]
pub fn to_physical_size(size: Size, scale_factor: f32) -> PhysicalSize {
    (size * scale_factor).round().cast::<i32>().cast_unit()
}
