use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::text::{CustomGlyphID, TextProperties};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align, Align2, Padding};
use crate::math::{Rect, Size, ZIndex};
use crate::prelude::ResourceCtx;
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

use super::button::{ButtonState, StateChangeResult};
use super::icon_label::{
    IconLabelClipMode, IconLabelInner, IconLabelLayout, IconLabelPrimitives, IconLabelStyle,
};
use super::icon_label_button::IconLabelButtonStylePart;
use super::icon_toggle_button::ToggleIcons;
use super::toggle_button::{DualText, ToggleText};

/// The style of a [`IconLabelToggleButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconLabelToggleButtonStyle {
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

    pub idle_on: IconLabelButtonStylePart,
    pub hovered_on: IconLabelButtonStylePart,
    pub disabled_on: IconLabelButtonStylePart,

    pub idle_off: IconLabelButtonStylePart,
    pub hovered_off: IconLabelButtonStylePart,
    pub disabled_off: IconLabelButtonStylePart,
}

impl Default for IconLabelToggleButtonStyle {
    fn default() -> Self {
        let idle_on = IconLabelButtonStylePart {
            text_color: color::WHITE,
            icon_color: color::WHITE,
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

        let idle_off = IconLabelButtonStylePart {
            back_quad: QuadStyle {
                bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                ..idle_on.back_quad
            },
            ..idle_on
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

            idle_on: idle_on.clone(),
            hovered_on: IconLabelButtonStylePart {
                back_quad: QuadStyle {
                    border: BorderStyle {
                        color: RGBA8::new(135, 135, 135, 255),
                        ..idle_on.back_quad.border
                    },
                    ..idle_on.back_quad
                },
                ..idle_on
            },
            disabled_on: IconLabelButtonStylePart {
                text_color: RGBA8::new(150, 150, 150, 255),
                icon_color: RGBA8::new(150, 150, 150, 255),
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
            hovered_off: IconLabelButtonStylePart {
                back_quad: QuadStyle {
                    border: BorderStyle {
                        color: RGBA8::new(135, 135, 135, 255),
                        ..idle_off.back_quad.border
                    },
                    ..idle_off.back_quad
                },
                ..idle_off
            },
            disabled_off: IconLabelButtonStylePart {
                text_color: RGBA8::new(150, 150, 150, 255),
                icon_color: RGBA8::new(150, 150, 150, 255),
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

impl IconLabelToggleButtonStyle {
    pub fn icon_label_style(&self, state: ButtonState, toggled: bool) -> IconLabelStyle {
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
pub struct IconLabelToggleButtonInner {
    icon_label: IconLabelInner,
    dual_text: Option<DualText>,
    toggle_icons: Option<ToggleIcons>,
    state: ButtonState,
    toggled: bool,
}

impl IconLabelToggleButtonInner {
    pub fn new(
        text: Option<ToggleText>,
        toggle_icons: Option<ToggleIcons>,
        text_offset: Point,
        icon_offset: Point,
        icon_scale: f32,
        toggled: bool,
        disabled: bool,
        style: &IconLabelToggleButtonStyle,
        res: &mut ResourceCtx,
    ) -> Self {
        let (text, dual_text) = text
            .map(|text| match text {
                ToggleText::Single(text) => (Some(text), None),
                ToggleText::Dual { off, on } => (
                    Some(if toggled { on.clone() } else { off.clone() }),
                    Some(DualText { off, on }),
                ),
            })
            .unwrap_or((None, None));

        let icon_id = toggle_icons.map(|i| i.icon(toggled));

        let state = ButtonState::new(disabled);

        let icon_label = IconLabelInner::new(
            text,
            icon_id,
            text_offset,
            icon_offset,
            icon_scale,
            &style.icon_label_style(state, toggled),
            res,
        );

        Self {
            icon_label,
            dual_text,
            toggle_icons,
            state,
            toggled,
        }
    }

    pub fn set_state(
        &mut self,
        state: ButtonState,
        style: &IconLabelToggleButtonStyle,
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

    pub fn set_style(&mut self, style: &IconLabelToggleButtonStyle, res: &mut ResourceCtx) {
        self.icon_label
            .set_style(&self.icon_label_style(style), res);
    }

    pub fn set_toggled(
        &mut self,
        toggled: bool,
        style: &IconLabelToggleButtonStyle,
        res: &mut ResourceCtx,
    ) -> StateChangeResult {
        if self.toggled != toggled {
            self.toggled = toggled;

            if let Some(dual_text) = &self.dual_text {
                let new_text = if toggled {
                    &dual_text.on
                } else {
                    &dual_text.off
                };

                self.icon_label.set_text(
                    new_text,
                    &style.icon_label_style(self.state, self.toggled),
                    res,
                );
            }

            self.icon_label.icon_id = self.toggle_icons.map(|i| i.icon(toggled));

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
    pub fn desired_padded_size(&mut self, style: &IconLabelToggleButtonStyle) -> Size {
        self.icon_label
            .desired_padded_size(&self.icon_label_style(style))
    }

    /// Returns `true` if the text has changed.
    pub fn set_text(
        &mut self,
        text: ToggleText,
        style: &IconLabelToggleButtonStyle,
        res: &mut ResourceCtx,
    ) -> bool {
        let (text, dual_text) = match text {
            ToggleText::Single(text) => (text, None),
            ToggleText::Dual { off, on } => (
                if self.toggled {
                    on.clone()
                } else {
                    off.clone()
                },
                Some(DualText { off, on }),
            ),
        };

        self.dual_text = dual_text;

        self.icon_label.set_text(
            &text,
            &style.icon_label_style(self.state, self.toggled),
            res,
        )
    }

    /// Returns `true` if the icons have changed.
    pub fn set_icons(&mut self, toggle_icons: Option<ToggleIcons>) -> bool {
        let changed = self.toggle_icons != toggle_icons;

        self.toggle_icons = toggle_icons;
        self.icon_label.icon_id = toggle_icons.map(|i| i.icon(self.toggled));

        changed
    }

    pub fn text(&self) -> &str {
        self.icon_label.text()
    }

    pub fn icons(&self) -> Option<ToggleIcons> {
        self.toggle_icons
    }

    pub fn icon_label_style(&self, style: &IconLabelToggleButtonStyle) -> IconLabelStyle {
        style.icon_label_style(self.state, self.toggled)
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &IconLabelToggleButtonStyle,
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

pub struct IconLabelToggleButtonBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(bool) -> A>>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub toggled: bool,
    pub text: Option<ToggleText>,
    pub icons: Option<ToggleIcons>,
    pub icon_scale: f32,
    pub text_offset: Point,
    pub icon_offset: Point,
    pub style: Rc<IconLabelToggleButtonStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub disabled: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl<A: Clone + 'static> IconLabelToggleButtonBuilder<A> {
    pub fn new(style: &Rc<IconLabelToggleButtonStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            toggled: false,
            text: None,
            icons: None,
            icon_scale: 1.0,
            text_offset: Point::default(),
            icon_offset: Point::default(),
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            disabled: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> IconLabelToggleButton {
        IconLabelToggleButtonElement::create(self, cx)
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

    pub fn text(mut self, text: Option<impl Into<String>>) -> Self {
        self.text = text.map(|t| ToggleText::Single(t.into()));
        self
    }

    pub fn dual_text(
        mut self,
        off_on_text: Option<(impl Into<String>, impl Into<String>)>,
    ) -> Self {
        self.text = off_on_text.map(|(off_text, on_text)| ToggleText::Dual {
            off: off_text.into(),
            on: on_text.into(),
        });
        self
    }

    pub fn icon(mut self, id: Option<impl Into<CustomGlyphID>>) -> Self {
        self.icons = id.map(|id| ToggleIcons::Single(id.into()));
        self
    }

    pub const fn icons(mut self, icons: Option<ToggleIcons>) -> Self {
        self.icons = icons;
        self
    }

    pub fn dual_icons(
        mut self,
        off_on_ids: Option<(impl Into<CustomGlyphID>, impl Into<CustomGlyphID>)>,
    ) -> Self {
        self.icons = off_on_ids.map(|(off_id, on_id)| ToggleIcons::Dual {
            off: off_id.into(),
            on: on_id.into(),
        });
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

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = scissor_rect_id;
        self
    }
}

/// A button element with a label.
pub struct IconLabelToggleButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(bool) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
}

impl<A: Clone + 'static> IconLabelToggleButtonElement<A> {
    pub fn create(
        builder: IconLabelToggleButtonBuilder<A>,
        cx: &mut WindowContext<'_, A>,
    ) -> IconLabelToggleButton {
        let IconLabelToggleButtonBuilder {
            action,
            tooltip_message,
            tooltip_align,
            toggled,
            text,
            icons,
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

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: IconLabelToggleButtonInner::new(
                text,
                icons,
                text_offset,
                icon_offset,
                icon_scale,
                toggled,
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

        IconLabelToggleButton { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for IconLabelToggleButtonElement<A> {
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
                    let res2 = inner.set_toggled(!inner.toggled(), style, &mut cx.res);

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

/// A handle to a [`IconLabelToggleButtonElement`], a button with a label.
pub struct IconLabelToggleButton {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    inner: IconLabelToggleButtonInner,
    style: Rc<IconLabelToggleButtonStyle>,
}

impl IconLabelToggleButton {
    pub fn builder<A: Clone + 'static>(
        style: &Rc<IconLabelToggleButtonStyle>,
    ) -> IconLabelToggleButtonBuilder<A> {
        IconLabelToggleButtonBuilder::new(style)
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

    pub fn set_text(&mut self, text: impl Into<String>, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        if inner.set_text(ToggleText::Single(text.into()), style, res) {
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_dual_text(
        &mut self,
        off_text: impl Into<String>,
        on_text: impl Into<String>,
        res: &mut ResourceCtx,
    ) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        if inner.set_text(
            ToggleText::Dual {
                off: off_text.into(),
                on: on_text.into(),
            },
            style,
            res,
        ) {
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_icon(&mut self, icon_id: Option<impl Into<CustomGlyphID>>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state
            .inner
            .set_icons(icon_id.map(|i| ToggleIcons::Single(i.into())))
        {
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_icons(&mut self, icons: Option<ToggleIcons>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icons(icons) {
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_dual_icons(
        &mut self,
        off_on_ids: Option<(impl Into<CustomGlyphID>, impl Into<CustomGlyphID>)>,
    ) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state
            .inner
            .set_icons(off_on_ids.map(|(off_id, on_id)| ToggleIcons::Dual {
                off: off_id.into(),
                on: on_id.into(),
            }))
        {
            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn icons(&self) -> Option<ToggleIcons> {
        RefCell::borrow(&self.shared_state).inner.toggle_icons
    }

    pub fn set_style(&mut self, style: &Rc<IconLabelToggleButtonStyle>, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(style, res);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<IconLabelToggleButtonStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    pub fn set_toggled(&mut self, toggled: bool, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        if inner.toggled() != toggled {
            inner.set_toggled(toggled, style, res);
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
