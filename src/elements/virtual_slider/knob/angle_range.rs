use rootvg::math::Angle;
use std::f32::consts::PI;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        radians: 35.0 * PI / 180.0,
    };
    /// The default maximum angle of a rotating element such as a Knob
    pub const DEFAULT_MAX: Angle = Angle {
        radians: (360.0 - 35.0) * PI / 180.0,
    };

    /// The span between the `min` and `max` angle (in radians) the knob
    /// will rotate.
    ///
    /// `0.0` radians points straight down at the bottom of the knob, with the
    /// angles rotating clockwise towards `2*PI` radians.
    ///
    /// Values < `0.0` and >= `2*PI` will be set to `0.0`.
    ///
    /// The default minimum is `35` degrees, and the default maximum is `325`
    /// degrees, giving a span of `290` degrees, and a halfway point pointing
    /// strait up.
    ///
    /// # Panics
    ///
    /// This will panic if `min` > `max`.
    pub fn new(mut min: Angle, mut max: Angle) -> Self {
        assert!(min <= max);

        if min.radians < 0.0 || min.radians >= 2.0 * PI {
            log::warn!(
                "KnobAngleRange min value {} is out of range of [0.0, 2.0*PI), using 0.0 instead",
                min.radians
            );
            min.radians = 0.0;
        }
        if max.radians < 0.0 || max.radians >= 2.0 * PI {
            log::warn!(
                "KnobAngleRange max value {} is out of range of [0.0, 2.0*PI), using 0.0 instead",
                max.radians
            );
            max.radians = 0.0;
        }

        Self { min, max }
    }

    /// The range between the `min` and `max` angle (in degrees) a knob
    /// will rotate.
    ///
    /// `0.0` degrees points straight down at the bottom of the knob, with the
    /// angles rotating clockwise towards `360` degrees.
    ///
    /// Values < `0.0` and >= `360.0` will be set to `0.0`.
    ///
    /// The default minimum is `35` degrees, and the default maximum is `325`
    /// degrees, giving a span of `290` degrees, and a halfway point pointing
    /// strait up.
    ///
    /// # Panics
    ///
    /// This will panic if `min` > `max`.
    pub fn from_degrees(min: f32, max: f32) -> Self {
        let min_rad = Angle {
            radians: min * PI / 180.0,
        };
        let max_rad = Angle {
            radians: max * PI / 180.0,
        };

        Self::new(min_rad, max_rad)
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
