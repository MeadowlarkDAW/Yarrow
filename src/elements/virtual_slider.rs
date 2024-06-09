use std::{cell::RefCell, ops::Range, rc::Rc};

use keyboard_types::Modifiers;
use rootvg::{
    math::{Point, Rect, Vector, ZIndex},
    PrimitiveGroup,
};

use crate::{
    event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent, WheelDeltaType},
    layout::Align2,
    view::element::{
        Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
    },
    ScissorRectID, WindowContext, MAIN_SCISSOR_RECT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureState {
    /// The user has just starting gesturing (dragging) this element.
    GestureStarted,
    /// The user is in the process of gesturing (dragging) this element.
    Gesturing,
    /// The user has just finished gesturing (dragging) this element.
    GestureFinished,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamUpdate {
    /// The parameter ID
    pub param_id: u32,
    /// The normalized value in the range `[0.0, 1.0]`
    pub normal_value: f64,
    /// The stepped value (if this parameter is stepped)
    pub stepped_value: Option<u32>,
    /// The current state of gesturing (dragging)
    pub gesture_state: GestureState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamOpenTextEntryInfo {
    /// The parameter ID
    pub param_id: u32,
    /// The normalized value in the range `[0.0, 1.0]`
    pub normal_value: f64,
    /// The stepped value (if this parameter is stepped)
    pub stepped_value: Option<u32>,
    /// The bounding rectangle of this element
    pub bounds: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualSliderConfig {
    /// The scalar (points to normalized units) to use when dragging.
    ///
    /// By default this is set to `0.00275`.
    pub drag_scalar: f32,

    /// The scalar (points to normalized units) to use when scrolling.
    ///
    /// By default this is set to `0.0004`.
    pub scroll_wheel_scalar: f32,

    /// How many points per line when using the scroll wheel (for backends
    /// that send a scroll wheel amount in lines instead of points).
    ///
    /// By default this is set to `24.0`.
    pub scroll_wheel_points_per_line: f32,

    /// An additional scalar to apply when the modifier key is held down.
    ///
    /// By default this is set to `0.02`.
    pub fine_adjustment_scalar: f32,

    /// Whether or not the scroll wheel should adjust this parameter.
    ///
    /// By default this is set to `true`.
    pub use_scroll_wheel: bool,

    /// The modifier key to use when making fine adjustments.
    ///
    /// Set this to `None` to disable the fine adjustment modifier.
    ///
    /// By default this is set to `Some(Modifiers::SHIFT)`
    pub fine_adjustment_modifier: Option<Modifiers>,

    /// Activate the `on_open_text_entry` event when the user selects
    /// this element with this modifier held done.
    ///
    /// Set this to `None` to disable this.
    ///
    /// By default this is set to `Some(Modifiers::CONTROL)`
    pub open_text_entry_modifier: Option<Modifiers>,

    /// Whether or not to activate the `on_open_text_entry` event when
    /// the user middle-clicks this element.
    ///
    /// By default this is set to `true`.
    pub open_text_entry_on_middle_click: bool,

    /// Whether or not to disabled locking the pointer in place while
    /// dragging this element.
    ///
    /// By default this is set to `false`.
    pub disable_pointer_locking: bool,
}

impl Default for VirtualSliderConfig {
    fn default() -> Self {
        Self {
            drag_scalar: 0.00275,
            scroll_wheel_scalar: 0.0004,
            scroll_wheel_points_per_line: 24.0,
            fine_adjustment_scalar: 0.02,
            use_scroll_wheel: true,
            fine_adjustment_modifier: Some(Modifiers::SHIFT),
            open_text_entry_modifier: Some(Modifiers::CONTROL),
            open_text_entry_on_middle_click: true,
            disable_pointer_locking: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BeginGestureType {
    Dragging {
        pointer_start_pos: Point,
        start_normal: f64,
    },
    ScrollWheel,
}

#[derive(Clone, Copy)]
struct SteppedParamState {
    value: u32,
    num_steps: u32,
}

/// A reusable "virtual slider" struct that can be used to make
/// elements like knobs and sliders.
pub struct VirtualSliderInner {
    pub param_id: u32,
    pub config: VirtualSliderConfig,
    pub drag_vertically: bool,
    pub scroll_vertically: bool,

    normal_value: f64,
    default_normal: f64,
    continuous_gesture_normal: f64,
    stepped_state: Option<SteppedParamState>,
    current_gesture: Option<BeginGestureType>,
}

impl VirtualSliderInner {
    pub fn new(
        param_id: u32,
        normal_value: f64,
        default_normal: f64,
        num_quantized_steps: Option<u32>,
        config: VirtualSliderConfig,
        drag_vertically: bool,
        scroll_vertically: bool,
    ) -> Self {
        let (normal_value, default_normal, stepped_state) =
            if let Some(num_steps) = num_quantized_steps {
                let stepped_value = param_normal_to_quantized(normal_value, num_steps);

                (
                    param_quantized_to_normal(stepped_value, num_steps),
                    param_snap_normal(default_normal, num_steps),
                    Some(SteppedParamState {
                        value: stepped_value,
                        num_steps,
                    }),
                )
            } else {
                (
                    normal_value.clamp(0.0, 1.0),
                    default_normal.clamp(0.0, 1.0),
                    None,
                )
            };

        Self {
            param_id,
            config,
            drag_vertically,
            scroll_vertically,
            normal_value,
            default_normal,
            stepped_state,
            continuous_gesture_normal: normal_value,
            current_gesture: None,
        }
    }

    pub fn begin_drag_gesture(&mut self, pointer_start_pos: Point) -> Option<ParamUpdate> {
        if self.current_gesture.is_some() {
            None
        } else {
            self.current_gesture = Some(BeginGestureType::Dragging {
                pointer_start_pos,
                start_normal: self.normal_value,
            });

            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: GestureState::GestureStarted,
            })
        }
    }

    pub fn begin_scroll_wheel_gesture(&mut self) -> Option<ParamUpdate> {
        if self.current_gesture.is_some() {
            None
        } else {
            self.current_gesture = Some(BeginGestureType::ScrollWheel);

            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: GestureState::GestureStarted,
            })
        }
    }

    pub fn is_gesturing(&self) -> bool {
        self.current_gesture.is_some()
    }

    pub fn handle_pointer_moved(
        &mut self,
        pointer_pos: Point,
        pointer_delta: Option<Point>,
        modifiers: Modifiers,
    ) -> Option<ParamUpdate> {
        if let Some(BeginGestureType::Dragging {
            pointer_start_pos,
            start_normal,
        }) = &mut self.current_gesture
        {
            let use_pointer_delta = !self.config.disable_pointer_locking && pointer_delta.is_some();

            let apply_fine_adjustment_scalar = if let Some(m) = self.config.fine_adjustment_modifier
            {
                modifiers == m
            } else {
                false
            };

            let (new_gesture_normal, reset_start_pos) = if use_pointer_delta {
                let delta = pointer_delta.unwrap();
                let delta_points = if self.drag_vertically {
                    -delta.y
                } else {
                    delta.x
                };

                let mut delta_normal = delta_points * self.config.drag_scalar;
                if apply_fine_adjustment_scalar {
                    delta_normal *= self.config.fine_adjustment_scalar;
                }

                (
                    self.continuous_gesture_normal + f64::from(delta_normal),
                    true,
                )
            } else if apply_fine_adjustment_scalar {
                let delta_points = if self.drag_vertically {
                    pointer_start_pos.y - pointer_pos.y
                } else {
                    pointer_pos.x - pointer_start_pos.x
                };

                let delta_normal =
                    delta_points * self.config.drag_scalar * self.config.fine_adjustment_scalar;

                (
                    self.continuous_gesture_normal + f64::from(delta_normal),
                    true,
                )
            } else {
                // Use absolute positions instead of deltas for a "better feel".
                let offset = if self.drag_vertically {
                    pointer_start_pos.y - pointer_pos.y
                } else {
                    pointer_pos.x - pointer_start_pos.x
                };

                (
                    *start_normal + f64::from(offset * self.config.drag_scalar),
                    false,
                )
            };

            if reset_start_pos {
                *pointer_start_pos = pointer_pos;
                *start_normal = self.continuous_gesture_normal;
            }

            self.set_new_gesture_normal(new_gesture_normal)
        } else {
            None
        }
    }

    pub fn handle_scroll_wheel(
        &mut self,
        delta_type: WheelDeltaType,
        modifiers: Modifiers,
    ) -> Option<ParamUpdate> {
        if !self.config.use_scroll_wheel {
            return None;
        }

        let apply_fine_adjustment_scalar = if let Some(m) = self.config.fine_adjustment_modifier {
            modifiers == m
        } else {
            false
        };

        let delta = match delta_type {
            WheelDeltaType::Points(points) => points,
            WheelDeltaType::Lines(lines) => lines * self.config.scroll_wheel_points_per_line,
            // Don't handle scrolling by pages.
            WheelDeltaType::Pages(_) => Vector::default(),
        };

        let delta_points = if self.drag_vertically {
            delta.y
        } else {
            delta.x
        };

        if delta_points == 0.0 {
            return None;
        }

        let mut delta_normal = delta_points * self.config.scroll_wheel_scalar;
        if apply_fine_adjustment_scalar {
            delta_normal *= self.config.fine_adjustment_scalar;
        }

        let new_gesture_normal = self.continuous_gesture_normal - f64::from(delta_normal);

        self.set_new_gesture_normal(new_gesture_normal)
    }

    fn set_new_gesture_normal(&mut self, mut new_gesture_normal: f64) -> Option<ParamUpdate> {
        new_gesture_normal = new_gesture_normal.clamp(0.0, 1.0);

        if new_gesture_normal == self.continuous_gesture_normal {
            return None;
        }

        self.continuous_gesture_normal = new_gesture_normal;

        let value_changed = if let Some(stepped_state) = &mut self.stepped_state {
            let new_val = param_normal_to_quantized(new_gesture_normal, stepped_state.num_steps);
            let changed = stepped_state.value != new_val;
            stepped_state.value = new_val;
            changed
        } else {
            let changed = self.normal_value != new_gesture_normal;
            self.normal_value = new_gesture_normal;
            changed
        };

        if value_changed {
            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: GestureState::Gesturing,
            })
        } else {
            None
        }
    }

    pub fn finish_gesture(&mut self) -> Option<ParamUpdate> {
        self.current_gesture.take().map(|_| ParamUpdate {
            param_id: self.param_id,
            normal_value: self.normal_value,
            stepped_value: self.stepped_value(),
            gesture_state: GestureState::GestureFinished,
        })
    }

    pub fn reset_to_default(&mut self) -> Option<ParamUpdate> {
        self.continuous_gesture_normal = self.default_normal;

        if let Some(_) = self.current_gesture.take() {
            self.normal_value = self.default_normal;

            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: GestureState::GestureFinished,
            })
        } else if self.normal_value != self.default_normal {
            self.normal_value = self.default_normal;

            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: GestureState::GestureFinished,
            })
        } else {
            None
        }
    }

    pub fn stepped_value(&self) -> Option<u32> {
        self.stepped_state.map(|s| s.value)
    }

    pub fn num_quantized_steps(&self) -> Option<u32> {
        self.stepped_state.map(|s| s.num_steps)
    }

    /// Set the normalized value of the virtual slider.
    ///
    /// If the slider is currently gesturing, then the gesture will
    /// be canceled.
    pub fn set_normal_value(&mut self, new_normal: f64) -> Option<ParamUpdate> {
        let new_normal = if let Some(stepped_state) = &mut self.stepped_state {
            stepped_state.value = param_normal_to_quantized(new_normal, stepped_state.num_steps);

            param_quantized_to_normal(stepped_state.value, stepped_state.num_steps)
        } else {
            new_normal.clamp(0.0, 1.0)
        };

        let state_changed = self.current_gesture.is_some() || self.normal_value != new_normal;

        self.normal_value = new_normal;
        self.continuous_gesture_normal = new_normal;
        self.current_gesture = None;

        if state_changed {
            Some(ParamUpdate {
                param_id: self.param_id,
                normal_value: self.normal_value,
                stepped_value: self.stepped_value(),
                gesture_state: GestureState::GestureFinished,
            })
        } else {
            None
        }
    }

    /// Set the normalized default value of the virtual slider.
    ///
    /// Returns `true` if the default value has changed.
    pub fn set_default_normal(&mut self, new_normal: f64) -> bool {
        let new_normal = self.snap_normal(new_normal);

        let changed = self.default_normal != new_normal;
        self.default_normal = new_normal;
        changed
    }

    pub fn snap_normal(&self, normal: f64) -> f64 {
        if let Some(stepped_state) = self.stepped_state {
            param_snap_normal(normal, stepped_state.num_steps)
        } else {
            normal.clamp(0.0, 1.0)
        }
    }

    pub fn normal_value(&self) -> f64 {
        self.normal_value
    }

    pub fn default_normal(&self) -> f64 {
        self.default_normal
    }
}

