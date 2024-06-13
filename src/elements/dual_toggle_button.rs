use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::text::glyphon::FontSystem;
use rootvg::text::TextProperties;
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align, Align2, Padding};
use crate::math::{Rect, Size, ZIndex};
use crate::style::{
    Background, BorderStyle, QuadStyle, DEFAULT_ACCENT_COLOR, DEFAULT_TEXT_ATTRIBUTES,
};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;
use crate::CursorIcon;

use super::button::ButtonState;
use super::dual_button::DualButtonStylePart;
use super::dual_label::{
    DualLabelClipMode, DualLabelInner, DualLabelLayout, DualLabelPrimitives, DualLabelStyle,
};

/// The style of a [`DualToggleButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct DualToggleButtonStyle {
    /// The properties of the left text.
    pub left_properties: TextProperties,
    /// The properties of the right text.
    pub right_properties: TextProperties,

    /// The vertical alignment of the text.
    ///
    /// By default this is set to `Align::Center`.
    pub vertical_align: Align,

    pub layout: DualLabelLayout,

    /// The minimum size of the clipped text area for the left text.
    ///
    /// By default this is set to `Size::new(5.0, 5.0)`.
    pub left_min_clipped_size: Size,
    /// The minimum size of the clipped text area for the right text.
    ///
    /// By default this is set to `Size::new(5.0, 5.0)`.
    pub right_min_clipped_size: Size,

    pub clip_mode: DualLabelClipMode,

    /// The padding between the left text and the bounding rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub left_padding: Padding,
    /// The padding between the right text and the bounding rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub right_padding: Padding,

    pub idle_on: DualButtonStylePart,
    pub hovered_on: DualButtonStylePart,
    pub disabled_on: DualButtonStylePart,

    pub idle_off: DualButtonStylePart,
    pub hovered_off: DualButtonStylePart,
    pub disabled_off: DualButtonStylePart,
}

impl Default for DualToggleButtonStyle {
    fn default() -> Self {
        let idle_on = DualButtonStylePart {
            left_font_color: color::WHITE,
            right_font_color: color::WHITE,
            back_quad: QuadStyle {
                bg: Background::Solid(DEFAULT_ACCENT_COLOR),
                border: BorderStyle {
                    radius: 4.0.into(),
                    color: RGBA8::new(105, 105, 105, 255),
                    width: 1.0,
                    ..Default::default()
                },
            },
        };

        let idle_off = DualButtonStylePart {
            back_quad: QuadStyle {
                bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                ..idle_on.back_quad
            },
            ..idle_on
        };

        Self {
            left_properties: TextProperties {
                attrs: DEFAULT_TEXT_ATTRIBUTES,
                ..Default::default()
            },
            right_properties: TextProperties {
                attrs: DEFAULT_TEXT_ATTRIBUTES,
                ..Default::default()
            },
            vertical_align: Align::Center,
            left_min_clipped_size: Size::new(5.0, 5.0),
            right_min_clipped_size: Size::new(5.0, 5.0),
            left_padding: Padding::new(6.0, 6.0, 6.0, 6.0),
            right_padding: Padding::new(6.0, 6.0, 6.0, 6.0),

            clip_mode: DualLabelClipMode::default(),
            layout: DualLabelLayout::default(),

            idle_on: idle_on.clone(),
            hovered_on: DualButtonStylePart {
                back_quad: QuadStyle {
                    border: BorderStyle {
                        color: RGBA8::new(135, 135, 135, 255),
                        ..idle_on.back_quad.border
                    },
                    ..idle_on.back_quad
                },
                ..idle_on
            },
            disabled_on: DualButtonStylePart {
                left_font_color: RGBA8::new(150, 150, 150, 255),
                right_font_color: RGBA8::new(150, 150, 150, 255),
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(76, 76, 76, 255)),
                    border: BorderStyle {
                        color: RGBA8::new(80, 80, 80, 255),
                        ..idle_on.back_quad.border
                    },
                },
                ..idle_on
            },

            idle_off: idle_off.clone(),
            hovered_off: DualButtonStylePart {
                back_quad: QuadStyle {
                    border: BorderStyle {
                        color: RGBA8::new(135, 135, 135, 255),
                        ..idle_off.back_quad.border
                    },
                    ..idle_off.back_quad
                },
                ..idle_off
            },
            disabled_off: DualButtonStylePart {
                left_font_color: RGBA8::new(150, 150, 150, 255),
                right_font_color: RGBA8::new(150, 150, 150, 255),
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    border: BorderStyle {
                        color: RGBA8::new(80, 80, 80, 255),
                        ..idle_off.back_quad.border
                    },
                },
                ..idle_off
            },
        }
    }
}

