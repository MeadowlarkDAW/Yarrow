use derive_where::derive_where;
use std::cell::RefCell;
use std::rc::Rc;

use crate::derive::*;
use crate::prelude::*;

// TODO: Sliding animation for switch

/// The style of a [`Switch`] element
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStyle {
    pub size: f32,
    pub rounding: f32,

    pub outer_border_width: f32,

    pub outer_border_color_off: RGBA8,
    pub outer_border_color_off_hover: Option<RGBA8>,
    pub outer_border_color_off_disabled: DisabledColor,
    pub outer_border_color_on: Option<RGBA8>,
    pub outer_border_color_on_hover: Option<RGBA8>,
    pub outer_border_color_on_disabled: DisabledColor,

    pub off_bg: Background,
    pub off_bg_hover: Option<Background>,
    pub off_bg_disabled: DisabledBackground,
    pub on_bg: Option<Background>,
    pub on_bg_hover: Option<Background>,
    pub on_bg_disabled: DisabledBackground,

    pub slider_padding: f32,

    pub slider_bg_off: Background,
    pub slider_bg_off_hover: Option<Background>,
    pub slider_bg_off_disabled: DisabledBackground,
    pub slider_bg_on: Option<Background>,
    pub slider_bg_on_hover: Option<Background>,
    pub slider_bg_on_disabled: DisabledBackground,

    pub slider_border_width: f32,
    pub slider_border_width_hover: Option<f32>,

    pub slider_border_color_off: RGBA8,
    pub slider_border_color_off_hover: Option<RGBA8>,
    pub slider_border_color_off_disabled: DisabledColor,
    pub slider_border_color_on: Option<RGBA8>,
    pub slider_border_color_on_hover: Option<RGBA8>,
    pub slider_border_color_on_disabled: DisabledColor,

    /// The cursor icon to show when the user hovers over this element.
    ///
    /// If this is `None`, then the cursor icon will not be changed.
    ///
    /// By default this is set to `None`.
    pub cursor_icon: Option<CursorIcon>,

    /// Additional flags for the quad primitives.
    ///
    /// By default this is set to `QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL`.
    pub quad_flags: QuadFlags,
}

impl Default for SwitchStyle {
    fn default() -> Self {
        Self {
            size: 20.0,
            rounding: 20.0,
            outer_border_width: 0.0,
            outer_border_color_off: color::TRANSPARENT,
            outer_border_color_off_hover: None,
            outer_border_color_off_disabled: Default::default(),
            outer_border_color_on: None,
            outer_border_color_on_hover: None,
            outer_border_color_on_disabled: Default::default(),
            off_bg: Background::TRANSPARENT,
            off_bg_hover: None,
            off_bg_disabled: Default::default(),
            on_bg: None,
            on_bg_hover: None,
            on_bg_disabled: Default::default(),
            slider_padding: 2.0,
            slider_bg_off: Background::TRANSPARENT,
            slider_bg_off_hover: None,
            slider_bg_off_disabled: Default::default(),
            slider_bg_on: None,
            slider_bg_on_hover: None,
            slider_bg_on_disabled: Default::default(),
            slider_border_width: 0.0,
            slider_border_width_hover: None,
            slider_border_color_off: color::TRANSPARENT,
            slider_border_color_off_hover: None,
            slider_border_color_off_disabled: Default::default(),
            slider_border_color_on: None,
            slider_border_color_on_hover: None,
            slider_border_color_on_disabled: Default::default(),
            cursor_icon: None,
            quad_flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        }
    }
}

impl ElementStyle for SwitchStyle {
    const ID: &'static str = "switch";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self::default()
    }
}

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[element_builder_disabled]
#[element_builder_tooltip]
#[derive_where(Default)]
pub struct SwitchBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(bool) -> A>>,
    pub toggled: bool,
}

impl<A: Clone + 'static> SwitchBuilder<A> {
    pub fn on_toggled<F: FnMut(bool) -> A + 'static>(mut self, f: F) -> Self {
        self.action = Some(Box::new(f));
        self
    }

    pub const fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }

    pub fn build(self, window_cx: &mut WindowContext<'_, A>) -> Switch {
        let SwitchBuilder {
            action,
            tooltip_data,
            toggled,
            disabled,
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
        } = self;

        let style = window_cx
            .res
            .style_system
            .get::<SwitchStyle>(window_cx.builder_class(class));
        let cursor_icon = style.cursor_icon;

        let shared_state = Rc::new(RefCell::new(SharedState {
            toggled,
            disabled,
            tooltip_inner: TooltipInner::new(tooltip_data),
        }));

        let el = ElementBuilder::new(SwitchElement {
            shared_state: Rc::clone(&shared_state),
            action,
            hovered: false,
            cursor_icon,
        })
        .builder_values(z_index, scissor_rect, class, window_cx)
        .rect(rect)
        .hidden(manually_hidden)
        .flags(ElementFlags::PAINTS | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
        .build(window_cx);

        Switch { el, shared_state }
    }
}

