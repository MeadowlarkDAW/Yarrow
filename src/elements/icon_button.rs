use std::cell::RefCell;
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::text::CustomGlyphID;
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align2, Padding};
use crate::math::{Rect, Size, ZIndex};
use crate::style::{Background, BorderStyle, QuadStyle};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;
use crate::CursorIcon;

use super::button::{ButtonState, ButtonStylePart, StateChangeResult};
use super::icon::{IconInner, IconStyle};
use super::label::LabelPrimitives;

/// The style of a [`IconButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconButtonStyle {
    /// The size of the icon in points.
    ///
    /// By default this is set to `20.0`.
    pub size: f32,

    /// The padding between the icon and the bounding rectangle.
    ///
    /// By default this is set to `Padding::new(6.0, 6.0, 6.0, 6.0)`.
    pub padding: Padding,

    pub idle: ButtonStylePart,
    pub hovered: ButtonStylePart,
    pub down: ButtonStylePart,
    pub disabled: ButtonStylePart,
}

impl IconButtonStyle {
    pub fn icon_style(&self, state: ButtonState) -> IconStyle {
        let part = match state {
            ButtonState::Idle => &self.idle,
            ButtonState::Hovered => &self.hovered,
            ButtonState::Down => &self.down,
            ButtonState::Disabled => &self.disabled,
        };

        IconStyle {
            size: self.size,
            color: part.font_color,
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
            padding: Padding::new(2.0, 3.0, 2.0, 3.0),
            ..Default::default()
        }
    }
}

impl Default for IconButtonStyle {
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
            size: 20.0,
            padding: Padding::new(4.0, 6.0, 4.0, 6.0),

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

/// A reusable button struct that can be used by other elements.
pub struct IconButtonInner {
    pub icon: IconInner,
    pub state: ButtonState,
}

impl IconButtonInner {
    pub fn new(icon_id: CustomGlyphID, scale: f32, offset: Point, disabled: bool) -> Self {
        let icon = IconInner::new(icon_id, scale, offset);

        let state = ButtonState::new(disabled);

        Self { icon, state }
    }

    pub fn set_state(&mut self, state: ButtonState, style: &IconButtonStyle) -> StateChangeResult {
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

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    pub fn desired_padded_size(&mut self, style: &IconButtonStyle) -> Size {
        self.icon.desired_padded_size(&self.icon_style(style))
    }

    /// Returns the rectangular area of the icon from the given bounds size
    /// (icons are assumed to be square).
    pub fn icon_rect(&self, style: &IconStyle, bounds_size: Size) -> Rect {
        self.icon.icon_rect(style, bounds_size)
    }

    pub fn icon_style(&self, style: &IconButtonStyle) -> IconStyle {
        style.icon_style(self.state)
    }

    pub fn render_primitives(&mut self, bounds: Rect, style: &IconButtonStyle) -> LabelPrimitives {
        self.icon.render_primitives(bounds, &self.icon_style(style))
    }
}

pub struct IconButtonBuilder<A: Clone + 'static> {
    pub action: Option<A>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub icon: CustomGlyphID,
    pub scale: f32,
    pub offset: Point,
    pub style: Rc<IconButtonStyle>,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub disabled: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> IconButtonBuilder<A> {
    pub fn new(style: &Rc<IconButtonStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            icon: CustomGlyphID::MAX,
            scale: 1.0,
            offset: Point::default(),
            style: Rc::clone(style),
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            disabled: false,
            scissor_rect_id: None,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> IconButton {
        IconButtonElement::create(self, cx)
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

    pub fn icon(mut self, id: impl Into<CustomGlyphID>) -> Self {
        self.icon = id.into();
        self
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
    pub const fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn offset(mut self, offset: Point) -> Self {
        self.offset = offset;
        self
    }

    pub const fn z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = Some(z_index);
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

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = Some(scissor_rect_id);
        self
    }
}

/// A button element with a label.
pub struct IconButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<A>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
}

impl<A: Clone + 'static> IconButtonElement<A> {
    pub fn create(builder: IconButtonBuilder<A>, cx: &mut WindowContext<'_, A>) -> IconButton {
        let IconButtonBuilder {
            action,
            tooltip_message,
            tooltip_align,
            icon,
            scale,
            offset,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            disabled,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id) = cx.z_index_and_scissor_rect_id(z_index, scissor_rect_id);

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: IconButtonInner::new(icon, scale, offset, disabled),
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
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        IconButton { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for IconButtonElement<A> {
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

        let label_primitives = inner.render_primitives(Rect::from_size(cx.bounds_size), style);

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
    inner: IconButtonInner,
    style: Rc<IconButtonStyle>,
}

/// A handle to a [`IconButtonElement`], a button with a label.
pub struct IconButton {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl IconButton {
    pub fn builder<A: Clone + 'static>(style: &Rc<IconButtonStyle>) -> IconButtonBuilder<A> {
        IconButtonBuilder::new(style)
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

    pub fn set_icon_id(&mut self, icon_id: impl Into<CustomGlyphID>) {
        let icon_id: CustomGlyphID = icon_id.into();

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon.icon_id != icon_id {
            shared_state.inner.icon.icon_id = icon_id;
            self.el.notify_custom_state_change();
        }
    }

    pub fn icon_id(&self) -> CustomGlyphID {
        RefCell::borrow(&self.shared_state).inner.icon.icon_id
    }

    pub fn set_style(&mut self, style: &Rc<IconButtonStyle>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<IconButtonStyle> {
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
    pub fn set_offset(&mut self, offset: Point) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon.offset != offset {
            shared_state.inner.icon.offset = offset;
            self.el.notify_custom_state_change();
        }
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
    pub fn set_scale(&mut self, scale: f32) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon.scale != scale {
            shared_state.inner.icon.scale = scale;
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
