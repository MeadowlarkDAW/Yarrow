use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::derive::*;
use crate::prelude::*;
use crate::theme::DEFAULT_ICON_SIZE;
use crate::vg::{
    quad::QuadPrimitive,
    text::{RcTextBuffer, TextPrimitive},
};

#[cfg(feature = "svg-icons")]
use crate::vg::text::CustomGlyph;

/// The style of a [`Label`] element
#[derive(Debug, Clone, PartialEq)]
pub struct LabelStyle {
    /// The properties of the text.
    pub text_properties: TextProperties,

    /// The width and height of the icon in points (if the user hasn't
    /// manually set a size for the icon).
    ///
    /// By default this is set to `20.0`.
    pub default_icon_size: f32,

    /// Whether or not the icon should be snapped to the nearset physical
    /// pixel when rendering.
    ///
    /// By default this is set to `true`.
    pub snap_icon_to_physical_pixel: bool,

    /// The color of the text
    ///
    /// By default this is set to `color::WHITE`.
    pub text_color: RGBA8,
    /// The color of the icon.
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub icon_color: Option<RGBA8>,

    /// The padding around the text.
    ///
    /// By default this has all values set to `0.0`.
    pub text_padding: Padding,
    /// The padding around the icon.
    ///
    /// By default this has all values set to `0.0`.
    pub icon_padding: Padding,
    /// Extra spacing between the text and icon. (This can be negative to
    /// move them closer together).
    ///
    /// By default this set to `0.0`.
    pub text_icon_spacing: f32,

    /// The style of the padded background rectangle behind the text and icon.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background.
    ///
    /// By default this is set to `QuadStyle::TRANSPARENT`.
    pub back_quad: QuadStyle,

    /// The vertical alignment.
    ///
    /// By default this is set to `Align::Center`.
    pub vertical_align: crate::layout::Align,
}

impl LabelStyle {
    pub fn padding_info(&self) -> LabelPaddingInfo {
        LabelPaddingInfo {
            default_icon_size: self.default_icon_size,
            text_padding: self.text_padding,
            icon_padding: self.icon_padding,
            text_icon_spacing: self.text_icon_spacing,
        }
    }
}

impl Default for LabelStyle {
    fn default() -> Self {
        Self {
            text_properties: Default::default(),
            default_icon_size: DEFAULT_ICON_SIZE,
            snap_icon_to_physical_pixel: true,
            text_color: color::WHITE,
            icon_color: None,
            text_padding: Padding::default(),
            icon_padding: Padding::default(),
            text_icon_spacing: 0.0,
            back_quad: QuadStyle::TRANSPARENT,
            vertical_align: crate::layout::Align::Center,
        }
    }
}

impl ElementStyle for LabelStyle {
    const ID: &'static str = "lb";

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

/// How to align the text and the icon.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextIconLayout {
    #[default]
    LeftAlignIconThenText,
    LeftAlignTextThenIcon,
    RightAlignIconThenText,
    RightAlignTextThenIcon,
    LeftAlignIconRightAlignText,
    LeftAlignTextRightAlignIcon,
}

// Information used to calculate label padding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LabelPaddingInfo {
    pub default_icon_size: f32,
    pub text_padding: Padding,
    pub icon_padding: Padding,
    pub text_icon_spacing: f32,
}

#[derive(Debug, Clone)]
pub struct LabelPrimitives {
    pub icon: Option<TextPrimitive>,
    pub text: Option<TextPrimitive>,
    pub bg_quad: Option<QuadPrimitive>,
}

struct TextInner {
    text: String,
    text_buffer: RcTextBuffer,
}

/// A reusable label with text and icon struct that can be used by other elements.
pub struct LabelInner {
    /// An offset that can be used mainly to correct the position of text.
    /// This does not effect the position of the background quad.
    pub text_offset: Vector,
    /// An offset that can be used mainly to correct the position of icons.
    /// This does not effect the position of the background quad.
    pub icon_offset: Vector,
    pub icon_scale: IconScale,
    icon_size: Option<Size>,
    text_inner: Option<TextInner>,
    unclipped_text_size: Size,
    text_size_needs_calculated: bool,
    prev_bounds_size: Size,
    text_bounds_rect: Rect,
    icon_bounds_rect: Rect,
    padded_size: Size,
    padded_size_needs_calculated: bool,
    text_icon_layout: TextIconLayout,
    icon: Option<IconID>,
}

