use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::derive::*;
use crate::prelude::*;
use crate::vg::text::{RcTextBuffer, TextPrimitive};

use super::label::LabelPrimitives;

// TODO: Add ability to select, copy, and right-click text.

/// The style of a [`Paragraph`] element
#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphStyle {
    /// The text properties.
    pub text_properties: TextProperties,

    /// The color of the font
    ///
    /// By default this is set to `color::WHITE`.
    pub text_color: RGBA8,

    /// The vertical alignment of the text.
    ///
    /// By default this is set to `Align::Center`.
    pub vertical_align: Align,

    /// The style of the padded background rectangle behind the text.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    ///
    /// By default this is set to `QuadStyle::TRANSPARENT`.
    pub back_quad: QuadStyle,

    /// The padding between the text and the bounding rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub padding: Padding,
}

impl Default for ParagraphStyle {
    fn default() -> Self {
        Self {
            text_properties: TextProperties {
                shaping: rootvg::text::Shaping::Advanced,
                wrap: rootvg::text::Wrap::WordOrGlyph,
                ..Default::default()
            },
            text_color: color::WHITE,
            vertical_align: Align::Center,
            back_quad: QuadStyle::TRANSPARENT,
            padding: Padding::default(),
        }
    }
}

impl ElementStyle for ParagraphStyle {
    const ID: &'static str = "prgph";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self {
            text_color: color::BLACK,
            ..Default::default()
        }
    }
}

/// A reusable Paragraph struct that can be used by other elements.
pub struct ParagraphInner {
    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub text_offset: Vector,
    text: String,
    text_buffer: RcTextBuffer,
    bounds_width: f32,
    unclipped_text_size: Size,
    text_size_needs_calculated: bool,
    padded_size_needs_calculated: bool,
    prev_bounds_size: Size,
    text_bounds_rect: Rect,
    padded_size: Size,
}

impl ParagraphInner {
    pub fn new(
        text: impl Into<String>,
        style: &ParagraphStyle,
        bounds_width: f32,
        font_system: &mut FontSystem,
        text_offset: Vector,
    ) -> Self {
        let text: String = text.into();

        let width = (bounds_width - style.padding.left - style.padding.right)
            .max(style.padding.left + style.padding.right);

        let text_buffer = RcTextBuffer::new(
            &text,
            style.text_properties,
            Some(width),
            None,
            false,
            font_system,
        );

        Self {
            text_offset,
            text,
            text_buffer,
            bounds_width,
            // This will be overwritten later.
            unclipped_text_size: Size::default(),
            text_size_needs_calculated: true,
            padded_size_needs_calculated: true,
            prev_bounds_size: Size::new(-1.0, -1.0),
            // This will be overwritten later.
            text_bounds_rect: Rect::default(),
            padded_size: Size::default(),
        }
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    ///
    /// If the padded size needs calculated, then the given closure will be used to
    /// extract the padding from the style.
    pub fn desired_size<F: FnOnce() -> Padding>(&mut self, get_padding: F) -> Size {
        if self.padded_size_needs_calculated {
            self.padded_size_needs_calculated = false;

            let padding = (get_padding)();

            let text_size = self.unclipped_text_size();

            self.padded_size = Size::new(
                text_size.width + padding.left + padding.right,
                text_size.height + padding.top + padding.bottom,
            )
        }

        self.padded_size
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
    pub fn set_text<T: AsRef<str> + Into<String>>(
        &mut self,
        text: T,
        font_system: &mut FontSystem,
    ) -> bool {
        // TODO: If the text is sufficiently large, use a hash for comparison
        // for better performance.
        if self.text.as_str() != text.as_ref() {
            self.text = text.into();
            self.text_size_needs_calculated = true;
            self.padded_size_needs_calculated = true;

            self.text_buffer.set_text(&self.text, font_system);

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
        style: &ParagraphStyle,
        font_system: &mut FontSystem,
    ) {
        if self.bounds_width != bounds_width {
            self.bounds_width = bounds_width;

            let text_width = self.text_width(style);

            self.text_buffer
                .set_bounds(Some(text_width), None, font_system);
            self.text_size_needs_calculated = true;
            self.padded_size_needs_calculated = true;
        }
    }

    pub fn bounds_width(&self) -> f32 {
        self.bounds_width
    }

    pub fn text_width(&self, style: &ParagraphStyle) -> f32 {
        (self.bounds_width - style.padding.left - style.padding.right)
            .max(style.padding.left + style.padding.right)
    }

    pub fn sync_new_style(&mut self, style: &ParagraphStyle, font_system: &mut FontSystem) {
        self.text_buffer
            .set_text_and_props(&self.text, style.text_properties, font_system);
        self.text_size_needs_calculated = true;
        self.padded_size_needs_calculated = true;
    }

    pub fn render(&mut self, bounds: Rect, style: &ParagraphStyle) -> LabelPrimitives {
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
                style.vertical_align,
            );
        }

        let text = if !self.text.is_empty() {
            Some(TextPrimitive::new(
                self.text_buffer.clone(),
                bounds.origin + self.text_bounds_rect.origin.to_vector() + self.text_offset,
                style.text_color,
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

        LabelPrimitives {
            icon: None,
            text,
            bg_quad,
        }
    }
}

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[derive(Default)]
pub struct ParagraphBuilder {
    pub text: String,
    pub text_offset: Vector,
    pub bounds_width: Option<f32>,
}

impl ParagraphBuilder {
    /// The text of the paragraph
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// The width of the paragraph
    ///
    /// If this method isn't used, then the width of the bounding rectangle will
    /// be used instead.
    pub const fn bounds_width(mut self, width: f32) -> Self {
        self.bounds_width = Some(width);
        self
    }

    /// An offset that can be used mainly to correct the position of the text.
    /// This does not effect the position of the background quad.
    ///
    /// By default this is set to an offset of zero.
    pub const fn text_offset(mut self, offset: Vector) -> Self {
        self.text_offset = offset;
        self
    }

    pub fn build<A: Clone + 'static>(self, window_cx: &mut WindowContext<'_, A>) -> Paragraph {
        let ParagraphBuilder {
            text,
            text_offset,
            bounds_width,
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
        } = self;

        let style = window_cx
            .res
            .style_system
            .get(window_cx.builder_class(class));

        let bounds_width = bounds_width.unwrap_or(rect.width());

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: ParagraphInner::new(
                text,
                &style,
                bounds_width,
                &mut window_cx.res.font_system,
                text_offset,
            ),
        }));

        let el = ElementBuilder::new(ParagraphElement {
            shared_state: Rc::clone(&shared_state),
        })
        .builder_values(z_index, scissor_rect, class, window_cx)
        .rect(rect)
        .hidden(manually_hidden)
        .flags(ElementFlags::PAINTS)
        .build(window_cx);

        Paragraph { el, shared_state }
    }
}

