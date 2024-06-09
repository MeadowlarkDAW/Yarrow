use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::text::glyphon::FontSystem;
use rootvg::text::TextProperties;
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align, Align2, Padding};
use crate::math::{Rect, Size, ZIndex};
use crate::style::{Background, BorderStyle, QuadStyle, DEFAULT_TEXT_ATTRIBUTES};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, ElementTooltipInfo,
    RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;
use crate::CursorIcon;

use super::button::ButtonState;
use super::dual_label::{
    DualLabelClipMode, DualLabelInner, DualLabelLayout, DualLabelPrimitives, DualLabelStyle,
};

#[derive(Debug, Clone, PartialEq)]
pub struct DualButtonStylePart {
    /// The color of the left text
    ///
    /// By default this is set to `color::WHITE`.
    pub left_font_color: RGBA8,
    /// The color of the right text
    ///
    /// By default this is set to `color::WHITE`.
    pub right_font_color: RGBA8,

    /// The style of the padded background rectangle behind the text.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    ///
    /// By default this is set to `QuadStyle::TRANSPARENT`.
    pub back_quad: QuadStyle,
}

/// The style of a [`DualButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct DualButtonStyle {
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

    pub idle: DualButtonStylePart,
    pub hovered: DualButtonStylePart,
    pub down: DualButtonStylePart,
    pub disabled: DualButtonStylePart,
}

impl Default for DualButtonStyle {
    fn default() -> Self {
        let idle = DualButtonStylePart {
            left_font_color: color::WHITE,
            right_font_color: color::WHITE,
            back_quad: QuadStyle {
                bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                border: BorderStyle {
                    radius: 4.0.into(),
                    color: RGBA8::new(105, 105, 105, 255),
                    width: 1.0,
                    ..Default::default()
                },
            },
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

            idle: idle.clone(),
            hovered: DualButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(55, 55, 55, 255)),
                    border: BorderStyle {
                        color: RGBA8::new(135, 135, 135, 255),
                        ..idle.back_quad.border
                    },
                },
                ..idle
            },
            down: DualButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    ..idle.back_quad
                },
                ..idle
            },
            disabled: DualButtonStylePart {
                left_font_color: RGBA8::new(150, 150, 150, 255),
                right_font_color: RGBA8::new(150, 150, 150, 255),
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    border: BorderStyle {
                        color: RGBA8::new(65, 65, 65, 255),
                        ..idle.back_quad.border
                    },
                },
                ..idle
            },
        }
    }
}

impl DualButtonStyle {
    pub fn dual_label_style(&self, state: ButtonState) -> DualLabelStyle {
        let part = match state {
            ButtonState::Idle => &self.idle,
            ButtonState::Hovered => &self.hovered,
            ButtonState::Down => &self.down,
            ButtonState::Disabled => &self.disabled,
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
pub struct DualButtonInner {
    state: ButtonState,
    dual_label: DualLabelInner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateChangeResult {
    pub state_changed: bool,
    pub needs_repaint: bool,
}

impl DualButtonInner {
    pub fn new(
        left_text: String,
        right_text: String,
        left_text_offset: Point,
        right_text_offset: Point,
        style: &DualButtonStyle,
        font_system: &mut FontSystem,
    ) -> Self {
        let dual_label = DualLabelInner::new(
            left_text,
            right_text,
            left_text_offset,
            right_text_offset,
            &style.dual_label_style(ButtonState::Idle),
            font_system,
        );

        Self {
            dual_label,
            state: ButtonState::Idle,
        }
    }

    pub fn set_state(&mut self, state: ButtonState, style: &DualButtonStyle) -> StateChangeResult {
        if self.state != state {
            let old_part = match self.state {
                ButtonState::Idle => &style.idle,
                ButtonState::Hovered => &style.hovered,
                ButtonState::Down => &style.down,
                ButtonState::Disabled => &style.disabled,
            };
            let new_part = match state {
                ButtonState::Idle => &style.idle,
                ButtonState::Hovered => &style.hovered,
                ButtonState::Down => &style.down,
                ButtonState::Disabled => &style.disabled,
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

    pub fn set_style(&mut self, style: &DualButtonStyle, font_system: &mut FontSystem) {
        self.dual_label
            .set_style(&self.dual_label_style(style), font_system);
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    pub fn desired_padded_size(&mut self, style: &DualButtonStyle) -> Size {
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
        style: &DualButtonStyle,
        font_system: &mut FontSystem,
    ) -> bool {
        let style = self.dual_label_style(style);
        self.dual_label.set_right_text(text, &style, font_system)
    }

    pub fn text(&self) -> (&str, &str) {
        self.dual_label.text()
    }

    pub fn dual_label_style(&self, style: &DualButtonStyle) -> DualLabelStyle {
        style.dual_label_style(self.state)
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &DualButtonStyle,
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

pub struct DualButtonBuilder<A: Clone + 'static> {
    pub action: Option<A>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub left_text: String,
    pub right_text: String,
    pub left_text_offset: Point,
    pub right_text_offset: Point,
    pub style: Rc<DualButtonStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl<A: Clone + 'static> DualButtonBuilder<A> {
    pub fn new(style: &Rc<DualButtonStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
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

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> DualButton {
        DualButtonElement::create(self, cx)
    }

    pub fn on_select(mut self, action: A) -> Self {
        self.action = Some(action);
        self
    }

    pub fn on_select_optional(mut self, action: Option<A>) -> Self {
        self.action = action;
        self
    }

    pub fn tooltip_message(mut self, message: impl Into<String>, align: Align2) -> Self {
        self.tooltip_message = Some(message.into());
        self.tooltip_align = align;
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
pub struct DualButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<A>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
}

impl<A: Clone + 'static> DualButtonElement<A> {
    pub fn create(builder: DualButtonBuilder<A>, cx: &mut WindowContext<'_, A>) -> DualButton {
        let DualButtonBuilder {
            action,
            tooltip_message,
            tooltip_align,
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
            inner: DualButtonInner::new(
                left_text,
                right_text,
                left_text_offset,
                right_text_offset,
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

        DualButton { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for DualButtonElement<A> {
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
                let SharedState { inner, style } = &mut *shared_state;

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
                let SharedState { inner, style } = &mut *shared_state;

                if button == PointerButton::Primary
                    && (inner.state == ButtonState::Idle || inner.state == ButtonState::Hovered)
                {
                    let res = inner.set_state(ButtonState::Down, style);

                    if res.needs_repaint {
                        cx.request_repaint();
                    }

                    if let Some(action) = &self.action {
                        cx.send_action(action.clone()).unwrap();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                position, button, ..
            }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState { inner, style } = &mut *shared_state;

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
                    cx.show_tooltip(ElementTooltipInfo {
                        message: message.clone(),
                        element_bounds: cx.rect(),
                        align: self.tooltip_align,
                    });
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

/// A handle to a [`DualButtonElement`], a button with a label.
pub struct DualButton {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    inner: DualButtonInner,
    style: Rc<DualButtonStyle>,
}

impl DualButton {
    pub fn builder<A: Clone + 'static>(style: &Rc<DualButtonStyle>) -> DualButtonBuilder<A> {
        DualButtonBuilder::new(style)
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

    pub fn set_style(&mut self, style: &Rc<DualButtonStyle>, font_system: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(style, font_system);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<DualButtonStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
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
