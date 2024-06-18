use std::rc::Rc;

use rootvg::PrimitiveGroup;

use crate::view::element::{ElementRenderCache, RenderContext};

use super::{AutomationInfo, ParamMarkersConfig, SteppedValue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UpdateResult {
    pub repaint: bool,
    pub animating: bool,
}

impl Default for UpdateResult {
    fn default() -> Self {
        Self {
            repaint: false,
            animating: false,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualSliderState {
    #[default]
    Idle,
    Hovered,
    Gesturing,
    Disabled,
}

#[derive(Debug)]
pub struct VirtualSliderRenderInfo<'a> {
    pub normal_value: f64,
    pub default_normal: f64,
    pub automation_info: AutomationInfo,
    pub stepped_value: Option<SteppedValue>,
    pub state: VirtualSliderState,
    pub markers: &'a ParamMarkersConfig,
    pub bipolar: bool,
}

pub trait VirtualSliderRenderer: Default + 'static {
    type Style;

    fn does_paint(&self) -> bool {
        true
    }

    #[allow(unused)]
    fn on_state_changed(
        &mut self,
        prev_state: VirtualSliderState,
        new_state: VirtualSliderState,
        style: &Rc<Self::Style>,
    ) -> UpdateResult {
        UpdateResult::default()
    }

    #[allow(unused)]
    /// Return `true` if the element should be repainted.
    fn on_automation_info_update(
        &mut self,
        info: &AutomationInfo,
        style: &Rc<Self::Style>,
    ) -> bool {
        false
    }

    #[allow(unused)]
    fn on_animation(
        &mut self,
        delta_seconds: f64,
        style: &Rc<Self::Style>,
        info: VirtualSliderRenderInfo<'_>,
    ) -> UpdateResult {
        UpdateResult::default()
    }

    #[allow(unused)]
    fn render_primitives(
        &mut self,
        style: &Rc<Self::Style>,
        info: VirtualSliderRenderInfo<'_>,
        cx: RenderContext<'_>,
        primitives: &mut PrimitiveGroup,
    ) {
    }

    /// A unique identifier for the optional global render cache.
    ///
    /// All instances of this element type must return the same value.
    fn global_render_cache_id(&self) -> Option<u32> {
        None
    }

    /// An optional struct that is shared across all instances of this element type
    /// which can be used to cache rendering primitives.
    ///
    /// This will only be called once at the creation of the first instance of this
    /// element type.
    fn global_render_cache(&self) -> Option<Box<dyn ElementRenderCache>> {
        None
    }
}
