use std::cell::RefCell;
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{self, Align2};
use crate::math::{Rect, Size, ZIndex};
use crate::style::{Background, BorderStyle, QuadStyle, DEFAULT_ACCENT_COLOR};
use crate::vg::color::RGBA8;
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;
use crate::CursorIcon;

// TODO: Sliding animation for switch

/// The style of a [`Switch`] element
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStyle {
    pub size: f32,
    pub rounding: f32,

    pub outer_border_width: f32,
    pub outer_border_color_idle: RGBA8,
    pub outer_border_color_hover: RGBA8,
    pub outer_border_color_disabled: RGBA8,

    pub off_bg: Background,
    pub on_bg: Background,

    pub off_bg_disabled: Background,
    pub on_bg_disabled: Background,

    pub slider_padding: f32,

    pub slider_bg_idle: Background,
    pub slider_bg_hover: Background,
    pub slider_bg_disabled: Background,

    pub slider_border_width_idle: f32,
    pub slider_border_width_hover: f32,

    pub slider_border_color_idle: RGBA8,
    pub slider_border_color_hover: RGBA8,
    pub slider_border_color_disabled: RGBA8,
}

impl Default for SwitchStyle {
    fn default() -> Self {
        Self {
            size: 20.0,
            rounding: 20.0,

            outer_border_width: 1.0,

            outer_border_color_idle: RGBA8::new(105, 105, 105, 255),
            outer_border_color_hover: RGBA8::new(135, 135, 135, 255),
            outer_border_color_disabled: RGBA8::new(105, 105, 105, 150),

            off_bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
            on_bg: Background::Solid(DEFAULT_ACCENT_COLOR),

            off_bg_disabled: Background::Solid(RGBA8::new(40, 40, 40, 150)),
            on_bg_disabled: Background::Solid(RGBA8::new(150, 150, 150, 150)),

            slider_padding: 2.0,

            slider_bg_idle: Background::Solid(RGBA8::new(255, 255, 255, 180)),
            slider_bg_hover: Background::Solid(RGBA8::new(255, 255, 255, 225)),
            slider_bg_disabled: Background::Solid(RGBA8::new(255, 255, 255, 100)),

            slider_border_width_idle: 1.0,
            slider_border_width_hover: 1.0,

            slider_border_color_idle: RGBA8::new(255, 255, 255, 220),
            slider_border_color_hover: RGBA8::new(255, 255, 255, 255),
            slider_border_color_disabled: RGBA8::new(255, 255, 255, 150),
        }
    }
}

