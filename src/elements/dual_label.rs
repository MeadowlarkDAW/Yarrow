use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::quad::QuadPrimitive;
use rootvg::text::glyphon::FontSystem;
use rootvg::text::{Align, RcTextBuffer, TextPrimitive, TextProperties};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::layout::{Align2, Padding};
use crate::math::{Point, Rect, Size, ZIndex};
use crate::style::{QuadStyle, DEFAULT_TEXT_ATTRIBUTES};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DualLabelLayout {
    #[default]
    LeftAlign,
    RightAlign,
    LeftAndRightAlign,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DualLabelClipMode {
    #[default]
    ClipLeftThenRight,
    ClipRightThenLeft,
}

/// The style of a [`DualLabel`] element
#[derive(Debug, Clone, PartialEq)]
pub struct DualLabelStyle {
    /// The properties of the left text.
    pub left_properties: TextProperties,
    /// The properties of the right text.
    pub right_properties: TextProperties,

    /// The color of the left font
    ///
    /// By default this is set to `color::WHITE`.
    pub left_font_color: RGBA8,
    /// The color of the right font
    ///
    /// By default this is set to `color::WHITE`.
    pub right_font_color: RGBA8,

    /// The vertical alignment of the text.
    ///
    /// By default this is set to `Align::Center`.
    pub vertical_align: crate::layout::Align,

    pub layout: DualLabelLayout,

    /// The minimum size of the clipped text area for the left text.
    ///
    /// By default this is set to `Size::new(5.0, 5.0)`.
    pub left_min_clipped_size: Size,
    /// The minimum size of the clipped text area for the right text.
    ///
    /// By default this is set to `Size::new(5.0, 5.0)`.
    pub right_min_clipped_size: Size,

    pub clip_mode: DualLabelClipMode,

    /// The style of the padded background rectangle behind the text.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    ///
    /// By default this is set to `QuadStyle::TRANSPARENT`.
    pub back_quad: QuadStyle,

    /// The padding between the left text and the bounding rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub left_padding: Padding,
    /// The padding between the right text and the bounding rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub right_padding: Padding,
}

impl Default for DualLabelStyle {
    fn default() -> Self {
        Self {
            left_properties: TextProperties {
                attrs: DEFAULT_TEXT_ATTRIBUTES,
                ..Default::default()
            },
            right_properties: TextProperties {
                attrs: DEFAULT_TEXT_ATTRIBUTES,
                ..Default::default()
            },
            left_font_color: color::WHITE,
            right_font_color: color::WHITE,
            vertical_align: crate::layout::Align::Center,
            layout: DualLabelLayout::default(),
            left_min_clipped_size: Size::new(5.0, 5.0),
            right_min_clipped_size: Size::new(5.0, 5.0),
            clip_mode: DualLabelClipMode::default(),
            back_quad: QuadStyle::TRANSPARENT,
            left_padding: Padding::default(),
            right_padding: Padding::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DualLabelPrimitives {
    pub left_text: Option<TextPrimitive>,
    pub right_text: Option<TextPrimitive>,
    pub bg_quad: Option<QuadPrimitive>,
}

/// A reusable label struct that can be used by other elements.
pub struct DualLabelInner {
    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub left_text_offset: Point,
    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub right_text_offset: Point,
    left_text: String,
    right_text: String,
    left_text_buffer: RcTextBuffer,
    right_text_buffer: Option<RcTextBuffer>,
    left_unclipped_text_size: Size,
    right_unclipped_text_size: Size,
    left_text_size_needs_calculated: bool,
    right_text_size_needs_calculated: bool,
    prev_bounds_size: Size,
    left_text_bounds_rect: Rect,
    right_text_bounds_rect: Rect,
    padded_size: Size,
    padded_size_needs_calculated: bool,
}

impl DualLabelInner {
    pub fn new(
        left_text: impl Into<String>,
        right_text: impl Into<String>,
        left_text_offset: Point,
        right_text_offset: Point,
        style: &DualLabelStyle,
        font_system: &mut FontSystem,
    ) -> Self {
        let left_text: String = left_text.into();
        let right_text: String = right_text.into();

        let mut left_properties = style.left_properties.clone();
        left_properties.align = Some(match style.layout {
            DualLabelLayout::LeftAlign | DualLabelLayout::LeftAndRightAlign => Align::Left,
            _ => Align::Right,
        });

        // Use a temporary size for the text buffer.
        let left_text_buffer = RcTextBuffer::new(
            &left_text,
            left_properties,
            Size::new(1000.0, 200.0),
            false,
            font_system,
        );
        let has_right_text = !right_text.is_empty();
        let right_text_buffer = if has_right_text {
            let mut right_properties = style.right_properties.clone();
            right_properties.align = Some(match style.layout {
                DualLabelLayout::LeftAlign => Align::Left,
                _ => Align::Right,
            });

            Some(RcTextBuffer::new(
                &right_text,
                right_properties,
                Size::new(1000.0, 200.0),
                false,
                font_system,
            ))
        } else {
            None
        };

        Self {
            left_text_offset,
            right_text_offset,
            left_text,
            right_text,
            left_text_buffer,
            right_text_buffer,
            // This will be overwritten later.
            left_unclipped_text_size: Size::default(),
            right_unclipped_text_size: Size::default(),
            left_text_size_needs_calculated: true,
            right_text_size_needs_calculated: has_right_text,
            prev_bounds_size: Size::new(-1.0, -1.0),
            // This will be overwritten later.
            left_text_bounds_rect: Rect::default(),
            right_text_bounds_rect: Rect::default(),
            padded_size: Size::default(),
            padded_size_needs_calculated: true,
        }
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    pub fn desired_padded_size(&mut self, style: &DualLabelStyle) -> Size {
        if self.padded_size_needs_calculated {
            self.padded_size_needs_calculated = false;

            let (left_size, right_size) = self.unclipped_text_size();

            let left_padded_width = if self.left_text.is_empty() {
                0.0
            } else {
                left_size.width + style.left_padding.left + style.left_padding.right
            };
            let right_padded_width = if self.right_text.is_empty() {
                0.0
            } else {
                right_size.width + style.right_padding.left + style.right_padding.right
            };

            let height = (left_size.height + style.left_padding.top + style.left_padding.bottom)
                .max(right_size.height + style.right_padding.top + style.right_padding.bottom);

            self.padded_size = Size::new(left_padded_width + right_padded_width, height);
        }

        self.padded_size
    }

    /// Returns the size of the unclipped left and right text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn unclipped_text_size(&mut self) -> (Size, Size) {
        if self.left_text_size_needs_calculated {
            self.left_text_size_needs_calculated = false;

            self.left_unclipped_text_size = self.left_text_buffer.measure();
        }

        if let Some(right_text_buffer) = self.right_text_buffer.as_mut() {
            if self.right_text_size_needs_calculated {
                self.right_text_size_needs_calculated = false;

                self.right_unclipped_text_size = right_text_buffer.measure();
            }
        }

        (
            self.left_unclipped_text_size,
            self.right_unclipped_text_size,
        )
    }

    /// Returns `true` if the text has changed.
    pub fn set_left_text(&mut self, text: &str, font_system: &mut FontSystem) -> bool {
        if &self.left_text != text {
            self.left_text = String::from(text);
            self.left_text_size_needs_calculated = true;
            self.padded_size_needs_calculated = true;

            self.left_text_buffer.set_text(text, font_system);

            true
        } else {
            false
        }
    }

    /// Returns `true` if the text has changed.
    pub fn set_right_text(
        &mut self,
        text: &str,
        style: &DualLabelStyle,
        font_system: &mut FontSystem,
    ) -> bool {
        if &self.right_text != text {
            self.right_text = String::from(text);
            self.right_text_size_needs_calculated = true;
            self.padded_size_needs_calculated = true;

            if let Some(right_text_buffer) = self.right_text_buffer.as_mut() {
                right_text_buffer.set_text(text, font_system);
            } else {
                self.right_text_buffer = Some(RcTextBuffer::new(
                    text,
                    style.right_properties,
                    Size::new(1000.0, 200.0),
                    false,
                    font_system,
                ));
            }

            true
        } else {
            false
        }
    }

    pub fn text(&self) -> (&str, &str) {
        (&self.left_text, &self.right_text)
    }

    pub fn set_style(&mut self, style: &DualLabelStyle, font_system: &mut FontSystem) {
        let mut left_properties = style.left_properties.clone();
        left_properties.align = Some(match style.layout {
            DualLabelLayout::LeftAlign | DualLabelLayout::LeftAndRightAlign => Align::Left,
            _ => Align::Right,
        });

        self.left_text_buffer
            .set_text_and_props(&self.left_text, left_properties, font_system);
        self.left_text_size_needs_calculated = true;

        if let Some(right_text_buffer) = self.right_text_buffer.as_mut() {
            let mut right_properties = style.right_properties.clone();
            right_properties.align = Some(match style.layout {
                DualLabelLayout::LeftAlign => Align::Left,
                _ => Align::Right,
            });

            right_text_buffer.set_text_and_props(
                &self.right_text,
                style.right_properties,
                font_system,
            );
            self.right_text_size_needs_calculated = true;
        }

        self.padded_size_needs_calculated = true;
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &DualLabelStyle,
        font_system: &mut FontSystem,
    ) -> DualLabelPrimitives {
        let mut needs_layout =
            self.left_text_size_needs_calculated || self.right_text_size_needs_calculated;

        if self.prev_bounds_size != bounds.size {
            self.prev_bounds_size = bounds.size;
            needs_layout = true;
        }

        if needs_layout {
            let _ = self.unclipped_text_size();

            let (left_rect, right_rect) = layout_text_bounds(
                bounds.size,
                self.left_unclipped_text_size,
                self.right_unclipped_text_size,
                style,
            );

            self.left_text_bounds_rect = left_rect;
            self.right_text_bounds_rect = right_rect;

            self.left_text_buffer.set_bounds(
                Size::new(
                    self.left_text_bounds_rect.width(),
                    // Add some extra padding below so that text doesn't get clipped.
                    self.left_text_bounds_rect.height() + 2.0,
                ),
                font_system,
            );

            if let Some(right_text_buffer) = self.right_text_buffer.as_mut() {
                right_text_buffer.set_bounds(
                    Size::new(
                        self.right_text_bounds_rect.width(),
                        // Add some extra padding below so that text doesn't get clipped.
                        self.right_text_bounds_rect.height() + 2.0,
                    ),
                    font_system,
                );
            }
        }

        let left_text = if !self.left_text.is_empty() {
            Some(TextPrimitive::new(
                self.left_text_buffer.clone(),
                bounds.origin
                    + self.left_text_bounds_rect.origin.to_vector()
                    + self.left_text_offset.to_vector(),
                style.left_font_color,
                None,
            ))
        } else {
            None
        };

        let right_text = if !self.right_text.is_empty() {
            let right_text_buffer = self.right_text_buffer.as_mut().unwrap().clone();

            Some(TextPrimitive::new(
                right_text_buffer,
                bounds.origin
                    + self.right_text_bounds_rect.origin.to_vector()
                    + self.right_text_offset.to_vector(),
                style.right_font_color,
                None,
            ))
        } else {
            None
        };

        let bg_quad = if !style.back_quad.is_transparent() {
            Some(style.back_quad.create_primitive(bounds))
        } else {
            None
        };

        DualLabelPrimitives {
            left_text,
            right_text,
            bg_quad,
        }
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_left_text_offset(&mut self, offset: Point) -> bool {
        if self.left_text_offset != offset {
            self.left_text_offset = offset;
            true
        } else {
            false
        }
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_right_text_offset(&mut self, offset: Point) -> bool {
        if self.right_text_offset != offset {
            self.right_text_offset = offset;
            true
        } else {
            false
        }
    }
}

pub struct DualLabelBuilder {
    pub left_text: String,
    pub right_text: String,
    pub left_text_offset: Point,
    pub right_text_offset: Point,
    pub style: Rc<DualLabelStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl DualLabelBuilder {
    pub fn new(style: &Rc<DualLabelStyle>) -> Self {
        Self {
            left_text: String::new(),
            right_text: String::new(),
            left_text_offset: Point::default(),
            right_text_offset: Point::default(),
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> DualLabel {
        DualLabelElement::create(self, cx)
    }

    pub fn left_text(mut self, text: impl Into<String>) -> Self {
        self.left_text = text.into();
        self
    }

    pub fn right_text(mut self, text: impl Into<String>) -> Self {
        self.right_text = text.into();
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn left_text_offset(mut self, offset: Point) -> Self {
        self.left_text_offset = offset;
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn right_text_offset(mut self, offset: Point) -> Self {
        self.right_text_offset = offset;
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

/// A label element with an optional quad background.
pub struct DualLabelElement {
    shared_state: Rc<RefCell<SharedState>>,
}

impl DualLabelElement {
    pub fn create<A: Clone + 'static>(
        builder: DualLabelBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> DualLabel {
        let DualLabelBuilder {
            left_text,
            right_text,
            left_text_offset,
            right_text_offset,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: DualLabelInner::new(
                left_text,
                right_text,
                left_text_offset,
                right_text_offset,
                &style,
                cx.font_system,
            ),
            style,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, cx.font_system, cx.clipboard);

        DualLabel { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for DualLabelElement {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        if let ElementEvent::CustomStateChanged = event {
            cx.request_repaint();
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        let label_primitives =
            inner.render_primitives(Rect::from_size(cx.bounds_size), style, cx.font_system);

        if let Some(quad_primitive) = label_primitives.bg_quad {
            primitives.add(quad_primitive);
        }

        if let Some(text_primitive) = label_primitives.left_text {
            primitives.set_z_index(1);
            primitives.add_text(text_primitive);
        }

        if let Some(text_primitive) = label_primitives.right_text {
            primitives.set_z_index(1);
            primitives.add_text(text_primitive);
        }
    }
}

struct SharedState {
    inner: DualLabelInner,
    style: Rc<DualLabelStyle>,
}

/// A handle to a [`DualLabelElement`], a label with an optional quad background.
pub struct DualLabel {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl DualLabel {
    pub fn builder(style: &Rc<DualLabelStyle>) -> DualLabelBuilder {
        DualLabelBuilder::new(style)
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

    /// Returns the size of the unclipped left and right text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn unclipped_text_size(&self) -> (Size, Size) {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .unclipped_text_size()
    }

    pub fn set_left_text(&mut self, text: &str, font_system: &mut FontSystem) {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_left_text(text, font_system);
        self.el.notify_custom_state_change();
    }

    pub fn set_right_text(&mut self, text: &str, font_system: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        inner.set_right_text(text, style, font_system);
        self.el.notify_custom_state_change();
    }

    pub fn left_text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text().0)
    }

    pub fn right_text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text().1)
    }

    pub fn set_style(&mut self, style: &Rc<DualLabelStyle>, font_system: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(style, font_system);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<DualLabelStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub fn set_left_text_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_left_text_offset(offset);

        if changed {
            self.el.notify_custom_state_change();
        }
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub fn set_right_text_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_right_text_offset(offset);

        if changed {
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

fn layout_text_bounds(
    bounds_size: Size,
    left_unclipped_text_size: Size,
    right_unclipped_text_size: Size,
    style: &DualLabelStyle,
) -> (Rect, Rect) {
    let left_empty = left_unclipped_text_size.is_empty();
    let right_empty = right_unclipped_text_size.is_empty();

    if left_empty || right_empty {
        return (
            if left_empty {
                Rect::default()
            } else {
                super::label::layout_text_bounds(
                    bounds_size,
                    left_unclipped_text_size,
                    style.left_padding,
                    style.left_min_clipped_size,
                    style.vertical_align,
                )
            },
            if right_empty {
                Rect::default()
            } else {
                super::label::layout_text_bounds(
                    bounds_size,
                    right_unclipped_text_size,
                    style.right_padding,
                    style.right_min_clipped_size,
                    style.vertical_align,
                )
            },
        );
    }

    let left_padded_width =
        left_unclipped_text_size.width + style.left_padding.left + style.left_padding.right;
    let right_padded_width =
        right_unclipped_text_size.width + style.right_padding.left + style.right_padding.right;

    let total_padded_width = left_padded_width + right_padded_width;

    let (left_clipped_padded_width, right_clipped_padded_width) = if total_padded_width
        <= bounds_size.width
    {
        match style.layout {
            DualLabelLayout::LeftAlign | DualLabelLayout::LeftAndRightAlign => {
                (left_padded_width, (bounds_size.width - left_padded_width))
            }
            DualLabelLayout::RightAlign => {
                ((bounds_size.width - right_padded_width), right_padded_width)
            }
        }
    } else {
        let min_left_padded_width =
            style.left_min_clipped_size.width + style.left_padding.left + style.left_padding.right;
        let min_right_padded_width = style.right_min_clipped_size.width
            + style.right_padding.left
            + style.right_padding.right;
        let min_total_padded_width = min_left_padded_width + min_right_padded_width;

        match style.clip_mode {
            DualLabelClipMode::ClipLeftThenRight => {
                if min_total_padded_width >= bounds_size.width {
                    (min_left_padded_width, min_right_padded_width)
                } else if min_left_padded_width + right_padded_width >= bounds_size.width {
                    (
                        style.left_min_clipped_size.width,
                        bounds_size.width - min_left_padded_width,
                    )
                } else {
                    (bounds_size.width - right_padded_width, right_padded_width)
                }
            }
            DualLabelClipMode::ClipRightThenLeft => {
                if min_total_padded_width >= bounds_size.width {
                    (min_left_padded_width, min_right_padded_width)
                } else if min_right_padded_width + left_padded_width >= bounds_size.width {
                    (
                        bounds_size.width - min_right_padded_width,
                        style.right_min_clipped_size.width,
                    )
                } else {
                    (left_padded_width, bounds_size.width - left_padded_width)
                }
            }
        }
    };

    let left_content_rect = crate::layout::layout_inner_rect_with_min_size(
        style.left_padding,
        Rect::new(
            Point::zero(),
            Size::new(left_clipped_padded_width, bounds_size.height),
        ),
        style.left_min_clipped_size,
    );

    // We need to vertically align the text ourselves as rootvg/glyphon does not do this.
    let left_text_bounds_y = match style.vertical_align {
        crate::layout::Align::Start => left_content_rect.min_y(),
        crate::layout::Align::Center => {
            left_content_rect.min_y()
                + ((left_content_rect.height() - left_unclipped_text_size.height) * 0.5)
        }
        /*
        crate::layout::Align::Center => {
            left_content_rect.min_y()
                + ((left_content_rect.height() - style.left_properties.metrics.font_size) / 2.0)
                + 1.0
        }
        */
        crate::layout::Align::End => left_content_rect.max_y() - left_unclipped_text_size.height,
    };

    let right_content_rect = crate::layout::layout_inner_rect_with_min_size(
        style.right_padding,
        Rect::new(
            Point::new(left_clipped_padded_width, 0.0),
            Size::new(right_clipped_padded_width, bounds_size.height),
        ),
        style.right_min_clipped_size,
    );

    // We need to vertically align the text ourselves as rootvg/glyphon does not do this.
    let right_text_bounds_y = match style.vertical_align {
        crate::layout::Align::Start => right_content_rect.min_y(),
        crate::layout::Align::Center => {
            right_content_rect.min_y()
                + ((right_content_rect.height() - right_unclipped_text_size.height) * 0.5)
        }
        /*
        crate::layout::Align::Center => {
            right_content_rect.min_y()
                + ((right_content_rect.height() - style.right_properties.metrics.font_size) / 2.0)
                + 1.0
        }
        */
        crate::layout::Align::End => right_content_rect.max_y() - right_unclipped_text_size.height,
    };

    (
        Rect::new(
            Point::new(left_content_rect.min_x(), left_text_bounds_y),
            left_content_rect.size,
        ),
        Rect::new(
            Point::new(right_content_rect.min_x(), right_text_bounds_y),
            right_content_rect.size,
        ),
    )
}
