use std::{cell::RefCell, rc::Rc};

use rootvg::{
    color::RGBA8,
    math::{Point, Rect, Size, ZIndex},
    PrimitiveGroup,
};

use crate::{
    event::{ElementEvent, EventCaptureStatus},
    layout::Align,
    style::{Background, BorderStyle, QuadStyle},
    view::element::{
        Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
    },
    ScissorRectID, WindowContext, MAIN_SCISSOR_RECT,
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SeparatorSizeType {
    Scale(f32),
    FixedPoints(f32),
    FixedPadding {
        padding_points: f32,
        min_size_points: f32,
    },
}

impl SeparatorSizeType {
    pub fn points(&self, span_points: f32) -> f32 {
        match self {
            Self::Scale(s) => *s * span_points,
            Self::FixedPoints(p) => *p,
            Self::FixedPadding {
                padding_points,
                min_size_points,
            } => (span_points - *padding_points - *padding_points).max(*min_size_points),
        }
    }
}

impl Default for SeparatorSizeType {
    fn default() -> Self {
        Self::Scale(1.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeparatorStyle {
    pub quad_style: QuadStyle,
    pub size: SeparatorSizeType,
    pub align: Align,
}

impl Default for SeparatorStyle {
    fn default() -> Self {
        Self {
            quad_style: QuadStyle {
                bg: Background::Solid(RGBA8::new(150, 150, 150, 40)),
                border: BorderStyle::default(),
            },
            size: SeparatorSizeType::default(),
            align: Align::Center,
        }
    }
}

pub struct SeparatorBuilder {
    pub style: Rc<SeparatorStyle>,
    pub vertical: bool,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl SeparatorBuilder {
    pub fn new(style: &Rc<SeparatorStyle>) -> Self {
        Self {
            style: Rc::clone(style),
            vertical: false,
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> Separator {
        SeparatorElement::create(self, cx)
    }

    pub const fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
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

/// A simple separator element.
pub struct SeparatorElement {
    shared_state: Rc<RefCell<SharedState>>,
    vertical: bool,
}

impl SeparatorElement {
    pub fn create<A: Clone + 'static>(
        builder: SeparatorBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> Separator {
        let SeparatorBuilder {
            style,
            vertical,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState { style }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                vertical,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        Separator { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for SeparatorElement {
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
        let shared_state = RefCell::borrow(&self.shared_state);

        let rect = if self.vertical {
            let span = shared_state.style.size.points(cx.bounds_size.height);

            let y = match shared_state.style.align {
                Align::Start => 0.0,
                Align::Center => (cx.bounds_size.height - span) * 0.5,
                Align::End => cx.bounds_size.height - span,
            };

            Rect::new(Point::new(0.0, y), Size::new(cx.bounds_size.width, span))
        } else {
            let span = shared_state.style.size.points(cx.bounds_size.width);

            let x = match shared_state.style.align {
                Align::Start => 0.0,
                Align::Center => (cx.bounds_size.width - span) * 0.5,
                Align::End => cx.bounds_size.width - span,
            };

            Rect::new(Point::new(x, 0.0), Size::new(span, cx.bounds_size.height))
        };

        primitives.add(shared_state.style.quad_style.create_primitive(rect));
    }
}

/// A simple separator element.
pub struct Separator {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl Separator {
    pub fn builder(style: &Rc<SeparatorStyle>) -> SeparatorBuilder {
        SeparatorBuilder::new(style)
    }

    pub fn set_style(&mut self, style: &Rc<SeparatorStyle>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<SeparatorStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }
}

struct SharedState {
    style: Rc<SeparatorStyle>,
}