pub struct SwitchBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(bool) -> A>>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub toggled: bool,
    pub disabled: bool,
    pub style: Rc<SwitchStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl<A: Clone + 'static> SwitchBuilder<A> {
    pub fn new(style: &Rc<SwitchStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            toggled: false,
            disabled: false,
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> Switch {
        SwitchElement::create(self, cx)
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

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

pub struct SwitchElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(bool) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
    hovered: bool,
}

impl<A: Clone + 'static> SwitchElement<A> {
    pub fn create(builder: SwitchBuilder<A>, cx: &mut WindowContext<'_, A>) -> Switch {
        let SwitchBuilder {
            action,
            tooltip_message,
            tooltip_align,
            toggled,
            disabled,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState {
            toggled,
            style,
            disabled,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                action,
                tooltip_message,
                tooltip_align,
                hovered: false,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, cx.font_system, cx.clipboard);

        Switch { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for SwitchElement<A> {
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
                if RefCell::borrow(&self.shared_state).disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                cx.cursor_icon = CursorIcon::Pointer;

                if just_entered && self.tooltip_message.is_some() {
                    cx.start_hover_timeout();
                }

                if !self.hovered {
                    self.hovered = true;
                    cx.request_repaint();
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                if RefCell::borrow(&self.shared_state).disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if self.hovered {
                    self.hovered = false;
                    cx.request_repaint();

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed { button, .. }) => {
                if RefCell::borrow(&self.shared_state).disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if button == PointerButton::Primary {
                    let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                    shared_state.toggled = !shared_state.toggled;
                    cx.request_repaint();

                    if let Some(action) = &mut self.action {
                        cx.send_action((action)(shared_state.toggled)).unwrap();
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
        let shared_state = RefCell::borrow(&self.shared_state);

        let bg_quad_style = if shared_state.disabled {
            QuadStyle {
                bg: if shared_state.toggled {
                    shared_state.style.on_bg_disabled.clone()
                } else {
                    shared_state.style.off_bg_disabled.clone()
                },
                border: BorderStyle {
                    color: shared_state.style.outer_border_color_disabled,
                    width: shared_state.style.outer_border_width,
                    radius: shared_state.style.rounding.into(),
                },
            }
        } else {
            QuadStyle {
                bg: if shared_state.toggled {
                    shared_state.style.on_bg.clone()
                } else {
                    shared_state.style.off_bg.clone()
                },
                border: BorderStyle {
                    color: if self.hovered {
                        shared_state.style.outer_border_color_hover
                    } else {
                        shared_state.style.outer_border_color_idle
                    },
                    width: shared_state.style.outer_border_width,
                    radius: shared_state.style.rounding.into(),
                },
            }
        };

        let slider_quad_style = if shared_state.disabled {
            QuadStyle {
                bg: shared_state.style.slider_bg_disabled.clone(),
                border: BorderStyle {
                    color: shared_state.style.slider_border_color_disabled,
                    width: shared_state.style.slider_border_width_idle,
                    radius: shared_state.style.rounding.into(),
                },
            }
        } else if self.hovered {
            QuadStyle {
                bg: shared_state.style.slider_bg_hover.clone(),
                border: BorderStyle {
                    color: shared_state.style.slider_border_color_hover,
                    width: shared_state.style.slider_border_width_hover,
                    radius: shared_state.style.rounding.into(),
                },
            }
        } else {
            QuadStyle {
                bg: shared_state.style.slider_bg_idle.clone(),
                border: BorderStyle {
                    color: shared_state.style.slider_border_color_idle,
                    width: shared_state.style.slider_border_width_idle,
                    radius: shared_state.style.rounding.into(),
                },
            }
        };

        let bounds_rect = Rect::from_size(cx.bounds_size);

        let padding = shared_state.style.slider_padding;
        let size = shared_state.style.size;

        let bg_bounds = layout::centered_rect(bounds_rect.center(), Size::new(size * 2.0, size));

        let slider_bounds = if shared_state.toggled {
            Rect::new(
                bg_bounds.origin
                    + Point::new(bg_bounds.width() - size + padding, padding).to_vector(),
                Size::new(size - (padding * 2.0), size - (padding * 2.0)),
            )
        } else {
            Rect::new(
                bg_bounds.origin + Point::new(padding, padding).to_vector(),
                Size::new(size - (padding * 2.0), size - (padding * 2.0)),
            )
        };

        primitives.add(bg_quad_style.create_primitive(bg_bounds));
        primitives.set_z_index(1);
        primitives.add(slider_quad_style.create_primitive(slider_bounds));
    }
}

/// A handle to a [`SwitchElement`].
pub struct Switch {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    toggled: bool,
    style: Rc<SwitchStyle>,
    disabled: bool,
}

impl Switch {
    pub fn builder<A: Clone + 'static>(style: &Rc<SwitchStyle>) -> SwitchBuilder<A> {
        SwitchBuilder::new(style)
    }

    pub fn min_size(&self) -> Size {
        let size = RefCell::borrow(&self.shared_state).style.size;
        Size::new(size * 2.0, size)
    }

    pub fn set_style(&mut self, style: &Rc<SwitchStyle>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<SwitchStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    pub fn set_toggled(&mut self, toggled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.toggled != toggled {
            shared_state.toggled = toggled;
            self.el.notify_custom_state_change();
        }
    }

    pub fn toggled(&self) -> bool {
        RefCell::borrow(&self.shared_state).toggled
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.disabled != disabled {
            shared_state.disabled = disabled;
            self.el.notify_custom_state_change();
        }
    }
}
