use std::{
    cell::{Ref, RefCell},
    ops::Range,
    rc::Rc,
};

use keyboard_types::Modifiers;
use rootvg::{
    math::{Point, Rect, Size, ZIndex},
    PrimitiveGroup,
};
use smallvec::SmallVec;

use crate::{
    event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent},
    layout::Align2,
    view::element::{
        Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, ElementRenderCache,
        RenderContext,
    },
    ScissorRectID, WindowContext,
};

mod inner;
mod renderer;

pub use inner::*;
pub use renderer::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParamOpenTextEntryInfo {
    pub param_info: ParamInfo,
    /// The bounding rectangle of this element
    pub bounds: Rect,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParamerMarkerType {
    #[default]
    Primary,
    Secondary,
    Third,
}

/// A marker on a parameter element.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct ParamMarker {
    pub normal_val: f32,
    pub label: Option<String>,
    pub type_: ParamerMarkerType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamMarkersConfig {
    /// Use the default configuration.
    ///
    /// * Standard linear parametrs: 1 main at `0.0`, 1 main at `1.0`
    /// * Bipolar linear parameters: 1 main at `0.0`, 1 main at `0.5`, 1 main at `1.0`
    /// * Quantized parameters: 1 main at each step
    Default,
    Custom(SmallVec<[ParamMarker; 8]>),
}

impl ParamMarkersConfig {
    pub fn with_markers<F: FnMut(&ParamMarker)>(
        &self,
        bipolar: bool,
        num_quantized_steps: Option<u32>,
        mut f: F,
    ) {
        match self {
            Self::Default => {
                if let Some(num_steps) = num_quantized_steps {
                    if num_steps < 2 {
                        (f)(&ParamMarker {
                            normal_val: 0.0,
                            label: None,
                            type_: ParamerMarkerType::Primary,
                        });
                    } else if num_steps > 16 {
                        // Don't clutter the view.
                        for normal_val in [0.0, 0.5, 1.0] {
                            (f)(&ParamMarker {
                                normal_val,
                                label: None,
                                type_: ParamerMarkerType::Primary,
                            });
                        }
                    } else {
                        let num_steps_recip = ((num_steps - 1) as f32).recip();

                        for i in 0..(num_steps - 1) {
                            (f)(&ParamMarker {
                                normal_val: (i as f32) * num_steps_recip,
                                label: None,
                                type_: ParamerMarkerType::Primary,
                            });
                        }

                        (f)(&ParamMarker {
                            normal_val: 1.0,
                            label: None,
                            type_: ParamerMarkerType::Primary,
                        });
                    }
                } else if bipolar {
                    for normal_val in [0.0, 0.5, 1.0] {
                        (f)(&ParamMarker {
                            normal_val,
                            label: None,
                            type_: ParamerMarkerType::Primary,
                        });
                    }
                } else {
                    for normal_val in [0.0, 1.0] {
                        (f)(&ParamMarker {
                            normal_val,
                            label: None,
                            type_: ParamerMarkerType::Primary,
                        });
                    }
                }
            }
            Self::Custom(m) => {
                for marker in m.iter() {
                    (f)(marker);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualSliderConfig {
    /// The scalar (points to normalized units) to use when dragging.
    ///
    /// By default this is set to `0.003`.
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

    /// Whether or not to activate the `on_open_text_entry` event when
    /// the user right-clicks this element.
    ///
    /// If the use has defined a right-click action, then that action
    /// will take precedence.
    ///
    /// By default this is set to `true`.
    pub open_text_entry_on_right_click: bool,

    /// Whether or not to disabled locking the pointer in place while
    /// dragging this element.
    ///
    /// By default this is set to `false`.
    pub disable_pointer_locking: bool,
}

impl Default for VirtualSliderConfig {
    fn default() -> Self {
        Self {
            drag_scalar: 0.003,
            scroll_wheel_scalar: 0.0004,
            scroll_wheel_points_per_line: 24.0,
            fine_adjustment_scalar: 0.02,
            use_scroll_wheel: true,
            fine_adjustment_modifier: Some(Modifiers::SHIFT),
            open_text_entry_modifier: Some(Modifiers::CONTROL),
            open_text_entry_on_middle_click: true,
            open_text_entry_on_right_click: true,
            disable_pointer_locking: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamElementTooltipInfo {
    pub param_info: ParamInfo,
    pub bounding_rect: Rect,
    pub tooltip_align: Align2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamRightClickInfo {
    pub param_info: ParamInfo,
    pub pointer_pos: Point,
}

pub struct VirtualSliderBuilder<A: Clone + 'static> {
    pub on_gesture: Option<Box<dyn FnMut(ParamUpdate) -> A>>,
    pub on_right_click: Option<Box<dyn FnMut(ParamRightClickInfo) -> A>>,
    pub on_open_text_entry: Option<Box<dyn FnMut(ParamOpenTextEntryInfo) -> A>>,
    pub on_tooltip_request: Option<Box<dyn FnMut(ParamElementTooltipInfo) -> A>>,
    pub class: Option<&'static str>,
    pub tooltip_align: Align2,
    pub param_id: u32,
    pub normal_value: f64,
    pub default_normal: f64,
    pub num_quantized_steps: Option<u32>,
    pub markers: ParamMarkersConfig,
    pub bipolar: bool,
    pub config: VirtualSliderConfig,
    pub drag_horizontally: bool,
    pub scroll_horizontally: bool,
    pub horizontal: bool,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub disabled: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> VirtualSliderBuilder<A> {
    pub fn new(param_id: u32) -> Self {
        Self {
            on_gesture: None,
            on_right_click: None,
            on_open_text_entry: None,
            on_tooltip_request: None,
            class: None,
            tooltip_align: Align2::TOP_CENTER,
            param_id,
            normal_value: 0.0,
            default_normal: 0.0,
            num_quantized_steps: None,
            markers: ParamMarkersConfig::Default,
            bipolar: false,
            config: VirtualSliderConfig::default(),
            drag_horizontally: false,
            scroll_horizontally: false,
            horizontal: false,
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            disabled: false,
            scissor_rect_id: None,
        }
    }

    pub fn build<R: VirtualSliderRenderer>(
        self,
        cx: &mut WindowContext<'_, A>,
    ) -> VirtualSlider<R> {
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

    pub fn on_tooltip_request<F: FnMut(ParamElementTooltipInfo) -> A + 'static>(
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

    pub fn markers(mut self, markers: ParamMarkersConfig) -> Self {
        self.markers = markers;
        self
    }

    pub const fn bipolar(mut self, bipolar: bool) -> Self {
        self.bipolar = bipolar;
        self
    }

    pub const fn config(mut self, config: VirtualSliderConfig) -> Self {
        self.config = config;
        self
    }

    pub const fn drag_horizontally(mut self, drag_horizontally: bool) -> Self {
        self.drag_horizontally = drag_horizontally;
        self
    }

    pub const fn scroll_horizontally(mut self, scroll_horizontally: bool) -> Self {
        self.scroll_horizontally = scroll_horizontally;
        self
    }

    pub const fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// The style class name
    ///
    /// If this method is not used, then the current class from the window context will
    /// be used.
    pub const fn class(mut self, class: &'static str) -> Self {
        self.class = Some(class);
        self
    }

    /// The z index of the element
    ///
    /// If this method is not used, then the current z index from the window context will
    /// be used.
    pub const fn z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = Some(z_index);
        self
    }

    /// The bounding rectangle of the element
    ///
    /// If this method is not used, then the element will have a size and position of
    /// zero and will not be visible until its bounding rectangle is set.
    pub const fn bounding_rect(mut self, rect: Rect) -> Self {
        self.bounding_rect = rect;
        self
    }

    /// Whether or not this element is manually hidden
    ///
    /// By default this is set to `false`.
    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.manually_hidden = hidden;
        self
    }

    /// Whether or not this element is in the disabled state
    ///
    /// By default this is set to `false`.
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The ID of the scissoring rectangle this element belongs to.
    ///
    /// If this method is not used, then the current scissoring rectangle ID from the
    /// window context will be used.
    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = Some(scissor_rect_id);
        self
    }
}

pub struct VirtualSliderElement<A: Clone + 'static, R: VirtualSliderRenderer + 'static> {
    shared_state: Rc<RefCell<SharedState<R>>>,

    on_gesture: Option<Box<dyn FnMut(ParamUpdate) -> A>>,
    on_right_click: Option<Box<dyn FnMut(ParamRightClickInfo) -> A>>,
    on_open_text_entry: Option<Box<dyn FnMut(ParamOpenTextEntryInfo) -> A>>,
    on_tooltip_request: Option<Box<dyn FnMut(ParamElementTooltipInfo) -> A>>,
    tooltip_align: Align2,
    horizontal: bool,

    hovered: bool,
    state: VirtualSliderState,
    global_render_cache_id: Option<u32>,
}

impl<A: Clone + 'static, R: VirtualSliderRenderer + 'static> VirtualSliderElement<A, R> {
    pub fn create(
        builder: VirtualSliderBuilder<A>,
        cx: &mut WindowContext<'_, A>,
    ) -> VirtualSlider<R> {
        let VirtualSliderBuilder {
            on_gesture,
            on_right_click,
            on_open_text_entry,
            on_tooltip_request,
            class,
            tooltip_align,
            param_id,
            normal_value,
            default_normal,
            num_quantized_steps,
            markers,
            bipolar,
            config,
            drag_horizontally,
            scroll_horizontally,
            horizontal,
            z_index,
            bounding_rect,
            manually_hidden,
            disabled,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);
        let style = cx.res.style_system.get_rc::<R::Style>(class);

        let renderer = R::new(style);
        let global_render_cache_id = renderer.global_render_cache_id();

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: VirtualSliderInner::new(
                param_id,
                normal_value,
                default_normal,
                num_quantized_steps,
                config,
                drag_horizontally,
                scroll_horizontally,
            ),
            renderer,
            automation_info: AutomationInfo::default(),
            markers,
            bipolar,
            automation_info_changed: false,
            needs_repaint: false,
            disabled,
            queued_new_val: None,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                on_gesture,
                on_right_click,
                on_open_text_entry,
                on_tooltip_request,
                tooltip_align,
                horizontal,
                hovered: false,
                state: if disabled {
                    VirtualSliderState::Disabled
                } else {
                    VirtualSliderState::Idle
                },
                global_render_cache_id,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

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

        if RefCell::borrow(&self.shared_state).renderer.does_paint() {
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
            renderer,
            automation_info,
            markers,
            bipolar,
            automation_info_changed,
            disabled,
            needs_repaint,
            queued_new_val,
        } = &mut *shared_state;

        let send_param_update =
            |param_update: InnerParamUpdate,
             cx: &mut ElementContext<'_, A>,
             renderer: &mut R,
             prev_state: Option<VirtualSliderState>,
             state: VirtualSliderState,
             on_gesture: &mut Option<Box<dyn FnMut(ParamUpdate) -> A>>| {
                if let Some(f) = on_gesture.as_mut() {
                    cx.send_action((f)(param_update.inner)).unwrap();
                }

                if renderer.does_paint() {
                    cx.request_repaint();
                }

                if let Some(prev_state) = prev_state {
                    let res = renderer.on_state_changed(prev_state, state);
                    cx.set_animating(res.animating);
                }

                if let Some(lock) = param_update.pointer_lock_request {
                    cx.request_pointer_lock(lock);
                }
            };

        let finish_gesture =
            |inner: &mut VirtualSliderInner,
             cx: &mut ElementContext<'_, A>,
             hovered: bool,
             state: &mut VirtualSliderState,
             renderer: &mut R,
             disabled: bool,
             on_gesture: &mut Option<Box<dyn FnMut(ParamUpdate) -> A>>| {
                if let Some(param_update) = inner.finish_gesture() {
                    let prev_state = if disabled {
                        let p = Some(*state);
                        *state = VirtualSliderState::Disabled;
                        p
                    } else if !hovered && *state != VirtualSliderState::Idle {
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

                    send_param_update(param_update, cx, renderer, prev_state, *state, on_gesture);
                }
            };

        match event {
            ElementEvent::Animation { delta_seconds } => {
                if *disabled {
                    cx.set_animating(false);
                    return EventCaptureStatus::NotCaptured;
                }

                let res = renderer.on_animation(
                    delta_seconds,
                    VirtualSliderRenderInfo {
                        normal_value: inner.normal_value(),
                        default_normal: inner.default_normal(),
                        automation_info: automation_info.clone(),
                        stepped_value: inner.stepped_value(),
                        state: self.state,
                        bipolar: *bipolar,
                        markers,
                        horizontal: self.horizontal,
                    },
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

                    let repaint = renderer.on_automation_info_update(automation_info);
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
                        renderer,
                        *disabled,
                        &mut self.on_gesture,
                    );

                    cx.set_animating(false);
                } else if self.state == VirtualSliderState::Disabled {
                    self.state = VirtualSliderState::Idle;
                    let res = renderer
                        .on_state_changed(VirtualSliderState::Disabled, VirtualSliderState::Idle);
                    if res.repaint {
                        cx.request_repaint();
                    }
                    cx.set_animating(res.animating);
                }

                if let Some(new_val) = queued_new_val.take() {
                    if inner.value() != new_val {
                        if let Some(param_update) = inner.set_value(new_val) {
                            send_param_update(
                                param_update,
                                cx,
                                renderer,
                                None,
                                self.state,
                                &mut self.on_gesture,
                            );
                        }
                    }
                }
            }
            ElementEvent::StyleChanged => {
                let new_style = cx.res.style_system.get_rc::<R::Style>(cx.class());
                renderer.style_changed(new_style);
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
                        let res = renderer.on_state_changed(prev_state, self.state);
                        if res.repaint {
                            cx.request_repaint();
                        }
                        cx.set_animating(res.animating);
                    }
                }

                if just_entered && self.on_tooltip_request.is_some() && !inner.is_gesturing() {
                    cx.start_hover_timeout();
                }

                let delta = if let Some(delta) = delta {
                    if cx.is_pointer_locked() {
                        Some(delta)
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(param_update) = inner.handle_pointer_moved(position, delta, modifiers) {
                    send_param_update(
                        InnerParamUpdate {
                            inner: param_update,
                            pointer_lock_request: None,
                        },
                        cx,
                        renderer,
                        None,
                        self.state,
                        &mut self.on_gesture,
                    );
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                if self.hovered {
                    if !inner.is_gesturing() {
                        if self.state != VirtualSliderState::Idle {
                            let prev_state = self.state;
                            self.state = VirtualSliderState::Idle;

                            let res = renderer.on_state_changed(prev_state, self.state);
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
                modifiers,
                ..
            }) => {
                if *disabled || !cx.rect.contains(position) {
                    return EventCaptureStatus::NotCaptured;
                }

                let mut open_text_entry = false;

                if button == PointerButton::Auxiliary
                    && inner.config.open_text_entry_on_middle_click
                    && self.on_open_text_entry.is_some()
                {
                    open_text_entry = true;
                }

                if button == PointerButton::Secondary {
                    if let Some(f) = self.on_right_click.as_mut() {
                        finish_gesture(
                            inner,
                            cx,
                            self.hovered,
                            &mut self.state,
                            renderer,
                            *disabled,
                            &mut self.on_gesture,
                        );

                        cx.send_action((f)(ParamRightClickInfo {
                            param_info: inner.param_info(),
                            pointer_pos: position,
                        }))
                        .unwrap();

                        return EventCaptureStatus::Captured;
                    } else if inner.config.open_text_entry_on_right_click
                        && self.on_open_text_entry.is_some()
                    {
                        open_text_entry = true;
                    }
                }

                if button == PointerButton::Primary {
                    if let Some(m) = inner.config.open_text_entry_modifier {
                        if modifiers == m && self.on_open_text_entry.is_some() {
                            open_text_entry = true;
                        }
                    }
                }

                if open_text_entry {
                    if let Some(f) = self.on_open_text_entry.as_mut() {
                        finish_gesture(
                            inner,
                            cx,
                            self.hovered,
                            &mut self.state,
                            renderer,
                            *disabled,
                            &mut self.on_gesture,
                        );

                        cx.send_action((f)(ParamOpenTextEntryInfo {
                            param_info: inner.param_info(),
                            bounds: cx.rect(),
                        }))
                        .unwrap();
                    }

                    return EventCaptureStatus::Captured;
                } else if button != PointerButton::Primary {
                    return EventCaptureStatus::NotCaptured;
                }

                finish_gesture(
                    inner,
                    cx,
                    self.hovered,
                    &mut self.state,
                    renderer,
                    *disabled,
                    &mut self.on_gesture,
                );

                if click_count == 1 {
                    if let Some(param_update) = inner.begin_drag_gesture(position) {
                        let prev_state = Some(self.state);
                        self.state = VirtualSliderState::Gesturing;

                        send_param_update(
                            param_update,
                            cx,
                            renderer,
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
                            renderer,
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

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::HoverTimeout { position }) => {
                if *disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if cx.rect().contains(position) {
                    if let Some(f) = self.on_tooltip_request.as_mut() {
                        cx.send_action((f)(ParamElementTooltipInfo {
                            param_info: inner.param_info(),
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
                        InnerParamUpdate {
                            inner: param_update,
                            pointer_lock_request: None,
                        },
                        cx,
                        renderer,
                        prev_state,
                        self.state,
                        &mut self.on_gesture,
                    );

                    cx.steal_focus();
                    cx.start_scroll_wheel_timeout();
                }

                if let Some(param_update) = inner.handle_scroll_wheel(delta_type, modifiers) {
                    send_param_update(
                        InnerParamUpdate {
                            inner: param_update,
                            pointer_lock_request: None,
                        },
                        cx,
                        renderer,
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
                        renderer,
                        *disabled,
                        &mut self.on_gesture,
                    );
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState {
            inner,
            renderer,
            automation_info,
            markers,
            bipolar,
            ..
        } = &mut *shared_state;

        renderer.render_primitives(
            VirtualSliderRenderInfo {
                normal_value: inner.normal_value(),
                default_normal: inner.default_normal(),
                automation_info: automation_info.clone(),
                stepped_value: inner.stepped_value(),
                state: self.state,
                bipolar: *bipolar,
                markers: markers,
                horizontal: self.horizontal,
            },
            cx,
            primitives,
        )
    }

    fn global_render_cache_id(&self) -> Option<u32> {
        self.global_render_cache_id
    }

    fn global_render_cache(&self) -> Option<Box<dyn ElementRenderCache>> {
        RefCell::borrow(&self.shared_state)
            .renderer
            .global_render_cache()
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

struct SharedState<R: VirtualSliderRenderer + 'static> {
    inner: VirtualSliderInner,
    renderer: R,
    automation_info: AutomationInfo,
    markers: ParamMarkersConfig,
    bipolar: bool,
    automation_info_changed: bool,
    disabled: bool,
    needs_repaint: bool,
    queued_new_val: Option<ParamValue>,
}

/// A handle to a [`VirtualSliderElement`].
pub struct VirtualSlider<R: VirtualSliderRenderer> {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState<R>>>,
}

impl<R: VirtualSliderRenderer> VirtualSlider<R> {
    pub fn builder<A: Clone + 'static>(param_id: u32) -> VirtualSliderBuilder<A> {
        VirtualSliderBuilder::new(param_id)
    }

    pub fn desired_size(&self) -> Option<Size> {
        RefCell::borrow(&self.shared_state).renderer.desired_size()
    }

    pub fn set_normal_value(&mut self, new_normal: f64) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.normal_value() != new_normal {
            shared_state.queued_new_val = Some(ParamValue::Normal(new_normal));
            self.el._notify_custom_state_change();
        }
    }

    pub fn set_stepped_value(&mut self, new_val: u32) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if let Some(stepped_value) = shared_state.inner.stepped_value() {
            if stepped_value.value != new_val {
                shared_state.queued_new_val = Some(ParamValue::Stepped(new_val));
                self.el._notify_custom_state_change();
            }
        }
    }

    pub fn set_value(&mut self, new_val: ParamValue) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.value() != new_val {
            if let ParamValue::Stepped(_) = new_val {
                if shared_state.inner.stepped_value().is_none() {
                    return;
                }
            }

            shared_state.queued_new_val = Some(new_val);
            self.el._notify_custom_state_change();
        }
    }

    pub fn set_default_normal(&mut self, new_normal: f64) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let state_changed = shared_state.inner.set_default_normal(new_normal);
        if state_changed {
            shared_state.needs_repaint = true;
            self.el._notify_custom_state_change();
        }
    }

    pub fn set_automation_info(&mut self, mut info: AutomationInfo) {
        info.clamp();

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        if shared_state.automation_info != info {
            shared_state.automation_info = info;
            shared_state.automation_info_changed = true;
            self.el._notify_custom_state_change();
        }
    }

    /// Reset the parameter to the default value.
    pub fn reset_to_default(&mut self) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.normal_value() != shared_state.inner.default_normal() {
            shared_state.queued_new_val =
                Some(ParamValue::Normal(shared_state.inner.default_normal()));
            self.el._notify_custom_state_change();
        }
    }

    pub fn param_info(&self) -> ParamInfo {
        RefCell::borrow(&self.shared_state).inner.param_info()
    }

    pub fn normal_value(&self) -> f64 {
        RefCell::borrow(&self.shared_state).inner.normal_value()
    }

    pub fn default_normal(&self) -> f64 {
        RefCell::borrow(&self.shared_state).inner.default_normal()
    }

    pub fn stepped_value(&self) -> Option<SteppedValue> {
        RefCell::borrow(&self.shared_state).inner.stepped_value()
    }

    pub fn value(&self) -> ParamValue {
        RefCell::borrow(&self.shared_state)
            .inner
            .param_info()
            .value()
    }

    pub fn set_markers(&mut self, markers: ParamMarkersConfig) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.markers != markers {
            shared_state.markers = markers;
            shared_state.needs_repaint = true;
            self.el._notify_custom_state_change();
        }
    }

    pub fn markers<'a>(&'a self) -> Ref<'a, ParamMarkersConfig> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| &s.markers)
    }

    pub fn set_bipolar(&mut self, bipolar: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.bipolar != bipolar {
            shared_state.bipolar = bipolar;
            shared_state.needs_repaint = true;
            self.el._notify_custom_state_change();
        }
    }

    pub fn bipolar(&self) -> bool {
        RefCell::borrow(&self.shared_state).bipolar
    }

    pub fn set_class(&mut self, class: &'static str) {
        if self.el.class() != class {
            self.el._notify_class_change(class);
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.disabled != disabled {
            shared_state.disabled = disabled;
            shared_state.needs_repaint = true;
            self.el._notify_custom_state_change();
        }
    }

    pub fn disabled(&self) -> bool {
        RefCell::borrow(&self.shared_state).disabled
    }
}
