use std::cell::RefCell;
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::text::CustomGlyphID;
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align2, Padding};
use crate::math::{Rect, Size, ZIndex};
use crate::style::{Background, BorderStyle, QuadStyle, DEFAULT_ACCENT_COLOR};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;
use crate::CursorIcon;

use super::button::{ButtonState, ButtonStylePart, StateChangeResult};
use super::icon::{IconInner, IconStyle};
use super::label::LabelPrimitives;

/// The style of a [`IconToggleButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconToggleButtonStyle {
    /// The size of the icon in points.
    ///
    /// By default this is set to `20.0`.
    pub size: f32,

    /// The padding between the icon and the bounding rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub padding: Padding,

    pub idle_on: ButtonStylePart,
    pub hovered_on: ButtonStylePart,
    //pub down_on: ButtonStylePart,
    pub disabled_on: ButtonStylePart,

    pub idle_off: ButtonStylePart,
    pub hovered_off: ButtonStylePart,
    //pub down_off: ButtonStylePart,
    pub disabled_off: ButtonStylePart,
}

impl IconToggleButtonStyle {
    pub fn icon_style(&self, state: ButtonState, toggled: bool) -> IconStyle {
        let part = if toggled {
            match state {
                ButtonState::Idle => &self.idle_on,
                ButtonState::Hovered | ButtonState::Down => &self.hovered_on,
                ButtonState::Disabled => &self.disabled_on,
            }
        } else {
            match state {
                ButtonState::Idle => &self.idle_off,
                ButtonState::Hovered | ButtonState::Down => &self.hovered_off,
                ButtonState::Disabled => &self.disabled_off,
            }
        };

        IconStyle {
            size: self.size,
            color: part.font_color,
            back_quad: part.back_quad.clone(),
            padding: self.padding,
        }
    }
}

