use std::cell::RefCell;
use std::rc::Rc;

use crate::prelude::*;

use super::label::LabelInner;

/// Tooltip data assigned to an element
#[derive(Default, Debug, Clone, PartialEq)]
pub struct TooltipData {
    /// The tooltip text
    pub text: String,
    /// Where to align the tooltip relative to this element
    pub align: Align2,
}

impl TooltipData {
    /// Construct tooltip data for an element
    ///
    /// * `text` - The tooltip text
    /// * `align` - Where to align the tooltip relative to this element
    pub fn new(text: impl Into<String>, align: Align2) -> Self {
        Self {
            text: text.into(),
            align,
        }
    }
}

/// A struct that can be used by elements to simplify tooltip handling
pub struct TooltipInner {
    pub data: Option<TooltipData>,
}

impl TooltipInner {
    pub fn new(data: Option<TooltipData>) -> Self {
        Self { data }
    }

    pub fn set_data(&mut self, text: Option<&str>, align: Align2) -> bool {
        let mut state_changed = false;

        if let Some(old_data) = &mut self.data {
            if let Some(text) = text {
                if &old_data.text != text || old_data.align != align {
                    old_data.text = String::from(text);
                    old_data.align = align;
                    state_changed = true;
                }
            } else {
                self.data = None;
                state_changed = true;
            }
        } else if let Some(text) = text {
            self.data = Some(TooltipData {
                text: String::from(text),
                align,
            });
            state_changed = true;
        }

        state_changed
    }

    pub fn handle_event<A: Clone + 'static>(
        &self,
        event: &ElementEvent,
        disabled: bool,
        cx: &mut ElementContext<'_, A>,
    ) {
        if disabled || self.data.is_none() {
            return;
        }

        match event {
            ElementEvent::Pointer(PointerEvent::Moved { just_entered, .. }) => {
                if *just_entered {
                    cx.start_hover_timeout();
                }
            }
            ElementEvent::Pointer(PointerEvent::HoverTimeout { .. }) => {
                cx.show_tooltip(self.data.clone().unwrap(), true);
            }
            _ => {}
        }
    }
}

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
    pub text_offset: Vector,
    pub class: Option<ClassID>,
    pub element_padding: Padding,
    pub z_index: Option<ZIndex>,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl TooltipBuilder {
    pub fn new() -> Self {
        Self {
            text_offset: Vector::default(),
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
    pub const fn text_offset(mut self, offset: Vector) -> Self {
        self.text_offset = offset;
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

        let (z_index, scissor_rect, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let style: &TooltipStyle = cx.res.style_system.get(class);

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: LabelInner::new(
                Some(String::new()),
                None,
                text_offset,
                Vector::default(),
                None,
                IconScale::default(),
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
            rect: Rect::default(),
            manually_hidden: true,
            scissor_rect,
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

                    cx.set_rect(rect);
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
#[element_handle]
#[element_handle_class]
pub struct Tooltip {
    shared_state: Rc<RefCell<SharedState>>,
}

impl Tooltip {
    pub fn builder() -> TooltipBuilder {
        TooltipBuilder::new()
    }

    pub fn show(&mut self, text: &str, align: Align2, element_bounds: Rect, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        shared_state
            .inner
            .set_text(Some(text), &mut res.font_system, || {
                res.style_system
                    .get::<TooltipStyle>(self.el.class())
                    .text_properties
            });

        shared_state.show_with_info = Some((element_bounds, align));

        self.el.notify_custom_state_change();
        self.el.set_hidden(false);
    }

    pub fn hide(&mut self) {
        RefCell::borrow_mut(&self.shared_state).show_with_info = None;

        self.el.set_hidden(true);
    }

    /// An offset that can be used mainly to correct the position of the text.
    ///
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the offset has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_text_offset(&mut self, offset: Vector) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.text_offset != offset {
            shared_state.inner.text_offset = offset;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }
}
