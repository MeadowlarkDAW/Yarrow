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
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;
use crate::CursorIcon;

use super::label::{LabelInner, LabelPrimitives, LabelStyle};

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonStylePart {
    /// The color of the font
    ///
    /// By default this is set to `color::WHITE`.
    pub font_color: RGBA8,

    /// The style of the padded background rectangle behind the text.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    pub back_quad: QuadStyle,
}

impl Default for ButtonStylePart {
    fn default() -> Self {
        Self {
            font_color: color::WHITE,
            back_quad: QuadStyle {
                bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                border: BorderStyle {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        }
    }
}

/// The style of a [`Button`] element
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonStyle {
    /// The text properties.
    pub properties: TextProperties,

    /// The vertical alignment of the text.
    ///
    /// By default this is set to `Align::Center`.
    pub vertical_align: Align,

    /// The minimum size of the clipped text area.
    ///
    /// By default this is set to `Size::new(5.0, 5.0)`.
    pub min_clipped_size: Size,

    /// The padding between the text and the bounding rectangle.
    ///
    /// By default this is set to `Padding::new(6.0, 6.0, 6.0, 6.0)`.
    pub padding: Padding,

    pub idle: ButtonStylePart,
    pub hovered: ButtonStylePart,
    pub down: ButtonStylePart,
    pub disabled: ButtonStylePart,
}

impl ButtonStyle {
    pub fn label_style(&self, state: ButtonState) -> LabelStyle {
        let part = match state {
            ButtonState::Idle => &self.idle,
            ButtonState::Hovered => &self.hovered,
            ButtonState::Down => &self.down,
            ButtonState::Disabled => &self.disabled,
        };

        LabelStyle {
            properties: self.properties,
            font_color: part.font_color,
            vertical_align: self.vertical_align,
            min_clipped_size: self.min_clipped_size,
            back_quad: part.back_quad.clone(),
            padding: self.padding,
        }
    }

    pub fn default_menu_style() -> Self {
        let hovered = ButtonStylePart {
            font_color: color::WHITE,
            back_quad: QuadStyle {
                bg: Background::Solid(RGBA8::new(75, 75, 75, 255)),
                border: BorderStyle {
                    radius: 4.0.into(),
                    ..Default::default()
                },
            },
        };

        Self {
            idle: ButtonStylePart {
                font_color: color::WHITE,
                back_quad: QuadStyle::TRANSPARENT,
            },
            hovered: hovered.clone(),
            down: hovered.clone(),
            disabled: ButtonStylePart {
                font_color: RGBA8::new(150, 150, 150, 255),
                back_quad: QuadStyle::TRANSPARENT,
            },
            padding: Padding::new(3.0, 6.0, 3.0, 6.0),
            ..Default::default()
        }
    }
}

impl Default for ButtonStyle {
    fn default() -> Self {
        let idle = ButtonStylePart {
            font_color: color::WHITE,
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
            properties: TextProperties {
                attrs: DEFAULT_TEXT_ATTRIBUTES,
                align: Some(rootvg::text::Align::Center),
                ..Default::default()
            },
            vertical_align: Align::Center,
            min_clipped_size: Size::new(5.0, 5.0),
            padding: Padding::new(6.0, 6.0, 6.0, 6.0),

            idle: idle.clone(),
            hovered: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(55, 55, 55, 255)),
                    border: BorderStyle {
                        color: RGBA8::new(135, 135, 135, 255),
                        ..idle.back_quad.border
                    },
                },
                ..idle
            },
            down: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    ..idle.back_quad
                },
                ..idle
            },
            disabled: ButtonStylePart {
                font_color: RGBA8::new(150, 150, 150, 255),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Idle,
    Hovered,
    Down,
    Disabled,
}