pub fn param_quantized_to_normal(value: u32, num_steps: u32) -> f64 {
    if value == 0 || num_steps == 0 {
        0.0
    } else if value >= num_steps {
        1.0
    } else {
        f64::from(value) / f64::from(num_steps)
    }
}

pub fn param_normal_to_quantized(normal: f64, num_steps: u32) -> u32 {
    if normal <= 0.0 || num_steps == 0 {
        0
    } else if normal >= 1.0 {
        num_steps
    } else {
        (normal * f64::from(num_steps)).round() as u32
    }
}

pub fn param_snap_normal(normal: f64, num_steps: u32) -> f64 {
    param_quantized_to_normal(param_normal_to_quantized(normal, num_steps), num_steps)
}

// --------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct NormalsState {
    pub normal_value: f64,
    pub default_normal: f64,
    pub automation_info: AutomationInfo,
    pub num_quantized_steps: Option<u32>,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualSliderState {
    Idle,
    Hovered,
    Gesturing,
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
        style: &Self::Style,
    ) -> UpdateResult {
        UpdateResult::default()
    }

    #[allow(unused)]
    /// Return `true` if the element should be repainted.
    fn on_automation_info_update(&mut self, info: &AutomationInfo, style: &Self::Style) -> bool {
        false
    }

    #[allow(unused)]
    fn on_animation(
        &mut self,
        delta_seconds: f64,
        normals: NormalsState,
        state: VirtualSliderState,
        style: &Self::Style,
    ) -> UpdateResult {
        UpdateResult::default()
    }

    #[allow(unused)]
    fn render_primitives(
        &mut self,
        style: &Self::Style,
        normals: NormalsState,
        state: VirtualSliderState,
        disabled: bool,
        cx: RenderContext<'_>,
        primitives: &mut PrimitiveGroup,
    ) {
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamTooltipInfo {
    /// The parameter ID
    pub param_id: u32,
    /// The normalized value in the range `[0.0, 1.0]`
    pub normal_value: f64,
    /// The stepped value (if this parameter is stepped)
    pub stepped_value: Option<u32>,
    pub bounding_rect: Rect,
    pub tooltip_align: Align2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamRightClickInfo {
    /// The parameter ID
    pub param_id: u32,
    /// The normalized value in the range `[0.0, 1.0]`
    pub normal_value: f64,
    /// The stepped value (if this parameter is stepped)
    pub stepped_value: Option<u32>,
    pub pointer_pos: Point,
}

pub struct VirtualSliderBuilder<A: Clone + 'static, R: VirtualSliderRenderer> {
    pub on_gesture: Option<Box<dyn FnMut(ParamUpdate) -> A>>,
    pub on_right_click: Option<Box<dyn FnMut(ParamRightClickInfo) -> A>>,
    pub on_open_text_entry: Option<Box<dyn FnMut(ParamOpenTextEntryInfo) -> A>>,
    pub on_tooltip_request: Option<Box<dyn FnMut(ParamTooltipInfo) -> A>>,
    pub style: Rc<R::Style>,
    pub tooltip_align: Align2,
    pub param_id: u32,
    pub normal_value: f64,
    pub default_normal: f64,
    pub num_quantized_steps: Option<u32>,
    pub config: VirtualSliderConfig,
    pub drag_vertically: bool,
    pub scroll_vertically: bool,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
    pub disabled: bool,
}

impl<A: Clone + 'static, R: VirtualSliderRenderer> VirtualSliderBuilder<A, R> {
    pub fn new(param_id: u32, style: &Rc<R::Style>) -> Self {
        Self {
            on_gesture: None,
            on_right_click: None,
            on_open_text_entry: None,
            on_tooltip_request: None,
            style: Rc::clone(style),
            tooltip_align: Align2::TOP_CENTER,
            param_id,
            normal_value: 0.0,
            default_normal: 0.0,
            num_quantized_steps: None,
            config: VirtualSliderConfig::default(),
            drag_vertically: true,
            scroll_vertically: true,
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
            disabled: false,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> VirtualSlider<R> {
        VirtualSliderElement::create(self, cx)
    }

    pub fn on_gesture<F: FnMut(ParamUpdate) -> A + 'static>(mut self, f: F) -> Self {
        self.on_gesture = Some(Box::new(f));
        self
    }

    pub fn on_right_click<F: FnMut(ParamRightClickInfo) -> A + 'static>(mut self, f: F) -> Self {
        self.on_right_click = Some(Box::new(f));
        self
    }

    pub fn on_open_text_entry<F: FnMut(ParamOpenTextEntryInfo) -> A + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.on_open_text_entry = Some(Box::new(f));
        self
    }

    pub fn on_tooltip_request<F: FnMut(ParamTooltipInfo) -> A + 'static>(
        mut self,
        f: F,
        align: Align2,
    ) -> Self {
        self.on_tooltip_request = Some(Box::new(f));
        self.tooltip_align = align;
        self
    }

    pub const fn normal_value(mut self, normal: f64) -> Self {
        self.normal_value = normal;
        self
    }

    pub const fn default_normal(mut self, normal: f64) -> Self {
        self.default_normal = normal;
        self
    }

    pub const fn num_quantized_steps(mut self, num_steps: Option<u32>) -> Self {
        self.num_quantized_steps = num_steps;
        self
    }

    pub const fn config(mut self, config: VirtualSliderConfig) -> Self {
        self.config = config;
        self
    }

    pub const fn drag_vertically(mut self, drag_vertically: bool) -> Self {
        self.drag_vertically = drag_vertically;
        self
    }

    pub const fn scroll_vertically(mut self, scroll_vertically: bool) -> Self {
        self.scroll_vertically = scroll_vertically;
        self
    }

    pub const fn z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = z_index;
        self
    }

    pub const fn bounding_rect(mut self, rect: Rect) -> Self {
        self.bounding_rect = rect;
        self
    }

    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.manually_hidden = hidden;
        self
    }

    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = scissor_rect_id;
        self
    }

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct VirtualSliderElement<A: Clone + 'static, R: VirtualSliderRenderer + 'static> {
    shared_state: Rc<RefCell<SharedState<R>>>,

    on_gesture: Option<Box<dyn FnMut(ParamUpdate) -> A>>,
    on_right_click: Option<Box<dyn FnMut(ParamRightClickInfo) -> A>>,
    on_open_text_entry: Option<Box<dyn FnMut(ParamOpenTextEntryInfo) -> A>>,
    on_tooltip_request: Option<Box<dyn FnMut(ParamTooltipInfo) -> A>>,
    tooltip_align: Align2,

    renderer: R,
    hovered: bool,
    state: VirtualSliderState,
}

