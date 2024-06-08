use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::quad::QuadPrimitive;
use rootvg::text::glyphon::FontSystem;
use rootvg::text::{RcTextBuffer, TextPrimitive, TextProperties};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::layout::{Align, Padding};
use crate::math::{Point, Rect, Size, ZIndex};
use crate::style::{Background, BorderStyle, QuadStyle};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;

/// The style of a [`Label`] element
#[derive(Debug, Clone, PartialEq)]
pub struct LabelStyle {
    /// The text properties.
    pub properties: TextProperties,

    /// The color of the font
    ///
    /// By default this is set to `color::WHITE`.
    pub font_color: RGBA8,

    /// The vertical alignment of the text.
    ///
    /// By default this is set to `Align::Center`.
    pub vertical_align: Align,

    /// The minimum size of the clipped text area.
    ///
    /// By default this is set to `Size::new(5.0, 5.0)`.
    pub min_clipped_size: Size,

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

impl LabelStyle {
    pub fn default_tooltip_style() -> Self {
        Self {
            back_quad: QuadStyle {
                bg: Background::Solid(RGBA8::new(30, 30, 30, 255)),
                border: BorderStyle {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: RGBA8::new(105, 105, 105, 255),
                    ..Default::default()
                },
            },
            padding: Padding::new(5.0, 5.0, 5.0, 5.0),
            ..Default::default()
        }
    }
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            properties: TextProperties::default(),
            font_color: color::WHITE,
            vertical_align: Align::Center,
            min_clipped_size: Size::new(5.0, 5.0),
            back_quad: QuadStyle::TRANSPARENT,
            padding: Padding::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LabelPrimitives {
    pub text: Option<TextPrimitive>,
    pub bg_quad: Option<QuadPrimitive>,
}

/// A reusable label struct that can be used by other elements.
pub struct LabelInner {
    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub text_offset: Point,
    text: String,
    text_buffer: RcTextBuffer,
    unclipped_text_size: Size,
    text_size_needs_calculated: bool,
    prev_bounds_size: Size,
    text_bounds_rect: Rect,
}

impl LabelInner {
    pub fn new(
        text: impl Into<String>,
        style: &LabelStyle,
        font_system: &mut FontSystem,
        text_offset: Point,
    ) -> Self {
        let text: String = text.into();

        // Use a temporary size for the text buffer.
        let text_buffer = RcTextBuffer::new(
            &text,
            style.properties,
            Size::new(1000.0, 200.0),
            false,
            font_system,
        );

        Self {
            text_offset,
            text,
            text_buffer,
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
    pub fn set_text(&mut self, text: &str, font_system: &mut FontSystem) -> bool {
        if &self.text != text {
            self.text = String::from(text);
            self.text_size_needs_calculated = true;

            self.text_buffer.set_text(text, font_system);

            true
        } else {
            false
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_style(&mut self, style: &LabelStyle, font_system: &mut FontSystem) {
        self.text_buffer
            .set_text_and_props(&self.text, style.properties, font_system);
        self.text_size_needs_calculated = true;
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &LabelStyle,
        font_system: &mut FontSystem,
    ) -> LabelPrimitives {
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

            self.text_buffer.set_bounds(
                Size::new(
                    self.text_bounds_rect.width(),
                    // Add some extra padding below so that text doesn't get clipped.
                    self.text_bounds_rect.height() + 2.0,
                ),
                font_system,
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

pub struct LabelBuilder {
    pub text: String,
    pub text_offset: Point,
    pub style: Rc<LabelStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl LabelBuilder {
    pub fn new(style: &Rc<LabelStyle>) -> Self {
        Self {
            text: String::new(),
            text_offset: Point::default(),
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> Label {
        LabelElement::create(self, cx)
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
pub struct LabelElement {
    shared_state: Rc<RefCell<SharedState>>,
}

impl LabelElement {
    pub fn create<A: Clone + 'static>(
        builder: LabelBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> Label {
        let LabelBuilder {
            text,
            text_offset,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: LabelInner::new(text, &style, cx.font_system, text_offset),
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

        Label { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for LabelElement {
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

        if let Some(text_primitive) = label_primitives.text {
            primitives.set_z_index(1);
            primitives.add_text(text_primitive);
        }
    }
}

struct SharedState {
    inner: LabelInner,
    style: Rc<LabelStyle>,
}

/// A handle to a [`LabelElement`], a label with an optional quad background.
pub struct Label {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl Label {
    pub fn builder(style: &Rc<LabelStyle>) -> LabelBuilder {
        LabelBuilder::new(style)
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

    pub fn set_text(&mut self, text: &str, font_sytem: &mut FontSystem) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_text(text, font_sytem);

        if changed {
            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn set_style(&mut self, style: &Rc<LabelStyle>, font_sytem: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(style, font_sytem);
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
