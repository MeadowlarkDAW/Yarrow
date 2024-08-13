use std::cell::RefCell;
use std::rc::Rc;

use rootvg::color::{self, RGBA8};
use rootvg::math::Point;
use rootvg::quad::SolidQuadBuilder;
use rootvg::PrimitiveGroup;

use crate::prelude::ElementStyle;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::math::{Rect, Size, ZIndex};
use crate::style::ClassID;
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;
use crate::CursorIcon;

// TODO: Make this configurable?
const DRAG_HANDLE_WIDTH: f32 = 5.0;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ResizeDirection {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}

/// The style of a [`ResizeHandle`] element
#[derive(Debug, Clone, PartialEq)]
pub struct ResizeHandleStyle {
    pub drag_handle_width: f32,
    pub drag_handle_color: RGBA8,
    pub drag_handle_width_hover: Option<f32>,
    pub drag_handle_color_hover: Option<RGBA8>,
    pub edge_padding_start: f32,
    pub edge_padding_end: f32,
}

impl Default for ResizeHandleStyle {
    fn default() -> Self {
        Self {
            drag_handle_width: 0.0,
            drag_handle_color: color::TRANSPARENT,
            drag_handle_width_hover: None,
            drag_handle_color_hover: None,
            edge_padding_start: 0.0,
            edge_padding_end: 0.0,
        }
    }
}

impl ElementStyle for ResizeHandleStyle {
    const ID: &'static str = "rszhndl";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self::default()
    }
}

/// The style of a [`ResizeHandle`] element
#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResizeHandleLayout {
    pub anchor: Point,
    pub length: f32,
}

impl ResizeHandleLayout {
    fn resize_bounds(&self, direction: ResizeDirection, current_span: f32) -> Rect {
        if self.length <= 0.0 {
            return Rect::zero();
        }

        match direction {
            ResizeDirection::Left => Rect::new(
                Point::new(self.anchor.x - current_span, self.anchor.y),
                Size::new(current_span, self.length),
            ),
            ResizeDirection::Right => Rect::new(self.anchor, Size::new(current_span, self.length)),
            ResizeDirection::Top => Rect::new(
                Point::new(self.anchor.x, self.anchor.y - current_span),
                Size::new(self.length, current_span),
            ),
            ResizeDirection::Bottom => Rect::new(self.anchor, Size::new(self.length, current_span)),
        }
    }
}

pub struct ResizeHandleBuilder<A: Clone + 'static> {
    pub resized_action: Option<Box<dyn FnMut(f32) -> A>>,
    pub resize_finished_action: Option<Box<dyn FnMut(f32) -> A>>,
    pub direction: ResizeDirection,
    pub min_span: f32,
    pub max_span: f32,
    pub default_span: f32,
    pub current_span: f32,
    pub layout: Option<ResizeHandleLayout>,
    pub class: Option<ClassID>,
    pub z_index: Option<ZIndex>,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
    pub disabled: bool,
}

