use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::text::{CustomGlyphID, TextProperties};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align2, LayoutDirection, Padding};
use crate::math::{Rect, Size, ZIndex};
use crate::prelude::ResourceCtx;
use crate::style::{
    Background, BorderStyle, QuadStyle, DEFAULT_ACCENT_COLOR, DEFAULT_TEXT_ATTRIBUTES,
};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;
use crate::CursorIcon;

use super::button::ButtonState;
use super::icon_label::{IconLabelClipMode, IconLabelLayout};
use super::icon_label_button::IconLabelButtonStylePart;
use super::icon_label_toggle_button::{IconLabelToggleButtonInner, IconLabelToggleButtonStyle};
use super::icon_toggle_button::ToggleIcons;
use super::tab::IndicatorLinePlacement;
use super::toggle_button::ToggleText;

/// The style of a [`IconLabelTab`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconLabelTabStyle {
    pub toggle_btn_style: IconLabelToggleButtonStyle,

    pub on_indicator_line_width: f32,
    pub on_indicator_line_style: QuadStyle,
    pub on_indicator_line_padding_to_edges: f32,
}

impl Default for IconLabelTabStyle {
    fn default() -> Self {
        let idle_on = IconLabelButtonStylePart {
            text_color: color::WHITE,
            icon_color: color::WHITE,
            back_quad: QuadStyle {
                bg: Background::Solid(RGBA8::new(70, 70, 70, 255)),
                border: BorderStyle::default(),
            },
        };

        let idle_off = IconLabelButtonStylePart {
            text_color: RGBA8::new(255, 255, 255, 125),
            icon_color: RGBA8::new(255, 255, 255, 125),
            back_quad: QuadStyle {
                bg: Background::Solid(color::TRANSPARENT),
                ..idle_on.back_quad
            },
            ..idle_on
        };

        Self {
            toggle_btn_style: IconLabelToggleButtonStyle {
                text_properties: TextProperties {
                    attrs: DEFAULT_TEXT_ATTRIBUTES,
                    ..Default::default()
                },
                icon_size: 20.0,

                text_min_clipped_size: Size::new(5.0, 5.0),

                text_padding: Padding::new(6.0, 6.0, 6.0, 6.0),
                icon_padding: Padding::new(0.0, 0.0, 0.0, 6.0),

                clip_mode: IconLabelClipMode::default(),
                layout: IconLabelLayout::default(),

                idle_on: idle_on.clone(),
                down_on: idle_on.clone(),
                hovered_on: IconLabelButtonStylePart {
                    back_quad: QuadStyle {
                        ..idle_on.back_quad
                    },
                    ..idle_on
                },
                disabled_on: IconLabelButtonStylePart {
                    text_color: RGBA8::new(255, 255, 255, 150),
                    icon_color: RGBA8::new(255, 255, 255, 150),
                    back_quad: QuadStyle {
                        bg: Background::Solid(RGBA8::new(50, 50, 50, 255)),
                        ..idle_on.back_quad
                    },
                    ..idle_on
                },

                idle_off: idle_off.clone(),
                down_off: idle_off.clone(),
                hovered_off: IconLabelButtonStylePart {
                    back_quad: QuadStyle {
                        bg: Background::Solid(RGBA8::new(50, 50, 50, 255)),
                        ..idle_off.back_quad
                    },
                    ..idle_off
                },
                disabled_off: IconLabelButtonStylePart {
                    text_color: RGBA8::new(255, 255, 255, 100),
                    icon_color: RGBA8::new(255, 255, 255, 100),
                    ..idle_off
                },
            },
            on_indicator_line_width: 3.0,
            on_indicator_line_style: QuadStyle {
                bg: Background::Solid(DEFAULT_ACCENT_COLOR),
                border: BorderStyle::default(),
            },
            on_indicator_line_padding_to_edges: 0.0,
        }
    }
}

