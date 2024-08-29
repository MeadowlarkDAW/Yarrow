use derive_where::derive_where;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::derive::*;
use crate::prelude::*;

use super::button::ButtonState;
use super::toggle_button::ToggleButtonInner;

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

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[element_builder_disabled]
#[element_builder_tooltip]
#[derive_where(Default)]
pub struct TabBuilder<A: Clone + 'static> {
    pub action: Option<A>,
    pub toggled: bool,
    pub text: Option<String>,
    pub icon: Option<IconID>,
    pub icon_size: Option<Size>,
    pub icon_scale: IconScale,
    pub text_offset: Vector,
    pub icon_offset: Vector,
    pub text_icon_layout: TextIconLayout,
    pub on_indicator_line_placement: IndicatorLinePlacement,
}

impl<A: Clone + 'static> TabBuilder<A> {
    pub fn on_toggled_on(mut self, action: A) -> Self {
        self.action = Some(action);
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

    pub fn build(self, window_cx: &mut WindowContext<'_, A>) -> Tab {
        let TabBuilder {
            action,
            tooltip_data,
            toggled,
            text,
            icon,
            icon_size,
            icon_scale,
            text_offset,
            icon_offset,
            text_icon_layout,
            class,
            on_indicator_line_placement,
            z_index,
            rect,
            manually_hidden,
            disabled,
            scissor_rect,
        } = self;

        let style = window_cx
            .res
            .style_system
            .get::<TabStyle>(window_cx.builder_class(class));
        let cursor_icon = style.toggle_btn_style.cursor_icon;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: ToggleButtonInner::new(
                text,
                icon,
                text_offset,
                icon_offset,
                icon_size,
                icon_scale,
                toggled,
                disabled,
                text_icon_layout,
                &style.toggle_btn_style,
                &mut window_cx.res.font_system,
            ),
            tooltip_inner: TooltipInner::new(tooltip_data),
        }));

        let el = ElementBuilder::new(TabElement {
            shared_state: Rc::clone(&shared_state),
            action,
            on_indicator_line_placement,
            cursor_icon,
        })
        .builder_values(z_index, scissor_rect, class, window_cx)
        .rect(rect)
        .hidden(manually_hidden)
        .flags(ElementFlags::PAINTS | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
        .build(window_cx);

        Tab { el, shared_state }
    }
}

/// A button element with a label.
struct TabElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<A>,
    on_indicator_line_placement: IndicatorLinePlacement,
    cursor_icon: Option<CursorIcon>,
}

impl<A: Clone + 'static> Element<A> for TabElement<A> {
    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        shared_state
            .tooltip_inner
            .handle_event(&event, shared_state.inner.disabled(), cx);

        match event {
            ElementEvent::CustomStateChanged => {
                cx.request_repaint();
            }
            ElementEvent::StyleChanged => {
                let style = cx.res.style_system.get::<TabStyle>(cx.class());
                self.cursor_icon = style.toggle_btn_style.cursor_icon;
            }
            ElementEvent::Pointer(PointerEvent::Moved { .. }) => {
                if shared_state.inner.state() == ButtonState::Disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                if let Some(cursor_icon) = self.cursor_icon {
                    cx.cursor_icon = cursor_icon;
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
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render(&mut self, cx: RenderContext, primitives: &mut PrimitiveGroup) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let style: &TabStyle = cx.res.style_system.get(cx.class);

        let label_primitives = shared_state.inner.render(
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

struct SharedState {
    inner: ToggleButtonInner,
    tooltip_inner: TooltipInner,
}

/// A handle to a [`TabElement`].
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
#[element_handle_set_tooltip]
pub struct Tab {
    shared_state: Rc<RefCell<SharedState>>,
}
impl Tab {
    pub fn builder<A: Clone + 'static>() -> TabBuilder<A> {
        TabBuilder::default()
    }

    /// Set the toggled state of this element.
    ///
    /// Returns `true` if the toggle state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_toggled(&mut self, toggled: bool) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.toggled != toggled {
            shared_state.inner.toggled = toggled;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn toggled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.toggled
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
                    .get::<TabStyle>(self.el.class())
                    .toggle_btn_style
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
                .get::<TabStyle>(self.el.class())
                .toggle_btn_style
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
    /// This size is automatically cached, so it should be relatively
    /// inexpensive to call.
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
        RefCell::borrow(&self.shared_state).inner.icon()
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

        if shared_state.inner.set_text_offset(offset) {
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

        if shared_state.inner.set_icon_offset(offset) {
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
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.set_icon_scale(scale.into()) {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// Set the disabled state of this element.
    ///
    /// Returns `true` if the disabled state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_disabled(&mut self, disabled: bool) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if disabled && shared_state.inner.state() != ButtonState::Disabled {
            shared_state.inner.set_state(ButtonState::Disabled);
            self.el.notify_custom_state_change();
            true
        } else if !disabled && shared_state.inner.state() == ButtonState::Disabled {
            shared_state.inner.set_state(ButtonState::Idle);
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn disabled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.state() == ButtonState::Disabled
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

#[derive(Default, Debug, Clone)]
pub struct TabGroupOption {
    pub text: Option<String>,
    pub icon: Option<IconID>,
    pub tooltip_text: Option<String>,
    pub text_offset: Vector,
    pub icon_offset: Vector,
    pub icon_scale: f32,
    pub disabled: bool,
}

impl TabGroupOption {
    pub fn new(
        text: Option<String>,
        icon: Option<IconID>,
        tooltip_text: Option<impl Into<String>>,
    ) -> Self {
        Self {
            text,
            icon,
            tooltip_text: tooltip_text.map(|t| t.into()),
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
        class: Option<ClassID>,
        on_indicator_line_placement: IndicatorLinePlacement,
        tooltip_align: Align2,
        z_index: Option<ZIndex>,
        scissor_rect: Option<ScissorRectID>,
        window_cx: &mut WindowContext<A>,
    ) -> Self
    where
        F: FnMut(usize) -> A + 'static,
    {
        let z_index = z_index.unwrap_or_else(|| window_cx.z_index());
        let class = class.unwrap_or_else(|| window_cx.class());
        let scissor_rect = scissor_rect.unwrap_or_else(|| window_cx.scissor_rect());

        let tabs: Vec<Tab> = options
            .into_iter()
            .enumerate()
            .map(|(i, option)| {
                let option: TabGroupOption = option.into();

                let mut tab = Tab::builder()
                    .text_optional(option.text)
                    .icon_optional(option.icon)
                    .icon_scale(option.icon_scale)
                    .class(class)
                    .on_toggled_on((on_selected)(i))
                    .toggled(i == selected_index)
                    .on_indicator_line_placement(on_indicator_line_placement)
                    .z_index(z_index)
                    .scissor_rect(scissor_rect)
                    .text_offset(option.text_offset)
                    .icon_offset(option.icon_offset)
                    .disabled(option.disabled);

                if let Some(text) = option.tooltip_text {
                    tab = tab.tooltip(text, tooltip_align);
                }

                tab.build(window_cx)
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
