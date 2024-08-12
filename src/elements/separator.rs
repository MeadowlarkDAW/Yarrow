use rootvg::{
    color::RGBA8,
    math::{Point, Rect, Size, ZIndex},
    PrimitiveGroup,
};

use crate::{
    event::{ElementEvent, EventCaptureStatus},
    layout::Align,
    prelude::ElementStyle,
    style::{Background, BorderStyle, QuadStyle},
    view::element::{
        Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
    },
    ScissorRectID, WindowContext,
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

impl ElementStyle for SeparatorStyle {
    const ID: &'static str = "sptr";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self::default()
    }
}

pub struct SeparatorBuilder {
    pub class: Option<&'static str>,
    pub vertical: bool,
    pub z_index: Option<ZIndex>,
    pub rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl SeparatorBuilder {
    pub fn new() -> Self {
        Self {
            class: None,
            vertical: false,
            z_index: None,
            rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: None,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> Separator {
        SeparatorElement::create(self, cx)
    }

    pub const fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
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

/// A simple separator element.
pub struct SeparatorElement {
    vertical: bool,
}

impl SeparatorElement {
    pub fn create<A: Clone + 'static>(
        builder: SeparatorBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> Separator {
        let SeparatorBuilder {
            class,
            vertical,
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let element_builder = ElementBuilder {
            element: Box::new(Self { vertical }),
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        Separator { el }
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
        let style = cx.res.style_system.get::<SeparatorStyle>(cx.class);

        let rect = if self.vertical {
            let span = style.size.points(cx.bounds_size.height);

            let y = match style.align {
                Align::Start => 0.0,
                Align::Center => (cx.bounds_size.height - span) * 0.5,
                Align::End => cx.bounds_size.height - span,
            };

            Rect::new(Point::new(0.0, y), Size::new(cx.bounds_size.width, span))
        } else {
            let span = style.size.points(cx.bounds_size.width);

            let x = match style.align {
                Align::Start => 0.0,
                Align::Center => (cx.bounds_size.width - span) * 0.5,
                Align::End => cx.bounds_size.width - span,
            };

            Rect::new(Point::new(x, 0.0), Size::new(span, cx.bounds_size.height))
        };

        primitives.add(style.quad_style.create_primitive(rect));
    }
}

/// A simple separator element.
pub struct Separator {
    pub el: ElementHandle,
}

impl Separator {
    pub fn builder() -> SeparatorBuilder {
        SeparatorBuilder::new()
    }

    pub fn set_class(&mut self, class: &'static str) {
        if self.el.class() != class {
            self.el._notify_class_change(class);
        }
    }
}
