use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::text::{RcTextBuffer, TextPrimitive};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::layout::{Align, Align2, Padding};
use crate::math::{Point, Rect, Size, ZIndex};
use crate::prelude::ResourceCtx;
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;

use super::label::{LabelPrimitives, LabelStyle};

// TODO: Add ability to select, copy, and right-click text.

/// A reusable Paragraph struct that can be used by other elements.
pub struct ParagraphInner {
    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub text_offset: Point,
    text: String,
    text_buffer: RcTextBuffer,
    bounds_width: f32,
    unclipped_text_size: Size,
    text_size_needs_calculated: bool,
    prev_bounds_size: Size,
    text_bounds_rect: Rect,
}

impl ParagraphInner {
    pub fn new(
        text: impl Into<String>,
        style: &LabelStyle,
        bounds_width: f32,
        res: &mut ResourceCtx,
        text_offset: Point,
    ) -> Self {
        let text: String = text.into();

        let width = (bounds_width - style.padding.left - style.padding.right)
            .max(style.padding.left + style.padding.right + style.min_clipped_size.width);

        // Use a temporary height for the text buffer.
        let text_buffer = RcTextBuffer::new(
            &text,
            style.properties,
            Some(width),
            None,
            false,
            &mut res.font_system,
        );

        Self {
            text_offset,
            text,
            text_buffer,
            bounds_width,
            // This will be overwritten later.
            unclipped_text_size: Size::default(),
            text_size_needs_calculated: true,
            prev_bounds_size: Size::new(-1.0, -1.0),
            // This will be overwritten later.
            text_bounds_rect: Rect::default(),
        }
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    pub fn desired_padded_size(&mut self, style: &LabelStyle) -> Size {
        let text_size = self.unclipped_text_size();

        Size::new(
            text_size.width + style.padding.left + style.padding.right,
            text_size.height + style.padding.top + style.padding.bottom,
        )
    }

    /// Returns the size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn unclipped_text_size(&mut self) -> Size {
        if self.text_size_needs_calculated {
            self.text_size_needs_calculated = false;

            self.unclipped_text_size = self.text_buffer.measure();
        }

        self.unclipped_text_size
    }

