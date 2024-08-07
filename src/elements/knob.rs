use rootvg::{
    math::{Rect, Size},
    PrimitiveGroup,
};
use std::{any::Any, rc::Rc};

use crate::{
    layout::SizeType,
    prelude::ElementStyle,
    view::element::{ElementRenderCache, RenderContext},
};

use super::virtual_slider::{
    UpdateResult, VirtualSlider, VirtualSliderRenderInfo, VirtualSliderRenderer, VirtualSliderState,
};

use self::arc::CachedKnobMarkerArcFrontMesh;
use self::cache::{KnobRenderCache, KnobRenderCacheInner};

mod angle_range;
mod arc;
mod cache;
mod markers_dot;
mod notch_line;
mod quad;

pub use angle_range::KnobAngleRange;
pub use arc::KnobMarkersArcStyle;
pub use markers_dot::KnobMarkersDotStyle;
pub use notch_line::{KnobNotchLinePrimitives, KnobNotchStyleLine, KnobNotchStyleLineBg};
pub use quad::{KnobBackStyleQuad, KnobNotchStyleQuad};

#[derive(Default, Debug, Clone)]
pub struct KnobStyle {
    pub back: KnobBackStyle,
    pub notch: KnobNotchStyle,
    pub markers: KnobMarkersStyle,
    pub angle_range: KnobAngleRange,
}

impl KnobStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        self.back.states_differ(a, b)
            || self.notch.states_differ(a, b)
            || self.markers.states_differ(a, b)
    }

    pub fn back_bounds(&self, element_size: Size) -> Rect {
        self.back.back_bounds(element_size)
    }
}

impl ElementStyle for KnobStyle {
    const ID: &'static str = "vs-knob";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KnobBackStyle {
    Quad(KnobBackStyleQuad),
    None,
}

impl KnobBackStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        match self {
            Self::Quad(s) => s.states_differ(a, b),
            Self::None => false,
        }
    }

    pub fn size(&self) -> SizeType {
        match self {
            Self::Quad(s) => s.size,
            Self::None => SizeType::Scale(1.0),
        }
    }

    pub fn back_bounds(&self, element_size: Size) -> Rect {
        match self {
            Self::Quad(s) => s.back_bounds(element_size),
            Self::None => Rect::from_size(element_size),
        }
    }
}

impl Default for KnobBackStyle {
    fn default() -> Self {
        Self::Quad(KnobBackStyleQuad::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KnobNotchStyle {
    Quad(KnobNotchStyleQuad),
    Line(KnobNotchStyleLine),
    None,
}

impl KnobNotchStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        match self {
            Self::Quad(s) => s.states_differ(a, b),
            Self::Line(s) => s.states_differ(a, b),
            Self::None => false,
        }
    }
}

impl Default for KnobNotchStyle {
    fn default() -> Self {
        Self::Quad(KnobNotchStyleQuad::default())
    }
}

#[derive(Debug, Clone)]
pub enum KnobMarkersStyle {
    Dots(KnobMarkersDotStyle),
    Arc(KnobMarkersArcStyle),
    None,
}

impl KnobMarkersStyle {
    pub fn states_differ(&self, a: VirtualSliderState, b: VirtualSliderState) -> bool {
        match self {
            Self::Dots(_) => false,
            Self::Arc(s) => s.states_differ(a, b),
            Self::None => false,
        }
    }
}

impl Default for KnobMarkersStyle {
    fn default() -> Self {
        //Self::Dots(KnobMarkersDotStyle::default())
        Self::Arc(KnobMarkersArcStyle::default())
    }
}

pub struct KnobRenderer {
    cached_arc_marker_front_mesh: CachedKnobMarkerArcFrontMesh,
    style: Rc<dyn Any>,
}

impl VirtualSliderRenderer for KnobRenderer {
    type Style = KnobStyle;

    fn new(style: Rc<dyn Any>) -> Self {
        Self {
            cached_arc_marker_front_mesh: Default::default(),
            style,
        }
    }

    fn style_changed(&mut self, new_style: Rc<dyn Any>) {
        self.style = new_style;
    }

    fn desired_size(&self) -> Option<Size> {
        let style = self.style.downcast_ref::<KnobStyle>().unwrap();

        match style.back.size() {
            SizeType::FixedPoints(size) => Some(Size::new(size, size)),
            SizeType::Scale(_) => None,
        }
    }

    fn on_state_changed(
        &mut self,
        prev_state: VirtualSliderState,
        new_state: VirtualSliderState,
    ) -> UpdateResult {
        let style = self.style.downcast_ref::<KnobStyle>().unwrap();

        // Only repaint if the appearance is different.
        UpdateResult {
            repaint: style.states_differ(prev_state, new_state),
            animating: false,
        }
    }

    fn render_primitives(
        &mut self,
        info: VirtualSliderRenderInfo<'_>,
        mut cx: RenderContext<'_>,
        primitives: &mut PrimitiveGroup,
    ) {
        let style = self.style.downcast_ref::<KnobStyle>().unwrap();

        let back_bounds = style.back_bounds(cx.bounds_size);

        match &style.back {
            KnobBackStyle::Quad(s) => {
                primitives.add(s.create_primitive(info.state, back_bounds));
            }
            KnobBackStyle::None => {}
        }

        match &style.markers {
            KnobMarkersStyle::Dots(s) => {
                s.add_primitives(
                    &info.markers,
                    back_bounds,
                    info.bipolar,
                    info.stepped_value.map(|s| s.num_steps),
                    style.angle_range,
                    primitives,
                );
            }
            KnobMarkersStyle::Arc(_) => {
                let render_cache = cx
                    .render_cache
                    .as_mut()
                    .unwrap()
                    .get_mut()
                    .downcast_mut::<KnobRenderCacheInner>()
                    .unwrap();

                primitives.add(
                    render_cache
                        .marker_arc_back_mesh(cx.class, style, back_bounds)
                        .unwrap(),
                );

                let normal_val = info
                    .automation_info
                    .current_normal
                    .unwrap_or(info.normal_value) as f32;

                if let Some(front_mesh) = self.cached_arc_marker_front_mesh.create_primitive(
                    cx.class,
                    style,
                    back_bounds,
                    normal_val,
                    info.state,
                    info.bipolar,
                ) {
                    primitives.set_z_index(1);
                    primitives.add_mesh(front_mesh);
                }
            }
            KnobMarkersStyle::None => {}
        }

        match &style.notch {
            KnobNotchStyle::Quad(s) => {
                let normal_val = info
                    .automation_info
                    .current_normal
                    .unwrap_or(info.normal_value) as f32;

                primitives.set_z_index(1);
                primitives.add(s.create_primitive(
                    normal_val,
                    style.angle_range,
                    info.state,
                    back_bounds,
                ));
            }
            KnobNotchStyle::Line(_) => {
                let normal_val = info
                    .automation_info
                    .current_normal
                    .unwrap_or(info.normal_value) as f32;

                let render_cache = cx
                    .render_cache
                    .as_mut()
                    .unwrap()
                    .get_mut()
                    .downcast_mut::<KnobRenderCacheInner>()
                    .unwrap();

                let meshes = render_cache
                    .notch_line_mesh(cx.class, style, back_bounds.width())
                    .unwrap();

                primitives.set_z_index(1);
                primitives.add(meshes.transformed_mesh(
                    normal_val,
                    style.angle_range,
                    info.state,
                    back_bounds,
                ));
            }
            KnobNotchStyle::None => {}
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
