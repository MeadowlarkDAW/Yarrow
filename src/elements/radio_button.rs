use derive_where::derive_where;
use std::cell::RefCell;
use std::rc::Rc;

use crate::derive::*;
use crate::prelude::*;

use super::label::Label;

/// The style of a [`RadioButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct RadioButtonStyle {
    pub size: f32,
    pub radius: Radius,

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

    pub dot_padding: f32,

    pub dot_bg: Background,
    pub dot_bg_hover: Option<Background>,
    pub dot_bg_disabled: DisabledBackground,

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

impl Default for RadioButtonStyle {
    fn default() -> Self {
        Self {
            size: 20.0,
            radius: 20.0.into(),
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
            dot_padding: 6.0,
            dot_bg: Background::TRANSPARENT,
            dot_bg_hover: None,
            dot_bg_disabled: Default::default(),
            cursor_icon: None,
            quad_flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        }
    }
}

impl ElementStyle for RadioButtonStyle {
    const ID: &'static str = "rdbtn";

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
pub struct RadioButtonBuilder<A: Clone + 'static> {
    pub action: Option<A>,
    pub toggled: bool,
}

impl<A: Clone + 'static> RadioButtonBuilder<A> {
    pub fn build(self, cx: &mut WindowContext<'_, A>) -> RadioButton {
        RadioButtonElement::create(self, cx)
    }

    pub fn on_toggled_on(mut self, action: A) -> Self {
        self.action = Some(action);
        self
    }

    pub const fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }
}

pub struct RadioButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<A>,
    hovered: bool,
    cursor_icon: Option<CursorIcon>,
}