    /// Returns `true` if the text has changed.
    pub fn set_text(&mut self, text: &str, res: &mut ResourceCtx) -> bool {
        if &self.text != text {
            self.text = String::from(text);
            self.text_size_needs_calculated = true;

            self.text_buffer.set_text(text, &mut res.font_system);

            true
        } else {
            false
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_bounds_width(
        &mut self,
        bounds_width: f32,
        style: &LabelStyle,
        res: &mut ResourceCtx,
    ) {
        if self.bounds_width != bounds_width {
            self.bounds_width = bounds_width;

            let text_width = self.text_width(style);

            self.text_buffer
                .set_bounds(Some(text_width), None, &mut res.font_system);
            self.text_size_needs_calculated = true;
        }
    }

    pub fn bounds_width(&self) -> f32 {
        self.bounds_width
    }

    pub fn text_width(&self, style: &LabelStyle) -> f32 {
        (self.bounds_width - style.padding.left - style.padding.right)
            .max(style.padding.left + style.padding.right + style.min_clipped_size.width)
    }

    pub fn set_style(&mut self, style: &LabelStyle, res: &mut ResourceCtx) {
        self.text_buffer
            .set_text_and_props(&self.text, style.properties, &mut res.font_system);
        self.text_size_needs_calculated = true;
    }

    pub fn render_primitives(&mut self, bounds: Rect, style: &LabelStyle) -> LabelPrimitives {
        let mut needs_layout = self.text_size_needs_calculated;

        if self.prev_bounds_size != bounds.size {
            self.prev_bounds_size = bounds.size;
            needs_layout = true;
        }

        if needs_layout {
            if self.text_size_needs_calculated {
                self.text_size_needs_calculated = false;

                self.unclipped_text_size = self.text_buffer.measure();
            }

            self.text_bounds_rect = layout_text_bounds(
                bounds.size,
                self.unclipped_text_size,
                style.padding,
                style.min_clipped_size,
                style.vertical_align,
            );
        }

        let text = if !self.text.is_empty() {
            Some(TextPrimitive::new(
                self.text_buffer.clone(),
                bounds.origin
                    + self.text_bounds_rect.origin.to_vector()
                    + self.text_offset.to_vector(),
                style.font_color,
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

        LabelPrimitives { text, bg_quad }
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_text_offset(&mut self, offset: Point) -> bool {
        if self.text_offset != offset {
            self.text_offset = offset;
            true
        } else {
            false
        }
    }
}

pub struct ParagraphBuilder {
    pub text: String,
    pub text_offset: Point,
    pub bounds_width: Option<f32>,
    pub style: Rc<LabelStyle>,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl ParagraphBuilder {
    pub fn new(style: &Rc<LabelStyle>) -> Self {
        Self {
            text: String::new(),
            text_offset: Point::default(),
            bounds_width: None,
            style: Rc::clone(style),
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: None,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> Paragraph {
        ParagraphElement::create(self, cx)
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn text_offset(mut self, offset: Point) -> Self {
        self.text_offset = offset;
        self
    }

    pub const fn bounds_width(mut self, width: f32) -> Self {
        self.bounds_width = Some(width);
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

    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = Some(scissor_rect_id);
        self
    }
}

/// A Paragraph element with an optional quad background.
pub struct ParagraphElement {
    shared_state: Rc<RefCell<SharedState>>,
}

impl ParagraphElement {
    pub fn create<A: Clone + 'static>(
        builder: ParagraphBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> Paragraph {
        let ParagraphBuilder {
            text,
            text_offset,
            bounds_width,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id) = cx.z_index_and_scissor_rect_id(z_index, scissor_rect_id);

        let bounds_width = bounds_width.unwrap_or(bounding_rect.width());

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: ParagraphInner::new(text, &style, bounds_width, &mut cx.res, text_offset),
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
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        Paragraph { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for ParagraphElement {
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

        let paragraph_primitives = inner.render_primitives(Rect::from_size(cx.bounds_size), style);

        if let Some(quad_primitive) = paragraph_primitives.bg_quad {
            primitives.add(quad_primitive);
        }

        if let Some(text_primitive) = paragraph_primitives.text {
            primitives.set_z_index(1);
            primitives.add_text(text_primitive);
        }
    }
}

struct SharedState {
    inner: ParagraphInner,
    style: Rc<LabelStyle>,
}

/// A handle to a [`ParagraphElement`], a Paragraph with an optional quad background.
pub struct Paragraph {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl Paragraph {
    pub fn builder(style: &Rc<LabelStyle>) -> ParagraphBuilder {
        ParagraphBuilder::new(style)
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

    /// Returns the size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn unclipped_text_size(&self) -> Size {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .unclipped_text_size()
    }

    pub fn set_text(&mut self, text: &str, res: &mut ResourceCtx) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_text(text, res);

        if changed {
            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn set_bounds_width(&mut self, width: f32, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        if inner.bounds_width() != width {
            inner.set_bounds_width(width, style, res);
            self.el.notify_custom_state_change();
        }
    }

    pub fn bounds_width(&self) -> f32 {
        RefCell::borrow(&self.shared_state).inner.bounds_width()
    }

    pub fn set_style(&mut self, style: &Rc<LabelStyle>, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(style, res);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<LabelStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub fn set_text_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_text_offset(offset);

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

pub fn layout_text_bounds(
    bounds_size: Size,
    unclipped_text_size: Size,
    padding: Padding,
    min_clipped_size: Size,
    vertical_align: Align,
) -> Rect {
    if unclipped_text_size.is_empty() {
        return Rect::default();
    }

    let content_rect = crate::layout::layout_inner_rect_with_min_size(
        padding,
        Rect::from_size(bounds_size),
        min_clipped_size,
    );

    // We need to vertically align the text ourselves as rootvg/glyphon does not do this.
    let text_bounds_y = match vertical_align {
        Align::Start => content_rect.min_y(),
        Align::Center => {
            content_rect.min_y() + ((content_rect.height() - unclipped_text_size.height) * 0.5)
        }
        Align::End => content_rect.max_y() - unclipped_text_size.height,
    };

    Rect::new(
        Point::new(content_rect.min_x(), text_bounds_y),
        content_rect.size,
    )
}
