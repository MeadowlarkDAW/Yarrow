use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::text::CustomGlyphID;
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align2, LayoutDirection};
use crate::math::{Rect, Size, ZIndex, Vector};
use crate::prelude::{ElementStyle, ResourceCtx};
use crate::style::QuadStyle;
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;
use crate::CursorIcon;

use super::button::ButtonState;
use super::label::TextIconLayout;
use super::toggle_button::{ToggleButtonInner, ToggleButtonStyle};

/// The style of a [`Tab`] element
#[derive(Debug, Clone, PartialEq)]
pub struct TabStyle {
    pub toggle_btn_style: ToggleButtonStyle,

    pub on_indicator_line_width: f32,
    pub on_indicator_line_style: QuadStyle,
    pub on_indicator_line_padding_to_edges: f32,
}

impl Default for TabStyle {
    fn default() -> Self {
        Self {
            toggle_btn_style: ToggleButtonStyle::default(),
            on_indicator_line_width: 0.0,
            on_indicator_line_style: QuadStyle::TRANSPARENT,
            on_indicator_line_padding_to_edges: 0.0,
        }
    }
}

impl ElementStyle for TabStyle {
    const ID: &'static str = "tab";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        todo!()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IndicatorLinePlacement {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

pub struct TabBuilder<A: Clone + 'static> {
    pub action: Option<A>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub toggled: bool,
    pub text: Option<String>,
    pub icon: Option<CustomGlyphID>,
    pub icon_scale: f32,
    pub text_offset: Vector,
    pub icon_offset: Vector,
    pub text_icon_layout: TextIconLayout,
    pub on_indicator_line_placement: IndicatorLinePlacement,
    pub class: Option<&'static str>,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub disabled: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> TabBuilder<A> {
    pub fn new() -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            toggled: false,
            text: None,
            icon: None,
            icon_scale: 1.0,
            text_offset: Vector::default(),
            icon_offset: Vector::default(),
            text_icon_layout: TextIconLayout::default(),
            class: None,
            on_indicator_line_placement: IndicatorLinePlacement::Top,
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            disabled: false,
            scissor_rect_id: None,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> Tab {
        TabElement::create(self, cx)
    }

    pub fn on_toggled_on(mut self, action: A) -> Self {
        self.action = Some(action);
        self
    }

    pub fn tooltip_message(mut self, message: impl Into<String>, align: Align2) -> Self {
        let msg: String = message.into();
        self.tooltip_message = if msg.is_empty() { None } else { Some(msg) };
        self.tooltip_align = align;
        self
    }

    pub const fn on_indicator_line_placement(mut self, placement: IndicatorLinePlacement) -> Self {
        self.on_indicator_line_placement = placement;
        self
    }

    pub const fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }

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
    pub fn icon(mut self, icon: impl Into<CustomGlyphID>) -> Self {
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
    pub fn icon_optional(mut self, icon: Option<impl Into<CustomGlyphID>>) -> Self {
        self.icon = icon.map(|i| i.into());
        self
    }

    /// The scaling factor for the icon
    ///
    /// By default this is set to `1.0`.
    pub const fn icon_scale(mut self, scale: f32) -> Self {
        self.icon_scale = scale;
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
    pub const fn bounding_rect(mut self, rect: Rect) -> Self {
        self.bounding_rect = rect;
        self
    }

    /// Whether or not this element is manually hidden
    ///
    /// By default this is set to `false`.
    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.manually_hidden = hidden;
        self
    }

    /// Whether or not this element is in the disabled state
    ///
    /// By default this is set to `false`.
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

/// A button element with a label.
pub struct TabElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<A>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
    on_indicator_line_placement: IndicatorLinePlacement,
    cursor_icon: Option<CursorIcon>,
}