impl DualToggleButtonStyle {
    pub fn dual_label_style(&self, state: ButtonState, toggled: bool) -> DualLabelStyle {
        let part = if toggled {
            match state {
                ButtonState::Idle => &self.idle_on,
                ButtonState::Hovered => &self.hovered_on,
                ButtonState::Down => &self.hovered_on,
                ButtonState::Disabled => &self.disabled_on,
            }
        } else {
            match state {
                ButtonState::Idle => &self.idle_off,
                ButtonState::Hovered => &self.hovered_off,
                ButtonState::Down => &self.hovered_off,
                ButtonState::Disabled => &self.disabled_off,
            }
        };

        DualLabelStyle {
            left_properties: self.left_properties,
            right_properties: self.right_properties,
            left_font_color: part.left_font_color,
            right_font_color: part.right_font_color,
            vertical_align: self.vertical_align,
            left_min_clipped_size: self.left_min_clipped_size,
            right_min_clipped_size: self.left_min_clipped_size,
            back_quad: part.back_quad.clone(),
            left_padding: self.left_padding,
            right_padding: self.right_padding,
            layout: self.layout,
            clip_mode: self.clip_mode,
        }
    }
}

/// A reusable button struct that can be used by other elements.
pub struct DualToggleButtonInner {
    state: ButtonState,
    dual_label: DualLabelInner,
    toggled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateChangeResult {
    pub state_changed: bool,
    pub needs_repaint: bool,
}

impl DualToggleButtonInner {
    pub fn new(
        left_text: String,
        right_text: String,
        left_text_offset: Point,
        right_text_offset: Point,
        toggled: bool,
        style: &DualToggleButtonStyle,
        font_system: &mut FontSystem,
    ) -> Self {
        let dual_label = DualLabelInner::new(
            left_text,
            right_text,
            left_text_offset,
            right_text_offset,
            &style.dual_label_style(ButtonState::Idle, toggled),
            font_system,
        );

        Self {
            dual_label,
            state: ButtonState::Idle,
            toggled,
        }
    }

    pub fn set_state(
        &mut self,
        state: ButtonState,
        style: &DualToggleButtonStyle,
    ) -> StateChangeResult {
        if self.state != state {
            let old_part = if self.toggled {
                match self.state {
                    ButtonState::Idle => &style.idle_on,
                    ButtonState::Hovered => &style.hovered_on,
                    ButtonState::Down => &style.hovered_on,
                    ButtonState::Disabled => &style.disabled_on,
                }
            } else {
                match self.state {
                    ButtonState::Idle => &style.idle_off,
                    ButtonState::Hovered => &style.hovered_off,
                    ButtonState::Down => &style.hovered_off,
                    ButtonState::Disabled => &style.disabled_off,
                }
            };

            let new_part = if self.toggled {
                match state {
                    ButtonState::Idle => &style.idle_on,
                    ButtonState::Hovered => &style.hovered_on,
                    ButtonState::Down => &style.hovered_on,
                    ButtonState::Disabled => &style.disabled_on,
                }
            } else {
                match state {
                    ButtonState::Idle => &style.idle_off,
                    ButtonState::Hovered => &style.hovered_off,
                    ButtonState::Down => &style.hovered_off,
                    ButtonState::Disabled => &style.disabled_off,
                }
            };

            let needs_repaint = old_part != new_part;

            self.state = state;

            StateChangeResult {
                state_changed: true,
                needs_repaint,
            }
        } else {
            StateChangeResult {
                state_changed: false,
                needs_repaint: false,
            }
        }
    }

    pub fn state(&self) -> ButtonState {
        self.state
    }

    pub fn set_style(&mut self, style: &DualToggleButtonStyle, font_system: &mut FontSystem) {
        self.dual_label
            .set_style(&self.dual_label_style(style), font_system);
    }

    pub fn set_toggled(&mut self, toggled: bool) -> StateChangeResult {
        if self.toggled != toggled {
            self.toggled = toggled;

            StateChangeResult {
                state_changed: true,
                needs_repaint: true,
            }
        } else {
            StateChangeResult {
                state_changed: false,
                needs_repaint: false,
            }
        }
    }

