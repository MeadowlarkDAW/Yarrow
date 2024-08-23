use std::cell::RefCell;
use std::rc::Rc;

use crate::prelude::*;
use crate::theme::DEFAULT_ICON_SIZE;
use crate::vg::{
    quad::QuadPrimitive,
    text::{CustomGlyphDesc, TextPrimitive},
};

/// The style of an [`Icon`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconStyle {
    /// The size of the icon in points.
    ///
    /// By default this is set to `24.0`.
    pub size: f32,

    /// The color of the icon
    ///
    /// By default this is set to `color::WHITE`.
    pub color: RGBA8,

    /// The style of the padded background rectangle behind the icon.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    ///
    /// By default this is set to `QuadStyle::TRANSPARENT`.
    pub back_quad: QuadStyle,

    /// The padding between the icon and the bounding rectangle.
    ///
    /// By default this has all values set to `0.0`.
    pub padding: Padding,
}

impl IconStyle {
    pub fn padding_info(&self) -> IconPaddingInfo {
        IconPaddingInfo {
            icon_size: self.size,
            padding: self.padding,
        }
    }
}

impl Default for IconStyle {
    fn default() -> Self {
        Self {
            size: DEFAULT_ICON_SIZE,
            color: color::WHITE,
            back_quad: QuadStyle::TRANSPARENT,
            padding: Padding::zero(),
        }
    }
}

impl ElementStyle for IconStyle {
    const ID: &'static str = "icn";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self {
            color: color::WHITE,
            ..Default::default()
        }
    }
}

// Information used to calculate icon padding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IconPaddingInfo {
    pub icon_size: f32,
    pub padding: Padding,
}

#[derive(Debug, Clone)]
pub struct IconPrimitives {
    pub icon: TextPrimitive,
    pub bg_quad: Option<QuadPrimitive>,
}

/// A reusable icon struct that can be used by other elements.
///
/// Icons are assumed to be square.
pub struct IconInner {
    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub offset: Vector,
    pub icon_id: IconID,
    pub scale: IconScale,
    desired_size: Size,
    size_needs_calculated: bool,
}

impl IconInner {
    pub fn new(icon_id: IconID, scale: IconScale, offset: Vector) -> Self {
        Self {
            offset,
            icon_id,
            scale,
            desired_size: Size::default(),
            size_needs_calculated: true,
        }
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the icon.
    pub fn desired_size<F: FnOnce() -> IconPaddingInfo>(&mut self, get_padding_info: F) -> Size {
        if self.size_needs_calculated {
            self.size_needs_calculated = false;

            let info = (get_padding_info)();

            self.desired_size = Size::new(
                info.icon_size + info.padding.left + info.padding.right,
                info.icon_size + info.padding.top + info.padding.bottom,
            );
        }

        self.desired_size
    }

    /// Returns the rectangular area of the icon from the given bounds size
    /// (icons are assumed to be square).
    pub fn icon_rect(&self, style: &IconStyle, bounds_size: Size) -> Rect {
        layout(style.size, &style.padding, bounds_size)
    }

    pub fn notify_style_change(&mut self) {
        self.size_needs_calculated = true;
    }

    pub fn render_primitives(&mut self, bounds: Rect, style: &IconStyle) -> IconPrimitives {
        let icon_rect = self.icon_rect(style, bounds.size);

        let (size, offset) = if self.scale.0 != 1.0 {
            (
                style.size * self.scale.0,
                (style.size - (style.size * self.scale.0)) * 0.5,
            )
        } else {
            (style.size, 0.0)
        };

        IconPrimitives {
            icon: TextPrimitive::new_with_icons(
                None,
                bounds.origin + icon_rect.origin.to_vector() + self.offset,
                style.color,
                None,
                smallvec::smallvec![CustomGlyphDesc {
                    id: self.icon_id,
                    left: offset,
                    top: offset,
                    size,
                    color: None,
                    metadata: 0,
                }],
            ),
            bg_quad: if !style.back_quad.is_transparent() {
                Some(style.back_quad.create_primitive(bounds))
            } else {
                None
            },
        }
    }
}

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[derive(Default)]
pub struct IconBuilder {
    pub icon: IconID,
    pub scale: IconScale,
    pub offset: Vector,
}

impl IconBuilder {
    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> Icon {
        IconElement::create(self, cx)
    }

    pub fn icon(mut self, id: impl Into<IconID>) -> Self {
        self.icon = id.into();
        self
    }

    /// The scale of an icon, used to make icons look more consistent.
    ///
    /// Note this does not affect any layout, this is just a visual thing.
    pub fn icon_scale(mut self, scale: impl Into<IconScale>) -> Self {
        self.scale = scale.into();
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn offset(mut self, offset: Vector) -> Self {
        self.offset = offset;
        self
    }
}

/// An icon element with an optional quad background.
pub struct IconElement {
    shared_state: Rc<RefCell<SharedState>>,
}

impl IconElement {
    pub fn create<A: Clone + 'static>(builder: IconBuilder, cx: &mut WindowContext<'_, A>) -> Icon {
        let IconBuilder {
            icon,
            scale,
            offset,
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
        } = builder;

        let (z_index, scissor_rect, class) = cx.builder_values(z_index, scissor_rect, class);

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: IconInner::new(icon, scale, offset),
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
            }),
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        Icon { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for IconElement {
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

        let icon_primitives = shared_state.inner.render_primitives(
            Rect::from_size(cx.bounds_size),
            cx.res.style_system.get(cx.class),
        );

        if let Some(quad_primitive) = icon_primitives.bg_quad {
            primitives.add(quad_primitive);
        }

        primitives.set_z_index(1);
        primitives.add_text(icon_primitives.icon);
    }
}

struct SharedState {
    inner: IconInner,
}

/// A handle to a [`IconElement`], an icon with an optional quad background.
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
pub struct Icon {
    shared_state: Rc<RefCell<SharedState>>,
}

impl Icon {
    pub fn builder() -> IconBuilder {
        IconBuilder::default()
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the icon.
    ///
    /// This size is automatically cached, so it should be relatively
    /// inexpensive to call.
    pub fn desired_size(&self, res: &mut ResourceCtx) -> Size {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        shared_state.inner.desired_size(|| {
            res.style_system
                .get::<IconStyle>(self.el.class())
                .padding_info()
        })
    }

    /// Set the icon.
    ///
    /// Returns `true` if the icon has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_icon(&mut self, icon_id: impl Into<IconID>) -> bool {
        let icon_id: IconID = icon_id.into();

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon_id != icon_id {
            shared_state.inner.icon_id = icon_id;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn icon_id(&self) -> IconID {
        RefCell::borrow(&self.shared_state).inner.icon_id
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the offset has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_offset(&mut self, offset: Vector) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.offset != offset {
            shared_state.inner.offset = offset;
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
    pub fn set_scale(&mut self, scale: impl Into<IconScale>) -> bool {
        let scale: IconScale = scale.into();

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.scale != scale {
            shared_state.inner.scale = scale;
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

fn layout(size: f32, padding: &Padding, bounds_size: Size) -> Rect {
    let padded_size = Size::new(
        size + padding.left + padding.right,
        size + padding.top + padding.bottom,
    );

    let padded_rect =
        crate::layout::centered_rect(Rect::from_size(bounds_size).center(), padded_size);

    Rect::new(
        Point::new(
            padded_rect.min_x() + padding.left,
            padded_rect.min_y() + padding.top,
        ),
        Size::new(size, size),
    )
}
