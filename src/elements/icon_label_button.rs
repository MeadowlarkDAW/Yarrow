use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::text::{CustomGlyphID, TextProperties};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align, Align2, Padding};
use crate::math::{Rect, Size, ZIndex};
use crate::prelude::ResourceCtx;
use crate::style::{Background, BorderStyle, QuadStyle, DEFAULT_TEXT_ATTRIBUTES};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;
use crate::CursorIcon;

use super::button::{ButtonState, StateChangeResult};
use super::icon_label::{
    IconLabelClipMode, IconLabelInner, IconLabelLayout, IconLabelPrimitives, IconLabelStyle,
};

#[derive(Debug, Clone, PartialEq)]
pub struct IconLabelButtonStylePart {
    /// The color of the text
    ///
    /// By default this is set to `color::WHITE`.
    pub text_color: RGBA8,
    /// The color of the icon
    ///
    /// By default this is set to `color::WHITE`.
    pub icon_color: RGBA8,

    /// The style of the padded background rectangle behind the text.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    ///
    /// By default this is set to `QuadStyle::TRANSPARENT`.
    pub back_quad: QuadStyle,
}

/// The style of a [`IconLabelButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconLabelButtonStyle {
    /// The properties of the text
    pub text_properties: TextProperties,

    /// The size of the icon in points.
    ///
    /// By default this is set to `20.0`.
    pub icon_size: f32,

    pub layout: IconLabelLayout,

    /// The minimum size of the clipped text area.
    ///
    /// By default this is set to `Size::new(5.0, 5.0)`.
    pub text_min_clipped_size: Size,

    pub clip_mode: IconLabelClipMode,

    /// The padding between the text and the bounding rectangle.
    ///
    /// By default this has all values set to `6.0`.
    pub text_padding: Padding,
    /// The padding between the icon and the bounding rectangle.
    ///
    /// By default this has all values set to `6.0`.
    pub icon_padding: Padding,

    pub idle: IconLabelButtonStylePart,
    pub hovered: IconLabelButtonStylePart,
    pub down: IconLabelButtonStylePart,
    pub disabled: IconLabelButtonStylePart,
}

impl IconLabelButtonStyle {
    pub fn default_dropdown_style() -> Self {
        Self {
            layout: IconLabelLayout::LeftAlignTextRightAlignIcon,
            icon_padding: Padding::new(0.0, 6.0, 0.0, 2.0),
            ..Default::default()
        }
    }
}

