use rootvg::PrimitiveGroup;

use crate::derive::*;
use crate::prelude::*;

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[derive(Default)]
pub struct QuadElementBuilder {}

impl QuadElementBuilder {
    pub fn build<A: Clone + 'static>(self, window_cx: &mut WindowContext<'_, A>) -> QuadElement {
        let QuadElementBuilder {
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
        } = self;

        let el = ElementBuilder::new(QuadElementInternal)
            .builder_values(z_index, scissor_rect, class, window_cx)
            .rect(rect)
            .hidden(manually_hidden)
            .flags(ElementFlags::PAINTS)
            .build(window_cx);

        QuadElement { el }
    }
}

/// A simple element with a quad background.
struct QuadElementInternal;

impl<A: Clone + 'static> Element<A> for QuadElementInternal {
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
        primitives.add(
            cx.res
                .style_system
                .get::<QuadStyle>(cx.class)
                .create_primitive(Rect::from_size(cx.bounds_size)),
        );
    }
}

/// A simple element with a quad background.
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
#[element_handle_layout_aligned]
pub struct QuadElement {}

impl QuadElement {
    pub fn builder() -> QuadElementBuilder {
        QuadElementBuilder::default()
    }
}