impl LabelInner {
    pub fn new(
        text: Option<impl Into<String>>,
        icon: Option<IconID>,
        text_offset: Vector,
        icon_offset: Vector,
        icon_size: Option<Size>,
        icon_scale: IconScale,
        text_icon_layout: TextIconLayout,
        style: &LabelStyle,
        font_system: &mut FontSystem,
    ) -> Self {
        let text_inner = text.map(|text| {
            let text: String = text.into();

            let mut text_properties = style.text_properties.clone();
            text_properties.align = Some(match text_icon_layout {
                TextIconLayout::LeftAlignTextThenIcon
                | TextIconLayout::LeftAlignIconThenText
                | TextIconLayout::LeftAlignTextRightAlignIcon => rootvg::text::Align::Left,
                _ => rootvg::text::Align::Right,
            });

            let text_buffer =
                RcTextBuffer::new(&text, text_properties, None, None, false, font_system);

            TextInner { text, text_buffer }
        });

        Self {
            text_offset,
            icon_offset,
            icon,
            icon_size,
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
            text_icon_layout,
        }
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    ///
    /// If the padded size needs calculated, then the given closure will be used to
    /// extract the padding from the current style (text_padding, icon_padding).
    pub fn desired_size<F: FnOnce() -> LabelPaddingInfo>(&mut self, get_padding: F) -> Size {
        if self.padded_size_needs_calculated {
            self.padded_size_needs_calculated = false;

            let padding_info = (get_padding)();

            let text_size = if self.text_inner.is_some() {
                let unclipped_text_size = self.unclipped_text_size();

                Size::new(
                    unclipped_text_size.width
                        + padding_info.text_padding.left
                        + padding_info.text_padding.right,
                    unclipped_text_size.height
                        + padding_info.text_padding.top
                        + padding_info.text_padding.bottom,
                )
            } else {
                Size::zero()
            };

            let icon_size = if self.icon.is_some() {
                let size = self.icon_size.unwrap_or(Size::new(
                    padding_info.default_icon_size,
                    padding_info.default_icon_size,
                ));

                Size::new(
                    size.width + padding_info.icon_padding.left + padding_info.icon_padding.right,
                    size.height + padding_info.icon_padding.top + padding_info.icon_padding.bottom,
                )
            } else {
                Size::zero()
            };

            self.padded_size = if self.text_inner.is_some() && self.icon.is_some() {
                Size::new(
                    (text_size.width + icon_size.width + padding_info.text_icon_spacing).max(0.0),
                    text_size.height.max(icon_size.height),
                )
            } else {
                Size::new(
                    text_size.width.max(icon_size.width),
                    text_size.height.max(icon_size.height),
                )
            };
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
    pub fn set_text<T: AsRef<str> + Into<String>, F: FnOnce() -> TextProperties>(
        &mut self,
        text: Option<T>,
        font_system: &mut FontSystem,
        get_text_props: F,
    ) -> bool {
        if let Some(inner) = &mut self.text_inner {
            if let Some(new_text) = text {
                if inner.text.as_str() != new_text.as_ref() {
                    inner.text = new_text.into();
                    self.text_size_needs_calculated = true;
                    self.padded_size_needs_calculated = true;

                    inner.text_buffer.set_text(&inner.text, font_system);

                    true
                } else {
                    false
                }
            } else {
                self.text_inner = None;
                self.text_size_needs_calculated = true;
                self.padded_size_needs_calculated = true;
                true
            }
        } else if let Some(new_text) = text {
            let new_text: String = new_text.into();

            let mut text_properties = (get_text_props)();

            text_properties.align = Some(match self.text_icon_layout {
                TextIconLayout::LeftAlignTextThenIcon
                | TextIconLayout::LeftAlignIconThenText
                | TextIconLayout::LeftAlignTextRightAlignIcon => rootvg::text::Align::Left,
                _ => rootvg::text::Align::Right,
            });

            let text_buffer =
                RcTextBuffer::new(&new_text, text_properties, None, None, false, font_system);

            self.text_inner = Some(TextInner {
                text: new_text,
                text_buffer,
            });

            true
        } else {
            false
        }
    }

    pub fn text(&self) -> Option<&str> {
        self.text_inner.as_ref().map(|i| i.text.as_str())
    }

    pub fn set_icon(&mut self, icon: Option<IconID>) -> bool {
        if self.icon == icon {
            false
        } else {
            if self.icon.is_some() != icon.is_some() {
                self.padded_size_needs_calculated = true;
            }

            self.icon = icon;

            true
        }
    }

    pub fn icon(&self) -> Option<IconID> {
        self.icon
    }

    pub fn set_icon_size(&mut self, size: Option<Size>) -> bool {
        if self.icon_size != size {
            self.icon_size = size;
            self.padded_size_needs_calculated = true;
            true
        } else {
            false
        }
    }

    pub fn icon_size(&self) -> Option<Size> {
        self.icon_size
    }

    pub fn sync_new_style(&mut self, style: &LabelStyle, font_system: &mut FontSystem) {
        if let Some(inner) = &mut self.text_inner {
            let mut text_properties = style.text_properties.clone();
            text_properties.align = Some(match self.text_icon_layout {
                TextIconLayout::LeftAlignTextThenIcon
                | TextIconLayout::LeftAlignIconThenText
                | TextIconLayout::LeftAlignTextRightAlignIcon => rootvg::text::Align::Left,
                _ => rootvg::text::Align::Right,
            });

            inner
                .text_buffer
                .set_text_and_props(&inner.text, text_properties, font_system);

            self.text_size_needs_calculated = true;
        }

        self.padded_size_needs_calculated = true;
    }

    pub fn render(
        &mut self,
        bounds: Rect,
        style: &LabelStyle,
        font_system: &mut FontSystem,
    ) -> LabelPrimitives {
        let mut needs_layout = self.text_size_needs_calculated || self.padded_size_needs_calculated;

        if self.prev_bounds_size != bounds.size {
            self.prev_bounds_size = bounds.size;
            needs_layout = true;
        }

        if needs_layout {
            let _ = self.unclipped_text_size();

            let icon_size = self
                .icon_size
                .unwrap_or(Size::new(style.default_icon_size, style.default_icon_size));

            let layout_res = layout(
                bounds.size,
                self.unclipped_text_size,
                self.icon,
                icon_size,
                self.text_icon_layout,
                style,
            );

            self.text_bounds_rect = layout_res.text_bounds_rect;
            self.icon_bounds_rect = layout_res.icon_bounds_rect;

            if let Some(inner) = &mut self.text_inner {
                inner.text_buffer.set_bounds(
                    Some(self.text_bounds_rect.width()),
                    None,
                    font_system,
                );
            }
        }

        let text = if let Some(inner) = &self.text_inner {
            Some(TextPrimitive::new(
                inner.text_buffer.clone(),
                bounds.origin + self.text_bounds_rect.origin.to_vector() + self.text_offset,
                style.text_color,
                Some(Rect::new(
                    Point::new(-1.0, -1.0),
                    Size::new(bounds.width() + 2.0, bounds.height() + 2.0),
                )),
            ))
        } else {
            None
        };

        #[cfg(feature = "svg-icons")]
        let icon = if let Some(icon) = self.icon {
            let size = self
                .icon_size
                .unwrap_or(Size::new(style.default_icon_size, style.default_icon_size));

            let (size, offset) = if self.icon_scale.0 != 1.0 {
                (
                    size * self.icon_scale.0,
                    ((size - (size * self.icon_scale.0)) * 0.5).to_vector(),
                )
            } else {
                (size, Vector::zero())
            };

            Some(TextPrimitive::new_with_icons(
                None,
                bounds.origin + self.icon_bounds_rect.origin.to_vector() + self.icon_offset,
                style.icon_color.unwrap_or(style.text_color),
                Some(Rect::new(
                    Point::new(-1.0, -1.0),
                    Size::new(bounds.width() + 2.0, bounds.height() + 2.0),
                )),
                smallvec::smallvec![CustomGlyph {
                    id: icon,
                    left: offset.x,
                    top: offset.y,
                    width: size.width,
                    height: size.height,
                    snap_to_physical_pixel: style.snap_icon_to_physical_pixel,
                    color: None,
                    metadata: 0,
                }],
            ))
        } else {
            None
        };

        #[cfg(not(feature = "svg-icons"))]
        let icon = None;

        let bg_quad = if !style.back_quad.is_transparent() {
            Some(style.back_quad.create_primitive(bounds))
        } else {
            None
        };

        LabelPrimitives {
            text,
            icon,
            bg_quad,
        }
    }
}

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[derive(Default)]
pub struct LabelBuilder {
    pub text: Option<String>,
    pub icon: Option<IconID>,
    pub icon_size: Option<Size>,
    pub icon_scale: IconScale,
    pub text_offset: Vector,
    pub icon_offset: Vector,
    pub text_icon_layout: TextIconLayout,
}

impl LabelBuilder {
    /// The text of the label
    ///
    /// If this method isn't used, then the label will have no text (unless
    /// [`LabelBulder::text_optional`] is used).
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// The icon of the label
    ///
    /// If this method isn't used, then the label will have no icon (unless
    /// [`LabelBulder::icon_optional`] is used).
    pub fn icon(mut self, icon: impl Into<IconID>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// The optional text of the label
    ///
    /// If this is set to `None`, then the label will have no text.
    pub fn text_optional(mut self, text: Option<impl Into<String>>) -> Self {
        self.text = text.map(|t| t.into());
        self
    }

    /// The optional icon of the label
    ///
    /// If this is set to `None`, then the label will have no icon.
    pub fn icon_optional(mut self, icon: Option<impl Into<IconID>>) -> Self {
        self.icon = icon.map(|i| i.into());
        self
    }

    /// The size of the icon (Overrides the size in the style.)
    pub fn icon_size(mut self, size: impl Into<Option<Size>>) -> Self {
        self.icon_size = size.into();
        self
    }

    /// The scale of an icon, used to make icons look more consistent.
    ///
    /// Note this does not affect any layout, this is just a visual thing.
    pub fn icon_scale(mut self, scale: impl Into<IconScale>) -> Self {
        self.icon_scale = scale.into();
        self
    }

    /// An offset that can be used mainly to correct the position of the text.
    /// This does not effect the position of the background quad.
    ///
    /// By default this is set to an offset of zero.
    pub const fn text_offset(mut self, offset: Vector) -> Self {
        self.text_offset = offset;
        self
    }

    /// An offset that can be used mainly to correct the position of the icon.
    /// This does not effect the position of the background quad.
    ///
    /// By default this is set to an offset of zero.
    pub const fn icon_offset(mut self, offset: Vector) -> Self {
        self.icon_offset = offset;
        self
    }

    /// How to layout the text and the icon inside the label's bounds.
    ///
    /// By default this is set to `TextIconLayout::LeftAlignIconThenText`
    pub const fn text_icon_layout(mut self, layout: TextIconLayout) -> Self {
        self.text_icon_layout = layout;
        self
    }

    pub fn build<A: Clone + 'static>(self, window_cx: &mut WindowContext<'_, A>) -> Label {
        let LabelBuilder {
            text,
            icon,
            icon_size,
            icon_scale,
            text_offset,
            icon_offset,
            text_icon_layout,
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
        } = self;

        let style = window_cx
            .res
            .style_system
            .get(window_cx.builder_class(class));

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: LabelInner::new(
                text,
                icon,
                text_offset,
                icon_offset,
                icon_size,
                icon_scale,
                text_icon_layout,
                &style,
                &mut window_cx.res.font_system,
            ),
        }));

        let el = ElementBuilder::new(LabelElement {
            shared_state: Rc::clone(&shared_state),
        })
        .builder_values(z_index, scissor_rect, class, window_cx)
        .rect(rect)
        .hidden(manually_hidden)
        .flags(ElementFlags::PAINTS)
        .build(window_cx);

        Label { el, shared_state }
    }
}