impl<A: Clone + 'static> ResizeHandleBuilder<A> {
    pub fn new() -> Self {
        Self {
            resized_action: None,
            resize_finished_action: None,
            direction: ResizeDirection::default(),
            min_span: 150.0,
            max_span: 500.0,
            default_span: 200.0,
            current_span: 200.0,
            class: None,
            z_index: None,
            layout: None,
            manually_hidden: false,
            scissor_rect_id: None,
            disabled: false,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> ResizeHandle {
        ResizeHandleElement::create(self, cx)
    }

    pub fn on_resized<F: FnMut(f32) -> A + 'static>(mut self, f: F) -> Self {
        self.resized_action = Some(Box::new(f));
        self
    }

    pub fn on_resize_finished<F: FnMut(f32) -> A + 'static>(mut self, f: F) -> Self {
        self.resize_finished_action = Some(Box::new(f));
        self
    }

    pub const fn direction(mut self, direction: ResizeDirection) -> Self {
        self.direction = direction;
        self
    }

    pub const fn min_span(mut self, min_span: f32) -> Self {
        self.min_span = min_span;
        self
    }

    pub const fn max_span(mut self, max_span: f32) -> Self {
        self.max_span = max_span;
        self
    }

    pub const fn default_span(mut self, default_span: f32) -> Self {
        self.default_span = default_span;
        self
    }

    pub const fn current_span(mut self, current_span: f32) -> Self {
        self.current_span = current_span;
        self
    }

    pub const fn layout(mut self, layout: ResizeHandleLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    /// The style class ID
    ///
    /// If this method is not used, then the current class from the window context will
    /// be used.
    pub const fn class(mut self, class: ClassID) -> Self {
        self.class = Some(class);
        self
    }

    /// The z index of the element
    ///
    /// If this method is not used, then the current z index from the window context will
    /// be used.
    pub const fn z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = Some(z_index);
        self
    }

    /// Whether or not this element is manually hidden
    ///
    /// By default this is set to `false`.
    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.manually_hidden = hidden;
        self
    }

    /// Whether or not this element is in the disabled state
    ///
    /// By default this is set to `false`.
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The ID of the scissoring rectangle this element belongs to.
    ///
    /// If this method is not used, then the current scissoring rectangle ID from the
    /// window context will be used.
    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = Some(scissor_rect_id);
        self
    }
}

struct DragState {
    drag_start_pos: Point,
    drag_start_span: f32,
}

pub struct ResizeHandleElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,

    resized_action: Option<Box<dyn FnMut(f32) -> A>>,
    resize_finished_action: Option<Box<dyn FnMut(f32) -> A>>,

    direction: ResizeDirection,
    min_span: f32,
    max_span: f32,
    default_span: f32,

    drag_state: Option<DragState>,
    queued_resize_finished_span: Option<f32>,
    show_drag_handle: bool,
}

