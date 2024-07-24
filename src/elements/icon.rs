use std::cell::RefCell;
use std::rc::Rc;

use rootvg::text::{CustomGlyphDesc, CustomGlyphID, TextPrimitive};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus};
use crate::layout::{Align2, Padding};
use crate::math::{Point, Rect, Size, ZIndex};
use crate::style::QuadStyle;
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;

use super::label::LabelPrimitives;

/// The style of an [`Icon`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconStyle {
    /// The size of the icon in points.
    ///
    /// By default this is set to `20.0`.
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

impl Default for IconStyle {
    fn default() -> Self {
        Self {
            size: 20.0,
            color: color::WHITE,
            back_quad: QuadStyle::TRANSPARENT,
            padding: Padding::zero(),
        }
    }
}

/// A reusable icon struct that can be used by other elements.
///
/// Icons are assumed to be square.
pub struct IconInner {
    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub offset: Point,
    pub icon_id: CustomGlyphID,
    pub scale: f32,
}

impl IconInner {
    pub fn new(icon_id: CustomGlyphID, scale: f32, offset: Point) -> Self {
        Self {
            offset,
            icon_id,
            scale,
        }
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the icon.
    pub fn desired_padded_size(&self, style: &IconStyle) -> Size {
        Size::new(
            style.size + style.padding.left + style.padding.right,
            style.size + style.padding.top + style.padding.bottom,
        )
    }

    /// Returns the rectangular area of the icon from the given bounds size
    /// (icons are assumed to be square).
    pub fn icon_rect(&self, style: &IconStyle, bounds_size: Size) -> Rect {
        layout(style.size, &style.padding, bounds_size)
    }

    pub fn render_primitives(&mut self, bounds: Rect, style: &IconStyle) -> LabelPrimitives {
        let icon_rect = self.icon_rect(style, bounds.size);

        let (size, offset) = if self.scale != 1.0 {
            (
                style.size * self.scale,
                (style.size - (style.size * self.scale)) * 0.5,
            )
        } else {
            (style.size, 0.0)
        };

        LabelPrimitives {
            text: Some(TextPrimitive::new_with_icons(
                None,
                bounds.origin + icon_rect.origin.to_vector() + self.offset.to_vector(),
                style.color,
                Some(Rect::new(
                    Point::new(-1.0, -1.0),
                    Size::new(icon_rect.size.width + 2.0, icon_rect.size.height + 2.0),
                )),
                smallvec::smallvec![CustomGlyphDesc {
                    id: self.icon_id,
                    left: offset,
                    top: offset,
                    size,
                    color: None,
                    metadata: 0,
                }],
            )),
            bg_quad: if !style.back_quad.is_transparent() {
                Some(style.back_quad.create_primitive(bounds))
            } else {
                None
            },
        }
    }
}

pub struct IconBuilder {
    pub icon: CustomGlyphID,
    pub scale: f32,
    pub offset: Point,
    pub style: Rc<IconStyle>,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl IconBuilder {
    pub fn new(style: &Rc<IconStyle>) -> Self {
        Self {
            icon: CustomGlyphID::MAX,
            scale: 1.0,
            offset: Point::default(),
            style: Rc::clone(style),
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: None,
        }
    }

    pub fn build<A: Clone + 'static>(self, cx: &mut WindowContext<'_, A>) -> Icon {
        IconElement::create(self, cx)
    }

    pub fn icon(mut self, id: impl Into<CustomGlyphID>) -> Self {
        self.icon = id.into();
        self
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
    pub const fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn offset(mut self, offset: Point) -> Self {
        self.offset = offset;
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
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id) = cx.z_index_and_scissor_rect_id(z_index, scissor_rect_id);

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: IconInner::new(icon, scale, offset),
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
        let SharedState { inner, style } = &mut *shared_state;

        let label_primitives = inner.render_primitives(Rect::from_size(cx.bounds_size), style);

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
    inner: IconInner,
    style: Rc<IconStyle>,
}

/// A handle to a [`IconElement`], an icon with an optional quad background.
pub struct Icon {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl Icon {
    pub fn builder(style: &Rc<IconStyle>) -> IconBuilder {
        IconBuilder::new(style)
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the icon.
    ///
    /// This can be useful to lay out elements that depend on icon size.
    pub fn desired_padded_size(&self) -> Size {
        let shared_state = RefCell::borrow(&self.shared_state);

        shared_state.inner.desired_padded_size(&shared_state.style)
    }

    pub fn set_icon_id(&mut self, icon_id: impl Into<CustomGlyphID>) {
        let icon_id: CustomGlyphID = icon_id.into();

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.icon_id != icon_id {
            shared_state.inner.icon_id = icon_id;
            self.el.notify_custom_state_change();
        }
    }

    pub fn icon_id(&self) -> CustomGlyphID {
        RefCell::borrow(&self.shared_state).inner.icon_id
    }

    pub fn set_style(&mut self, style: &Rc<IconStyle>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<IconStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub fn set_offset(&mut self, offset: Point) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.offset != offset {
            shared_state.inner.offset = offset;
            self.el.notify_custom_state_change();
        }
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
    pub fn set_scale(&mut self, scale: f32) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.scale != scale {
            shared_state.inner.scale = scale;
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

pub(crate) fn layout(size: f32, padding: &Padding, bounds_size: Size) -> Rect {
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