    pub fn toggled(&self) -> bool {
        self.toggled
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    pub fn desired_padded_size(&mut self, style: &DualToggleButtonStyle) -> Size {
        self.dual_label
            .desired_padded_size(&self.dual_label_style(style))
    }

    /// Returns the size of the unclipped left and right text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn unclipped_text_size(&mut self) -> (Size, Size) {
        self.dual_label.unclipped_text_size()
    }

    /// Returns `true` if the text has changed.
    pub fn set_left_text(&mut self, text: &str, font_system: &mut FontSystem) -> bool {
        self.dual_label.set_left_text(text, font_system)
    }

    /// Returns `true` if the text has changed.
    pub fn set_right_text(
        &mut self,
        text: &str,
        style: &DualToggleButtonStyle,
        font_system: &mut FontSystem,
    ) -> bool {
        let style = self.dual_label_style(style);
        self.dual_label.set_right_text(text, &style, font_system)
    }

    pub fn text(&self) -> (&str, &str) {
        self.dual_label.text()
    }

    pub fn dual_label_style(&self, style: &DualToggleButtonStyle) -> DualLabelStyle {
        style.dual_label_style(self.state, self.toggled)
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &DualToggleButtonStyle,
        font_system: &mut FontSystem,
    ) -> DualLabelPrimitives {
        self.dual_label
            .render_primitives(bounds, &self.dual_label_style(style), font_system)
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_left_text_offset(&mut self, offset: Point) -> bool {
        self.dual_label.set_left_text_offset(offset)
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_right_text_offset(&mut self, offset: Point) -> bool {
        self.dual_label.set_right_text_offset(offset)
    }

    pub fn left_text_offset(&self) -> Point {
        self.dual_label.left_text_offset
    }

    pub fn right_text_offset(&self) -> Point {
        self.dual_label.right_text_offset
    }
}

pub struct DualToggleButtonBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(bool) -> A>>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub toggled: bool,
    pub left_text: String,
    pub right_text: String,
    pub left_text_offset: Point,
    pub right_text_offset: Point,
    pub style: Rc<DualToggleButtonStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl<A: Clone + 'static> DualToggleButtonBuilder<A> {
    pub fn new(style: &Rc<DualToggleButtonStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            toggled: false,
            left_text: String::new(),
            right_text: String::new(),
            left_text_offset: Point::default(),
            right_text_offset: Point::default(),
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> DualToggleButton {
        DualToggleButtonElement::create(self, cx)
    }

    pub fn on_toggled<F: FnMut(bool) -> A + 'static>(mut self, f: F) -> Self {
        self.action = Some(Box::new(f));
        self
    }

    pub fn tooltip_message(mut self, message: impl Into<String>, align: Align2) -> Self {
        self.tooltip_message = Some(message.into());
        self.tooltip_align = align;
        self
    }

    pub const fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }

    pub fn left_text(mut self, text: impl Into<String>) -> Self {
        self.left_text = text.into();
        self
    }