impl<A: Clone + 'static> ResizeHandleElement<A> {
    pub fn create(builder: ResizeHandleBuilder<A>, cx: &mut WindowContext<'_, A>) -> ResizeHandle {
        let ResizeHandleBuilder {
            resized_action,
            resize_finished_action,
            direction,
            min_span,
            max_span,
            default_span,
            current_span,
            layout,
            class,
            z_index,
            manually_hidden,
            scissor_rect_id,
            disabled,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let max_span = if max_span < min_span {
            min_span
        } else {
            max_span
        };
        let default_span = default_span.clamp(min_span, max_span);
        let current_span = current_span.clamp(min_span, max_span);

        let layout = layout.unwrap_or_default();
        let resize_bounds = layout.resize_bounds(direction, current_span);
        let rect = calc_drag_handle_rect(resize_bounds, direction, DRAG_HANDLE_WIDTH);

        let shared_state = Rc::new(RefCell::new(SharedState {
            layout,
            current_span,
            resized_by_handle: false,
            disabled,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                resized_action,
                resize_finished_action,
                direction,
                min_span,
                max_span,
                default_span,
                drag_state: None,
                queued_resize_finished_span: None,
                show_drag_handle: false,
            }),
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        ResizeHandle {
            el,
            shared_state,
            layout,
            direction,
            min_span,
            max_span,
        }
    }
}

impl<A: Clone + 'static> Element<A> for ResizeHandleElement<A> {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS
            | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
            | ElementFlags::LISTENS_TO_POINTER_OUTSIDE_BOUNDS_WHEN_FOCUSED
            | ElementFlags::LISTENS_TO_FOCUS_CHANGE
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        match event {
            ElementEvent::CustomStateChanged => {
                let resize_bounds = shared_state
                    .layout
                    .resize_bounds(self.direction, shared_state.current_span);
                let rect = calc_drag_handle_rect(resize_bounds, self.direction, DRAG_HANDLE_WIDTH);

                cx.set_rect(rect);
                cx.request_repaint();

                if shared_state.resized_by_handle {
                    shared_state.resized_by_handle = false;

                    self.queued_resize_finished_span = None;
                    self.drag_state = None;
                    self.show_drag_handle = false;
                }
            }
            ElementEvent::Pointer(PointerEvent::Moved { position, .. }) => {
                if shared_state.disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                let pointer_hovered = cx.rect().contains(position);

                if pointer_hovered || self.drag_state.is_some() {
                    cx.cursor_icon = match self.direction {
                        ResizeDirection::Left | ResizeDirection::Right => CursorIcon::ColResize,
                        ResizeDirection::Top | ResizeDirection::Bottom => CursorIcon::RowResize,
                    };
                }

                if let Some(drag_state) = &mut self.drag_state {
                    let delta = match self.direction {
                        ResizeDirection::Left | ResizeDirection::Right => {
                            position.x - drag_state.drag_start_pos.x
                        }
                        ResizeDirection::Top | ResizeDirection::Bottom => {
                            position.y - drag_state.drag_start_pos.y
                        }
                    };

                    let new_span =
                        (drag_state.drag_start_span + delta).clamp(self.min_span, self.max_span);

                    if shared_state.current_span != new_span {
                        shared_state.current_span = new_span;

                        let resize_bounds = shared_state
                            .layout
                            .resize_bounds(self.direction, shared_state.current_span);
                        let rect =
                            calc_drag_handle_rect(resize_bounds, self.direction, DRAG_HANDLE_WIDTH);

                        cx.set_rect(rect);
                        cx.request_repaint();

                        self.queued_resize_finished_span = Some(new_span);

                        if let Some(f) = &mut self.resized_action {
                            cx.send_action((f)(new_span)).unwrap();
                        }
                    }
                } else if pointer_hovered {
                    cx.start_hover_timeout();
                } else if self.show_drag_handle {
                    self.show_drag_handle = false;
                    cx.request_repaint();
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                if self.drag_state.is_none() && self.show_drag_handle {
                    self.show_drag_handle = false;
                    cx.request_repaint();
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed {
                position, button, ..
            }) => {
                if cx.rect().contains(position)
                    && button == PointerButton::Primary
                    && !shared_state.disabled
                {
                    let current_span = shared_state.current_span;

                    self.drag_state = Some(DragState {
                        drag_start_pos: position,
                        drag_start_span: current_span,
                    });

                    cx.steal_temporary_focus();

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                button,
                click_count,
                position,
                ..
            }) => {
                if shared_state.disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if button == PointerButton::Primary {
                    self.drag_state = None;
                    cx.release_focus();

                    if click_count == 2 && cx.rect().contains(position) {
                        if shared_state.current_span != self.default_span {
                            shared_state.current_span = self.default_span;

                            let resize_bounds = shared_state
                                .layout
                                .resize_bounds(self.direction, shared_state.current_span);
                            let rect = calc_drag_handle_rect(
                                resize_bounds,
                                self.direction,
                                DRAG_HANDLE_WIDTH,
                            );

                            cx.set_rect(rect);
                            cx.request_repaint();

                            if let Some(f) = &mut self.resized_action {
                                cx.send_action((f)(self.default_span)).unwrap();
                            }

                            self.queued_resize_finished_span = None;
                            if let Some(f) = &mut self.resize_finished_action {
                                cx.send_action((f)(self.default_span)).unwrap();
                            }
                        }
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::HoverTimeout { position }) => {
                if shared_state.disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if !self.show_drag_handle && cx.rect().contains(position) {
                    self.show_drag_handle = true;
                    cx.request_repaint();
                }
            }
            ElementEvent::Focus(false) => {
                cx.cursor_icon = CursorIcon::Default;

                self.drag_state = None;
                self.show_drag_handle = false;
                cx.request_repaint();

                if let Some(span) = self.queued_resize_finished_span.take() {
                    if let Some(f) = &mut self.resize_finished_action {
                        cx.send_action((f)(span)).unwrap();
                    }
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let bounds_rect = Rect::new(Point::zero(), cx.bounds_size);

        let style = cx.res.style_system.get::<ResizeHandleStyle>(cx.class);

        struct DragHandleDrawOpts {
            width: f32,
            color: RGBA8,
        }

        let handle_opts = if self.show_drag_handle || self.drag_state.is_some() {
            let color = style
                .drag_handle_color_hover
                .unwrap_or(style.drag_handle_color);
            let width = style
                .drag_handle_width_hover
                .unwrap_or(style.drag_handle_width);

            if color != color::TRANSPARENT && width > 0.0 {
                Some(DragHandleDrawOpts { width, color })
            } else {
                None
            }
        } else if style.drag_handle_color != color::TRANSPARENT && style.drag_handle_width > 0.0 {
            Some(DragHandleDrawOpts {
                width: style.drag_handle_width,
                color: style.drag_handle_color,
            })
        } else {
            None
        };

        if let Some(handle_opts) = handle_opts {
            let handle_rect = calc_drag_handle_rect(bounds_rect, self.direction, handle_opts.width);

            primitives.add_solid_quad(
                SolidQuadBuilder::new(handle_rect.size)
                    .bg_color(handle_opts.color)
                    .position(handle_rect.origin),
            );
        }
    }
}

/// A simple element with a quad background.
pub struct ResizeHandle {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
    // Keep a copy here so the pointer doesn't need to be dereferenced when
    // diffing.
    layout: ResizeHandleLayout,
    direction: ResizeDirection,
    min_span: f32,
    max_span: f32,
}

impl ResizeHandle {
    pub fn builder<A: Clone + 'static>() -> ResizeHandleBuilder<A> {
        ResizeHandleBuilder::new()
    }

    /// Set the class of the element.
    ///
    /// Returns `true` if the class has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// and the class ID is cached in the handle itself, so this is very
    /// cheap to call frequently.
    pub fn set_class(&mut self, class: ClassID) -> bool {
        if self.el.class() != class {
            self.el._notify_class_change(class);
            true
        } else {
            false
        }
    }

    /// Set the layout.
    ///
    /// Returns `true` if the layout has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_layout(&mut self, layout: ResizeHandleLayout) -> bool {
        if self.layout != layout {
            self.layout = layout;

            RefCell::borrow_mut(&self.shared_state).layout = layout;

            self.el._notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn layout(&self) -> &ResizeHandleLayout {
        &self.layout
    }

    /// Set the span.
    ///
    /// Returns `true` if the span has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_span(&mut self, span: f32) -> bool {
        let span = span.clamp(self.min_span, self.max_span);

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        if shared_state.current_span != span {
            shared_state.current_span = span;
            shared_state.resized_by_handle = true;
            self.el._notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn current_span(&self) -> f32 {
        RefCell::borrow(&self.shared_state).current_span
    }

    pub fn min_span(&self) -> f32 {
        self.min_span
    }

    pub fn max_span(&self) -> f32 {
        self.max_span
    }

    pub fn direction(&self) -> ResizeDirection {
        self.direction
    }

    pub fn disabled(&self) -> bool {
        RefCell::borrow(&self.shared_state).disabled
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
            self.el._notify_custom_state_change();
            true
        } else {
            false
        }
    }
}

struct SharedState {
    layout: ResizeHandleLayout,
    current_span: f32,
    resized_by_handle: bool,
    disabled: bool,
}

fn calc_drag_handle_rect(bounds: Rect, direction: ResizeDirection, handle_width: f32) -> Rect {
    if bounds.size.is_empty() {
        return Rect::zero();
    }

    match direction {
        ResizeDirection::Left => Rect::new(bounds.origin, Size::new(handle_width, bounds.height())),
        ResizeDirection::Right => Rect::new(
            Point::new(
                bounds.origin.x + bounds.width() - handle_width,
                bounds.origin.y,
            ),
            Size::new(handle_width, bounds.height()),
        ),
        ResizeDirection::Top => Rect::new(bounds.origin, Size::new(bounds.width(), handle_width)),
        ResizeDirection::Bottom => Rect::new(
            Point::new(
                bounds.origin.x,
                bounds.origin.y + bounds.height() - handle_width,
            ),
            Size::new(bounds.width(), handle_width),
        ),
    }
}
