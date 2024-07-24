use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::quad::QuadPrimitive;
use rootvg::text::{
    Align, CustomGlyphDesc, CustomGlyphID, RcTextBuffer, TextPrimitive, TextProperties,
};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::layout::{Align2, Padding};
use crate::math::{Point, Rect, Size, ZIndex};
use crate::prelude::ResourceCtx;
use crate::style::{QuadStyle, DEFAULT_TEXT_ATTRIBUTES};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconLabelLayout {
    #[default]
    LeftAlignIconThenText,
    LeftAlignTextThenIcon,
    RightAlignIconThenText,
    RightAlignTextThenIcon,
    LeftAlignIconRightAlignText,
    LeftAlignTextRightAlignIcon,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconLabelClipMode {
    #[default]
    ClipTextThenIcon,
    ClipIconThenText,
}

/// The style of a [`IconLabel`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconLabelStyle {
    /// The properties of the text.
    pub text_properties: TextProperties,

    /// The size of the icon in points.
    ///
    /// By default this is set to `20.0`.
    pub icon_size: f32,

    /// The color of the text
    ///
    /// By default this is set to `color::WHITE`.
    pub text_color: RGBA8,
    /// The color of the icon
    ///
    /// By default this is set to `color::WHITE`.
    pub icon_color: RGBA8,

    /// The vertical alignment of the text.
    ///
    /// By default this is set to `Align::Center`.
    pub vertical_align: crate::layout::Align,

    pub layout: IconLabelLayout,

    /// The minimum size of the clipped text area.
    ///
    /// By default this is set to `Size::new(5.0, 5.0)`.
    pub text_min_clipped_size: Size,

    pub clip_mode: IconLabelClipMode,

    /// The style of the padded background rectangle behind the text.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    ///
    /// By default this is set to `QuadStyle::TRANSPARENT`.
    pub back_quad: QuadStyle,

    /// The padding between the text and the bounding rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub text_padding: Padding,
    /// The padding between the icon and the bounding rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub icon_padding: Padding,
}