pub struct IconLabelTabBuilder<A: Clone + 'static> {
    pub action: Option<A>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub toggled: bool,
    pub text: Option<String>,
    pub icons: Option<ToggleIcons>,
    pub icon_scale: f32,
    pub text_offset: Point,
    pub icon_offset: Point,
    pub style: Rc<IconLabelTabStyle>,
    pub on_indicator_line_placement: IndicatorLinePlacement,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub disabled: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl<A: Clone + 'static> IconLabelTabBuilder<A> {
    pub fn new(style: &Rc<IconLabelTabStyle>) -> Self {
        Self {
            action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            toggled: false,
            text: None,
            icons: None,
            icon_scale: 1.0,
            text_offset: Point::default(),
            icon_offset: Point::default(),
            style: Rc::clone(style),
            on_indicator_line_placement: IndicatorLinePlacement::Top,
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            disabled: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> IconLabelTab {
        IconLabelTabElement::create(self, cx)
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

    pub const fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = toggled;
        self
    }

    pub fn text(mut self, text: Option<impl Into<String>>) -> Self {
        self.text = text.map(|t| t.into());
        self
    }

    pub fn icon(mut self, icon_id: Option<impl Into<CustomGlyphID>>) -> Self {
        self.icons = icon_id.map(|i| ToggleIcons::Single(i.into()));
        self
    }

    pub const fn icons(mut self, icons: Option<ToggleIcons>) -> Self {
        self.icons = icons;
        self
    }

    pub fn dual_icons(
        mut self,
        off_on_ids: Option<(impl Into<CustomGlyphID>, impl Into<CustomGlyphID>)>,
    ) -> Self {
        self.icons = off_on_ids.map(|(off_id, on_id)| ToggleIcons::Dual {
            off: off_id.into(),
            on: on_id.into(),
        });
        self
    }

    /// Scale the icon when rendering (used to help make icons look consistent).
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

    pub const fn on_indicator_line_placement(mut self, placement: IndicatorLinePlacement) -> Self {
        self.on_indicator_line_placement = placement;
        self
    }

    pub const fn z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = z_index;
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

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = scissor_rect_id;
        self
    }
}

/// A button element with a label.
pub struct IconLabelTabElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<A>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
    on_indicator_line_placement: IndicatorLinePlacement,
}