struct SwitchElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(bool) -> A>>,
    hovered: bool,
    cursor_icon: Option<CursorIcon>,
}

impl<A: Clone + 'static> Element<A> for SwitchElement<A> {
    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        shared_state
            .tooltip_inner
            .handle_event(&event, shared_state.disabled, cx);

        match event {
            ElementEvent::CustomStateChanged => {
                cx.request_repaint();
            }
            ElementEvent::StyleChanged => {
                let style = cx.res.style_system.get::<SwitchStyle>(cx.class());
                self.cursor_icon = style.cursor_icon;
            }
            ElementEvent::Pointer(PointerEvent::Moved { .. }) => {
                if shared_state.disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if let Some(cursor_icon) = self.cursor_icon {
                    cx.cursor_icon = cursor_icon;
                }

                if !self.hovered {
                    self.hovered = true;
                    cx.request_repaint();
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                if shared_state.disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if self.hovered {
                    self.hovered = false;
                    cx.request_repaint();

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed { button, .. }) => {
                if shared_state.disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if button == PointerButton::Primary {
                    shared_state.toggled = !shared_state.toggled;
                    cx.request_repaint();

                    if let Some(action) = &mut self.action {
                        cx.send_action((action)(shared_state.toggled)).unwrap();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render(&mut self, cx: RenderContext, primitives: &mut PrimitiveGroup) {
        let shared_state = RefCell::borrow(&self.shared_state);

        let style = cx.res.style_system.get::<SwitchStyle>(cx.class);

        let get_colors = || -> (Background, Background, RGBA8, RGBA8) {
            let bg_quad_bg = if shared_state.toggled {
                if self.hovered {
                    style.on_bg_hover.unwrap_or(
                        style
                            .on_bg
                            .unwrap_or(style.off_bg_hover.unwrap_or(style.off_bg)),
                    )
                } else {
                    style.on_bg.clone().unwrap_or_else(|| style.off_bg.clone())
                }
            } else {
                if self.hovered {
                    style.off_bg_hover.unwrap_or(style.off_bg)
                } else {
                    style.off_bg.clone()
                }
            };

            let slider_quad_bg = if shared_state.toggled {
                if self.hovered {
                    style.slider_bg_on_hover.unwrap_or(
                        style
                            .slider_bg_on
                            .unwrap_or(style.slider_bg_off_hover.unwrap_or(style.slider_bg_off)),
                    )
                } else {
                    style
                        .slider_bg_on
                        .clone()
                        .unwrap_or_else(|| style.slider_bg_off.clone())
                }
            } else {
                if self.hovered {
                    style.slider_bg_off_hover.unwrap_or(style.slider_bg_off)
                } else {
                    style.slider_bg_off.clone()
                }
            };

            let bg_border_color = if self.hovered {
                if shared_state.toggled {
                    style.outer_border_color_on_hover.unwrap_or(
                        style.outer_border_color_on.unwrap_or(
                            style
                                .outer_border_color_off_hover
                                .unwrap_or(style.outer_border_color_off),
                        ),
                    )
                } else {
                    style
                        .outer_border_color_off_hover
                        .unwrap_or(style.outer_border_color_off)
                }
            } else {
                if shared_state.toggled {
                    style
                        .outer_border_color_on
                        .unwrap_or(style.outer_border_color_off)
                } else {
                    style.outer_border_color_off
                }
            };

            let slider_border_color = if self.hovered {
                if shared_state.toggled {
                    style.slider_border_color_on_hover.unwrap_or(
                        style.slider_border_color_on.unwrap_or(
                            style
                                .slider_border_color_off_hover
                                .unwrap_or(style.slider_border_color_off),
                        ),
                    )
                } else {
                    style
                        .slider_border_color_off_hover
                        .unwrap_or(style.slider_border_color_off)
                }
            } else {
                if shared_state.toggled {
                    style
                        .slider_border_color_on
                        .unwrap_or(style.slider_border_color_off)
                } else {
                    style.slider_border_color_off
                }
            };

            (
                bg_quad_bg,
                slider_quad_bg,
                bg_border_color,
                slider_border_color,
            )
        };

        let (bg_quad_style, slider_quad_style) = if shared_state.disabled {
            let (bg_quad_bg, slider_quad_bg, bg_border_color, slider_border_color) = get_colors();

            (
                QuadStyle {
                    bg: if shared_state.toggled {
                        style.on_bg_disabled.get(bg_quad_bg)
                    } else {
                        style.off_bg_disabled.get(bg_quad_bg)
                    },
                    border: BorderStyle {
                        color: if shared_state.toggled {
                            style.outer_border_color_on_disabled.get(bg_border_color)
                        } else {
                            style.outer_border_color_off_disabled.get(bg_border_color)
                        },
                        width: style.outer_border_width,
                        radius: style.rounding.into(),
                    },
                    flags: style.quad_flags,
                },
                QuadStyle {
                    bg: if shared_state.toggled {
                        style.slider_bg_on_disabled.get(slider_quad_bg)
                    } else {
                        style.slider_bg_off_disabled.get(slider_quad_bg)
                    },
                    border: BorderStyle {
                        color: if shared_state.toggled {
                            style
                                .slider_border_color_on_disabled
                                .get(slider_border_color)
                        } else {
                            style
                                .slider_border_color_off_disabled
                                .get(slider_border_color)
                        },
                        width: style.slider_border_width,
                        radius: style.rounding.into(),
                    },
                    flags: style.quad_flags,
                },
            )
        } else {
            let (bg_quad_bg, slider_quad_bg, bg_border_color, slider_border_color) = get_colors();

            (
                QuadStyle {
                    bg: bg_quad_bg,
                    border: BorderStyle {
                        color: bg_border_color,
                        width: style.outer_border_width,
                        radius: style.rounding.into(),
                    },
                    flags: style.quad_flags,
                },
                QuadStyle {
                    bg: slider_quad_bg,
                    border: BorderStyle {
                        color: slider_border_color,
                        width: if self.hovered {
                            style
                                .slider_border_width_hover
                                .unwrap_or(style.slider_border_width)
                        } else {
                            style.slider_border_width
                        },
                        radius: style.rounding.into(),
                    },
                    flags: style.quad_flags,
                },
            )
        };

        let bounds_rect = Rect::from_size(cx.bounds_size);

        let padding = style.slider_padding;
        let size = style.size;

        let bg_bounds = centered_rect(bounds_rect.center(), Size::new(size * 2.0, size));

        let slider_bounds = if shared_state.toggled {
            Rect::new(
                bg_bounds.origin + Vector::new(bg_bounds.width() - size + padding, padding),
                Size::new(size - (padding * 2.0), size - (padding * 2.0)),
            )
        } else {
            Rect::new(
                bg_bounds.origin + Vector::new(padding, padding),
                Size::new(size - (padding * 2.0), size - (padding * 2.0)),
            )
        };

        primitives.add(bg_quad_style.create_primitive(bg_bounds));
        primitives.set_z_index(1);
        primitives.add(slider_quad_style.create_primitive(slider_bounds));
    }
}

/// A handle to a [`SwitchElement`].
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
#[element_handle_set_tooltip]
pub struct Switch {
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    toggled: bool,
    disabled: bool,
    tooltip_inner: TooltipInner,
}

impl Switch {
    pub fn builder<A: Clone + 'static>() -> SwitchBuilder<A> {
        SwitchBuilder::default()
    }

    pub fn desired_size(&self, res: &mut ResourceCtx) -> Size {
        let size = res.style_system.get::<SwitchStyle>(self.el.class()).size;
        Size::new(size * 2.0, size)
    }

    /// Set the toggled state of this element.
    ///
    /// Returns `true` if the toggle state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_toggled(&mut self, toggled: bool) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.toggled != toggled {
            shared_state.toggled = toggled;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn toggled(&self) -> bool {
        RefCell::borrow(&self.shared_state).toggled
    }

    /// Set the disabled state of this element.
    ///
    /// Returns `true` if the disabled state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_disabled(&mut self, disabled: bool) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.disabled != disabled {
            shared_state.disabled = disabled;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// Layout out the element (with the top-left corner of the bounds set to `origin`).
    ///
    /// Returns `true` if the layout has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn layout(&mut self, origin: Point, res: &mut ResourceCtx) -> bool {
        let size = self.desired_size(res);
        self.el.set_rect(Rect::new(origin, size))
    }

    /// Layout out the element aligned to the given point.
    ///
    /// Returns `true` if the layout has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn layout_aligned(&mut self, point: Point, align: Align2, res: &mut ResourceCtx) -> bool {
        let size = self.desired_size(res);
        self.el.set_rect(align.align_rect_to_point(point, size))
    }
}