impl<A: Clone + 'static> TabElement<A> {
    pub fn create(builder: TabBuilder<A>, cx: &mut WindowContext<'_, A>) -> Tab {
        let TabBuilder {
            action,
            tooltip_message,
            tooltip_align,
            toggled,
            text,
            icon,
            icon_scale,
            text_offset,
            icon_offset,
            text_icon_layout,
            class,
            on_indicator_line_placement,
            z_index,
            bounding_rect,
            manually_hidden,
            disabled,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);
        let style = cx.res.style_system.get::<TabStyle>(class);
        let cursor_icon = style.toggle_btn_style.cursor_icon;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: ToggleButtonInner::new(
                text,
                icon,
                text_offset,
                icon_offset,
                icon_scale,
                toggled,
                disabled,
                text_icon_layout,
                &style.toggle_btn_style,
                &mut cx.res.font_system,
            ),
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                action,
                tooltip_message,
                tooltip_align,
                on_indicator_line_placement,
                cursor_icon,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        Tab { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for TabElement<A> {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        match event {
            ElementEvent::CustomStateChanged => {
                cx.request_repaint();
            }
            ElementEvent::StyleChanged => {
                let style = cx.res.style_system.get::<TabStyle>(cx.class());
                self.cursor_icon = style.toggle_btn_style.cursor_icon;
            }
            ElementEvent::Pointer(PointerEvent::Moved { just_entered, .. }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                if shared_state.inner.state() == ButtonState::Disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if let Some(cursor_icon) = self.cursor_icon {
                    cx.cursor_icon = cursor_icon;
                }

                if just_entered && self.tooltip_message.is_some() {
                    cx.start_hover_timeout();
                }

                if shared_state.inner.state() == ButtonState::Idle {
                    let needs_repaint = shared_state.inner.set_state(ButtonState::Hovered);

                    if needs_repaint {
                        cx.request_repaint();
                    }
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                if shared_state.inner.state() == ButtonState::Hovered
                    || shared_state.inner.state() == ButtonState::Down
                {
                    let needs_repaint = shared_state.inner.set_state(ButtonState::Idle);

                    if needs_repaint {
                        cx.request_repaint();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed { button, .. }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                if button == PointerButton::Primary
                    && (shared_state.inner.state() == ButtonState::Idle
                        || shared_state.inner.state() == ButtonState::Hovered)
                {
                    shared_state.inner.set_state(ButtonState::Down);

                    if !shared_state.inner.toggled {
                        shared_state.inner.toggled = true;

                        if let Some(action) = &self.action {
                            cx.send_action(action.clone()).unwrap();
                        }

                        cx.request_repaint();
                    }

                    cx.request_repaint();
                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                position, button, ..
            }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                if button == PointerButton::Primary
                    && (shared_state.inner.state() == ButtonState::Down
                        || shared_state.inner.state() == ButtonState::Hovered)
                {
                    let new_state = if cx.is_point_within_visible_bounds(position) {
                        ButtonState::Hovered
                    } else {
                        ButtonState::Idle
                    };

                    let needs_repaint = shared_state.inner.set_state(new_state);

                    if needs_repaint {
                        cx.request_repaint();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::HoverTimeout { .. }) => {
                if let Some(message) = &self.tooltip_message {
                    cx.show_tooltip(message.clone(), self.tooltip_align, true);
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let style: &TabStyle = cx.res.style_system.get(cx.class);

        let label_primitives = shared_state.inner.render_primitives(
            Rect::from_size(cx.bounds_size),
            &style.toggle_btn_style,
            &mut cx.res.font_system,
        );

        if let Some(quad_primitive) = label_primitives.bg_quad {
            primitives.add(quad_primitive);
        }

        if let Some(p) = label_primitives.text {
            primitives.set_z_index(1);
            primitives.add_text(p);
        }
        if let Some(p) = label_primitives.icon {
            primitives.set_z_index(1);
            primitives.add_text(p);
        }

        if style.on_indicator_line_width > 0.0
            && !style.on_indicator_line_style.is_transparent()
            && shared_state.inner.toggled
        {
            primitives.set_z_index(1);

            let line_rect = match self.on_indicator_line_placement {
                IndicatorLinePlacement::Top => Rect::new(
                    Point::new(style.on_indicator_line_padding_to_edges, 0.0),
                    Size::new(
                        cx.bounds_size.width - (style.on_indicator_line_padding_to_edges * 2.0),
                        style.on_indicator_line_width,
                    ),
                ),
                IndicatorLinePlacement::Bottom => Rect::new(
                    Point::new(
                        style.on_indicator_line_padding_to_edges,
                        cx.bounds_size.height - style.on_indicator_line_width,
                    ),
                    Size::new(
                        cx.bounds_size.width - (style.on_indicator_line_padding_to_edges * 2.0),
                        style.on_indicator_line_width,
                    ),
                ),
                IndicatorLinePlacement::Left => Rect::new(
                    Point::new(0.0, style.on_indicator_line_padding_to_edges),
                    Size::new(
                        style.on_indicator_line_width,
                        cx.bounds_size.height - (style.on_indicator_line_padding_to_edges * 2.0),
                    ),
                ),
                IndicatorLinePlacement::Right => Rect::new(
                    Point::new(
                        cx.bounds_size.width - style.on_indicator_line_width,
                        style.on_indicator_line_padding_to_edges,
                    ),
                    Size::new(
                        style.on_indicator_line_width,
                        cx.bounds_size.height - (style.on_indicator_line_padding_to_edges * 2.0),
                    ),
                ),
            };

            primitives.add(style.on_indicator_line_style.create_primitive(line_rect));
        }
    }
}

/// A handle to a [`TabElement`].
pub struct Tab {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    inner: ToggleButtonInner,
}

impl Tab {
    pub fn builder<A: Clone + 'static>() -> TabBuilder<A> {
        TabBuilder::new()
    }

    pub fn set_toggled(&mut self, toggled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.toggled != toggled {
            shared_state.inner.toggled = toggled;
            self.el._notify_custom_state_change();
        }
    }

    pub fn toggled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.toggled
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the text and icon.
    pub fn desired_size(&self, res: &mut ResourceCtx) -> Size {
        RefCell::borrow_mut(&self.shared_state)
            .inner
            .desired_size(|| {
                res.style_system
                    .get::<TabStyle>(self.el.class())
                    .toggle_btn_style
                    .padding_info()
            })
    }

    pub fn set_text(&mut self, text: Option<&str>, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_text(text, &mut res.font_system, || {
            res.style_system
                .get::<TabStyle>(self.el.class())
                .toggle_btn_style
                .text_properties
        }) {
            self.el._notify_custom_state_change();
        }
    }

    pub fn set_icon(&mut self, icon: Option<impl Into<CustomGlyphID>>) {
        let icon: Option<CustomGlyphID> = icon.map(|i| i.into());

        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon(icon) {
            self.el._notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Option<Ref<'a, str>> {
        Ref::filter_map(RefCell::borrow(&self.shared_state), |s| s.inner.text()).ok()
    }

    pub fn icon(&self) -> Option<CustomGlyphID> {
        RefCell::borrow(&self.shared_state).inner.icon()
    }

    pub fn set_class(&mut self, class: &'static str, res: &mut ResourceCtx) {
        if self.el.class() != class {
            RefCell::borrow_mut(&self.shared_state)
                .inner
                .sync_new_style(res.style_system.get(class), &mut res.font_system);

            self.el._notify_class_change(class);
        }
    }

    /// An offset that can be used mainly to correct the position of the text.
    ///
    /// This does not effect the position of the background quad.
    pub fn set_text_offset(&mut self, offset: Vector) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_text_offset(offset) {
            self.el._notify_custom_state_change();
        }
    }

    /// An offset that can be used mainly to correct the position of the icon.
    ///
    /// This does not effect the position of the background quad.
    pub fn set_icon_offset(&mut self, offset: Vector) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon_offset(offset) {
            self.el._notify_custom_state_change();
        }
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
    ///
    /// This does no effect the padded size of the element.
    pub fn set_icon_scale(&mut self, scale: f32) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon_scale(scale) {
            self.el._notify_custom_state_change();
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if disabled && shared_state.inner.state() != ButtonState::Disabled {
            shared_state.inner.set_state(ButtonState::Disabled);
            self.el._notify_custom_state_change();
        } else if !disabled && shared_state.inner.state() == ButtonState::Disabled {
            shared_state.inner.set_state(ButtonState::Idle);
            self.el._notify_custom_state_change();
        }
    }

    pub fn disabled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.state() == ButtonState::Disabled
    }

    pub fn layout(&mut self, origin: Point, res: &mut ResourceCtx) {
        let size = self.desired_size(res);
        self.el.set_rect(Rect::new(origin, size));
    }

    pub fn layout_aligned(&mut self, point: Point, align: Align2, res: &mut ResourceCtx) {
        let size = self.desired_size(res);
        self.el.set_rect(align.align_rect_to_point(point, size));
    }
}

#[derive(Default, Debug, Clone)]
pub struct TabGroupOption {
    pub text: Option<String>,
    pub icon: Option<CustomGlyphID>,
    pub tooltip_message: String,
    pub text_offset: Vector,
    pub icon_offset: Vector,
    pub icon_scale: f32,
    pub disabled: bool,
}

impl TabGroupOption {
    pub fn new(
        text: Option<String>,
        icon: Option<CustomGlyphID>,
        tooltip_message: impl Into<String>,
    ) -> Self {
        Self {
            text,
            icon,
            tooltip_message: tooltip_message.into(),
            text_offset: Vector::default(),
            icon_offset: Vector::default(),
            icon_scale: 1.0,
            disabled: false,
        }
    }
}

pub struct TabGroup {
    tabs: Vec<Tab>,
    selected_index: usize,
    bounds: Rect,
}

impl TabGroup {
    pub fn new<'a, A: Clone + 'static, F>(
        options: impl IntoIterator<Item = impl Into<TabGroupOption>>,
        selected_index: usize,
        mut on_selected: F,
        class: Option<&'static str>,
        on_indicator_line_placement: IndicatorLinePlacement,
        tooltip_align: Align2,
        z_index: Option<ZIndex>,
        scissor_rect_id: Option<ScissorRectID>,
        cx: &mut WindowContext<A>,
    ) -> Self
    where
        F: FnMut(usize) -> A + 'static,
    {
        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let tabs: Vec<Tab> = options
            .into_iter()
            .enumerate()
            .map(|(i, option)| {
                let option: TabGroupOption = option.into();

                Tab::builder()
                    .text_optional(option.text)
                    .icon_optional(option.icon)
                    .icon_scale(option.icon_scale)
                    .class(class)
                    .tooltip_message(option.tooltip_message, tooltip_align)
                    .on_toggled_on((on_selected)(i))
                    .toggled(i == selected_index)
                    .on_indicator_line_placement(on_indicator_line_placement)
                    .z_index(z_index)
                    .scissor_rect(scissor_rect_id)
                    .text_offset(option.text_offset)
                    .icon_offset(option.icon_offset)
                    .disabled(option.disabled)
                    .build(cx)
            })
            .collect();

        Self {
            tabs,
            selected_index,
            bounds: Rect::default(),
        }
    }

    pub fn layout(
        &mut self,
        origin: Point,
        spacing: f32,
        direction: LayoutDirection,
        stretch_to_fit: Option<f32>,
        res: &mut ResourceCtx,
    ) {
        self.bounds.origin = origin;

        if self.tabs.is_empty() {
            self.bounds.size = Size::default();
            return;
        }

        if let LayoutDirection::Horizontal = direction {
            let mut x = origin.x;

            let max_height = stretch_to_fit.unwrap_or_else(|| {
                let mut max_height: f32 = 0.0;
                for tab in self.tabs.iter() {
                    let size = tab.desired_size(res);
                    max_height = max_height.max(size.height);
                }
                max_height
            });

            for tab in self.tabs.iter_mut() {
                let size = tab.desired_size(res);

                tab.el.set_rect(Rect::new(
                    Point::new(x, origin.y),
                    Size::new(size.width, max_height),
                ));

                x += size.width + spacing;
            }

            self.bounds = Rect::new(origin, Size::new(x - spacing, max_height));
        } else {
            let mut y = origin.y;

            let max_width = stretch_to_fit.unwrap_or_else(|| {
                let mut max_width: f32 = 0.0;
                for tab in self.tabs.iter() {
                    let size = tab.desired_size(res);
                    max_width = max_width.max(size.width);
                }
                max_width
            });

            for tab in self.tabs.iter_mut() {
                let size = tab.desired_size(res);

                tab.el.set_rect(Rect::new(
                    Point::new(origin.x, y),
                    Size::new(max_width, size.height),
                ));

                y += size.height + spacing;
            }

            self.bounds = Rect::new(origin, Size::new(max_width, y - spacing));
        }
    }

    pub fn updated_selected(&mut self, selected_index: usize) {
        let selected_index = if selected_index >= self.tabs.len() {
            0
        } else {
            selected_index
        };

        if self.selected_index == selected_index {
            return;
        }

        if let Some(prev_selected_tab) = self.tabs.get_mut(self.selected_index) {
            prev_selected_tab.set_toggled(false);
        }

        self.selected_index = selected_index;

        self.tabs[selected_index].set_toggled(true);
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        for tab in self.tabs.iter_mut() {
            tab.el.set_hidden(hidden);
        }
    }
}