impl<A: Clone + 'static> RadioButtonElement<A> {
    pub fn create(builder: RadioButtonBuilder<A>, cx: &mut WindowContext<'_, A>) -> RadioButton {
        let RadioButtonBuilder {
            action,
            tooltip_data,
            toggled,
            class,
            z_index,
            rect,
            manually_hidden,
            disabled,
            scissor_rect,
        } = builder;

        let (z_index, scissor_rect, class) = cx.builder_values(z_index, scissor_rect, class);
        let style = cx.res.style_system.get::<RadioButtonStyle>(class);
        let cursor_icon = style.cursor_icon;

        let shared_state = Rc::new(RefCell::new(SharedState {
            toggled,
            disabled,
            tooltip_inner: TooltipInner::new(tooltip_data),
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                action,
                hovered: false,
                cursor_icon,
            }),
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        RadioButton { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for RadioButtonElement<A> {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
    }

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
                let style = cx.res.style_system.get::<RadioButtonStyle>(cx.class());
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
                    if !shared_state.toggled {
                        shared_state.toggled = true;

                        if let Some(action) = &self.action {
                            cx.send_action(action.clone()).unwrap();
                        }

                        cx.request_repaint();
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

        let style = cx.res.style_system.get::<RadioButtonStyle>(cx.class);

        let bg_quad_style = if shared_state.disabled {
            if shared_state.toggled {
                QuadStyle {
                    bg: style
                        .on_bg_disabled
                        .get(style.on_bg.unwrap_or(style.off_bg)),
                    border: BorderStyle {
                        color: style.outer_border_color_on_disabled.get(
                            style
                                .outer_border_color_on
                                .unwrap_or(style.outer_border_color_off),
                        ),
                        width: style.outer_border_width,
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                }
            } else {
                QuadStyle {
                    bg: style.off_bg_disabled.get(style.off_bg),
                    border: BorderStyle {
                        color: style
                            .outer_border_color_off_disabled
                            .get(style.outer_border_color_off),
                        width: style.outer_border_width,
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                }
            }
        } else if shared_state.toggled {
            if self.hovered {
                QuadStyle {
                    bg: style
                        .on_bg_hover
                        .unwrap_or(style.on_bg.unwrap_or(style.off_bg)),
                    border: BorderStyle {
                        color: style.outer_border_color_on_hover.unwrap_or(
                            style
                                .outer_border_color_on
                                .unwrap_or(style.outer_border_color_off),
                        ),
                        width: style.outer_border_width,
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                }
            } else {
                QuadStyle {
                    bg: style.on_bg.unwrap_or(style.off_bg),
                    border: BorderStyle {
                        color: style
                            .outer_border_color_on
                            .unwrap_or(style.outer_border_color_off),
                        width: style.outer_border_width,
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                }
            }
        } else {
            if self.hovered {
                QuadStyle {
                    bg: style.off_bg_hover.unwrap_or(style.off_bg),
                    border: BorderStyle {
                        color: style
                            .outer_border_color_off_hover
                            .unwrap_or(style.outer_border_color_off),
                        width: style.outer_border_width,
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                }
            } else {
                QuadStyle {
                    bg: style.off_bg,
                    border: BorderStyle {
                        color: style.outer_border_color_off,
                        width: style.outer_border_width,
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                }
            }
        };

        let bounds_rect = Rect::from_size(cx.bounds_size);
        let size = style.size;

        let bg_bounds = centered_rect(bounds_rect.center(), Size::new(size, size));

        primitives.add(bg_quad_style.create_primitive(bg_bounds));

        if shared_state.toggled {
            let quad_style = if shared_state.disabled {
                QuadStyle {
                    bg: style.dot_bg_disabled.get(style.dot_bg),
                    border: BorderStyle {
                        radius: style.radius,
                        ..Default::default()
                    },
                    flags: style.quad_flags,
                }
            } else {
                QuadStyle {
                    bg: style.dot_bg,
                    border: BorderStyle {
                        radius: style.radius,
                        ..Default::default()
                    },
                    flags: style.quad_flags,
                }
            };

            let padding = style.dot_padding;

            let dot_bounds = Rect::new(
                bg_bounds.origin + Point::new(padding, padding).to_vector(),
                Size::new(size - (padding * 2.0), size - (padding * 2.0)),
            );

            primitives.set_z_index(1);
            primitives.add(quad_style.create_primitive(dot_bounds));
        }
    }
}

/// A handle to a [`RadioButtonElement`].
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
#[element_handle_set_tooltip]
pub struct RadioButton {
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    toggled: bool,
    disabled: bool,
    tooltip_inner: TooltipInner,
}

impl RadioButton {
    pub fn builder<A: Clone + 'static>() -> RadioButtonBuilder<A> {
        RadioButtonBuilder::default()
    }

    pub fn desired_size(&self, res: &mut ResourceCtx) -> Size {
        let size = res
            .style_system
            .get::<RadioButtonStyle>(self.el.class())
            .size;
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

// TODO: Different alignment options.
/// A helper struct to make a group of radio buttons with labels.
pub struct RadioButtonGroup {
    rows: Vec<(RadioButton, Label)>,
    selected_index: usize,
    bounds: Rect,
}

impl RadioButtonGroup {
    pub fn new<A: Clone + 'static, F>(
        options: impl IntoIterator<Item = impl Into<String>>,
        selected_index: usize,
        mut on_selected: F,
        label_class: Option<ClassID>,
        radio_btn_class: Option<ClassID>,
        z_index: Option<ZIndex>,
        scissor_rect: Option<ScissorRectID>,
        cx: &mut WindowContext<A>,
    ) -> Self
    where
        F: FnMut(usize) -> A + 'static,
    {
        let z_index = z_index.unwrap_or_else(|| cx.z_index());
        let scissor_rect = scissor_rect.unwrap_or_else(|| cx.scissor_rect());

        let label_class = label_class.unwrap_or_else(|| cx.class());
        let radio_btn_class = radio_btn_class.unwrap_or_else(|| cx.class());

        let rows: Vec<(RadioButton, Label)> = options
            .into_iter()
            .enumerate()
            .map(|(i, option)| {
                (
                    RadioButton::builder()
                        .on_toggled_on((on_selected)(i))
                        .toggled(i == selected_index)
                        .class(radio_btn_class)
                        .z_index(z_index)
                        .scissor_rect(scissor_rect)
                        .build(cx),
                    Label::builder()
                        .text(option.into())
                        .class(label_class)
                        .z_index(z_index)
                        .scissor_rect(scissor_rect)
                        .build(cx),
                )
            })
            .collect();

        Self {
            rows,
            selected_index,
            bounds: Rect::default(),
        }
    }

    pub fn layout(
        &mut self,
        origin: Point,
        row_padding: f32,
        column_padding: f32,
        max_width: Option<f32>,
        text_offset: Vector,
        res: &mut ResourceCtx,
    ) {
        self.bounds.origin = origin;

        if self.rows.is_empty() {
            self.bounds.size = Size::default();
            return;
        }

        let mut y = origin.y;
        let mut max_row_width: f32 = 0.0;

        let mut btn_size = None;

        for (radio_btn, label) in self.rows.iter_mut() {
            if btn_size.is_none() {
                btn_size = Some(radio_btn.desired_size(res));
            }

            let label_size = label.desired_size(res);
            let mut label_width = label_size.width;
            let mut row_width = btn_size.unwrap().width + column_padding + label_size.width;

            if let Some(max_width) = max_width {
                if row_width > max_width {
                    row_width = max_width;
                    label_width = max_width - btn_size.unwrap().width - column_padding;
                }
            }

            max_row_width = max_row_width.max(row_width);

            let row_height = label_size.height.max(btn_size.unwrap().height);

            radio_btn.el.set_rect(Rect::new(
                Point::new(
                    origin.x,
                    y + ((row_height - btn_size.unwrap().height) * 0.5),
                ),
                btn_size.unwrap(),
            ));

            label.set_text_offset(text_offset);
            label.set_rect(Rect::new(
                Point::new(origin.x + btn_size.unwrap().width + column_padding, y),
                Size::new(label_width, row_height),
            ));

            y += row_height + row_padding;
        }

        self.bounds.size.height = (btn_size.map(|s| s.height).unwrap_or_default()
            * self.rows.len() as f32)
            + (row_padding * (self.rows.len() as f32 - 1.0));
        self.bounds.size.width = max_row_width;
    }

    pub fn updated_selected(&mut self, selected_index: usize) {
        let selected_index = if selected_index >= self.rows.len() {
            0
        } else {
            selected_index
        };

        if self.selected_index == selected_index {
            return;
        }

        if let Some((prev_selected_btn, _)) = self.rows.get_mut(self.selected_index) {
            prev_selected_btn.set_toggled(false);
        }

        self.selected_index = selected_index;

        self.rows[selected_index].0.set_toggled(true);
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        for (radio_btn, label) in self.rows.iter_mut() {
            radio_btn.el.set_hidden(hidden);
            label.set_hidden(hidden);
        }
    }
}
