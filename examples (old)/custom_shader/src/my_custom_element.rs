mod pipeline;
use pipeline::MyCustomPrimitive;

use yarrow::derive::*;
use yarrow::prelude::*;
use yarrow::vg::pipeline::CustomPrimitive;

#[element_builder]
#[element_builder_rect]
#[derive(Default)]
pub struct MyCustomElementBuilder {}

impl MyCustomElementBuilder {
    pub fn build<A: Clone + 'static>(
        self,
        window_cx: &mut WindowContext<'_, A>,
    ) -> MyCustomElement {
        let el = ElementBuilder::new(MyCustomElementInternal::new())
            .builder_values(self.z_index, self.scissor_rect, None, window_cx)
            .rect(self.rect)
            .flags(ElementFlags::PAINTS)
            .build(window_cx);

        MyCustomElement { el }
    }
}

struct MyCustomElementInternal {
    // Note, if your custom primitive is particuarly large or if it
    // contains heap-allocated data, then consider wrapping that data
    // inside of an `Rc<RefCell<T>>` and storing it here so that you
    // don't have to clone/reconstruct the entire contents to create
    // a new updated primitive in `render()`.
}

impl MyCustomElementInternal {
    fn new() -> Self {
        Self {}
    }
}

impl<A: Clone + 'static> Element<A> for MyCustomElementInternal {
    fn render(&mut self, cx: RenderContext, primitives: &mut PrimitiveGroup) {
        // Custom primitives need to get the arena ID of the pipeline.
        let pipeline_id = cx.custom_pipelines.get_id(
            "my_custom_pipeline",
            // If the pipeline doesn't exist in this window yet, create one.
            || {
                pipeline::MyCustomPrimitivePipeline::new(
                    cx.device,
                    cx.texture_format,
                    cx.multisample,
                )
            },
            cx.vg,
        );

        primitives.add_custom_primitive(CustomPrimitive::new(
            MyCustomPrimitive::new(RGBA8::new(255, 0, 0, 255), Point::default(), cx.bounds_size),
            pipeline_id,
        ));
    }
}

#[element_handle]
#[element_handle_set_rect]
#[element_handle_layout_aligned]
pub struct MyCustomElement {}

impl MyCustomElement {
    pub fn builder() -> MyCustomElementBuilder {
        MyCustomElementBuilder::default()
    }
}
