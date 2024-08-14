use std::cell::RefCell;
use std::rc::Rc;

use rootvg::color::{self, RGBA8};
use rootvg::math::{Point, Rect, Size, Vector, ZIndex};
use rootvg::quad::{QuadFlags, Radius};
use rootvg::PrimitiveGroup;

use crate::prelude::ElementStyle;
use crate::style::{Background, BorderStyle, ClassID, QuadStyle};

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;

/// The style of a scroll bar in a [`ScrollArea`] element.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollBarStyle {
    pub back_quad_bg: Background,
    pub back_quad_bg_content_hover: Option<Background>,
    pub back_quad_bg_slider_hover: Option<Background>,

    pub back_quad_border_color: RGBA8,
    pub back_quad_border_color_content_hover: Option<RGBA8>,
    pub back_quad_border_color_slider_hover: Option<RGBA8>,

    pub back_quad_border_width: f32,
    pub back_quad_border_width_content_hover: Option<f32>,
    pub back_quad_border_width_slider_hover: Option<f32>,

    pub slider_bg: Background,
    pub slider_bg_content_hover: Option<Background>,
    pub slider_bg_slider_hover: Option<Background>,
    pub slider_bg_slider_dragging: Option<Background>,

    pub slider_border_color: RGBA8,
    pub slider_border_color_content_hover: Option<RGBA8>,
    pub slider_border_color_slider_hover: Option<RGBA8>,
    pub slider_border_color_slider_dragging: Option<RGBA8>,

    pub slider_border_width: f32,
    pub slider_border_width_content_hover: Option<f32>,
    pub slider_border_width_slider_hover: Option<f32>,
    pub slider_border_width_slider_dragging: Option<f32>,

    pub slider_width: f32,
    pub radius: Radius,

    /// Additional flags for the quad primitives.
    ///
    /// By default this is set to `QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL`.
    pub quad_flags: QuadFlags,
}

impl Default for ScrollBarStyle {
    fn default() -> Self {
        Self {
            back_quad_bg: Background::TRANSPARENT,
            back_quad_bg_content_hover: None,
            back_quad_bg_slider_hover: None,
            back_quad_border_color: color::TRANSPARENT,
            back_quad_border_color_content_hover: None,
            back_quad_border_color_slider_hover: None,
            back_quad_border_width: 0.0,
            back_quad_border_width_content_hover: None,
            back_quad_border_width_slider_hover: None,
            slider_bg: Background::TRANSPARENT,
            slider_bg_content_hover: None,
            slider_bg_slider_hover: None,
            slider_bg_slider_dragging: None,
            slider_border_color: color::TRANSPARENT,
            slider_border_color_content_hover: None,
            slider_border_color_slider_hover: None,
            slider_border_color_slider_dragging: None,
            slider_border_width: 0.0,
            slider_border_width_content_hover: None,
            slider_border_width_slider_hover: None,
            slider_border_width_slider_dragging: None,
            slider_width: 8.0,
            radius: Radius::default(),
            quad_flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        }
    }
}

impl ElementStyle for ScrollBarStyle {
    const ID: &'static str = "scrlbar";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self::default()
    }
}