/// A Paragraph element with an optional quad background.
struct ParagraphElement {
    shared_state: Rc<RefCell<SharedState>>,
}

impl<A: Clone + 'static> Element<A> for ParagraphElement {
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

    fn render(&mut self, cx: RenderContext, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let paragraph_primitives = shared_state.inner.render(
            Rect::from_size(cx.bounds_size),
            cx.res.style_system.get(cx.class),
        );

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
}

/// A handle to a [`ParagraphElement`], a Paragraph with an optional quad background.
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
pub struct Paragraph {
    shared_state: Rc<RefCell<SharedState>>,
}

impl Paragraph {
    pub fn builder() -> ParagraphBuilder {
        ParagraphBuilder::default()
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    ///
    /// This size is automatically cached, so it should be relatively
    /// inexpensive to call.
    pub fn desired_size(&self, res: &mut ResourceCtx) -> Size {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .desired_size(|| {
                res.style_system
                    .get::<ParagraphStyle>(self.el.class())
                    .padding
            })
    }

    /// Returns the size of the unclipped text (not including the padding
    /// background rectangle).
    ///
    /// This size is automatically cached, so it should be relatively
    /// inexpensive to call.
    pub fn unclipped_text_size(&self) -> Size {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .unclipped_text_size()
    }

    /// Set the text.
    ///
    /// Returns `true` if the text has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed.
    /// However, calling this method can be expensive if the text is particuarly
    /// long, so prefer to call this method sparingly.
    pub fn set_text<T: AsRef<str> + Into<String>>(
        &mut self,
        text: T,
        res: &mut ResourceCtx,
    ) -> bool {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_text(text, &mut res.font_system);

        if changed {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    /// Set the width of the bounding rectangle while correctly wrapping
    /// the text.
    ///
    /// Returns `true` if the bounds width has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_bounds_width(&mut self, width: f32, res: &mut ResourceCtx) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.bounds_width() != width {
            shared_state.inner.set_bounds_width(
                width,
                res.style_system.get(self.el.class()),
                &mut res.font_system,
            );
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn bounds_width(&self) -> f32 {
        RefCell::borrow(&self.shared_state).inner.bounds_width()
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the offset has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_text_offset(&mut self, offset: Vector) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.text_offset != offset {
            shared_state.inner.text_offset = offset;
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

pub fn layout_text_bounds(
    bounds_size: Size,
    unclipped_text_size: Size,
    padding: Padding,
    vertical_align: Align,
) -> Rect {
    if unclipped_text_size.is_empty() {
        return Rect::default();
    }

    let content_rect = crate::layout::layout_inner_rect_with_min_size(
        padding,
        Rect::from_size(bounds_size),
        Size::default(),
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