    pub fn right_text(mut self, text: impl Into<String>) -> Self {
        self.right_text = text.into();
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn left_text_offset(mut self, offset: Point) -> Self {
        self.left_text_offset = offset;
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn right_text_offset(mut self, offset: Point) -> Self {
        self.right_text_offset = offset;
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
}

/// A button element with a label.
pub struct DualToggleButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(bool) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
}

impl<A: Clone + 'static> DualToggleButtonElement<A> {
    pub fn create(
        builder: DualToggleButtonBuilder<A>,
        cx: &mut WindowContext<'_, A>,
    ) -> DualToggleButton {
        let DualToggleButtonBuilder {
            action,
            tooltip_message,
            tooltip_align,
            toggled,
            left_text,
            right_text,
            left_text_offset,
            right_text_offset,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: DualToggleButtonInner::new(
                left_text,
                right_text,
                left_text_offset,
                right_text_offset,
                toggled,
                &style,
                cx.font_system,
            ),
            style,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                action,
                tooltip_message,
                tooltip_align,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, cx.font_system, cx.clipboard);

        DualToggleButton { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for DualToggleButtonElement<A> {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        match event {
            ElementEvent::CustomStateChanged => {
                cx.request_repaint();
            }
            ElementEvent::Pointer(PointerEvent::Moved { just_entered, .. }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState { inner, style } = &mut *shared_state;

                if inner.state == ButtonState::Disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                cx.cursor_icon = CursorIcon::Pointer;

                if just_entered && self.tooltip_message.is_some() {
                    cx.start_hover_timeout();
                }

                if inner.state == ButtonState::Idle {
                    let res = inner.set_state(ButtonState::Hovered, style);

                    if res.needs_repaint {
                        cx.request_repaint();
                    }
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState { inner, style, .. } = &mut *shared_state;

                if inner.state == ButtonState::Hovered || inner.state == ButtonState::Down {
                    let res = inner.set_state(ButtonState::Idle, style);

                    if res.needs_repaint {
                        cx.request_repaint();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed { button, .. }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState { inner, style, .. } = &mut *shared_state;

                if button == PointerButton::Primary
                    && (inner.state == ButtonState::Idle || inner.state == ButtonState::Hovered)
                {
                    let res1 = inner.set_state(ButtonState::Down, style);
                    let res2 = inner.set_toggled(!inner.toggled());

                    if res1.needs_repaint || res2.needs_repaint {
                        cx.request_repaint();
                    }

                    if let Some(action) = &mut self.action {
                        cx.send_action((action)(inner.toggled())).unwrap();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                position, button, ..
            }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState { inner, style, .. } = &mut *shared_state;

                if button == PointerButton::Primary
                    && (inner.state == ButtonState::Down || inner.state == ButtonState::Hovered)
                {
                    let new_state = if cx.is_point_within_visible_bounds(position) {
                        ButtonState::Hovered
                    } else {
                        ButtonState::Idle
                    };

                    let res = inner.set_state(new_state, style);

                    if res.needs_repaint {
                        cx.request_repaint();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::HoverTimeout { .. }) => {
                if let Some(message) = &self.tooltip_message {
                    cx.show_tooltip(message.clone(), self.tooltip_align);
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        let label_primitives =
            inner.render_primitives(Rect::from_size(cx.bounds_size), style, cx.font_system);

        if let Some(quad_primitive) = label_primitives.bg_quad {
            primitives.add(quad_primitive);
        }

        if let Some(p) = label_primitives.left_text {
            primitives.set_z_index(1);
            primitives.add_text(p);
        }

        if let Some(p) = label_primitives.right_text {
            primitives.set_z_index(1);
            primitives.add_text(p);
        }
    }
}

/// A handle to a [`DualToggleButtonElement`], a button with a label.
pub struct DualToggleButton {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    inner: DualToggleButtonInner,
    style: Rc<DualToggleButtonStyle>,
}

impl DualToggleButton {
    pub fn builder<A: Clone + 'static>(
        style: &Rc<DualToggleButtonStyle>,
    ) -> DualToggleButtonBuilder<A> {
        DualToggleButtonBuilder::new(style)
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn desired_padded_size(&self) -> Size {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        inner.desired_padded_size(style)
    }

    /// Returns the size of the unclipped left and right text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn unclipped_text_size(&self) -> (Size, Size) {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .unclipped_text_size()
    }

    pub fn set_left_text(&mut self, text: &str, font_system: &mut FontSystem) {
        if RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_left_text(text, font_system)
        {
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_right_text(&mut self, text: &str, font_system: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        inner.set_right_text(text, style, font_system);
        self.el.notify_custom_state_change();
    }

    pub fn left_text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text().0)
    }

    pub fn right_text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text().1)
    }

    pub fn set_style(&mut self, style: &Rc<DualToggleButtonStyle>, font_system: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(style, font_system);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<DualToggleButtonStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    pub fn set_toggled(&mut self, toggled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.toggled() != toggled {
            shared_state.inner.set_toggled(toggled);
            self.el.notify_custom_state_change();
        }
    }

    pub fn toggled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.toggled()
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        if disabled && inner.state != ButtonState::Disabled {
            inner.set_state(ButtonState::Disabled, style);
            self.el.notify_custom_state_change();
        } else if !disabled && inner.state == ButtonState::Disabled {
            inner.set_state(ButtonState::Idle, style);
            self.el.notify_custom_state_change();
        }
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub fn set_left_text_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_left_text_offset(offset);

        if changed {
            self.el.notify_custom_state_change();
        }
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub fn set_right_text_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_right_text_offset(offset);

        if changed {
            self.el.notify_custom_state_change();
        }
    }
}
