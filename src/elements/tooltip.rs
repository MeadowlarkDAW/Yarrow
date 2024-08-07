use std::cell::RefCell;
use std::rc::Rc;

use rootvg::text::TextProperties;
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::layout::{Align2, Padding};
use crate::math::{Point, Rect, ZIndex};
use crate::prelude::{ElementStyle, ResourceCtx};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;

use super::label::{LabelInner, LabelStyle};
use super::quad::QuadStyle;

/// The style of a [`Tooltip`] element
#[derive(Debug, Clone, PartialEq)]
pub struct TooltipStyle {
    /// The properties of the text.
    pub text_properties: TextProperties,

    /// The color of the text
    ///
    /// By default this is set to `color::WHITE`.
    pub text_color: RGBA8,

    /// The padding around the text.
    ///
    /// By default this has all values set to `6.0`.
    pub text_padding: Padding,

    /// The style of the padded background rectangle behind the text and icon.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background.
    ///
    /// By default this is set to `QuadStyle::TRANSPARENT`.
    pub back_quad: QuadStyle,
}

impl TooltipStyle {
    pub fn label_style(&self) -> LabelStyle {
        LabelStyle {
            text_properties: self.text_properties.clone(),
            text_color: self.text_color,
            text_padding: self.text_padding,
            back_quad: self.back_quad.clone(),
            ..Default::default()
        }
    }
}

impl Default for TooltipStyle {
    fn default() -> Self {
        Self {
            text_properties: Default::default(),
            text_color: color::WHITE,
            text_padding: Padding::default(),
            back_quad: QuadStyle::TRANSPARENT,
        }
    }
}

impl ElementStyle for TooltipStyle {
    const ID: &'static str = "tltip";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self {
            text_color: color::BLACK,
            ..Default::default()
        }
    }
}

pub struct TooltipBuilder {
    pub text_offset: Point,
    pub class: Option<&'static str>,
    pub element_padding: Padding,
    pub z_index: Option<ZIndex>,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl TooltipBuilder {
    pub fn new() -> Self {
        Self {
            text_offset: Point::default(),
            class: None,
            element_padding: Padding::new(10.0, 10.0, 10.0, 10.0),
            z_index: None,
            scissor_rect_id: None,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> Tooltip {
        TooltipElement::create(self, cx)
    }

    /// The padding between the tooltip and the element that is being hovered.
    ///
    /// By default this has a padding with all values set to `10.0`.
    pub const fn element_padding(mut self, padding: Padding) -> Self {
        self.element_padding = padding;
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn text_offset(mut self, offset: Point) -> Self {
        self.text_offset = offset;
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

    /// The ID of the scissoring rectangle this element belongs to.
    ///
    /// If this method is not used, then the current scissoring rectangle ID from the
    /// window context will be used.
    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = Some(scissor_rect_id);
        self
    }
}

pub struct TooltipElement {
    shared_state: Rc<RefCell<SharedState>>,
    element_padding: Padding,
}

impl TooltipElement {
    pub fn create<A: Clone + 'static>(
        builder: TooltipBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> Tooltip {
        let TooltipBuilder {
            text_offset,
            class,
            element_padding,
            z_index,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let style: &TooltipStyle = cx.res.style_system.get(class);

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: LabelInner::new(
                Some(String::new()),
                None,
                text_offset,
                Point::default(),
                1.0,
                Default::default(),
                &style.label_style(),
                &mut cx.res.font_system,
            ),
            show_with_info: None,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                element_padding,
            }),
            z_index,
            bounding_rect: Rect::default(),
            manually_hidden: true,
            scissor_rect_id,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

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
        match event {
            ElementEvent::CustomStateChanged => {
                cx.request_repaint();

                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState {
                    inner,
                    show_with_info,
                } = &mut *shared_state;

                if let Some((element_rect, align)) = show_with_info.take() {
                    let size = inner.desired_size(|| {
                        cx.res
                            .style_system
                            .get::<TooltipStyle>(cx.class())
                            .label_style()
                            .padding_info()
                    });

                    let origin =
                        align.align_floating_element(element_rect, size, self.element_padding);

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
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let style: &TooltipStyle = cx.res.style_system.get(cx.class);

        let label_primitives = shared_state.inner.render_primitives(
            Rect::from_size(cx.bounds_size),
            false,
            &style.label_style(),
            &mut cx.res.font_system,
        );

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
    show_with_info: Option<(Rect, Align2)>,
}

/// A handle to a [`TooltipElement`]
pub struct Tooltip {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl Tooltip {
    pub fn builder() -> TooltipBuilder {
        TooltipBuilder::new()
    }

    pub fn show(
        &mut self,
        message: &str,
        element_bounds: Rect,
        align: Align2,
        res: &mut ResourceCtx,
    ) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        shared_state
            .inner
            .set_text(Some(message), &mut res.font_system, || {
                res.style_system
                    .get::<TooltipStyle>(self.el.class())
                    .text_properties
            });

        shared_state.show_with_info = Some((element_bounds, align));

        self.el._notify_custom_state_change();
        self.el.set_hidden(false);
    }

    pub fn hide(&mut self) {
        RefCell::borrow_mut(&self.shared_state).show_with_info = None;

        self.el.set_hidden(true);
    }

    pub fn set_class(&mut self, class: &'static str, res: &mut ResourceCtx) {
        if self.el.class() != class {
            RefCell::borrow_mut(&self.shared_state)
                .inner
                .sync_new_style(
                    &res.style_system.get::<TooltipStyle>(class).label_style(),
                    &mut res.font_system,
                );

            self.el._notify_class_change(class);
        }
    }

    /// An offset that can be used mainly to correct the position of the text.
    ///
    /// This does not effect the position of the background quad.
    pub fn set_text_offset(&mut self, offset: Point) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.text_offset != offset {
            shared_state.inner.text_offset = offset;
            self.el._notify_custom_state_change();
        }
    }
}