impl<A: Clone + 'static, R: VirtualSliderRenderer + 'static> VirtualSliderElement<A, R> {
    pub fn create(
        builder: VirtualSliderBuilder<A, R>,
        cx: &mut WindowContext<'_, A>,
    ) -> VirtualSlider<R> {
        let VirtualSliderBuilder {
            on_gesture,
            on_right_click,
            on_open_text_entry,
            on_tooltip_request,
            style,
            tooltip_align,
            param_id,
            normal_value,
            default_normal,
            num_quantized_steps,
            config,
            drag_vertically,
            scroll_vertically,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
            disabled,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: VirtualSliderInner::new(
                param_id,
                normal_value,
                default_normal,
                num_quantized_steps,
                config,
                drag_vertically,
                scroll_vertically,
            ),
            style,
            automation_info: AutomationInfo::default(),
            automation_info_changed: false,
            needs_repaint: false,
            disabled,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                on_gesture,
                on_right_click,
                on_open_text_entry,
                on_tooltip_request,
                tooltip_align,
                renderer: R::default(),
                hovered: false,
                state: VirtualSliderState::Idle,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, cx.font_system, cx.clipboard);

        VirtualSlider { el, shared_state }
    }
}

impl<A: Clone + 'static, R: VirtualSliderRenderer + 'static> Element<A>
    for VirtualSliderElement<A, R>
{
    fn flags(&self) -> ElementFlags {
        let mut flags = ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
            | ElementFlags::LISTENS_TO_POINTER_OUTSIDE_BOUNDS_WHEN_FOCUSED
            | ElementFlags::LISTENS_TO_FOCUS_CHANGE;

        if self.renderer.does_paint() {
            flags.insert(ElementFlags::PAINTS);
        }

        flags
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState {
            inner,
            style,
            automation_info,
            automation_info_changed,
            disabled,
            needs_repaint,
        } = &mut *shared_state;

        let send_param_update =
            |param_update: ParamUpdate,
             cx: &mut ElementContext<'_, A>,
             renderer: &mut R,
             style: &R::Style,
             prev_state: Option<VirtualSliderState>,
             state: VirtualSliderState,
             on_gesture: &mut Option<Box<dyn FnMut(ParamUpdate) -> A>>| {
                if let Some(f) = on_gesture.as_mut() {
                    cx.send_action((f)(param_update)).unwrap();
                }

                if renderer.does_paint() {
                    cx.request_repaint();
                }

                if let Some(prev_state) = prev_state {
                    let res = renderer.on_state_changed(prev_state, state, style);
                    cx.set_animating(res.animating);
                }
            };

        let finish_gesture =
            |inner: &mut VirtualSliderInner,
             cx: &mut ElementContext<'_, A>,
             hovered: bool,
             state: &mut VirtualSliderState,
             renderer: &mut R,
             style: &R::Style,
             on_gesture: &mut Option<Box<dyn FnMut(ParamUpdate) -> A>>| {
                if let Some(param_update) = inner.finish_gesture() {
                    let prev_state = if !hovered && *state != VirtualSliderState::Idle {
                        let p = Some(*state);
                        *state = VirtualSliderState::Idle;
                        p
                    } else if hovered && *state != VirtualSliderState::Hovered {
                        let p = Some(*state);
                        *state = VirtualSliderState::Hovered;
                        p
                    } else {
                        None
                    };

                    send_param_update(
                        param_update,
                        cx,
                        renderer,
                        style,
                        prev_state,
                        *state,
                        on_gesture,
                    );
                }
            };

        match event {
            ElementEvent::Animation { delta_seconds } => {
                if *disabled {
                    cx.set_animating(false);
                    return EventCaptureStatus::NotCaptured;
                }

                let res = self.renderer.on_animation(
                    delta_seconds,
                    NormalsState {
                        normal_value: inner.normal_value(),
                        default_normal: inner.default_normal(),
                        automation_info: automation_info.clone(),
                        num_quantized_steps: inner.num_quantized_steps(),
                    },
                    self.state,
                    style,
                );
                if res.repaint {
                    cx.request_repaint();
                }
                cx.set_animating(res.animating);
            }
            ElementEvent::CustomStateChanged => {
                if *needs_repaint {
                    *needs_repaint = false;
                    cx.request_repaint();
                }

                if *automation_info_changed {
                    *automation_info_changed = false;

                    let repaint = self
                        .renderer
                        .on_automation_info_update(automation_info, style);
                    if repaint {
                        cx.request_repaint();
                    }
                }

                if *disabled {
                    self.hovered = false;

                    finish_gesture(
                        inner,
                        cx,
                        self.hovered,
                        &mut self.state,
                        &mut self.renderer,
                        style,
                        &mut self.on_gesture,
                    );

                    cx.set_animating(false);
                }
            }
            ElementEvent::Pointer(PointerEvent::Moved {
                position,
                delta,
                modifiers,
                just_entered,
                ..
            }) => {
                if *disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                let hovered = cx.rect().contains(position);

                if self.hovered != hovered {
                    self.hovered = hovered;

                    if !inner.is_gesturing() {
                        let prev_state = self.state;
                        self.state = if hovered {
                            VirtualSliderState::Hovered
                        } else {
                            VirtualSliderState::Idle
                        };
                        let res = self
                            .renderer
                            .on_state_changed(prev_state, self.state, style);
                        if res.repaint {
                            cx.request_repaint();
                        }
                        cx.set_animating(res.animating);
                    }
                }

                if just_entered && self.on_tooltip_request.is_some() && !inner.is_gesturing() {
                    cx.start_hover_timeout();
                }

                if let Some(param_update) = inner.handle_pointer_moved(position, delta, modifiers) {
                    send_param_update(
                        param_update,
                        cx,
                        &mut self.renderer,
                        style,
                        None,
                        self.state,
                        &mut self.on_gesture,
                    );
                }
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                if self.hovered {
                    if !inner.is_gesturing() {
                        if self.state != VirtualSliderState::Idle {
                            let prev_state = self.state;
                            self.state = VirtualSliderState::Idle;

                            let res = self
                                .renderer
                                .on_state_changed(prev_state, self.state, style);
                            if res.repaint {
                                cx.request_repaint();
                            }
                            cx.set_animating(res.animating);
                        }
                    }

                    self.hovered = false;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed {
                position,
                button,
                click_count,
                ..
            }) => {
                if *disabled || !cx.rect.contains(position) {
                    return EventCaptureStatus::NotCaptured;
                }

                if button == PointerButton::Auxiliary
                    && inner.config.open_text_entry_on_middle_click
                {
                    if let Some(f) = self.on_open_text_entry.as_mut() {
                        finish_gesture(
                            inner,
                            cx,
                            self.hovered,
                            &mut self.state,
                            &mut self.renderer,
                            style,
                            &mut self.on_gesture,
                        );

                        cx.send_action((f)(ParamOpenTextEntryInfo {
                            param_id: inner.param_id,
                            normal_value: inner.normal_value(),
                            stepped_value: inner.stepped_value(),
                            bounds: cx.rect(),
                        }))
                        .unwrap();

                        return EventCaptureStatus::Captured;
                    }
                }

                if button == PointerButton::Secondary {
                    if let Some(f) = self.on_right_click.as_mut() {
                        finish_gesture(
                            inner,
                            cx,
                            self.hovered,
                            &mut self.state,
                            &mut self.renderer,
                            style,
                            &mut self.on_gesture,
                        );

                        cx.send_action((f)(ParamRightClickInfo {
                            param_id: inner.param_id,
                            normal_value: inner.normal_value(),
                            stepped_value: inner.stepped_value(),
                            pointer_pos: position,
                        }))
                        .unwrap();

                        return EventCaptureStatus::Captured;
                    }
                }

                if button != PointerButton::Primary {
                    return EventCaptureStatus::Captured;
                }

                finish_gesture(
                    inner,
                    cx,
                    self.hovered,
                    &mut self.state,
                    &mut self.renderer,
                    style,
                    &mut self.on_gesture,
                );

                if click_count == 1 {
                    if let Some(param_update) = inner.begin_drag_gesture(position) {
                        let prev_state = Some(self.state);
                        self.state = VirtualSliderState::Gesturing;

                        send_param_update(
                            param_update,
                            cx,
                            &mut self.renderer,
                            style,
                            prev_state,
                            self.state,
                            &mut self.on_gesture,
                        );

                        cx.steal_focus();
                    }
                } else if click_count == 2 {
                    if let Some(param_update) = inner.reset_to_default() {
                        let prev_state = if !self.hovered && self.state != VirtualSliderState::Idle
                        {
                            let p = Some(self.state);
                            self.state = VirtualSliderState::Idle;
                            p
                        } else if self.hovered && self.state != VirtualSliderState::Hovered {
                            let p = Some(self.state);
                            self.state = VirtualSliderState::Hovered;
                            p
                        } else {
                            None
                        };

                        send_param_update(
                            param_update,
                            cx,
                            &mut self.renderer,
                            style,
                            prev_state,
                            self.state,
                            &mut self.on_gesture,
                        );
                    }
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                button, position, ..
            }) => {
                if *disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                let in_bounds = cx.rect().contains(position);

                if button != PointerButton::Primary {
                    if in_bounds {
                        return EventCaptureStatus::Captured;
                    } else {
                        return EventCaptureStatus::NotCaptured;
                    }
                }

                if cx.has_focus() {
                    cx.release_focus();
                }
            }
            ElementEvent::Pointer(PointerEvent::HoverTimeout { position }) => {
                if *disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if cx.rect().contains(position) {
                    if let Some(f) = self.on_tooltip_request.as_mut() {
                        cx.send_action((f)(ParamTooltipInfo {
                            param_id: inner.param_id,
                            normal_value: inner.normal_value(),
                            stepped_value: inner.stepped_value(),
                            bounding_rect: cx.rect(),
                            tooltip_align: self.tooltip_align,
                        }))
                        .unwrap();
                    }
                }
            }
            ElementEvent::Pointer(PointerEvent::ScrollWheel {
                position,
                delta_type,
                modifiers,
                ..
            }) => {
                if *disabled || !cx.rect().contains(position) || !inner.config.use_scroll_wheel {
                    return EventCaptureStatus::NotCaptured;
                }

                if let Some(param_update) = inner.begin_scroll_wheel_gesture() {
                    let prev_state = Some(self.state);
                    self.state = VirtualSliderState::Gesturing;

                    send_param_update(
                        param_update,
                        cx,
                        &mut self.renderer,
                        style,
                        prev_state,
                        self.state,
                        &mut self.on_gesture,
                    );

                    cx.steal_focus();
                    cx.start_scroll_wheel_timeout();
                }

                if let Some(param_update) = inner.handle_scroll_wheel(delta_type, modifiers) {
                    send_param_update(
                        param_update,
                        cx,
                        &mut self.renderer,
                        style,
                        None,
                        self.state,
                        &mut self.on_gesture,
                    );
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::ScrollWheelTimeout) => {
                if cx.has_focus() {
                    cx.release_focus();
                }
            }
            ElementEvent::Focus(focused) => {
                if !focused {
                    finish_gesture(
                        inner,
                        cx,
                        self.hovered,
                        &mut self.state,
                        &mut self.renderer,
                        style,
                        &mut self.on_gesture,
                    );
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let shared_state = RefCell::borrow(&self.shared_state);

        self.renderer.render_primitives(
            &shared_state.style,
            NormalsState {
                normal_value: shared_state.inner.normal_value(),
                default_normal: shared_state.inner.default_normal(),
                automation_info: shared_state.automation_info.clone(),
                num_quantized_steps: shared_state.inner.num_quantized_steps(),
            },
            self.state,
            shared_state.disabled,
            cx,
            primitives,
        )
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AutomationInfo {
    pub current_normal: Option<f64>,
    pub range: Option<Range<f64>>,
}

impl AutomationInfo {
    pub fn clamp(&mut self) {
        if let Some(n) = &mut self.current_normal {
            *n = n.clamp(0.0, 1.0);
        }
        if let Some(r) = &mut self.range {
            let start = r.start.clamp(0.0, 1.0);
            let end = r.end.clamp(0.0, 1.0);
            *r = start..end
        }
    }
}

struct SharedState<R: VirtualSliderRenderer> {
    inner: VirtualSliderInner,
    style: Rc<R::Style>,
    automation_info: AutomationInfo,
    automation_info_changed: bool,
    disabled: bool,
    needs_repaint: bool,
}

/// A handle to a [`VirtualSliderElement`].
pub struct VirtualSlider<R: VirtualSliderRenderer> {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState<R>>>,
}

impl<R: VirtualSliderRenderer> VirtualSlider<R> {
    pub fn builder<A: Clone + 'static>(
        param_id: u32,
        style: &Rc<R::Style>,
    ) -> VirtualSliderBuilder<A, R> {
        VirtualSliderBuilder::new(param_id, style)
    }

    pub fn set_normal_value(&mut self, new_normal: f64) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let state_changed = shared_state.inner.set_normal_value(new_normal).is_some();
        if state_changed {
            shared_state.needs_repaint = true;
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_default_normal(&mut self, new_normal: f64) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let state_changed = shared_state.inner.set_default_normal(new_normal);
        if state_changed {
            shared_state.needs_repaint = true;
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_automation_info(&mut self, mut info: AutomationInfo) {
        info.clamp();

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        if shared_state.automation_info != info {
            shared_state.automation_info = info;
            shared_state.automation_info_changed = true;
            self.el.notify_custom_state_change();
        }
    }

    /// Reset the parameter to the default value.
    pub fn reset_to_default(&mut self) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let state_changed = shared_state.inner.reset_to_default().is_some();
        if state_changed {
            shared_state.needs_repaint = true;
            self.el.notify_custom_state_change();
        }
    }

    pub fn normal_value(&self) -> f64 {
        RefCell::borrow(&self.shared_state).inner.normal_value()
    }

    pub fn default_normal(&self) -> f64 {
        RefCell::borrow(&self.shared_state).inner.default_normal()
    }

    pub fn stepped_value(&self) -> Option<u32> {
        RefCell::borrow(&self.shared_state).inner.stepped_value()
    }

    pub fn num_quantized_steps(&self) -> Option<u32> {
        RefCell::borrow(&self.shared_state)
            .inner
            .num_quantized_steps()
    }

    pub fn set_style(&mut self, style: &Rc<R::Style>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.needs_repaint = true;
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<R::Style> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.disabled != disabled {
            shared_state.disabled = disabled;
            shared_state.needs_repaint = true;
            self.el.notify_custom_state_change();
        }
    }

    pub fn disabled(&self) -> bool {
        RefCell::borrow(&self.shared_state).disabled
    }
}