impl Default for IconLabelButtonStyle {
    fn default() -> Self {
        let idle = IconLabelButtonStylePart {
            text_color: color::WHITE,
            icon_color: color::WHITE,
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
            text_properties: TextProperties {
                attrs: DEFAULT_TEXT_ATTRIBUTES,
                ..Default::default()
            },
            icon_size: 20.0,
            text_min_clipped_size: Size::new(5.0, 5.0),
            text_padding: Padding::new(6.0, 6.0, 6.0, 6.0),
            icon_padding: Padding::new(0.0, 0.0, 0.0, 6.0),

            clip_mode: IconLabelClipMode::default(),
            layout: IconLabelLayout::default(),

            idle: idle.clone(),
            hovered: IconLabelButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(55, 55, 55, 255)),
                    border: BorderStyle {
                        color: RGBA8::new(135, 135, 135, 255),
                        ..idle.back_quad.border
                    },
                },
                ..idle
            },
            down: IconLabelButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    ..idle.back_quad
                },
                ..idle
            },
            disabled: IconLabelButtonStylePart {
                text_color: RGBA8::new(150, 150, 150, 255),
                icon_color: RGBA8::new(150, 150, 150, 255),
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

impl IconLabelButtonStyle {
    pub fn icon_label_style(&self, state: ButtonState) -> IconLabelStyle {
        let part = match state {
            ButtonState::Idle => &self.idle,
            ButtonState::Hovered => &self.hovered,
            ButtonState::Down => &self.down,
            ButtonState::Disabled => &self.disabled,
        };

        IconLabelStyle {
            text_properties: self.text_properties,
            icon_size: self.icon_size,
            text_color: part.text_color,
            icon_color: part.icon_color,
            vertical_align: Align::Center,
            text_min_clipped_size: self.text_min_clipped_size,
            back_quad: part.back_quad.clone(),
            text_padding: self.text_padding,
            icon_padding: self.icon_padding,
            layout: self.layout,
            clip_mode: self.clip_mode,
        }
    }
}

/// A reusable button struct that can be used by other elements.
pub struct IconLabelButtonInner {
    state: ButtonState,
    icon_label: IconLabelInner,
}

impl IconLabelButtonInner {
    pub fn new(
        text: Option<impl Into<String>>,
        icon_id: Option<CustomGlyphID>,
        text_offset: Point,
        icon_offset: Point,
        icon_scale: f32,
        disabled: bool,
        style: &IconLabelButtonStyle,
        res: &mut ResourceCtx,
    ) -> Self {
        let state = ButtonState::new(disabled);

        let icon_label = IconLabelInner::new(
            text,
            icon_id,
            text_offset,
            icon_offset,
            icon_scale,
            &style.icon_label_style(state),
            res,
        );

        Self { icon_label, state }
    }

    pub fn set_state(
        &mut self,
        state: ButtonState,
        style: &IconLabelButtonStyle,
    ) -> StateChangeResult {
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

    pub fn set_style(&mut self, style: &IconLabelButtonStyle, res: &mut ResourceCtx) {
        self.icon_label
            .set_style(&self.icon_label_style(style), res);
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    pub fn desired_padded_size(&mut self, style: &IconLabelButtonStyle) -> Size {
        self.icon_label
            .desired_padded_size(&self.icon_label_style(style))
    }

    /// Returns `true` if the text has changed.
    pub fn set_text(
        &mut self,
        text: &str,
        style: &IconLabelButtonStyle,
        res: &mut ResourceCtx,
    ) -> bool {
        self.icon_label
            .set_text(text, &style.icon_label_style(self.state), res)
    }

    pub fn text(&self) -> &str {
        self.icon_label.text()
    }

    pub fn icon_label_style(&self, style: &IconLabelButtonStyle) -> IconLabelStyle {
        style.icon_label_style(self.state)
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &IconLabelButtonStyle,
        res: &mut ResourceCtx,
    ) -> IconLabelPrimitives {
        self.icon_label
            .render_primitives(bounds, &self.icon_label_style(style), res)
    }

    /// An offset that can be used mainly to correct the position of text.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_text_offset(&mut self, offset: Point) -> bool {
        self.icon_label.set_text_offset(offset)
    }

    /// An offset that can be used mainly to correct the position of the icon.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_icon_offset(&mut self, offset: Point) -> bool {
        self.icon_label.set_icon_offset(offset)
    }
}

pub struct IconLabelButtonBuilder<A: Clone + 'static> {
    pub action: Option<A>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub text: Option<String>,
    pub icon_id: Option<CustomGlyphID>,
    pub icon_scale: f32,
    pub text_offset: Point,
    pub icon_offset: Point,
    pub style: Rc<IconLabelButtonStyle>,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub disabled: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> IconLabelButtonBuilder<A> {
    pub fn new(style: &Rc<IconLabelButtonStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            text: None,
            icon_id: None,
            icon_scale: 1.0,
            text_offset: Point::default(),
            icon_offset: Point::default(),
            style: Rc::clone(style),
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            disabled: false,
            scissor_rect_id: None,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> IconLabelButton {
        IconLabelButtonElement::create(self, cx)
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

    pub fn text(mut self, text: Option<impl Into<String>>) -> Self {
        self.text = text.map(|t| t.into());
        self
    }

    pub fn icon(mut self, icon_id: Option<impl Into<CustomGlyphID>>) -> Self {
        self.icon_id = icon_id.map(|i| i.into());
        self
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
    pub const fn icon_scale(mut self, scale: f32) -> Self {
        self.icon_scale = scale;
        self
    }

    /// An offset that can be used mainly to correct the position of the text.
    /// This does not effect the position of the background quad.
    pub const fn text_offset(mut self, offset: Point) -> Self {
        self.text_offset = offset;
        self
    }

    /// An offset that can be used mainly to correct the position of the icon.
    /// This does not effect the position of the background quad.
    pub const fn icon_offset(mut self, offset: Point) -> Self {
        self.icon_offset = offset;
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
pub struct IconLabelButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<A>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
}

impl<A: Clone + 'static> IconLabelButtonElement<A> {
    pub fn create(
        builder: IconLabelButtonBuilder<A>,
        cx: &mut WindowContext<'_, A>,
    ) -> IconLabelButton {
        let IconLabelButtonBuilder {
            action,
            tooltip_message,
            tooltip_align,
            text,
            icon_id,
            icon_scale,
            text_offset,
            icon_offset,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            disabled,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id) = cx.z_index_and_scissor_rect_id(z_index, scissor_rect_id);

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: IconLabelButtonInner::new(
                text,
                icon_id,
                text_offset,
                icon_offset,
                icon_scale,
                disabled,
                &style,
                &mut cx.res,
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
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        IconLabelButton { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for IconLabelButtonElement<A> {
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
            inner.render_primitives(Rect::from_size(cx.bounds_size), style, cx.res);

        if let Some(quad_primitive) = label_primitives.bg_quad {
            primitives.add(quad_primitive);
        }

        if let Some(p) = label_primitives.text {
            primitives.set_z_index(1);
            primitives.add_text(p);
        }

        if let Some(p) = label_primitives.icon {
            primitives.set_z_index(1);
            primitives.add_text(p);
        }
    }
}

/// A handle to a [`IconLabelButtonElement`], a button with a label.
pub struct IconLabelButton {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    inner: IconLabelButtonInner,
    style: Rc<IconLabelButtonStyle>,
}

impl IconLabelButton {
    pub fn builder<A: Clone + 'static>(
        style: &Rc<IconLabelButtonStyle>,
    ) -> IconLabelButtonBuilder<A> {
        IconLabelButtonBuilder::new(style)
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

    pub fn set_text(&mut self, text: &str, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        if inner.set_text(text, style, res) {
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_icon_id(&mut self, icon_id: Option<impl Into<CustomGlyphID>>) {
        let icon_id: Option<CustomGlyphID> = icon_id.map(|i| i.into());

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon_label.icon_id != icon_id {
            shared_state.inner.icon_label.icon_id = icon_id;
            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn icon_id(&self) -> Option<CustomGlyphID> {
        RefCell::borrow(&self.shared_state).inner.icon_label.icon_id
    }

    pub fn set_style(&mut self, style: &Rc<IconLabelButtonStyle>, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(style, res);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<IconLabelButtonStyle> {
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

    /// An offset that can be used mainly to correct the position of the text.
    /// This does not effect the position of the background quad.
    pub fn set_text_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_text_offset(offset);

        if changed {
            self.el.notify_custom_state_change();
        }
    }

    /// An offset that can be used mainly to correct the position of the icon.
    /// This does not effect the position of the background quad.
    pub fn set_icon_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_icon_offset(offset);

        if changed {
            self.el.notify_custom_state_change();
        }
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
    pub fn set_scale(&mut self, scale: f32) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon_label.icon_scale != scale {
            shared_state.inner.icon_label.icon_scale = scale;
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
