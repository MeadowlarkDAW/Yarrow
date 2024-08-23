use rootvg::PrimitiveGroup;

use crate::prelude::*;

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[derive(Default)]
pub struct QuadElementBuilder {}

impl QuadElementBuilder {
    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> QuadElement {
        QuadElementInternal::create(self, cx)
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
            scissor_rect,
        } = builder;

        let (z_index, scissor_rect, class) = cx.builder_values(z_index, scissor_rect, class);

        let element_builder = ElementBuilder {
            element: Box::new(Self),
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
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