/// A label element with an optional quad background.
struct LabelElement {
    shared_state: Rc<RefCell<SharedState>>,
}

impl<A: Clone + 'static> Element<A> for LabelElement {
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
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let label_primitives = shared_state.inner.render(
            Rect::from_size(cx.bounds_size),
            cx.res.style_system.get(cx.class),
            &mut cx.res.font_system,
        );

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
    inner: LabelInner,
}

/// A handle to a [`LabelElement`], a label with an optional quad background.
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
pub struct Label {
    shared_state: Rc<RefCell<SharedState>>,
}

impl Label {
    pub fn builder() -> LabelBuilder {
        LabelBuilder::default()
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the text and icon.
    ///
    /// This size is automatically cached, so it should be relatively
    /// inexpensive to call.
    pub fn desired_size(&self, res: &mut ResourceCtx) -> Size {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .desired_size(|| {
                res.style_system
                    .get::<LabelStyle>(self.el.class())
                    .padding_info()
            })
    }

    /// Set the text.
    ///
    /// Returns `true` if the text has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently. However, this method still
    /// involves a string comparison so you may want to call this method
    /// sparingly.
    pub fn set_text<T: AsRef<str> + Into<String>>(
        &mut self,
        text: Option<T>,
        res: &mut ResourceCtx,
    ) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_text(text, &mut res.font_system, || {
            res.style_system
                .get::<LabelStyle>(self.el.class())
                .text_properties
        }) {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// Set the icon.
    ///
    /// Returns `true` if the icon has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_icon(&mut self, icon: Option<impl Into<IconID>>) -> bool {
        let icon: Option<IconID> = icon.map(|i| i.into());

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon(icon) {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn text<'a>(&'a self) -> Option<Ref<'a, str>> {
        Ref::filter_map(RefCell::borrow(&self.shared_state), |s| s.inner.text()).ok()
    }

    pub fn icon(&self) -> Option<IconID> {
        RefCell::borrow(&self.shared_state).inner.icon
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

    /// An offset that can be used mainly to correct the position of the icon.
    ///
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the offset has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_icon_offset(&mut self, offset: Vector) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon_offset != offset {
            shared_state.inner.icon_offset = offset;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// Set the size of the icon
    ///
    /// If `size` is `None`, then the size specified by the style will be used.
    ///
    /// Returns `true` if the size has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_icon_size(&mut self, size: impl Into<Option<Size>>) -> bool {
        let size: Option<Size> = size.into();

        if RefCell::borrow_mut(&self.shared_state)
            .inner
            .set_icon_size(size.into())
        {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// The scale of the icon, used to make icons look more consistent.
    ///
    /// Note this does not affect any layout, this is just a visual thing.
    ///
    /// Returns `true` if the scale has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_icon_scale(&mut self, scale: impl Into<IconScale>) -> bool {
        let scale: IconScale = scale.into();

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon_scale != scale {
            shared_state.inner.icon_scale = scale;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// Layout out the element (with the top-left corner of the bounds set to `origin`).
    ///
    /// Returns `true` if the layout has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn layout(&mut self, origin: Point, res: &mut ResourceCtx) -> bool {
        let size = self.desired_size(res);
        self.el.set_rect(Rect::new(origin, size))
    }

    /// Layout out the element aligned to the given point.
    ///
    /// Returns `true` if the layout has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn layout_aligned(&mut self, point: Point, align: Align2, res: &mut ResourceCtx) -> bool {
        let size = self.desired_size(res);
        self.el.set_rect(align.align_rect_to_point(point, size))
    }
}

struct LayoutResult {
    text_bounds_rect: Rect,
    icon_bounds_rect: Rect,
}

fn layout(
    bounds_size: Size,
    unclipped_text_size: Size,
    icon: Option<IconID>,
    icon_size: Size,
    text_icon_layout: TextIconLayout,
    style: &LabelStyle,
) -> LayoutResult {
    if icon.is_none() {
        return LayoutResult {
            text_bounds_rect: layout_label_only(
                bounds_size,
                unclipped_text_size,
                style.text_padding,
                style.vertical_align,
            ),
            icon_bounds_rect: Rect::zero(),
        };
    }

    if unclipped_text_size.is_empty() {
        return LayoutResult {
            text_bounds_rect: Rect::zero(),
            icon_bounds_rect: layout_icon_only(icon_size, &style.icon_padding, bounds_size),
        };
    }

    let icon_padding = match text_icon_layout {
        TextIconLayout::LeftAlignIconRightAlignText
        | TextIconLayout::LeftAlignIconThenText
        | TextIconLayout::RightAlignIconThenText => {
            let mut icon_padding = style.icon_padding;
            icon_padding.right += style.text_icon_spacing;

            icon_padding
        }
        _ => {
            let mut icon_padding = style.icon_padding;
            icon_padding.left += style.text_icon_spacing;

            icon_padding
        }
    };
    let text_padding = style.text_padding;

    let text_padded_width = unclipped_text_size.width + text_padding.left + text_padding.right;
    let icon_padded_width = icon_size.width + icon_padding.left + icon_padding.right;

    let total_padded_width = text_padded_width + icon_padded_width;

    let (text_clipped_padded_width, icon_clipped_padded_width) =
        if total_padded_width <= bounds_size.width {
            (text_padded_width, icon_padded_width)
        } else {
            let min_text_padded_width = text_padding.left + text_padding.right;
            let min_icon_padded_width = icon_padding.left + icon_padding.right;
            let min_total_padded_width = min_text_padded_width + min_icon_padded_width;

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
        };

    let (text_padded_rect_x, icon_padded_rect_x) = match text_icon_layout {
        TextIconLayout::LeftAlignIconThenText => (icon_clipped_padded_width, 0.0),
        TextIconLayout::LeftAlignTextThenIcon => (0.0, text_clipped_padded_width),
        TextIconLayout::RightAlignIconThenText => (
            bounds_size.width - text_clipped_padded_width,
            bounds_size.width - text_clipped_padded_width - icon_clipped_padded_width,
        ),
        TextIconLayout::RightAlignTextThenIcon => (
            bounds_size.width - text_clipped_padded_width - icon_clipped_padded_width,
            bounds_size.width - icon_clipped_padded_width,
        ),
        TextIconLayout::LeftAlignIconRightAlignText => {
            (bounds_size.width - text_clipped_padded_width, 0.0)
        }
        TextIconLayout::LeftAlignTextRightAlignIcon => {
            (0.0, bounds_size.width - icon_clipped_padded_width)
        }
    };

    let text_bounds_height = if unclipped_text_size.height + text_padding.top + text_padding.bottom
        <= bounds_size.height
    {
        unclipped_text_size.height
    } else {
        (bounds_size.height - text_padding.top - text_padding.bottom).max(0.0)
    };

    let icon_bounds_height =
        if icon_size.height + icon_padding.top + icon_padding.bottom <= bounds_size.height {
            icon_size.height
        } else {
            (bounds_size.height - icon_padding.top - icon_padding.bottom).max(0.0)
        };

    // We need to vertically align the text ourselves as rootvg/glyphon does not do this.
    let text_bounds_y = match style.vertical_align {
        crate::layout::Align::Start => text_padding.top,
        crate::layout::Align::Center => (bounds_size.height - text_bounds_height) * 0.5,
        crate::layout::Align::End => bounds_size.height - text_bounds_height - text_padding.bottom,
    };
    let icon_bounds_y = match style.vertical_align {
        crate::layout::Align::Start => icon_padding.top,
        crate::layout::Align::Center => (bounds_size.height - icon_bounds_height) * 0.5,
        crate::layout::Align::End => bounds_size.height - icon_bounds_height - icon_padding.bottom,
    };

    LayoutResult {
        text_bounds_rect: Rect::new(
            Point::new(text_padded_rect_x + text_padding.left, text_bounds_y),
            Size::new(
                (text_clipped_padded_width - text_padding.left - text_padding.right).max(0.0),
                text_bounds_height,
            ),
        ),
        icon_bounds_rect: Rect::new(
            Point::new(icon_padded_rect_x + icon_padding.left, icon_bounds_y),
            Size::new(
                (icon_clipped_padded_width - icon_padding.left - icon_padding.right).max(0.0),
                icon_bounds_height,
            ),
        ),
    }
}

fn layout_label_only(
    bounds_size: Size,
    unclipped_text_size: Size,
    padding: Padding,
    vertical_align: Align,
) -> Rect {
    if unclipped_text_size.is_empty() {
        return Rect::default();
    }

    let content_rect = crate::layout::layout_inner_rect_with_min_size(
        padding,
        Rect::from_size(bounds_size),
        Size::default(),
    );

    // We need to vertically align the text ourselves as rootvg/glyphon does not do this.
    let text_bounds_y = match vertical_align {
        crate::layout::Align::Start => content_rect.min_y(),
        crate::layout::Align::Center => {
            content_rect.min_y() + ((content_rect.height() - unclipped_text_size.height) * 0.5)
        }
        crate::layout::Align::End => content_rect.max_y() - unclipped_text_size.height,
    };

    Rect::new(
        Point::new(content_rect.min_x(), text_bounds_y),
        content_rect.size,
    )
}

fn layout_icon_only(size: Size, padding: &Padding, bounds_size: Size) -> Rect {
    let padded_size = Size::new(
        size.width + padding.left + padding.right,
        size.height + padding.top + padding.bottom,
    );

    let padded_rect =
        crate::layout::centered_rect(Rect::from_size(bounds_size).center(), padded_size);

    Rect::new(
        Point::new(
            padded_rect.min_x() + padding.left,
            padded_rect.min_y() + padding.top,
        ),
        size,
    )
}
