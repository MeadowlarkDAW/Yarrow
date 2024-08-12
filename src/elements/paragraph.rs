use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::color::{self, RGBA8};
use rootvg::text::{FontSystem, RcTextBuffer, TextPrimitive, TextProperties};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::layout::{Align, Align2, Padding};
use crate::math::{Point, Rect, Size, Vector, ZIndex};
use crate::prelude::{ElementStyle, ResourceCtx};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;

use super::label::LabelPrimitives;
use super::quad::QuadStyle;

// TODO: Add ability to select, copy, and right-click text.

/// The style of a [`Paragraph`] element
#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphStyle {
    /// The text properties.
    pub properties: TextProperties,

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
            properties: TextProperties {
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
            style.properties,
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
    pub fn set_text(&mut self, text: &str, font_system: &mut FontSystem) -> bool {
        if &self.text != text {
            self.text = String::from(text);
            self.text_size_needs_calculated = true;
            self.padded_size_needs_calculated = true;

            self.text_buffer.set_text(text, font_system);

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
            .set_text_and_props(&self.text, style.properties, font_system);
        self.text_size_needs_calculated = true;
        self.padded_size_needs_calculated = true;
    }

    pub fn render_primitives(&mut self, bounds: Rect, style: &ParagraphStyle) -> LabelPrimitives {
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

pub struct ParagraphBuilder {
    pub text: String,
    pub text_offset: Vector,
    pub bounds_width: Option<f32>,
    pub class: Option<&'static str>,
    pub z_index: Option<ZIndex>,
    pub rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl ParagraphBuilder {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            text_offset: Vector::default(),
            bounds_width: None,
            class: None,
            z_index: None,
            rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: None,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> Paragraph {
        ParagraphElement::create(self, cx)
    }

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

    /// The style class name
    ///
    /// If this method is not used, then the current class from the window context will
    /// be used.
    pub const fn class(mut self, class: &'static str) -> Self {
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

    /// The bounding rectangle of the element
    ///
    /// If this method is not used, then the element will have a size and position of
    /// zero and will not be visible until its bounding rectangle is set.
    pub const fn rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    /// Whether or not this element is manually hidden
    ///
    /// By default this is set to `false`.
    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.manually_hidden = hidden;
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
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let style = cx.res.style_system.get(class);

        let bounds_width = bounds_width.unwrap_or(rect.width());

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: ParagraphInner::new(
                text,
                &style,
                bounds_width,
                &mut cx.res.font_system,
                text_offset,
            ),
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
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

        let paragraph_primitives = shared_state.inner.render_primitives(
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
pub struct Paragraph {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl Paragraph {
    pub fn builder() -> ParagraphBuilder {
        ParagraphBuilder::new()
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn desired_size(&self, res: &mut ResourceCtx) -> Size {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .desired_size(|| {
                res.style_system
                    .get::<ParagraphStyle>(self.el.class())
                    .padding
            })
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
            .set_text(text, &mut res.font_system);

        if changed {
            self.el._notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn set_bounds_width(&mut self, width: f32, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.bounds_width() != width {
            shared_state.inner.set_bounds_width(
                width,
                res.style_system.get(self.el.class()),
                &mut res.font_system,
            );
            self.el._notify_custom_state_change();
        }
    }

    pub fn bounds_width(&self) -> f32 {
        RefCell::borrow(&self.shared_state).inner.bounds_width()
    }

    pub fn set_class(&mut self, class: &'static str, res: &mut ResourceCtx) {
        if self.el.class() != class {
            RefCell::borrow_mut(&self.shared_state)
                .inner
                .sync_new_style(res.style_system.get(class), &mut res.font_system);

            self.el._notify_class_change(class);
        }
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub fn set_text_offset(&mut self, offset: Vector) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.text_offset != offset {
            shared_state.inner.text_offset = offset;
            self.el._notify_custom_state_change();
        }
    }

    pub fn layout(&mut self, origin: Point, res: &mut ResourceCtx) {
        let size = self.desired_size(res);
        self.el.set_rect(Rect::new(origin, size));
    }

    pub fn layout_aligned(&mut self, point: Point, align: Align2, res: &mut ResourceCtx) {
        let size = self.desired_size(res);
        self.el.set_rect(align.align_rect_to_point(point, size));
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
