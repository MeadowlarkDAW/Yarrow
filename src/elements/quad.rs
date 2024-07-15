use std::cell::RefCell;
use std::rc::Rc;

use rootvg::PrimitiveGroup;

pub use crate::style::QuadStyle;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::math::{Rect, ZIndex};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;

pub struct QuadElementBuilder {
    pub style: Rc<QuadStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl QuadElementBuilder {
    pub fn new(style: &Rc<QuadStyle>) -> Self {
        Self {
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> QuadElement {
        QuadElementInternal::create(self, cx)
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

/// A simple element with a quad background.
pub struct QuadElementInternal {
    shared_state: Rc<RefCell<SharedState>>,
}

impl QuadElementInternal {
    pub fn create<A: Clone + 'static>(
        builder: QuadElementBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> QuadElement {
        let QuadElementBuilder {
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState { style }));

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

        QuadElement { el, shared_state }
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
            RefCell::borrow(&self.shared_state)
                .style
                .create_primitive(Rect::from_size(cx.bounds_size)),
        );
    }
}

/// A simple element with a quad background.
pub struct QuadElement {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl QuadElement {
    pub fn builder(style: &Rc<QuadStyle>) -> QuadElementBuilder {
        QuadElementBuilder::new(style)
    }

    pub fn set_style(&mut self, style: &Rc<QuadStyle>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<QuadStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }
}

struct SharedState {
    style: Rc<QuadStyle>,
}
