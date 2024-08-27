use std::{cell::RefCell, rc::Rc};

use rootvg::{
    color::Rgba,
    math::{Point, Rect, Size, Vector, ZIndex},
    quad::Radius,
    Primitive,
};
use yarrow_derive::{
    element_builder, element_builder_class, element_builder_disabled, element_builder_hidden,
    element_builder_rect, element_handle, element_handle_class, element_handle_set_rect,
};

use crate::{
    event::{ElementEvent, EventCaptureStatus},
    prelude::{ElementHandle, ElementStyle},
    style::{Background, BorderStyle, ClassID, QuadStyle},
    view::element::{Element, ElementBuilder, ElementFlags},
    ScissorRectID, WindowContext,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressBarStyle {
    pub back_quad: QuadStyle,
    pub fill_quad: QuadStyle,
}

impl Default for ProgressBarStyle {
    fn default() -> Self {
        Self {
            back_quad: QuadStyle::new(
                Background::Solid(Rgba::new(255, 0, 0, 255)),
                BorderStyle::from_radius(Radius::all_same(1.5)),
            ),
            fill_quad: QuadStyle::new(
                Background::Solid(Rgba::new(255, 255, 255, 255)),
                BorderStyle::from_radius(Radius::all_same(1.5)),
            ),
        }
    }
}

impl ElementStyle for ProgressBarStyle {
    const ID: &'static str = "pgrsbar";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self::default()
    }
}

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[derive(Default)]
pub struct ProgressBarBuilder {
    pub percentage: f32,
}

impl ProgressBarBuilder {
    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> ProgressBar {
        ProgressBarElement::create(self, cx)
    }

    pub const fn percentage(mut self, percentage: f32) -> Self {
        self.percentage = percentage;
        self
    }
}

pub struct ProgressBarElement {
    shared_state: Rc<RefCell<SharedState>>,
}

impl<A: Clone + 'static> Element<A> for ProgressBarElement {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS
    }

    fn on_event(
        &mut self,
        event: crate::prelude::ElementEvent,
        cx: &mut crate::view::element::ElementContext<'_, A>,
    ) -> crate::prelude::EventCaptureStatus {
        if event == ElementEvent::CustomStateChanged {
            cx.request_repaint();
            EventCaptureStatus::Captured
        } else {
            EventCaptureStatus::NotCaptured
        }
    }

    fn render_primitives(
        &mut self,
        cx: crate::view::element::RenderContext<'_>,
        primitives: &mut rootvg::PrimitiveGroup,
    ) {
        println!("render called");
        let percentage = self.shared_state.borrow().percentage;

        let progress_bar_style = cx.res.style_system.get::<ProgressBarStyle>(cx.class);

        // TODO: figure out how to construct a proper rect
        //
        let rect = Rect::from_size(cx.bounds_size);

        primitives.add(progress_bar_style.back_quad.create_primitive(rect));
        primitives.set_z_index(1);
        let mut progress_rect = rect;
        progress_rect.size.width *= percentage;
        primitives.add(progress_bar_style.fill_quad.create_primitive(progress_rect))
    }
}

impl ProgressBarElement {
    pub fn create<A: Clone>(
        builder: ProgressBarBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> ProgressBar {
        let ProgressBarBuilder {
            percentage,
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
        } = builder;
        let (z_index, scissor_rect, class) = cx.builder_values(z_index, scissor_rect, class);

        let shared_state = Rc::new(RefCell::new(SharedState { percentage }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: shared_state.clone(),
            }),
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
            class,
        };

        let el = cx.view.add_element(element_builder, cx.res, cx.clipboard);

        ProgressBar { el, shared_state }
    }
}

#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
pub struct ProgressBar {
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    pub percentage: f32,
}

impl ProgressBar {
    pub fn builder() -> ProgressBarBuilder {
        ProgressBarBuilder::default()
    }

    pub fn set_percent(&mut self, percent: f32) {
        let mut shared_state = self.shared_state.borrow_mut();
        shared_state.percentage = percent;
        self.el.notify_custom_state_change();
    }
}
