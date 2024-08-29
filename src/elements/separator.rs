use crate::derive::*;
use crate::prelude::*;

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
                flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
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

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[derive(Default)]
pub struct SeparatorBuilder {
    pub vertical: bool,
}

impl SeparatorBuilder {
    pub const fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    pub fn build<A: Clone + 'static>(self, window_cx: &mut WindowContext<'_, A>) -> Separator {
        let SeparatorBuilder {
            class,
            vertical,
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
        } = self;

        let el = ElementBuilder::new(SeparatorElement { vertical })
            .builder_values(z_index, scissor_rect, class, window_cx)
            .rect(rect)
            .hidden(manually_hidden)
            .flags(ElementFlags::PAINTS)
            .build(window_cx);

        Separator { el }
    }
}

/// A simple separator element.
struct SeparatorElement {
    vertical: bool,
}

impl<A: Clone + 'static> Element<A> for SeparatorElement {
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
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
#[element_handle_layout_aligned]
pub struct Separator {}

impl Separator {
    pub fn builder() -> SeparatorBuilder {
        SeparatorBuilder::default()
    }
}
