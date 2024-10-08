use rootvg::{
    color::RGBA8,
    math::{Point, Rect, Size, Transform},
    mesh::{MeshPrimitive, SolidMeshPrimitive},
};

#[cfg(feature = "gradient")]
use rootvg::{gradient::Gradient, mesh::GradientMeshPrimitive};

use crate::{elements::virtual_slider::VirtualSliderState, layout::SizeType, style::DisabledColor};

#[cfg(feature = "gradient")]
use crate::style::DisabledGradient;

use super::KnobAngleRange;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum KnobNotchStyleLineBg {
    Solid {
        idle: RGBA8,
        hovered: Option<RGBA8>,
        gesturing: Option<RGBA8>,
        disabled: DisabledColor,
    },
    #[cfg(feature = "gradient")]
    Gradient {
        idle: Gradient,
        hovered: Option<Gradient>,
        gesturing: Option<Gradient>,
        disabled: DisabledGradient,
    },
}

impl Default for KnobNotchStyleLineBg {
    fn default() -> Self {
        Self::Solid {
            idle: RGBA8::new(255, 255, 255, 255),
            hovered: None,
            gesturing: None,
            disabled: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnobNotchStyleLine {
    pub bg: KnobNotchStyleLineBg,
    pub width: SizeType,
    pub height: SizeType,
    /// * When `SizeType::Fixed(value)`, the value is the distance from the
    /// edge of the knob to the center of the notch in points.
    /// * When `SizeType::Scale(value)`, a value of `0.0` is on the edge of
    /// the knob and a value of `1.0` is in the center of the knob.
    pub edge_offset: SizeType,
}

impl KnobNotchStyleLine {
    pub fn create_primitives(&self, back_size: f32) -> KnobNotchLinePrimitives {
        KnobNotchLinePrimitives::new(self, back_size)
    }
}

impl Default for KnobNotchStyleLine {
    fn default() -> Self {
        Self {
            bg: KnobNotchStyleLineBg::default(),
            width: SizeType::Scale(0.075),
            height: SizeType::Scale(0.25),
            edge_offset: SizeType::Scale(0.08),
        }
    }
}

pub struct KnobNotchLinePrimitives {
    pub idle: MeshPrimitive,
    pub hovered: MeshPrimitive,
    pub gesturing: MeshPrimitive,
    pub disabled: MeshPrimitive,
}

impl KnobNotchLinePrimitives {
    pub fn new(style: &KnobNotchStyleLine, back_size: f32) -> Self {
        let line_size = Size::new(
            match style.width {
                SizeType::FixedPoints(points) => points,
                SizeType::Scale(scale) => back_size * scale,
            },
            match style.height {
                SizeType::FixedPoints(points) => points,
                SizeType::Scale(scale) => back_size * scale,
            },
        );

        let edge_offset = match style.edge_offset {
            SizeType::FixedPoints(points) => points,
            SizeType::Scale(scale) => back_size * scale,
        };

        let rect = Rect::new(
            Point::new(
                line_size.width * -0.5,
                (back_size * 0.5) - line_size.height - edge_offset,
            ),
            line_size,
        );

        match &style.bg {
            KnobNotchStyleLineBg::Solid {
                idle,
                hovered,
                gesturing,
                disabled,
            } => {
                let hovered = hovered.unwrap_or(*idle);
                let gesturing = gesturing.unwrap_or(hovered);
                let disabled = disabled.get(*idle);

                let idle_mesh = SolidMeshPrimitive::from_rect(rect, *idle);
                let hovered_mesh = if hovered == *idle {
                    idle_mesh.clone()
                } else {
                    SolidMeshPrimitive::from_rect(rect, hovered)
                };
                let gesturing_mesh = if gesturing == *idle {
                    idle_mesh.clone()
                } else if gesturing == hovered {
                    hovered_mesh.clone()
                } else {
                    SolidMeshPrimitive::from_rect(rect, gesturing)
                };
                let disabled_mesh = if disabled == *idle {
                    idle_mesh.clone()
                } else {
                    SolidMeshPrimitive::from_rect(rect, disabled)
                };

                Self {
                    idle: MeshPrimitive::Solid(idle_mesh),
                    hovered: MeshPrimitive::Solid(hovered_mesh),
                    gesturing: MeshPrimitive::Solid(gesturing_mesh),
                    disabled: MeshPrimitive::Solid(disabled_mesh),
                }
            }
            #[cfg(feature = "gradient")]
            KnobNotchStyleLineBg::Gradient {
                idle,
                hovered,
                gesturing,
                disabled,
            } => {
                let hovered = hovered.unwrap_or(*idle);
                let gesturing = gesturing.unwrap_or(hovered);
                let disabled = disabled.get(*idle);

                let idle_mesh = GradientMeshPrimitive::from_rect(rect, idle);
                let hovered_mesh = if hovered == *idle {
                    idle_mesh.clone()
                } else {
                    GradientMeshPrimitive::from_rect(rect, &hovered)
                };
                let gesturing_mesh = if gesturing == *idle {
                    idle_mesh.clone()
                } else if gesturing == hovered {
                    hovered_mesh.clone()
                } else {
                    GradientMeshPrimitive::from_rect(rect, &gesturing)
                };
                let disabled_mesh = if disabled == *idle {
                    idle_mesh.clone()
                } else {
                    GradientMeshPrimitive::from_rect(rect, &disabled)
                };

                Self {
                    idle: MeshPrimitive::Gradient(idle_mesh),
                    hovered: MeshPrimitive::Gradient(hovered_mesh),
                    gesturing: MeshPrimitive::Gradient(gesturing_mesh),
                    disabled: MeshPrimitive::Gradient(disabled_mesh),
                }
            }
        }
    }

    pub fn mesh(&self, state: VirtualSliderState) -> &MeshPrimitive {
        match state {
            VirtualSliderState::Idle => &self.idle,
            VirtualSliderState::Hovered => &self.hovered,
            VirtualSliderState::Gesturing => &self.gesturing,
            VirtualSliderState::Disabled => &self.disabled,
        }
    }

    pub fn transformed_mesh(
        &self,
        normal_val: f32,
        angle_range: KnobAngleRange,
        state: VirtualSliderState,
        back_bounds: Rect,
    ) -> MeshPrimitive {
        let mut mesh = self.mesh(state).clone();

        mesh.set_offset(back_bounds.center().to_vector());

        let notch_angle = angle_range.min() + (angle_range.span() * normal_val);

        mesh.set_transform(Transform::identity().then_rotate(notch_angle));

        mesh
    }
}