impl Default for IconLabelStyle {
    fn default() -> Self {
        Self {
            text_properties: TextProperties {
                attrs: DEFAULT_TEXT_ATTRIBUTES,
                ..Default::default()
            },
            icon_size: 20.0,
            text_color: color::WHITE,
            icon_color: color::WHITE,
            vertical_align: crate::layout::Align::Center,
            layout: IconLabelLayout::default(),
            text_min_clipped_size: Size::new(5.0, 5.0),
            clip_mode: IconLabelClipMode::default(),
            back_quad: QuadStyle::TRANSPARENT,
            text_padding: Padding::default(),
            icon_padding: Padding::new(0.0, 5.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IconLabelPrimitives {
    pub icon: Option<TextPrimitive>,
    pub text: Option<TextPrimitive>,
    pub bg_quad: Option<QuadPrimitive>,
}

struct TextInner {
    text: String,
    text_buffer: RcTextBuffer,
}

/// A reusable label with icon struct that can be used by other elements.
pub struct IconLabelInner {
    /// An offset that can be used mainly to correct the position of text.
    /// This does not effect the position of the background quad.
    pub text_offset: Point,
    /// An offset that can be used mainly to correct the position of icons.
    /// This does not effect the position of the background quad.
    pub icon_offset: Point,
    pub icon_id: Option<CustomGlyphID>,
    pub icon_scale: f32,
    text_inner: Option<TextInner>,
    unclipped_text_size: Size,
    text_size_needs_calculated: bool,
    prev_bounds_size: Size,
    text_bounds_rect: Rect,
    icon_bounds_rect: Rect,
    padded_size: Size,
    padded_size_needs_calculated: bool,
}

impl IconLabelInner {
    pub fn new(
        text: Option<impl Into<String>>,
        icon_id: Option<CustomGlyphID>,
        text_offset: Point,
        icon_offset: Point,
        icon_scale: f32,
        style: &IconLabelStyle,
        res: &mut ResourceCtx,
    ) -> Self {
        let text_inner = text.map(|text| {
            let text: String = text.into();

            let mut text_properties = style.text_properties.clone();
            text_properties.align = Some(match style.layout {
                IconLabelLayout::LeftAlignTextThenIcon
                | IconLabelLayout::LeftAlignIconThenText
                | IconLabelLayout::LeftAlignTextRightAlignIcon => Align::Left,
                _ => Align::Right,
            });

            // Use a temporary size for the text buffer.
            let text_buffer = RcTextBuffer::new(
                &text,
                text_properties,
                Size::new(1000.0, 200.0),
                false,
                &mut res.font_system,
            );

            TextInner { text, text_buffer }
        });

        Self {
            text_offset,
            icon_offset,
            icon_id,
            icon_scale,
            text_inner,
            // This will be overwritten later.
            unclipped_text_size: Size::default(),
            text_size_needs_calculated: true,
            prev_bounds_size: Size::new(-1.0, -1.0),
            // This will be overwritten later.
            text_bounds_rect: Rect::default(),
            icon_bounds_rect: Rect::default(),
            padded_size: Size::default(),
            padded_size_needs_calculated: true,
        }
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    pub fn desired_padded_size(&mut self, style: &IconLabelStyle) -> Size {
        if self.padded_size_needs_calculated {
            self.padded_size_needs_calculated = false;

            let (text_size, text_padded_width) = if self.text_inner.is_some() {
                let text_size = self.unclipped_text_size();
                (
                    text_size,
                    text_size.width + style.text_padding.left + style.text_padding.right,
                )
            } else {
                (Size::zero(), 0.0)
            };

            let icon_padded_size = if self.icon_id.is_none() {
                Size::zero()
            } else {
                Size::new(
                    style.icon_size + style.icon_padding.left + style.icon_padding.right,
                    style.icon_size + style.icon_padding.top + style.icon_padding.bottom,
                )
            };

            let height = (text_size.height + style.text_padding.top + style.text_padding.bottom)
                .max(icon_padded_size.height);

            self.padded_size = Size::new(text_padded_width + icon_padded_size.width, height);
        }

        self.padded_size
    }

    /// Returns the size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn unclipped_text_size(&mut self) -> Size {
        if self.text_size_needs_calculated {
            self.text_size_needs_calculated = false;

            self.unclipped_text_size = self
                .text_inner
                .as_mut()
                .map(|i| i.text_buffer.measure())
                .unwrap_or(Size::default());
        }

        self.unclipped_text_size
    }

    /// Returns `true` if the text has changed.
    pub fn set_text(&mut self, text: &str, style: &IconLabelStyle, res: &mut ResourceCtx) -> bool {
        if let Some(inner) = &mut self.text_inner {
            if &inner.text != text {
                inner.text = String::from(text);
                self.text_size_needs_calculated = true;
                self.padded_size_needs_calculated = true;

                inner.text_buffer.set_text(text, &mut res.font_system);

                true
            } else {
                false
            }
        } else {
            let text: String = text.into();

            let mut text_properties = style.text_properties.clone();
            text_properties.align = Some(match style.layout {
                IconLabelLayout::LeftAlignTextThenIcon
                | IconLabelLayout::LeftAlignIconThenText
                | IconLabelLayout::LeftAlignTextRightAlignIcon => Align::Left,
                _ => Align::Right,
            });

            // Use a temporary size for the text buffer.
            let text_buffer = RcTextBuffer::new(
                &text,
                text_properties,
                Size::new(1000.0, 200.0),
                false,
                &mut res.font_system,
            );

            self.text_inner = Some(TextInner { text, text_buffer });

            true
        }
    }

    pub fn text(&self) -> &str {
        self.text_inner
            .as_ref()
            .map(|i| i.text.as_str())
            .unwrap_or_default()
    }

    pub fn set_style(&mut self, style: &IconLabelStyle, res: &mut ResourceCtx) {
        if let Some(inner) = &mut self.text_inner {
            let mut text_properties = style.text_properties.clone();
            text_properties.align = Some(match style.layout {
                IconLabelLayout::LeftAlignTextThenIcon
                | IconLabelLayout::LeftAlignIconThenText
                | IconLabelLayout::LeftAlignTextRightAlignIcon => Align::Left,
                _ => Align::Right,
            });

            inner.text_buffer.set_text_and_props(
                &inner.text,
                text_properties,
                &mut res.font_system,
            );

            self.text_size_needs_calculated = true;
        }

        self.padded_size_needs_calculated = true;
    }

    pub fn render_primitives(
        &mut self,
        bounds: Rect,
        style: &IconLabelStyle,
        res: &mut ResourceCtx,
    ) -> IconLabelPrimitives {
        let mut needs_layout = self.text_size_needs_calculated || self.padded_size_needs_calculated;

        if self.prev_bounds_size != bounds.size {
            self.prev_bounds_size = bounds.size;
            needs_layout = true;
        }

        if needs_layout {
            let _ = self.unclipped_text_size();

            let layout_res = layout(bounds.size, self.unclipped_text_size, self.icon_id, style);

            self.text_bounds_rect = layout_res.text_bounds_rect;
            self.icon_bounds_rect = layout_res.icon_bounds_rect;

            if let Some(inner) = &mut self.text_inner {
                inner.text_buffer.set_bounds(
                    Size::new(
                        self.text_bounds_rect.width(),
                        // Add some extra padding below so that text doesn't get clipped.
                        self.text_bounds_rect.height() + 2.0,
                    ),
                    &mut res.font_system,
                );
            }
        }

        let text = if let Some(inner) = &self.text_inner {
            Some(TextPrimitive::new(
                inner.text_buffer.clone(),
                bounds.origin
                    + self.text_bounds_rect.origin.to_vector()
                    + self.text_offset.to_vector(),
                style.text_color,
                None,
            ))
        } else {
            None
        };

        let icon = if let Some(icon_id) = self.icon_id {
            let (size, offset) = if self.icon_scale != 1.0 {
                (
                    style.icon_size * self.icon_scale,
                    (style.icon_size - (style.icon_size * self.icon_scale)) * 0.5,
                )
            } else {
                (style.icon_size, 0.0)
            };

            Some(TextPrimitive::new_with_icons(
                None,
                bounds.origin
                    + self.icon_bounds_rect.origin.to_vector()
                    + self.icon_offset.to_vector(),
                style.icon_color,
                Some(Rect::new(
                    Point::new(-1.0, -1.0),
                    Size::new(
                        self.icon_bounds_rect.size.width + 2.0,
                        self.icon_bounds_rect.size.height + 2.0,
                    ),
                )),
                smallvec::smallvec![CustomGlyphDesc {
                    id: icon_id,
                    left: offset,
                    top: offset,
                    size,
                    color: None,
                    metadata: 0,
                }],
            ))
        } else {
            None
        };

        let bg_quad = if !style.back_quad.is_transparent() {
            Some(style.back_quad.create_primitive(bounds))
        } else {
            None
        };

        IconLabelPrimitives {
            text,
            icon,
            bg_quad,
        }
    }

    /// An offset that can be used mainly to correct the position of text.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_text_offset(&mut self, offset: Point) -> bool {
        if self.text_offset != offset {
            self.text_offset = offset;
            true
        } else {
            false
        }
    }

    /// An offset that can be used mainly to correct the position of the icon.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the text offset has changed.
    pub fn set_icon_offset(&mut self, offset: Point) -> bool {
        if self.icon_offset != offset {
            self.icon_offset = offset;
            true
        } else {
            false
        }
    }
}

pub struct IconLabelBuilder {
    pub text: Option<String>,
    pub icon: Option<CustomGlyphID>,
    pub icon_scale: f32,
    pub text_offset: Point,
    pub icon_offset: Point,
    pub style: Rc<IconLabelStyle>,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl IconLabelBuilder {
    pub fn new(style: &Rc<IconLabelStyle>) -> Self {
        Self {
            text: None,
            icon: None,
            icon_scale: 1.0,
            text_offset: Point::default(),
            icon_offset: Point::default(),
            style: Rc::clone(style),
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: None,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> IconLabel {
        IconLabelElement::create(self, cx)
    }

    pub fn text(mut self, text: Option<impl Into<String>>) -> Self {
        self.text = text.map(|t| t.into());
        self
    }

    pub fn icon(mut self, icon_id: Option<impl Into<CustomGlyphID>>) -> Self {
        self.icon = icon_id.map(|i| i.into());
        self
    }

    pub const fn icon_scale(mut self, scale: f32) -> Self {
        self.icon_scale = scale;
        self
    }

    /// An offset that can be used mainly to correct the position of the text.
    /// This does not effect the position of the background quad.
    pub const fn text_offset(mut self, offset: Point) -> Self {
        self.text_offset = offset;
        self
    }

    /// An offset that can be used mainly to correct the position of the icon.
    /// This does not effect the position of the background quad.
    pub const fn icon_offset(mut self, offset: Point) -> Self {
        self.icon_offset = offset;
        self
    }

    pub const fn z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = Some(z_index);
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
        self.scissor_rect_id = Some(scissor_rect_id);
        self
    }
}

/// A label element with an optional quad background.
pub struct IconLabelElement {
    shared_state: Rc<RefCell<SharedState>>,
}

impl IconLabelElement {
    pub fn create<A: Clone + 'static>(
        builder: IconLabelBuilder,
        cx: &mut WindowContext<'_, A>,
    ) -> IconLabel {
        let IconLabelBuilder {
            text,
            icon,
            icon_scale,
            text_offset,
            icon_offset,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id) = cx.z_index_and_scissor_rect_id(z_index, scissor_rect_id);

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: IconLabelInner::new(
                text,
                icon,
                text_offset,
                icon_offset,
                icon_scale,
                &style,
                &mut cx.res,
            ),
            style,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        IconLabel { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for IconLabelElement {
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
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        let label_primitives =
            inner.render_primitives(Rect::from_size(cx.bounds_size), style, cx.res);

        if let Some(quad_primitive) = label_primitives.bg_quad {
            primitives.add(quad_primitive);
        }

        if let Some(text_primitive) = label_primitives.text {
            primitives.set_z_index(1);
            primitives.add_text(text_primitive);
        }

        if let Some(icon_primitive) = label_primitives.icon {
            primitives.set_z_index(1);
            primitives.add_text(icon_primitive);
        }
    }
}

struct SharedState {
    inner: IconLabelInner,
    style: Rc<IconLabelStyle>,
}

/// A handle to a [`IconLabelElement`], a label with an optional quad background.
pub struct IconLabel {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl IconLabel {
    pub fn builder(style: &Rc<IconLabelStyle>) -> IconLabelBuilder {
        IconLabelBuilder::new(style)
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn desired_padded_size(&self) -> Size {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        inner.desired_padded_size(style)
    }

    pub fn set_text(&mut self, text: &str, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style } = &mut *shared_state;

        if inner.set_text(text, style, res) {
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_icon_id(&mut self, icon_id: Option<impl Into<CustomGlyphID>>) {
        let icon_id: Option<CustomGlyphID> = icon_id.map(|i| i.into());

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon_id != icon_id {
            shared_state.inner.icon_id = icon_id;
            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn icon_id(&self) -> Option<CustomGlyphID> {
        RefCell::borrow(&self.shared_state).inner.icon_id
    }

    pub fn set_style(&mut self, style: &Rc<IconLabelStyle>, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(style, res);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<IconLabelStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    /// An offset that can be used mainly to correct the position of the text.
    /// This does not effect the position of the background quad.
    pub fn set_text_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_text_offset(offset);

        if changed {
            self.el.notify_custom_state_change();
        }
    }

    /// An offset that can be used mainly to correct the position of the icon.
    /// This does not effect the position of the background quad.
    pub fn set_icon_offset(&mut self, offset: Point) {
        let changed = RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_icon_offset(offset);

        if changed {
            self.el.notify_custom_state_change();
        }
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
    pub fn set_scale(&mut self, scale: f32) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon_scale != scale {
            shared_state.inner.icon_scale = scale;
            self.el.notify_custom_state_change();
        }
    }

    pub fn layout(&mut self, origin: Point) {
        let size = self.desired_padded_size();
        self.el.set_rect(Rect::new(origin, size));
    }

    pub fn layout_aligned(&mut self, point: Point, align: Align2) {
        let size = self.desired_padded_size();
        self.el.set_rect(align.align_rect_to_point(point, size));
    }
}

struct LayoutResult {
    text_bounds_rect: Rect,
    icon_bounds_rect: Rect,
}

fn layout(
    bounds_size: Size,
    unclipped_text_size: Size,
    icon_id: Option<CustomGlyphID>,
    style: &IconLabelStyle,
) -> LayoutResult {
    if icon_id.is_none() {
        return LayoutResult {
            text_bounds_rect: super::label::layout_text_bounds(
                bounds_size,
                unclipped_text_size,
                style.text_padding,
                style.text_min_clipped_size,
                style.vertical_align,
            ),
            icon_bounds_rect: Rect::zero(),
        };
    }

    if unclipped_text_size.is_empty() {
        return LayoutResult {
            text_bounds_rect: Rect::zero(),
            icon_bounds_rect: super::icon::layout(
                style.icon_size,
                &style.icon_padding,
                bounds_size,
            ),
        };
    }

    let text_padded_width =
        unclipped_text_size.width + style.text_padding.left + style.text_padding.right;
    let icon_padded_width = style.icon_size + style.icon_padding.left + style.icon_padding.right;

    let total_padded_width = text_padded_width + icon_padded_width;

    let (text_clipped_padded_width, icon_clipped_padded_width) = if total_padded_width
        <= bounds_size.width
    {
        (text_padded_width, icon_padded_width)
    } else {
        let min_text_padded_width =
            style.text_min_clipped_size.width + style.text_padding.left + style.text_padding.right;
        let min_icon_padded_width = style.icon_padding.left + style.icon_padding.right;
        let min_total_padded_width = min_text_padded_width + min_icon_padded_width;

        match style.clip_mode {
            IconLabelClipMode::ClipTextThenIcon => {
                if min_total_padded_width >= bounds_size.width {
                    (min_text_padded_width, min_icon_padded_width)
                } else if min_text_padded_width + icon_padded_width >= bounds_size.width {
                    (
                        min_text_padded_width,
                        bounds_size.width - min_text_padded_width,
                    )
                } else {
                    (bounds_size.width - icon_padded_width, icon_padded_width)
                }
            }
            IconLabelClipMode::ClipIconThenText => {
                if min_total_padded_width >= bounds_size.width {
                    (min_text_padded_width, min_icon_padded_width)
                } else if min_icon_padded_width + text_padded_width >= bounds_size.width {
                    (
                        bounds_size.width - min_icon_padded_width,
                        min_icon_padded_width,
                    )
                } else {
                    (text_padded_width, bounds_size.width - text_padded_width)
                }
            }
        }
    };

    let (text_padded_rect_x, icon_padded_rect_x) = match style.layout {
        IconLabelLayout::LeftAlignIconThenText => (icon_clipped_padded_width, 0.0),
        IconLabelLayout::LeftAlignTextThenIcon => (0.0, text_clipped_padded_width),
        IconLabelLayout::RightAlignIconThenText => (
            bounds_size.width - text_clipped_padded_width,
            bounds_size.width - text_clipped_padded_width - icon_clipped_padded_width,
        ),
        IconLabelLayout::RightAlignTextThenIcon => (
            bounds_size.width - text_clipped_padded_width - icon_clipped_padded_width,
            bounds_size.width - icon_clipped_padded_width,
        ),
        IconLabelLayout::LeftAlignIconRightAlignText => {
            (bounds_size.width - text_clipped_padded_width, 0.0)
        }
        IconLabelLayout::LeftAlignTextRightAlignIcon => {
            (0.0, bounds_size.width - icon_clipped_padded_width)
        }
    };

    let text_bounds_height =
        if unclipped_text_size.height + style.text_padding.top + style.text_padding.bottom
            <= bounds_size.height
        {
            unclipped_text_size.height
        } else {
            (bounds_size.height - style.text_padding.top - style.text_padding.bottom)
                .max(style.text_min_clipped_size.height)
        };

    let icon_bounds_height = if style.icon_size + style.icon_padding.top + style.icon_padding.bottom
        <= bounds_size.height
    {
        style.icon_size
    } else {
        (bounds_size.height - style.icon_padding.top - style.icon_padding.bottom).max(0.0)
    };

    // We need to vertically align the text ourselves as rootvg/glyphon does not do this.
    let text_bounds_y = match style.vertical_align {
        crate::layout::Align::Start => style.text_padding.top,
        crate::layout::Align::Center => (bounds_size.height - text_bounds_height) * 0.5,
        crate::layout::Align::End => {
            bounds_size.height - text_bounds_height - style.text_padding.bottom
        }
    };
    let icon_bounds_y = match style.vertical_align {
        crate::layout::Align::Start => style.icon_padding.top,
        crate::layout::Align::Center => (bounds_size.height - icon_bounds_height) * 0.5,
        crate::layout::Align::End => {
            bounds_size.height - icon_bounds_height - style.icon_padding.bottom
        }
    };

    LayoutResult {
        text_bounds_rect: Rect::new(
            Point::new(text_padded_rect_x + style.text_padding.left, text_bounds_y),
            Size::new(
                (text_clipped_padded_width - style.text_padding.left - style.text_padding.right)
                    .max(0.0),
                text_bounds_height,
            ),
        ),
        icon_bounds_rect: Rect::new(
            Point::new(icon_padded_rect_x + style.icon_padding.left, icon_bounds_y),
            Size::new(
                (icon_clipped_padded_width - style.icon_padding.left - style.icon_padding.right)
                    .max(0.0),
                icon_bounds_height,
            ),
        ),
    }
}