/// A reusable button struct that can be used by other elements.
pub struct ButtonInner {
    state: ButtonState,
    label: LabelInner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateChangeResult {
    pub state_changed: bool,
    pub needs_repaint: bool,
}

impl ButtonInner {
    pub fn new(
        text: String,
        style: &ButtonStyle,
        font_system: &mut FontSystem,
        text_offset: Point,
    ) -> Self {
        let label = LabelInner::new(
            text,
            &style.label_style(ButtonState::Idle),
            font_system,
            text_offset,
        );

        Self {
            label,
            state: ButtonState::Idle,
        }
    }

    pub fn set_state(&mut self, state: ButtonState, style: &ButtonStyle) -> StateChangeResult {
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

    pub fn set_style(&mut self, style: &ButtonStyle, font_system: &mut FontSystem) {
        self.label.set_style(&self.label_style(style), font_system);
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    pub fn desired_padded_size(&mut self, style: &ButtonStyle) -> Size {
        self.label.desired_padded_size(&self.label_style(style))
    }

    /// Returns the size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn unclipped_text_size(&mut self) -> Size {
        self.label.unclipped_text_size()
    }

    /// Returns `true` if the text has changed.
    pub fn set_text(&mut self, text: &str, font_system: &mut FontSystem) -> bool {
        self.label.set_text(text, font_system)
    }

    pub fn text(&self) -> &str {
        self.label.text()
    }

    pub fn label_style(&self, style: &ButtonStyle) -> LabelStyle {
        style.label_style(self.state)
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &ButtonStyle,
        font_system: &mut FontSystem,
    ) -> LabelPrimitives {
        self.label
            .render_primitives(bounds, &self.label_style(style), font_system)
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_text_offset(&mut self, offset: Point) -> bool {
        self.label.set_text_offset(offset)
    }

    pub fn text_offset(&self) -> Point {
        self.label.text_offset
    }
}

pub struct ButtonBuilder<A: Clone + 'static> {
    pub action: Option<A>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub text: String,
    pub text_offset: Point,
    pub style: Rc<ButtonStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl<A: Clone + 'static> ButtonBuilder<A> {
    pub fn new(style: &Rc<ButtonStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            text: String::new(),
            text_offset: Point::default(),
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> Button {
        ButtonElement::create(self, cx)
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

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn text_offset(mut self, offset: Point) -> Self {
        self.text_offset = offset;
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
pub struct ButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<A>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
}

impl<A: Clone + 'static> ButtonElement<A> {
    pub fn create(builder: ButtonBuilder<A>, cx: &mut WindowContext<'_, A>) -> Button {
        let ButtonBuilder {
            action,
            tooltip_message,
            tooltip_align,
            text,
            text_offset,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: ButtonInner::new(text, &style, cx.font_system, text_offset),
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

        Button { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for ButtonElement<A> {
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
                    cx.show_tooltip(message.clone(), self.tooltip_align, true);
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

        if let Some(text_primitive) = label_primitives.text {
            primitives.set_z_index(1);
            primitives.add_text(text_primitive);
        }
    }
}

struct SharedState {
    inner: ButtonInner,
    style: Rc<ButtonStyle>,
}

/// A handle to a [`ButtonElement`], a button with a label.
pub struct Button {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl Button {
    pub fn builder<A: Clone + 'static>(style: &Rc<ButtonStyle>) -> ButtonBuilder<A> {
        ButtonBuilder::new(style)
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

    /// Returns the size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn unclipped_text_size(&self) -> Size {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .unclipped_text_size()
    }

    pub fn set_text(&mut self, text: &str, font_system: &mut FontSystem) {
        if RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_text(text, font_system)
        {
            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn set_style(&mut self, style: &Rc<ButtonStyle>, font_system: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(style, font_system);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<ButtonStyle> {
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
    pub fn set_text_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_text_offset(offset);

        if changed {
            self.el.notify_custom_state_change();
        }
    }

    pub fn layout(&mut self, origin: Point) {
        let size = self.desired_padded_size();
        self.el.set_rect(Rect::new(origin, size));
    }

    pub fn layout_aligned(&mut self, point: Point, align: Align2) {
        let size = self.desired_padded_size();
        self.el.set_rect(align.align_rect_to_point(point, size));
    }
}
