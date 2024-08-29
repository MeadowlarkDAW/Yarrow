use std::{any::Any, rc::Rc};

use rootvg::{math::Size, PrimitiveGroup};

use crate::{
    element_system::element::{ElementRenderCache, RenderContext},
    prelude::ElementStyle,
};

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
    pub horizontal: bool,
}

pub trait VirtualSliderRenderer: 'static {
    type Style: ElementStyle;

    fn new(style: Rc<dyn Any>) -> Self;

    fn does_paint(&self) -> bool {
        true
    }

    fn style_changed(&mut self, new_style: Rc<dyn Any>);

    fn desired_size(&self) -> Option<Size> {
        None
    }

    #[allow(unused)]
    fn on_state_changed(
        &mut self,
        prev_state: VirtualSliderState,
        new_state: VirtualSliderState,
    ) -> UpdateResult {
        UpdateResult::default()
    }

    #[allow(unused)]
    /// Return `true` if the element should be repainted.
    fn on_automation_info_update(&mut self, info: &AutomationInfo) -> bool {
        false
    }

    #[allow(unused)]
    fn on_animation(
        &mut self,
        delta_seconds: f64,
        info: VirtualSliderRenderInfo<'_>,
    ) -> UpdateResult {
        UpdateResult::default()
    }

    #[allow(unused)]
    fn render(
        &mut self,
        info: VirtualSliderRenderInfo<'_>,
        cx: RenderContext,
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