pub struct ScrollAreaBuilder<A: Clone + 'static> {
    pub scrolled_action: Option<Box<dyn FnMut(Vector) -> A>>,
    pub bounds: Rect,
    pub content_size: Size,
    pub scroll_offset: Vector,
    pub scroll_horizontally: bool,
    pub scroll_vertically: bool,
    pub scroll_with_scroll_wheel: bool,
    pub show_slider_when_content_fits: bool,
    pub capture_scroll_wheel: bool,
    pub points_per_line: f32,
    pub class: Option<ClassID>,
    pub z_index: Option<ZIndex>,
    pub manually_hidden: bool,
    pub disabled: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> ScrollAreaBuilder<A> {
    pub fn new() -> Self {
        Self {
            scrolled_action: None,
            bounds: Rect::default(),
            content_size: Size::default(),
            scroll_offset: Vector::default(),
            scroll_horizontally: true,
            scroll_vertically: true,
            scroll_with_scroll_wheel: true,
            show_slider_when_content_fits: false,
            capture_scroll_wheel: true,
            points_per_line: 24.0,
            class: None,
            z_index: None,
            manually_hidden: false,
            disabled: false,
            scissor_rect_id: None,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> ScrollArea {
        ScrollAreaElement::create(self, cx)
    }

    pub fn on_scrolled<F: FnMut(Vector) -> A + 'static>(mut self, f: F) -> Self {
        self.scrolled_action = Some(Box::new(f));
        self
    }

    pub const fn bounds(mut self, bounds: Rect) -> Self {
        self.bounds = bounds;
        self
    }

    pub const fn content_size(mut self, size: Size) -> Self {
        self.content_size = size;
        self
    }

    pub const fn scroll_offset(mut self, offset: Vector) -> Self {
        self.scroll_offset = offset;
        self
    }

    pub const fn scroll_horizontally(mut self, do_scroll: bool) -> Self {
        self.scroll_horizontally = do_scroll;
        self
    }

    pub const fn scroll_vertically(mut self, do_scroll: bool) -> Self {
        self.scroll_vertically = do_scroll;
        self
    }

    pub const fn scroll_with_scroll_wheel(mut self, do_scroll: bool) -> Self {
        self.scroll_with_scroll_wheel = do_scroll;
        self
    }

    pub const fn show_slider_when_content_fits(mut self, do_show: bool) -> Self {
        self.show_slider_when_content_fits = do_show;
        self
    }

    pub const fn capture_scroll_wheel(mut self, do_capture: bool) -> Self {
        self.capture_scroll_wheel = do_capture;
        self
    }

    pub const fn points_per_line(mut self, points_per_line: f32) -> Self {
        self.points_per_line = points_per_line;
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
    drag_start_scroll_offset: Vector,
}

pub struct ScrollAreaElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,

    scrolled_action: Option<Box<dyn FnMut(Vector) -> A>>,

    scroll_horizontally: bool,
    scroll_vertically: bool,
    scroll_with_scroll_wheel: bool,
    show_slider_when_content_fits: bool,
    capture_scroll_wheel: bool,
    points_per_line: f32,

    vertical_state: ScrollBarState,
    horizontal_state: ScrollBarState,

    sliders_state: SlidersState,
    drag_state: Option<DragState>,

    slider_width: f32,
}

impl<A: Clone + 'static> ScrollAreaElement<A> {
    pub fn create(builder: ScrollAreaBuilder<A>, cx: &mut WindowContext<'_, A>) -> ScrollArea {
        let ScrollAreaBuilder {
            scrolled_action,
            bounds,
            content_size,
            scroll_offset,

            scroll_horizontally,
            scroll_vertically,
            scroll_with_scroll_wheel,
            show_slider_when_content_fits,
            capture_scroll_wheel,
            points_per_line,

            class,
            z_index,
            manually_hidden,
            scissor_rect_id,
            disabled,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let slider_width = cx
            .res
            .style_system
            .get::<ScrollBarStyle>(class)
            .slider_width;

        let res = update_sliders_state(
            bounds.size,
            content_size,
            scroll_offset,
            slider_width,
            scroll_horizontally,
            scroll_vertically,
            show_slider_when_content_fits,
        );

        let shared_state = Rc::new(RefCell::new(SharedState {
            content_size,
            scroll_offset: res.scroll_offset,
            disabled,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                scrolled_action,
                scroll_horizontally,
                scroll_vertically,
                scroll_with_scroll_wheel,
                show_slider_when_content_fits,
                capture_scroll_wheel,
                points_per_line,
                vertical_state: ScrollBarState::Idle,
                horizontal_state: ScrollBarState::Idle,
                sliders_state: res,
                drag_state: None,
                slider_width,
            }),
            z_index,
            rect: bounds,
            manually_hidden,
            scissor_rect_id,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        ScrollArea { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for ScrollAreaElement<A> {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS
            | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
            | ElementFlags::LISTENS_TO_POINTER_OUTSIDE_BOUNDS_WHEN_FOCUSED
            | ElementFlags::LISTENS_TO_FOCUS_CHANGE
            | ElementFlags::LISTENS_TO_SIZE_CHANGE
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        match event {
            ElementEvent::CustomStateChanged => {
                cx.request_repaint();

                self.sliders_state = update_sliders_state(
                    cx.rect().size,
                    shared_state.content_size,
                    shared_state.scroll_offset,
                    self.slider_width,
                    self.scroll_horizontally,
                    self.scroll_vertically,
                    self.show_slider_when_content_fits,
                );

                if shared_state.disabled {
                    self.drag_state = None;
                    self.vertical_state = ScrollBarState::Idle;
                    self.horizontal_state = ScrollBarState::Idle;
                }
            }
            ElementEvent::StyleChanged | ElementEvent::SizeChanged => {
                self.slider_width = cx
                    .res
                    .style_system
                    .get::<ScrollBarStyle>(cx.class())
                    .slider_width;

                let prev_scroll_offset = self.sliders_state.scroll_offset;

                self.sliders_state = update_sliders_state(
                    cx.rect().size,
                    shared_state.content_size,
                    shared_state.scroll_offset,
                    self.slider_width,
                    self.scroll_horizontally,
                    self.scroll_vertically,
                    self.show_slider_when_content_fits,
                );

                if prev_scroll_offset != self.sliders_state.scroll_offset {
                    shared_state.scroll_offset = self.sliders_state.scroll_offset;

                    if let Some(action) = self.scrolled_action.as_mut() {
                        cx.send_action((action)(shared_state.scroll_offset))
                            .unwrap();
                    }
                }
            }
            ElementEvent::Pointer(PointerEvent::Moved { position, .. }) => {
                if shared_state.disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                let relative_pos = position - cx.rect().origin.to_vector();

                if let Some(drag_state) = self.drag_state.as_mut() {
                    let mut scroll_offset_changed = false;

                    if self.vertical_state == ScrollBarState::Dragging
                        && self.sliders_state.max_scroll_offset.y > 0.0
                    {
                        let new_scroll_offset_y = (drag_state.drag_start_scroll_offset.y
                            + ((relative_pos.y - drag_state.drag_start_pos.y)
                                / self.sliders_state.slider_to_content_ratio.y))
                            .clamp(0.0, self.sliders_state.max_scroll_offset.y);

                        if self.sliders_state.scroll_offset.y != new_scroll_offset_y {
                            self.sliders_state.scroll_offset.y = new_scroll_offset_y;
                            scroll_offset_changed = true;
                        }
                    }

                    if self.horizontal_state == ScrollBarState::Dragging
                        && self.sliders_state.max_scroll_offset.x > 0.0
                    {
                        let new_scroll_offset_x = (drag_state.drag_start_scroll_offset.x
                            + ((relative_pos.x - drag_state.drag_start_pos.x)
                                / self.sliders_state.slider_to_content_ratio.x))
                            .clamp(0.0, self.sliders_state.max_scroll_offset.x);

                        if self.sliders_state.scroll_offset.x != new_scroll_offset_x {
                            self.sliders_state.scroll_offset.x = new_scroll_offset_x;
                            scroll_offset_changed = true;
                        }
                    }

                    if scroll_offset_changed {
                        shared_state.scroll_offset = self.sliders_state.scroll_offset;

                        self.sliders_state = update_sliders_state(
                            cx.rect().size,
                            shared_state.content_size,
                            shared_state.scroll_offset,
                            self.slider_width,
                            self.scroll_horizontally,
                            self.scroll_vertically,
                            self.show_slider_when_content_fits,
                        );

                        if let Some(action) = self.scrolled_action.as_mut() {
                            cx.send_action((action)(shared_state.scroll_offset))
                                .unwrap();
                        }

                        cx.request_repaint();
                    }

                    return EventCaptureStatus::Captured;
                } else {
                    if self.scroll_vertically {
                        if self
                            .sliders_state
                            .vertical_slider_bounds
                            .contains(relative_pos)
                        {
                            if self.vertical_state != ScrollBarState::SliderHovered {
                                self.vertical_state = ScrollBarState::SliderHovered;
                                cx.request_repaint();
                            }

                            return EventCaptureStatus::Captured;
                        } else if cx.rect().contains(position) {
                            if self.vertical_state != ScrollBarState::ContentHovered {
                                self.vertical_state = ScrollBarState::ContentHovered;
                                cx.request_repaint();
                            }
                        } else {
                            if self.vertical_state != ScrollBarState::Idle {
                                self.vertical_state = ScrollBarState::Idle;
                                cx.request_repaint();
                            }
                        }
                    }

                    if self.scroll_horizontally {
                        if self
                            .sliders_state
                            .horizontal_slider_bounds
                            .contains(relative_pos)
                        {
                            if self.horizontal_state != ScrollBarState::SliderHovered {
                                self.horizontal_state = ScrollBarState::SliderHovered;
                                cx.request_repaint();
                            }

                            return EventCaptureStatus::Captured;
                        } else if cx.rect().contains(position) {
                            if self.horizontal_state != ScrollBarState::ContentHovered {
                                self.horizontal_state = ScrollBarState::ContentHovered;
                                cx.request_repaint();
                            }
                        } else {
                            if self.horizontal_state != ScrollBarState::Idle {
                                self.horizontal_state = ScrollBarState::Idle;
                                cx.request_repaint();
                            }
                        }
                    }
                }
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                if self.drag_state.is_none() {
                    if self.vertical_state != ScrollBarState::Idle {
                        self.vertical_state = ScrollBarState::Idle;
                        cx.request_repaint();
                    }
                    if self.horizontal_state != ScrollBarState::Idle {
                        self.horizontal_state = ScrollBarState::Idle;
                        cx.request_repaint();
                    }
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed {
                position, button, ..
            }) => {
                if shared_state.disabled || button != PointerButton::Primary {
                    return EventCaptureStatus::NotCaptured;
                }

                let relative_pos = position - cx.rect().origin.to_vector();

                if self.scroll_vertically {
                    if self
                        .sliders_state
                        .vertical_slider_bounds
                        .contains(relative_pos)
                    {
                        self.vertical_state = ScrollBarState::Dragging;

                        self.drag_state = Some(DragState {
                            drag_start_pos: relative_pos,
                            drag_start_scroll_offset: self.sliders_state.scroll_offset,
                        });

                        cx.request_repaint();
                        cx.steal_temporary_focus();

                        return EventCaptureStatus::Captured;
                    } else if self.sliders_state.vertical_bg_bounds.contains(relative_pos) {
                        let new_scroll_offset_y = ((relative_pos.y
                            - (self.sliders_state.vertical_slider_bounds.height() * 0.5))
                            / self.sliders_state.slider_to_content_ratio.y)
                            .clamp(0.0, self.sliders_state.max_scroll_offset.y);

                        if self.sliders_state.scroll_offset.y != new_scroll_offset_y {
                            self.sliders_state.scroll_offset.y = new_scroll_offset_y;
                            shared_state.scroll_offset = self.sliders_state.scroll_offset;

                            self.sliders_state = update_sliders_state(
                                cx.rect().size,
                                shared_state.content_size,
                                shared_state.scroll_offset,
                                self.slider_width,
                                self.scroll_horizontally,
                                self.scroll_vertically,
                                self.show_slider_when_content_fits,
                            );

                            if let Some(action) = self.scrolled_action.as_mut() {
                                cx.send_action((action)(shared_state.scroll_offset))
                                    .unwrap();
                            }

                            cx.request_repaint();
                        }

                        return EventCaptureStatus::Captured;
                    }
                }

                if self.scroll_horizontally {
                    if self
                        .sliders_state
                        .horizontal_slider_bounds
                        .contains(relative_pos)
                    {
                        self.horizontal_state = ScrollBarState::Dragging;

                        self.drag_state = Some(DragState {
                            drag_start_pos: relative_pos,
                            drag_start_scroll_offset: self.sliders_state.scroll_offset,
                        });

                        cx.request_repaint();
                        cx.steal_temporary_focus();

                        return EventCaptureStatus::Captured;
                    } else if self
                        .sliders_state
                        .horizontal_bg_bounds
                        .contains(relative_pos)
                    {
                        let new_scroll_offset_x = ((relative_pos.x
                            - (self.sliders_state.horizontal_slider_bounds.width() * 0.5))
                            / self.sliders_state.slider_to_content_ratio.x)
                            .clamp(0.0, self.sliders_state.max_scroll_offset.x);

                        if self.sliders_state.scroll_offset.x != new_scroll_offset_x {
                            self.sliders_state.scroll_offset.x = new_scroll_offset_x;
                            shared_state.scroll_offset = self.sliders_state.scroll_offset;

                            self.sliders_state = update_sliders_state(
                                cx.rect().size,
                                shared_state.content_size,
                                shared_state.scroll_offset,
                                self.slider_width,
                                self.scroll_horizontally,
                                self.scroll_vertically,
                                self.show_slider_when_content_fits,
                            );

                            if let Some(action) = self.scrolled_action.as_mut() {
                                cx.send_action((action)(shared_state.scroll_offset))
                                    .unwrap();
                            }

                            cx.request_repaint();
                        }

                        return EventCaptureStatus::Captured;
                    }
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                button, position, ..
            }) => {
                if !(cx.has_focus() && button == PointerButton::Primary) {
                    return EventCaptureStatus::NotCaptured;
                }

                cx.release_focus();

                self.drag_state = None;

                let relative_pos = position - cx.rect().origin.to_vector();

                if self.scroll_vertically {
                    if self
                        .sliders_state
                        .vertical_slider_bounds
                        .contains(relative_pos)
                    {
                        if self.vertical_state != ScrollBarState::SliderHovered {
                            self.vertical_state = ScrollBarState::SliderHovered;
                            cx.request_repaint();
                        }
                    } else if cx.rect().contains(position) {
                        if self.vertical_state != ScrollBarState::ContentHovered {
                            self.vertical_state = ScrollBarState::ContentHovered;
                            cx.request_repaint();
                        }
                    } else {
                        if self.vertical_state != ScrollBarState::Idle {
                            self.vertical_state = ScrollBarState::Idle;
                            cx.request_repaint();
                        }
                    }
                }

                if self.scroll_horizontally {
                    if self
                        .sliders_state
                        .horizontal_slider_bounds
                        .contains(relative_pos)
                    {
                        if self.horizontal_state != ScrollBarState::SliderHovered {
                            self.horizontal_state = ScrollBarState::SliderHovered;
                            cx.request_repaint();
                        }
                    } else if cx.rect().contains(position) {
                        if self.horizontal_state != ScrollBarState::ContentHovered {
                            self.horizontal_state = ScrollBarState::ContentHovered;
                            cx.request_repaint();
                        }
                    } else {
                        if self.horizontal_state != ScrollBarState::Idle {
                            self.horizontal_state = ScrollBarState::Idle;
                            cx.request_repaint();
                        }
                    }
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::ScrollWheel {
                position,
                delta_type,
                ..
            }) => {
                if shared_state.disabled
                    || !self.scroll_with_scroll_wheel
                    || !cx.rect().contains(position)
                {
                    return EventCaptureStatus::NotCaptured;
                }

                let delta = delta_type.points(self.points_per_line, cx.rect().height());

                let new_scroll_offset = Vector::new(
                    (self.sliders_state.scroll_offset.x + (delta.x))
                        .clamp(0.0, self.sliders_state.max_scroll_offset.x),
                    (self.sliders_state.scroll_offset.y + (delta.y))
                        .clamp(0.0, self.sliders_state.max_scroll_offset.y),
                );

                if self.sliders_state.scroll_offset != new_scroll_offset {
                    self.sliders_state.scroll_offset = new_scroll_offset;
                    shared_state.scroll_offset = self.sliders_state.scroll_offset;

                    self.sliders_state = update_sliders_state(
                        cx.rect().size,
                        shared_state.content_size,
                        shared_state.scroll_offset,
                        self.slider_width,
                        self.scroll_horizontally,
                        self.scroll_vertically,
                        self.show_slider_when_content_fits,
                    );

                    if let Some(action) = self.scrolled_action.as_mut() {
                        cx.send_action((action)(shared_state.scroll_offset))
                            .unwrap();
                    }

                    cx.request_repaint();
                }

                if self.capture_scroll_wheel {
                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Focus(false) => {
                self.drag_state = None;
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let style = cx.res.style_system.get::<ScrollBarStyle>(cx.class);

        let bg_style = |state| -> QuadStyle {
            match state {
                ScrollBarState::Idle => QuadStyle {
                    bg: style.back_quad_bg,
                    border: BorderStyle {
                        color: style.back_quad_border_color,
                        width: style.back_quad_border_width,
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                },
                ScrollBarState::ContentHovered => QuadStyle {
                    bg: style
                        .back_quad_bg_content_hover
                        .unwrap_or(style.back_quad_bg),
                    border: BorderStyle {
                        color: style
                            .back_quad_border_color_content_hover
                            .unwrap_or(style.back_quad_border_color),
                        width: style
                            .back_quad_border_width_content_hover
                            .unwrap_or(style.back_quad_border_width),
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                },
                _ => QuadStyle {
                    bg: style
                        .back_quad_bg_slider_hover
                        .unwrap_or(style.back_quad_bg),
                    border: BorderStyle {
                        color: style
                            .back_quad_border_color_slider_hover
                            .unwrap_or(style.back_quad_border_color),
                        width: style
                            .back_quad_border_width_slider_hover
                            .unwrap_or(style.back_quad_border_width),
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                },
            }
        };

        if self.sliders_state.show_vertical {
            let bg_style = bg_style(self.vertical_state);

            if !bg_style.is_transparent() {
                primitives.add(bg_style.create_primitive(self.sliders_state.vertical_bg_bounds));
            }
        }

        if self.sliders_state.show_horizontal {
            let bg_style = bg_style(self.horizontal_state);

            if !bg_style.is_transparent() {
                primitives.add(bg_style.create_primitive(self.sliders_state.horizontal_bg_bounds));
            }
        }

        let slider_style = |state| -> QuadStyle {
            match state {
                ScrollBarState::Idle => QuadStyle {
                    bg: style.slider_bg,
                    border: BorderStyle {
                        color: style.slider_border_color,
                        width: style.slider_border_width,
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                },
                ScrollBarState::ContentHovered => QuadStyle {
                    bg: style.slider_bg_content_hover.unwrap_or(style.slider_bg),
                    border: BorderStyle {
                        color: style
                            .slider_border_color_content_hover
                            .unwrap_or(style.slider_border_color),
                        width: style
                            .slider_border_width_content_hover
                            .unwrap_or(style.slider_border_width),
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                },
                ScrollBarState::SliderHovered => QuadStyle {
                    bg: style
                        .slider_bg_slider_hover
                        .unwrap_or(style.slider_bg_content_hover.unwrap_or(style.slider_bg)),
                    border: BorderStyle {
                        color: style.slider_border_color_slider_hover.unwrap_or(
                            style
                                .slider_border_color_content_hover
                                .unwrap_or(style.slider_border_color),
                        ),
                        width: style.slider_border_width_slider_hover.unwrap_or(
                            style
                                .slider_border_width_content_hover
                                .unwrap_or(style.slider_border_width),
                        ),
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                },
                ScrollBarState::Dragging => QuadStyle {
                    bg: style.slider_bg_slider_dragging.unwrap_or(
                        style
                            .slider_bg_slider_hover
                            .unwrap_or(style.slider_bg_content_hover.unwrap_or(style.slider_bg)),
                    ),
                    border: BorderStyle {
                        color: style.slider_border_color_slider_dragging.unwrap_or(
                            style.slider_border_color_slider_hover.unwrap_or(
                                style
                                    .slider_border_color_content_hover
                                    .unwrap_or(style.slider_border_color),
                            ),
                        ),
                        width: style.slider_border_width_slider_dragging.unwrap_or(
                            style.slider_border_width_slider_hover.unwrap_or(
                                style
                                    .slider_border_width_content_hover
                                    .unwrap_or(style.slider_border_width),
                            ),
                        ),
                        radius: style.radius,
                    },
                    flags: style.quad_flags,
                },
            }
        };

        if self.sliders_state.show_vertical {
            let slider_style = slider_style(self.vertical_state);

            if !slider_style.is_transparent() {
                primitives.set_z_index(1);
                primitives
                    .add(slider_style.create_primitive(self.sliders_state.vertical_slider_bounds));
            }
        }

        if self.sliders_state.show_horizontal {
            let slider_style = slider_style(self.horizontal_state);

            if !slider_style.is_transparent() {
                primitives.set_z_index(1);
                primitives.add(
                    slider_style.create_primitive(self.sliders_state.horizontal_slider_bounds),
                );
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScrollBarState {
    Idle,
    ContentHovered,
    SliderHovered,
    Dragging,
}

struct SharedState {
    content_size: Size,
    scroll_offset: Vector,
    disabled: bool,
}

pub struct ScrollArea {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl ScrollArea {
    pub fn builder<A: Clone + 'static>() -> ScrollAreaBuilder<A> {
        ScrollAreaBuilder::new()
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

    /// Set the scroll offset.
    ///
    /// Returns `true` if the offset has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_scroll_offset(&mut self, scroll_offset: Vector) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.scroll_offset != scroll_offset {
            shared_state.scroll_offset = scroll_offset;
            self.el._notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn scroll_offset(&self) -> Vector {
        RefCell::borrow(&self.shared_state).scroll_offset
    }

    /// Set the content size.
    ///
    /// Returns `true` if the content size has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_content_size(&mut self, content_size: Size) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.content_size != content_size {
            shared_state.content_size = content_size;
            self.el._notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn content_size(&self) -> Size {
        RefCell::borrow(&self.shared_state).content_size
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

struct SlidersState {
    vertical_bg_bounds: Rect,
    vertical_slider_bounds: Rect,
    horizontal_bg_bounds: Rect,
    horizontal_slider_bounds: Rect,
    show_vertical: bool,
    show_horizontal: bool,
    scroll_offset: Vector,
    max_scroll_offset: Vector,
    slider_to_content_ratio: Vector,
}

fn update_sliders_state(
    bounds_size: Size,
    content_size: Size,
    scroll_offset: Vector,
    slider_width: f32,
    scroll_horizontally: bool,
    scroll_vertically: bool,
    show_slider_when_content_fits: bool,
) -> SlidersState {
    let show_vertical = if scroll_vertically {
        if show_slider_when_content_fits {
            true
        } else {
            content_size.height > bounds_size.height
        }
    } else {
        false
    };
    let show_horizontal = if scroll_horizontally {
        if show_slider_when_content_fits {
            true
        } else {
            content_size.width > bounds_size.width
        }
    } else {
        false
    };

    let mut vertical_bg_bounds = Rect::default();
    let mut vertical_slider_bounds = Rect::default();
    let mut scroll_offset_y = 0.0;
    let mut max_scroll_offset_y = 0.0;
    let mut slider_to_content_ratio_y = 1.0;

    if show_vertical {
        vertical_bg_bounds = Rect::new(
            Point::new(bounds_size.width as f32 - slider_width, 0.0),
            Size::new(slider_width, bounds_size.height as f32),
        );

        if content_size.height <= bounds_size.height {
            vertical_slider_bounds = vertical_bg_bounds;
        } else if content_size.height > 0.0 && bounds_size.height > 0.0 {
            max_scroll_offset_y = content_size.height - bounds_size.height;
            scroll_offset_y = scroll_offset.y.clamp(0.0, max_scroll_offset_y);

            slider_to_content_ratio_y = bounds_size.height / content_size.height;

            let slider_size_height = bounds_size.height * slider_to_content_ratio_y;
            let slider_y = scroll_offset_y * slider_to_content_ratio_y;

            vertical_slider_bounds = Rect::new(
                Point::new(vertical_bg_bounds.min_x(), slider_y as f32),
                Size::new(slider_width, slider_size_height as f32),
            );
        }
    }

    let horizontal_bounds_width = if show_vertical {
        bounds_size.width - (slider_width)
    } else {
        bounds_size.width
    };

    let mut horizontal_bg_bounds = Rect::default();
    let mut horizontal_slider_bounds = Rect::default();
    let mut scroll_offset_x = 0.0;
    let mut max_scroll_offset_x = 0.0;
    let mut slider_to_content_ratio_x = 1.0;

    if show_horizontal {
        horizontal_bg_bounds = Rect::new(
            Point::new(0.0, bounds_size.height as f32 - slider_width),
            Size::new(horizontal_bounds_width as f32, slider_width),
        );

        if content_size.width <= bounds_size.width {
            horizontal_slider_bounds = Rect::new(
                Point::new(0.0, horizontal_bg_bounds.min_y()),
                Size::new(horizontal_bounds_width as f32, slider_width),
            );
        } else if content_size.width > 0.0 && horizontal_bounds_width > 0.0 {
            max_scroll_offset_x = content_size.width - horizontal_bounds_width;
            scroll_offset_x = scroll_offset.x.clamp(0.0, max_scroll_offset_x);

            slider_to_content_ratio_x = horizontal_bounds_width / content_size.width;

            let slider_size_width = horizontal_bounds_width * slider_to_content_ratio_x;
            let slider_x = scroll_offset_x * slider_to_content_ratio_x;

            horizontal_slider_bounds = Rect::new(
                Point::new(slider_x as f32, horizontal_bg_bounds.min_y()),
                Size::new(slider_size_width as f32, slider_width),
            );
        }
    }

    SlidersState {
        vertical_bg_bounds,
        vertical_slider_bounds,
        horizontal_bg_bounds,
        horizontal_slider_bounds,
        show_vertical,
        show_horizontal,
        scroll_offset: Vector::new(scroll_offset_x, scroll_offset_y),
        max_scroll_offset: Vector::new(max_scroll_offset_x, max_scroll_offset_y),
        slider_to_content_ratio: Vector::new(slider_to_content_ratio_x, slider_to_content_ratio_y),
    }
}
