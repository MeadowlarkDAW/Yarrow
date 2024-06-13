use std::cell::RefCell;
use std::rc::Rc;

use rootvg::text::glyphon::FontSystem;
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::layout::{Align2, Padding};
use crate::math::{Point, Rect, ZIndex};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;

use super::label::{LabelInner, LabelStyle};

pub struct TooltipBuilder {
    pub text_offset: Point,
    pub style: Rc<LabelStyle>,
    pub padding: Padding,
    pub z_index: ZIndex,
    pub scissor_rect_id: ScissorRectID,
}

impl TooltipBuilder {
    pub fn new(style: &Rc<LabelStyle>) -> Self {
        Self {
            text_offset: Point::default(),
            style: Rc::clone(style),
            padding: Padding::new(5.0, 5.0, 5.0, 5.0),
            z_index: 0,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> Tooltip {
        TooltipElement::create(self, cx)
    }

    /// The padding between the tooltip and the element that is being hovered.
    ///
    /// By default this has a padding with all values set to `5.0`.
    pub const fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
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

    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = scissor_rect_id;
        self
    }
}

pub struct TooltipElement {
    shared_state: Rc<RefCell<SharedState>>,
    padding: Padding,
}

impl TooltipElement {
    pub fn create<A: Clone + 'static>(
        builder: TooltipBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> Tooltip {
        let TooltipBuilder {
            text_offset,
            style,
            padding,
            z_index,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: LabelInner::new(String::new(), &style, cx.font_system, text_offset),
            style,
            show_with_info: None,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                padding,
            }),
            z_index,
            bounding_rect: Rect::default(),
            manually_hidden: true,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, cx.font_system, cx.clipboard);

        Tooltip { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for TooltipElement {
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

            let mut shared_state = RefCell::borrow_mut(&self.shared_state);
            let SharedState {
                inner,
                style,
                show_with_info,
            } = &mut *shared_state;

            if let Some((element_rect, align)) = show_with_info.take() {
                let size = inner.desired_padded_size(style);

                let origin = align.align_floating_element(element_rect, size, self.padding);

                let mut rect = Rect::new(origin, size);
                let window_rect = Rect::from_size(cx.window_size());

                if rect.min_x() < window_rect.min_x() {
                    rect.origin.x = 0.0;
                }
                if rect.max_x() > window_rect.max_x() {
                    rect.origin.x = window_rect.max_x() - rect.size.width;
                }
                if rect.min_y() < window_rect.min_y() {
                    rect.origin.y = 0.0;
                }
                if rect.max_y() > window_rect.max_y() {
                    rect.origin.y = window_rect.max_y() - rect.size.height;
                }

                cx.set_bounding_rect(rect);
            }
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style, .. } = &mut *shared_state;

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
    show_with_info: Option<(Rect, Align2)>,
}

/// A handle to a [`TooltipElement`]
pub struct Tooltip {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl Tooltip {
    pub fn builder(style: &Rc<LabelStyle>) -> TooltipBuilder {
        TooltipBuilder::new(style)
    }

    pub fn show(
        &mut self,
        message: &str,
        element_bounds: Rect,
        align: Align2,
        font_sytem: &mut FontSystem,
    ) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        shared_state.inner.set_text(message, font_sytem);

        shared_state.show_with_info = Some((element_bounds, align));

        self.el.notify_custom_state_change();
        self.el.set_hidden(false);
    }

    pub fn hide(&mut self) {
        RefCell::borrow_mut(&self.shared_state).show_with_info = None;

        self.el.set_hidden(true);
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