impl Default for IconToggleButtonStyle {
    fn default() -> Self {
        let idle_on = ButtonStylePart {
            font_color: color::WHITE,
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

        let idle_off = ButtonStylePart {
            back_quad: QuadStyle {
                bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                ..idle_on.back_quad
            },
            ..idle_on
        };

        Self {
            size: 20.0,
            padding: Padding::new(4.0, 6.0, 4.0, 6.0),

            idle_on: idle_on.clone(),
            hovered_on: ButtonStylePart {
                back_quad: QuadStyle {
                    border: BorderStyle {
                        color: RGBA8::new(135, 135, 135, 255),
                        ..idle_on.back_quad.border
                    },
                    ..idle_on.back_quad
                },
                ..idle_on
            },
            /*
            down_on: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    ..idle_on.back_quad
                },
                ..idle_on
            },
            */
            disabled_on: ButtonStylePart {
                font_color: RGBA8::new(150, 150, 150, 255),
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
            hovered_off: ButtonStylePart {
                back_quad: QuadStyle {
                    border: BorderStyle {
                        color: RGBA8::new(135, 135, 135, 255),
                        ..idle_off.back_quad.border
                    },
                    ..idle_off.back_quad
                },
                ..idle_off
            },
            /*
            down_off: ButtonStylePart {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    ..idle_off.back_quad
                },
                ..idle_off
            },
            */
            disabled_off: ButtonStylePart {
                font_color: RGBA8::new(150, 150, 150, 255),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleIcons {
    Single(CustomGlyphID),
    Dual {
        off: CustomGlyphID,
        on: CustomGlyphID,
    },
}

impl ToggleIcons {
    pub fn icon(&self, toggled: bool) -> CustomGlyphID {
        match self {
            Self::Single(id) => *id,
            Self::Dual { off, on } => {
                if toggled {
                    *on
                } else {
                    *off
                }
            }
        }
    }
}

/// A reusable button struct that can be used by other elements.
pub struct IconToggleButtonInner {
    icon: IconInner,
    toggle_icons: ToggleIcons,
    state: ButtonState,
    toggled: bool,
}

impl IconToggleButtonInner {
    pub fn new(toggle_icons: ToggleIcons, scale: f32, offset: Point, toggled: bool) -> Self {
        let icon = IconInner::new(toggle_icons.icon(toggled), scale, offset);

        Self {
            icon,
            toggle_icons,
            state: ButtonState::Idle,
            toggled,
        }
    }

    pub fn set_state(
        &mut self,
        state: ButtonState,
        style: &IconToggleButtonStyle,
    ) -> StateChangeResult {
        if self.state != state {
            let old_part = if self.toggled {
                match self.state {
                    ButtonState::Idle => &style.idle_on,
                    ButtonState::Hovered | ButtonState::Down => &style.hovered_on,
                    ButtonState::Disabled => &style.disabled_on,
                }
            } else {
                match self.state {
                    ButtonState::Idle => &style.idle_off,
                    ButtonState::Hovered | ButtonState::Down => &style.hovered_off,
                    ButtonState::Disabled => &style.disabled_off,
                }
            };

            let new_part = if self.toggled {
                match state {
                    ButtonState::Idle => &style.idle_on,
                    ButtonState::Hovered | ButtonState::Down => &style.hovered_on,
                    ButtonState::Disabled => &style.disabled_on,
                }
            } else {
                match state {
                    ButtonState::Idle => &style.idle_off,
                    ButtonState::Hovered | ButtonState::Down => &style.hovered_off,
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

    pub fn set_toggled(&mut self, toggled: bool) -> StateChangeResult {
        if self.toggled != toggled {
            self.toggled = toggled;

            self.icon.icon_id = self.toggle_icons.icon(toggled);

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
    pub fn desired_padded_size(&self, style: &IconToggleButtonStyle) -> Size {
        self.icon.desired_padded_size(&self.icon_style(style))
    }

    /// Returns the rectangular area of the icon from the given bounds size
    /// (icons are assumed to be square).
    pub fn icon_rect(&self, style: &IconStyle, bounds_size: Size) -> Rect {
        self.icon.icon_rect(style, bounds_size)
    }

    pub fn icons(&self) -> ToggleIcons {
        self.toggle_icons
    }

    /// Returns `true` if the icons have changed.
    pub fn set_icons(&mut self, toggle_icons: ToggleIcons) -> bool {
        let changed = self.toggle_icons != toggle_icons;

        self.toggle_icons = toggle_icons;
        self.icon.icon_id = toggle_icons.icon(self.toggled);

        changed
    }

    pub fn icon_style(&self, style: &IconToggleButtonStyle) -> IconStyle {
        style.icon_style(self.state, self.toggled)
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &IconToggleButtonStyle,
    ) -> LabelPrimitives {
        let p = self.icon.render_primitives(bounds, &self.icon_style(style));

        LabelPrimitives {
            text: p.text,
            bg_quad: p.bg_quad,
        }
    }
}

pub struct IconToggleButtonBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(bool) -> A>>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub toggled: bool,
    pub icons: ToggleIcons,
    pub scale: f32,
    pub offset: Point,
    pub style: Rc<IconToggleButtonStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl<A: Clone + 'static> IconToggleButtonBuilder<A> {
    pub fn new(style: &Rc<IconToggleButtonStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            toggled: false,
            icons: ToggleIcons::Single(CustomGlyphID::MAX),
            scale: 1.0,
            offset: Point::default(),
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> IconToggleButton {
        IconToggleButtonElement::create(self, cx)
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

    pub fn icon(mut self, id: impl Into<CustomGlyphID>) -> Self {
        self.icons = ToggleIcons::Single(id.into());
        self
    }

    pub fn dual_icons(
        mut self,
        off_id: impl Into<CustomGlyphID>,
        on_id: impl Into<CustomGlyphID>,
    ) -> Self {
        self.icons = ToggleIcons::Dual {
            off: off_id.into(),
            on: on_id.into(),
        };
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
pub struct IconToggleButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(bool) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
}

impl<A: Clone + 'static> IconToggleButtonElement<A> {
    pub fn create(
        builder: IconToggleButtonBuilder<A>,
        cx: &mut WindowContext<'_, A>,
    ) -> IconToggleButton {
        let IconToggleButtonBuilder {
            action,
            tooltip_message,
            tooltip_align,
            toggled,
            icons,
            scale,
            offset,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: IconToggleButtonInner::new(icons, scale, offset, toggled),
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

        IconToggleButton { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for IconToggleButtonElement<A> {
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
                    cx.show_tooltip(message.clone(), self.tooltip_align, true);
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style, .. } = &mut *shared_state;

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

/// A handle to a [`IconToggleButtonElement`].
pub struct IconToggleButton {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    inner: IconToggleButtonInner,
    style: Rc<IconToggleButtonStyle>,
}

impl IconToggleButton {
    pub fn builder<A: Clone + 'static>(
        style: &Rc<IconToggleButtonStyle>,
    ) -> IconToggleButtonBuilder<A> {
        IconToggleButtonBuilder::new(style)
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn desired_padded_size(&self) -> Size {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style, .. } = &mut *shared_state;

        inner.desired_padded_size(style)
    }

    pub fn set_icon(&mut self, icon_id: impl Into<CustomGlyphID>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state
            .inner
            .set_icons(ToggleIcons::Single(icon_id.into()))
        {
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_dual_icons(
        &mut self,
        off_id: impl Into<CustomGlyphID>,
        on_id: impl Into<CustomGlyphID>,
    ) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icons(ToggleIcons::Dual {
            off: off_id.into(),
            on: on_id.into(),
        }) {
            self.el.notify_custom_state_change();
        }
    }

    pub fn icons(&self) -> ToggleIcons {
        RefCell::borrow(&self.shared_state).inner.toggle_icons
    }

    pub fn set_style(&mut self, style: &Rc<IconToggleButtonStyle>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<IconToggleButtonStyle> {
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
        let SharedState { inner, style, .. } = &mut *shared_state;

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
