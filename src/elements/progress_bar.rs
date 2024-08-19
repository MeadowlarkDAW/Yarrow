use std::{cell::RefCell, rc::Rc};

use rootvg::{
    color::Rgba,
    math::{Rect, ZIndex},
    quad::Radius,
    Primitive,
};

use crate::{
    event::{ElementEvent, EventCaptureStatus},
    prelude::{ElementHandle, ElementStyle},
    style::{Background, BorderStyle, ClassID},
    view::element::{Element, ElementBuilder, ElementFlags},
    ScissorRectID, WindowContext,
};

use super::quad::QuadStyle;

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressBarStyle {
    pub back_quad: QuadStyle,
    pub fill_quad: QuadStyle,
}

impl Default for ProgressBarStyle {
    fn default() -> Self {
        Self {
            back_quad: QuadStyle::new(
                Background::Solid(Rgba::new(0, 0, 0, 255)),
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

pub struct ProgressBarBuilder {
    pub percentage: f32,
    pub class: Option<ClassID>,
    pub z_index: Option<ZIndex>,
    pub rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl ProgressBarBuilder {
    pub fn new() -> Self {
        Self {
            percentage: 0.0,
            class: None,
            z_index: None,
            rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: None,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> ProgressBar {
        ProgressBarElement::create(self, cx)
    }

    pub const fn percentage(mut self, percentage: f32) -> Self {
        self.percentage = percentage;
        self
    }

    /// The style class ID
    ///
    /// If this method is not used, then the current class from the window context will
    /// be used.
    pub const fn class(mut self, class: ClassID) -> Self {
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
        let percentage = self.shared_state.borrow().percentage;

        let progress_bar_style = cx.res.style_system.get::<ProgressBarStyle>(cx.class);

        // TODO: figure out how to construct a proper rect

        primitives.add(
            progress_bar_style
                .back_quad
                .create_primitive(cx.visible_bounds),
        );
        primitives.set_z_index(1);
        let mut progress_rect = cx.visible_bounds;
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
            scissor_rect_id,
        } = builder;
        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let shared_state = Rc::new(RefCell::new(SharedState { percentage }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: shared_state.clone(),
            }),
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
            class,
        };

        let el = cx.view.add_element(element_builder, cx.res, cx.clipboard);

        ProgressBar { el, shared_state }
    }
}

pub struct ProgressBar {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    pub percentage: f32,
}

impl ProgressBar {
    pub fn builder() -> ProgressBarBuilder {
        ProgressBarBuilder::new()
    }

    pub fn set_percent(&mut self, percent: f32) {
        let mut shared_state = self.shared_state.borrow_mut();
        shared_state.percentage = percent;
        self.el._notify_custom_state_change();
    }
}