impl<A: Clone + 'static> IconLabelTabElement<A> {
    pub fn create(builder: IconLabelTabBuilder<A>, cx: &mut WindowContext<'_, A>) -> IconLabelTab {
        let IconLabelTabBuilder {
            action,
            tooltip_message,
            tooltip_align,
            toggled,
            text,
            icons,
            icon_scale,
            text_offset,
            icon_offset,
            style,
            on_indicator_line_placement,
            z_index,
            bounding_rect,
            manually_hidden,
            disabled,
            scissor_rect_id,
        } = builder;

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: IconLabelToggleButtonInner::new(
                text.map(|t| ToggleText::Single(t)),
                icons,
                text_offset,
                icon_offset,
                icon_scale,
                toggled,
                disabled,
                &style.toggle_btn_style,
                &mut cx.res,
            ),
            style,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                action,
                tooltip_message,
                tooltip_align,
                on_indicator_line_placement,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        IconLabelTab { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for IconLabelTabElement<A> {
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
            ElementEvent::Pointer(PointerEvent::Moved { just_entered, .. }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState { inner, style } = &mut *shared_state;

                if inner.state() == ButtonState::Disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                cx.cursor_icon = CursorIcon::Pointer;

                if just_entered && self.tooltip_message.is_some() {
                    cx.start_hover_timeout();
                }

                if inner.state() == ButtonState::Idle {
                    let res = inner.set_state(ButtonState::Hovered, &style.toggle_btn_style);

                    if res.needs_repaint {
                        cx.request_repaint();
                    }
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState { inner, style, .. } = &mut *shared_state;

                if inner.state() == ButtonState::Hovered || inner.state() == ButtonState::Down {
                    let res = inner.set_state(ButtonState::Idle, &style.toggle_btn_style);

                    if res.needs_repaint {
                        cx.request_repaint();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed { button, .. }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState { inner, style, .. } = &mut *shared_state;

                if button == PointerButton::Primary
                    && (inner.state() == ButtonState::Idle || inner.state() == ButtonState::Hovered)
                    && !inner.toggled()
                {
                    let res1 = inner.set_state(ButtonState::Down, &style.toggle_btn_style);
                    let res2 =
                        inner.set_toggled(!inner.toggled(), &style.toggle_btn_style, &mut cx.res);

                    if res1.needs_repaint || res2.needs_repaint {
                        cx.request_repaint();
                    }

                    if let Some(action) = &self.action {
                        cx.send_action(action.clone()).unwrap();
                    }

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                position, button, ..
            }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState { inner, style, .. } = &mut *shared_state;

                if button == PointerButton::Primary
                    && (inner.state() == ButtonState::Down || inner.state() == ButtonState::Hovered)
                {
                    let new_state = if cx.is_point_within_visible_bounds(position) {
                        ButtonState::Hovered
                    } else {
                        ButtonState::Idle
                    };

                    let res = inner.set_state(new_state, &style.toggle_btn_style);

                    if res.needs_repaint {
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
        let SharedState { inner, style, .. } = &mut *shared_state;

        let label_primitives = inner.render_primitives(
            Rect::from_size(cx.bounds_size),
            &style.toggle_btn_style,
            cx.res,
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
            && inner.toggled()
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

/// A handle to a [`IconLabelTabElement`].
pub struct IconLabelTab {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

struct SharedState {
    inner: IconLabelToggleButtonInner,
    style: Rc<IconLabelTabStyle>,
}

impl IconLabelTab {
    pub fn builder<A: Clone + 'static>(style: &Rc<IconLabelTabStyle>) -> IconLabelTabBuilder<A> {
        IconLabelTabBuilder::new(style)
    }

    /// Returns the size of the padded background rectangle if it were to
    /// cover the entire size of the unclipped text.
    ///
    /// This can be useful to lay out elements that depend on text size.
    pub fn desired_padded_size(&self) -> Size {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style, .. } = &mut *shared_state;

        inner.desired_padded_size(&style.toggle_btn_style)
    }

    pub fn set_text(&mut self, text: ToggleText, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style, .. } = &mut *shared_state;

        if inner.set_text(text, &style.toggle_btn_style, res) {
            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn set_icon(&mut self, icon_id: Option<impl Into<CustomGlyphID>>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state
            .inner
            .set_icons(icon_id.map(|i| ToggleIcons::Single(i.into())))
        {
            self.el.notify_custom_state_change();
        }
    }

    pub fn set_dual_icons(
        &mut self,
        off_on_ids: Option<(impl Into<CustomGlyphID>, impl Into<CustomGlyphID>)>,
    ) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state
            .inner
            .set_icons(off_on_ids.map(|(off_id, on_id)| ToggleIcons::Dual {
                off: off_id.into(),
                on: on_id.into(),
            }))
        {
            self.el.notify_custom_state_change();
        }
    }

    pub fn icons(&self) -> Option<ToggleIcons> {
        RefCell::borrow(&self.shared_state).inner.icons()
    }

    pub fn set_style(&mut self, style: &Rc<IconLabelTabStyle>, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !Rc::ptr_eq(&shared_state.style, style) {
            shared_state.style = Rc::clone(style);
            shared_state.inner.set_style(&style.toggle_btn_style, res);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<IconLabelTabStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    pub fn set_toggled(&mut self, toggled: bool, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style, .. } = &mut *shared_state;

        if inner.toggled() != toggled {
            inner.set_toggled(toggled, &style.toggle_btn_style, res);
            self.el.notify_custom_state_change();
        }
    }

    pub fn toggled(&self) -> bool {
        RefCell::borrow(&self.shared_state).inner.toggled()
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style, .. } = &mut *shared_state;

        if disabled && inner.state() != ButtonState::Disabled {
            inner.set_state(ButtonState::Disabled, &style.toggle_btn_style);
            self.el.notify_custom_state_change();
        } else if !disabled && inner.state() == ButtonState::Disabled {
            inner.set_state(ButtonState::Idle, &style.toggle_btn_style);
            self.el.notify_custom_state_change();
        }
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

    pub fn layout(&mut self, origin: Point) {
        let size = self.desired_padded_size();
        self.el.set_rect(Rect::new(origin, size));
    }

    pub fn layout_aligned(&mut self, point: Point, align: Align2) {
        let size = self.desired_padded_size();
        self.el.set_rect(align.align_rect_to_point(point, size));
    }
}

#[derive(Default, Debug, Clone)]
pub struct IconLabelTabGroupOption {
    pub text: Option<String>,
    pub icons: Option<ToggleIcons>,
    pub tooltip_message: String,
    pub text_offset: Point,
    pub icon_offset: Point,
    pub icon_scale: f32,
    pub disabled: bool,
}

impl IconLabelTabGroupOption {
    pub fn new(
        text: Option<impl Into<String>>,
        icons: Option<ToggleIcons>,
        tooltip_message: impl Into<String>,
    ) -> Self {
        Self {
            text: text.map(|t| t.into()),
            icons,
            tooltip_message: tooltip_message.into(),
            text_offset: Point::default(),
            icon_offset: Point::default(),
            icon_scale: 1.0,
            disabled: false,
        }
    }
}

pub struct IconLabelTabGroup {
    tabs: Vec<IconLabelTab>,
    selected_index: usize,
    bounds: Rect,
}

impl IconLabelTabGroup {
    pub fn new<'a, A: Clone + 'static, F>(
        options: impl IntoIterator<Item = impl Into<IconLabelTabGroupOption>>,
        selected_index: usize,
        mut on_selected: F,
        style: &Rc<IconLabelTabStyle>,
        z_index: u16,
        on_indicator_line_placement: IndicatorLinePlacement,
        tooltip_align: Align2,
        cx: &mut WindowContext<A>,
    ) -> Self
    where
        F: FnMut(usize) -> A + 'static,
    {
        let tabs: Vec<IconLabelTab> = options
            .into_iter()
            .enumerate()
            .map(|(i, option)| {
                let option: IconLabelTabGroupOption = option.into();

                IconLabelTab::builder(style)
                    .text(option.text)
                    .icons(option.icons)
                    .icon_scale(option.icon_scale)
                    .tooltip_message(option.tooltip_message, tooltip_align)
                    .on_toggled_on((on_selected)(i))
                    .toggled(i == selected_index)
                    .on_indicator_line_placement(on_indicator_line_placement)
                    .z_index(z_index)
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
                    let size = tab.desired_padded_size();
                    max_height = max_height.max(size.height);
                }
                max_height
            });

            for tab in self.tabs.iter_mut() {
                let size = tab.desired_padded_size();

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
                    let size = tab.desired_padded_size();
                    max_width = max_width.max(size.width);
                }
                max_width
            });

            for tab in self.tabs.iter_mut() {
                let size = tab.desired_padded_size();

                tab.el.set_rect(Rect::new(
                    Point::new(origin.x, y),
                    Size::new(max_width, size.height),
                ));

                y += size.height + spacing;
            }

            self.bounds = Rect::new(origin, Size::new(max_width, y - spacing));
        }
    }

    pub fn updated_selected(&mut self, selected_index: usize, res: &mut ResourceCtx) {
        let selected_index = if selected_index >= self.tabs.len() {
            0
        } else {
            selected_index
        };

        if self.selected_index == selected_index {
            return;
        }

        if let Some(prev_selected_tab) = self.tabs.get_mut(self.selected_index) {
            prev_selected_tab.set_toggled(false, res);
        }

        self.selected_index = selected_index;

        self.tabs[selected_index].set_toggled(true, res);
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
