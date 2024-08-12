use rootvg::PrimitiveGroup;

pub use crate::style::QuadStyle;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::math::{Rect, ZIndex};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;

pub struct QuadElementBuilder {
    pub class: Option<&'static str>,
    pub z_index: Option<ZIndex>,
    pub rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl QuadElementBuilder {
    pub fn new() -> Self {
        Self {
            class: None,
            z_index: None,
            rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: None,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> QuadElement {
        QuadElementInternal::create(self, cx)
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

/// A simple element with a quad background.
pub struct QuadElementInternal;

impl QuadElementInternal {
    pub fn create<A: Clone + 'static>(
        builder: QuadElementBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> QuadElement {
        let QuadElementBuilder {
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let element_builder = ElementBuilder {
            element: Box::new(Self),
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        QuadElement { el }
    }
}

impl<A: Clone + 'static> Element<A> for QuadElementInternal {
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
        primitives.add(
            cx.res
                .style_system
                .get::<QuadStyle>(cx.class)
                .create_primitive(Rect::from_size(cx.bounds_size)),
        );
    }
}

/// A simple element with a quad background.
pub struct QuadElement {
    pub el: ElementHandle,
}

impl QuadElement {
    pub fn builder() -> QuadElementBuilder {
        QuadElementBuilder::new()
    }

    pub fn set_class(&mut self, class: &'static str) {
        if self.el.class() != class {
            self.el._notify_class_change(class);
        }
    }
}
