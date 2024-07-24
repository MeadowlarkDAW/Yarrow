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
use crate::view::ScissorRectID;
use crate::window::WindowContext;
use crate::CursorIcon;

use super::label::{Label, LabelStyle};

/// The style of a [`RadioButton`] element
#[derive(Debug, Clone, PartialEq)]
pub struct RadioButtonStyle {
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

    pub dot_padding: f32,

    pub dot_bg_idle: Background,
    pub dot_bg_hover: Background,
    pub dot_bg_disabled: Background,
}

impl Default for RadioButtonStyle {
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

            dot_padding: 6.0,

            dot_bg_idle: Background::Solid(RGBA8::new(255, 255, 255, 180)),
            dot_bg_hover: Background::Solid(RGBA8::new(255, 255, 255, 225)),
            dot_bg_disabled: Background::Solid(RGBA8::new(255, 255, 255, 100)),
        }
    }
}

pub struct RadioButtonBuilder<A: Clone + 'static> {
    pub action: Option<A>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub toggled: bool,
    pub style: Rc<RadioButtonStyle>,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub disabled: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> RadioButtonBuilder<A> {
    pub fn new(style: &Rc<RadioButtonStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            toggled: false,
            style: Rc::clone(style),
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            disabled: false,
            scissor_rect_id: None,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> RadioButton {
        RadioButtonElement::create(self, cx)
    }

    pub fn on_toggled_on(mut self, action: A) -> Self {
        self.action = Some(action);
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

pub struct RadioButtonElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<A>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
    hovered: bool,
}

impl<A: Clone + 'static> RadioButtonElement<A> {
    pub fn create(builder: RadioButtonBuilder<A>, cx: &mut WindowContext<'_, A>) -> RadioButton {
        let RadioButtonBuilder {
            action,
            tooltip_message,
            tooltip_align,
            toggled,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            disabled,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id) = cx.z_index_and_scissor_rect_id(z_index, scissor_rect_id);

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

        let bounds_rect = Rect::from_size(cx.bounds_size);
        let size = shared_state.style.size;

        let bg_bounds = layout::centered_rect(bounds_rect.center(), Size::new(size, size));

        primitives.add(bg_quad_style.create_primitive(bg_bounds));

        if shared_state.toggled {
            let dot_quad_style = if shared_state.disabled {
                QuadStyle {
                    bg: shared_state.style.dot_bg_disabled.clone(),
                    border: BorderStyle {
                        radius: shared_state.style.rounding.into(),
                        ..Default::default()
                    },
                }
            } else if self.hovered {
                QuadStyle {
                    bg: shared_state.style.dot_bg_hover.clone(),
                    border: BorderStyle {
                        radius: shared_state.style.rounding.into(),
                        ..Default::default()
                    },
                }
            } else {
                QuadStyle {
                    bg: shared_state.style.dot_bg_idle.clone(),
                    border: BorderStyle {
                        radius: shared_state.style.rounding.into(),
                        ..Default::default()
                    },
                }
            };

            let padding = shared_state.style.dot_padding;

            let dot_bounds = Rect::new(
                bg_bounds.origin + Point::new(padding, padding).to_vector(),
                Size::new(size - (padding * 2.0), size - (padding * 2.0)),
            );

            primitives.set_z_index(1);
            primitives.add(dot_quad_style.create_primitive(dot_bounds));
        }
    }
}

/// A handle to a [`RadioButtonElement`].
pub struct RadioButton {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    toggled: bool,
    style: Rc<RadioButtonStyle>,
    disabled: bool,
}

impl RadioButton {
    pub fn builder<A: Clone + 'static>(style: &Rc<RadioButtonStyle>) -> RadioButtonBuilder<A> {
        RadioButtonBuilder::new(style)
    }

    pub fn min_size(&self) -> Size {
        let size = RefCell::borrow(&self.shared_state).style.size;
        Size::new(size * 2.0, size)
    }

    pub fn set_style(&mut self, style: &Rc<RadioButtonStyle>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<RadioButtonStyle> {
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

    pub fn layout(&mut self, origin: Point) {
        let size = self.min_size();
        self.el.set_rect(Rect::new(origin, size));
    }

    pub fn layout_aligned(&mut self, point: Point, align: Align2) {
        let size = self.min_size();
        self.el.set_rect(align.align_rect_to_point(point, size));
    }
}

// TODO: Different alignment options.
/// A helper struct to make a group of radio buttons with labels.
pub struct RadioButtonGroup {
    rows: Vec<(RadioButton, Label)>,
    selected_index: usize,
    btn_size: f32,
    bounds: Rect,
}

impl RadioButtonGroup {
    pub fn new<A: Clone + 'static, F>(
        options: impl IntoIterator<Item = impl Into<String>>,
        selected_index: usize,
        mut on_selected: F,
        label_style: &Rc<LabelStyle>,
        radio_btn_style: &Rc<RadioButtonStyle>,
        z_index: Option<ZIndex>,
        scissor_rect_id: Option<ScissorRectID>,
        cx: &mut WindowContext<A>,
    ) -> Self
    where
        F: FnMut(usize) -> A + 'static,
    {
        let (z_index, scissor_rect_id) = cx.z_index_and_scissor_rect_id(z_index, scissor_rect_id);

        let rows: Vec<(RadioButton, Label)> = options
            .into_iter()
            .enumerate()
            .map(|(i, option)| {
                (
                    RadioButton::builder(radio_btn_style)
                        .on_toggled_on((on_selected)(i))
                        .toggled(i == selected_index)
                        .z_index(z_index)
                        .scissor_rect(scissor_rect_id)
                        .build(cx),
                    Label::builder(label_style)
                        .text(option.into())
                        .z_index(z_index)
                        .scissor_rect(scissor_rect_id)
                        .build(cx),
                )
            })
            .collect();

        Self {
            rows,
            selected_index,
            btn_size: radio_btn_style.size,
            bounds: Rect::default(),
        }
    }

    pub fn layout(
        &mut self,
        origin: Point,
        row_padding: f32,
        column_padding: f32,
        max_width: Option<f32>,
        text_offset: Point,
    ) {
        self.bounds.origin = origin;

        if self.rows.is_empty() {
            self.bounds.size = Size::default();
            return;
        }

        let mut y = origin.y;
        let mut max_row_width: f32 = 0.0;

        for (radio_btn, label) in self.rows.iter_mut() {
            let label_size = label.desired_padded_size();
            let mut label_width = label_size.width;
            let mut row_width = self.btn_size + column_padding + label_size.width;

            if let Some(max_width) = max_width {
                if row_width > max_width {
                    row_width = max_width;
                    label_width = max_width - self.btn_size - column_padding;
                }
            }

            max_row_width = max_row_width.max(row_width);

            let row_height = label_size.height.max(self.btn_size);

            radio_btn.el.set_rect(Rect::new(
                Point::new(origin.x, y + ((row_height - self.btn_size) * 0.5)),
                Size::new(self.btn_size, self.btn_size),
            ));

            label.set_text_offset(text_offset);
            label.el.set_rect(Rect::new(
                Point::new(origin.x + self.btn_size + column_padding, y),
                Size::new(label_width, row_height),
            ));

            y += row_height + row_padding;
        }

        self.bounds.size.height = (self.btn_size * self.rows.len() as f32)
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
            label.el.set_hidden(hidden);
        }
    }
}
