use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::derive::*;
use crate::prelude::*;
use crate::theme::DEFAULT_ICON_SIZE;

use super::super::icon::{IconInner, IconStyle};
use super::super::tooltip::TooltipInner;
use super::{TextInputAction, TextInputInner, TextInputStyle, TextInputUpdateResult};

/// The style of an [`IconTextInput`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconTextInputStyle {
    pub text_input: TextInputStyle,

    /// The width and height of the icon in points (if the user hasn't
    /// manually set a size for the icon).
    ///
    /// By default this is set to `20.0`.
    pub default_icon_size: f32,
    pub icon_color: Option<RGBA8>,
    pub icon_color_hover: Option<RGBA8>,
    pub icon_color_focused: Option<RGBA8>,
    pub icon_color_disabled: DisabledColor,
    pub icon_padding: Padding,
    pub icon_align: StartEndAlign,

    /// Whether or not the icon should be snapped to the nearset physical
    /// pixel when rendering.
    ///
    /// By default this is set to `true`.
    pub snap_icon_to_physical_pixel: bool,
}

impl IconTextInputStyle {
    fn icon_style(&self, hovered: bool, focused: bool, disabled: bool) -> IconStyle {
        let color = if disabled {
            self.icon_color_disabled.get(
                self.icon_color.unwrap_or(
                    self.text_input
                        .text_color_placeholder
                        .unwrap_or(self.text_input.text_color),
                ),
            )
        } else if focused {
            self.icon_color_focused.unwrap_or(
                self.text_input.text_color_placeholder_focused.unwrap_or(
                    self.text_input
                        .text_color_focused
                        .unwrap_or(self.text_input.text_color),
                ),
            )
        } else if hovered {
            self.icon_color_hover.unwrap_or(
                self.text_input.text_color_placeholder_hover.unwrap_or(
                    self.text_input.text_color_placeholder.unwrap_or(
                        self.text_input
                            .text_color_hover
                            .unwrap_or(self.text_input.text_color),
                    ),
                ),
            )
        } else {
            self.icon_color.unwrap_or(
                self.text_input
                    .text_color_placeholder
                    .unwrap_or(self.text_input.text_color),
            )
        };

        IconStyle {
            default_size: self.default_icon_size,
            color,
            back_quad: QuadStyle::TRANSPARENT,
            padding: self.icon_padding,
            snap_to_physical_pixel: self.snap_icon_to_physical_pixel,
        }
    }
}

impl Default for IconTextInputStyle {
    fn default() -> Self {
        Self {
            text_input: TextInputStyle::default(),
            default_icon_size: DEFAULT_ICON_SIZE,
            icon_color: None,
            icon_color_hover: None,
            icon_color_focused: None,
            icon_color_disabled: Default::default(),
            icon_padding: Padding::default(),
            icon_align: StartEndAlign::Start,
            snap_icon_to_physical_pixel: true,
        }
    }
}

impl ElementStyle for IconTextInputStyle {
    const ID: &'static str = "icntxtinpt";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self {
            text_input: TextInputStyle {
                text_color: color::BLACK,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
#[element_builder_hidden]
#[element_builder_disabled]
#[element_builder_tooltip]
pub struct IconTextInputBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(String) -> A>>,
    pub right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    pub placeholder_text: String,
    pub text: String,
    pub text_offset: Vector,
    pub icon: IconID,
    pub icon_size: Option<Size>,
    pub icon_scale: IconScale,
    pub icon_offset: Vector,
    pub select_all_when_focused: bool,
    pub password_mode: bool,
    pub max_characters: usize,
}

impl<A: Clone + 'static> IconTextInputBuilder<A> {
    pub fn new() -> Self {
        Self {
            action: None,
            right_click_action: None,
            placeholder_text: String::new(),
            text: String::new(),
            text_offset: Vector::default(),
            icon: Default::default(),
            icon_scale: Default::default(),
            icon_size: None,
            icon_offset: Default::default(),
            select_all_when_focused: false,
            password_mode: false,
            max_characters: 256,
            z_index: Default::default(),
            scissor_rect: Default::default(),
            class: Default::default(),
            rect: Default::default(),
            manually_hidden: Default::default(),
            disabled: Default::default(),
            tooltip_data: Default::default(),
        }
    }

    pub fn on_changed<F: FnMut(String) -> A + 'static>(mut self, f: F) -> Self {
        self.action = Some(Box::new(f));
        self
    }

    pub fn on_right_click<F: FnMut(Point) -> A + 'static>(mut self, f: F) -> Self {
        self.right_click_action = Some(Box::new(f));
        self
    }

    pub fn placeholder_text(mut self, text: impl Into<String>) -> Self {
        self.placeholder_text = text.into();
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn text_offset(mut self, offset: Vector) -> Self {
        self.text_offset = offset;
        self
    }

    pub fn icon(mut self, id: impl Into<IconID>) -> Self {
        self.icon = id.into();
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

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn icon_offset(mut self, offset: Vector) -> Self {
        self.icon_offset = offset;
        self
    }

    /// If set to `true`, then all text will be selected whenever the element is
    /// focused.
    pub const fn select_all_when_focused(mut self, do_select_all: bool) -> Self {
        self.select_all_when_focused = do_select_all;
        self
    }

    /// If set the `true`, then text will be displayed in "password mode".
    pub const fn password_mode(mut self, do_use: bool) -> Self {
        self.password_mode = do_use;
        self
    }

    /// The maximum characters that can be in this text input.
    ///
    /// By default this is set to `256`.
    pub const fn max_characters(mut self, max: usize) -> Self {
        self.max_characters = max;
        self
    }

    pub fn build(self, window_cx: &mut WindowContext<'_, A>) -> IconTextInput {
        let IconTextInputBuilder {
            action,
            right_click_action,
            tooltip_data,
            placeholder_text,
            text,
            text_offset,
            icon,
            icon_size,
            icon_scale,
            icon_offset,
            select_all_when_focused,
            password_mode,
            max_characters,
            disabled,
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect,
        } = self;

        let style = window_cx
            .res
            .style_system
            .get::<IconTextInputStyle>(window_cx.builder_class(class));

        let icon = IconInner::new(icon, icon_size, icon_scale, icon_offset);

        let icon_size = icon
            .icon_size()
            .unwrap_or(Size::new(style.default_icon_size, style.default_icon_size));

        let layout_res = layout(rect.size, &style, icon_size);

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: TextInputInner::new(
                text,
                placeholder_text,
                password_mode,
                max_characters,
                rect.size,
                disabled,
                select_all_when_focused,
                &layout_res.text_input_style,
                &mut window_cx.res.font_system,
            ),
            text_offset,
            tooltip_inner: TooltipInner::new(tooltip_data),
        }));

        let el = ElementBuilder::new(IconTextInputElement {
            shared_state: Rc::clone(&shared_state),
            action,
            right_click_action,
            icon,
            icon_rect: layout_res.icon_rect,
            text_input_style: layout_res.text_input_style,
            hovered: false,
        })
        .builder_values(z_index, scissor_rect, class, window_cx)
        .rect(rect)
        .hidden(manually_hidden)
        .flags(
            ElementFlags::PAINTS
                | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
                | ElementFlags::LISTENS_TO_POINTER_OUTSIDE_BOUNDS_WHEN_FOCUSED
                | ElementFlags::LISTENS_TO_TEXT_COMPOSITION_WHEN_FOCUSED
                | ElementFlags::LISTENS_TO_KEYS_WHEN_FOCUSED
                | ElementFlags::LISTENS_TO_SIZE_CHANGE
                | ElementFlags::LISTENS_TO_FOCUS_CHANGE,
        )
        .build(window_cx);

        IconTextInput { el, shared_state }
    }
}

struct IconTextInputElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(String) -> A>>,
    right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    icon: IconInner,
    icon_rect: Rect,
    text_input_style: TextInputStyle,
    hovered: bool,
}

impl<A: Clone + 'static> Element<A> for IconTextInputElement<A> {
    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        shared_state
            .tooltip_inner
            .handle_event(&event, shared_state.inner.disabled(), cx);

        let res = match event {
            ElementEvent::Animation { .. } => shared_state.inner.on_animation(),
            ElementEvent::CustomStateChanged => shared_state
                .inner
                .on_custom_state_changed(cx.clipboard, &mut cx.res.font_system),
            ElementEvent::SizeChanged => {
                let bounds_size = cx.rect().size;
                let style: &IconTextInputStyle = cx.res.style_system.get(cx.class());

                let icon_size = self
                    .icon
                    .icon_size()
                    .unwrap_or(Size::new(style.default_icon_size, style.default_icon_size));

                let layout_res = layout(bounds_size, &style, icon_size);

                shared_state.inner.on_size_changed(
                    bounds_size,
                    &layout_res.text_input_style,
                    &mut cx.res.font_system,
                );

                self.icon_rect = layout_res.icon_rect;
                self.text_input_style = layout_res.text_input_style;

                TextInputUpdateResult::default()
            }
            ElementEvent::Pointer(PointerEvent::Moved { position, .. }) => shared_state
                .inner
                .on_pointer_moved(position, cx.rect(), &mut cx.res.font_system),
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed {
                position,
                button,
                click_count,
                ..
            }) => shared_state.inner.on_pointer_button_just_pressed(
                position,
                button,
                click_count,
                cx.rect(),
                &mut cx.res.font_system,
            ),
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                button, position, ..
            }) => shared_state
                .inner
                .on_pointer_button_just_released(position, button, cx.rect()),
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                shared_state.inner.on_pointer_left()
            }
            ElementEvent::Keyboard(key_event) => shared_state.inner.on_keyboard_event(
                &key_event,
                cx.clipboard,
                &mut cx.res.font_system,
            ),
            ElementEvent::TextComposition(comp_event) => shared_state
                .inner
                .on_text_composition_event(&comp_event, &mut cx.res.font_system),
            ElementEvent::Focus(has_focus) => shared_state.inner.on_focus_changed(
                has_focus,
                cx.clipboard,
                &mut cx.res.font_system,
            ),
            ElementEvent::ClickedOff => shared_state.inner.on_clicked_off(),
            _ => TextInputUpdateResult::default(),
        };

        if res.needs_repaint {
            cx.request_repaint();
        }
        if res.send_action {
            if let Some(action) = self.action.as_mut() {
                cx.send_action((action)(String::from(shared_state.inner.text())))
                    .unwrap();
            }
        }
        if let Some(pos) = res.right_clicked_at {
            if let Some(action) = self.right_click_action.as_mut() {
                cx.send_action((action)(pos)).unwrap();
            }
        }
        if let Some(focus) = res.set_focus {
            if focus {
                cx.steal_focus();
            } else {
                cx.release_focus();
            }
        }
        if res.hovered {
            self.hovered = true;
            cx.cursor_icon = CursorIcon::Text;
        } else {
            self.hovered = false;
        }
        if res.listen_to_pointer_clicked_off {
            cx.listen_to_pointer_clicked_off();
        }
        if let Some(animating) = res.set_animating {
            cx.set_animating(animating);
        }

        res.capture_status
    }

    fn render(&mut self, cx: RenderContext, primitives: &mut PrimitiveGroup) {
        let shared_state = RefCell::borrow(&self.shared_state);
        let style: &IconTextInputStyle = cx.res.style_system.get(cx.class);
        let disabled = shared_state.inner.disabled;

        let mut p = shared_state.inner.create_primitives(
            &self.text_input_style,
            Rect::from_size(cx.bounds_size),
            shared_state.text_offset,
            self.hovered,
        );

        if let Some(back_quad) = p.back_quad.take() {
            primitives.add(back_quad);
        }
        if let Some(highlight_range) = p.highlight_range.take() {
            primitives.set_z_index(1);
            primitives.add_solid_quad(highlight_range);
        }

        if let Some(text) = p.text.take() {
            primitives.set_z_index(2);
            primitives.add_text(text);
        }

        let icon_primitives = self.icon.render(
            self.icon_rect,
            &style.icon_style(self.hovered, shared_state.inner.focused(), disabled),
        );
        primitives.set_z_index(2);
        primitives.add_text(icon_primitives.icon);

        if let Some(cursor) = p.cursor.take() {
            primitives.set_z_index(3);
            primitives.add_solid_quad(cursor);
        }
    }
}

struct SharedState {
    inner: TextInputInner,
    text_offset: Vector,
    tooltip_inner: TooltipInner,
}

/// A handle to a [`IconTextInputElement`]
#[element_handle]
#[element_handle_class]
#[element_handle_set_rect]
#[element_handle_set_tooltip]
pub struct IconTextInput {
    shared_state: Rc<RefCell<SharedState>>,
}

impl IconTextInput {
    pub fn builder<A: Clone + 'static>() -> IconTextInputBuilder<A> {
        IconTextInputBuilder::new()
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
        text: T,
        res: &mut ResourceCtx,
        select_all: bool,
    ) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let result = shared_state
            .inner
            .set_text(text, &mut res.font_system, select_all);
        if result.needs_repaint {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    /// Set the placeholder text.
    ///
    /// Returns `true` if the text has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently. However, this method still
    /// involves a string comparison so you may want to call this method
    /// sparingly.
    pub fn set_placeholder_text<T: AsRef<str> + Into<String>>(
        &mut self,
        text: T,
        res: &mut ResourceCtx,
    ) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let result = shared_state
            .inner
            .set_placeholder_text(text, &mut res.font_system, || {
                res.style_system
                    .get::<TextInputStyle>(self.el.class())
                    .clone()
            });
        if result.needs_repaint {
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn placeholder_text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| {
            s.inner.placeholder_text()
        })
    }

    /// Set the disabled state of this element.
    ///
    /// Returns `true` if the disabled state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_disabled(&mut self, disabled: bool) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.disabled != disabled {
            shared_state.inner.disabled = true;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    ///
    /// Returns `true` if the offset has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_text_offset(&mut self, offset: Vector) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.text_offset != offset {
            shared_state.text_offset = offset;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }

    pub fn max_characters(&self) -> usize {
        RefCell::borrow(&self.shared_state).inner.max_characters()
    }

    /// Perform an action on the text input.
    ///
    /// This will do nothing if the element is currently disabled.
    pub fn perform_action(&mut self, action: TextInputAction) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !shared_state.inner.disabled {
            shared_state.inner.queue_action(action);
            self.el.notify_custom_state_change();
        }
    }

    /// Show/hide the password. This has no effect if the element wasn't created
    /// with password mode enabled.
    ///
    /// Returns `true` if the show password state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn show_password(&mut self, show: bool) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.show_password != show {
            shared_state.inner.show_password = show;
            self.el.notify_custom_state_change();
            true
        } else {
            false
        }
    }
}

struct LayoutResult {
    icon_rect: Rect,
    text_input_style: TextInputStyle,
}

fn layout(bounds_size: Size, style: &IconTextInputStyle, icon_size: Size) -> LayoutResult {
    let icon_padded_size = Size::new(
        icon_size.width + style.icon_padding.left + style.icon_padding.right,
        icon_size.height + style.icon_padding.top + style.icon_padding.bottom,
    );

    let mut text_input_style = style.text_input.clone();

    let icon_rect = match style.icon_align {
        StartEndAlign::Start => {
            text_input_style.padding.left += icon_padded_size.width;

            crate::layout::layout_inner_rect_with_min_size(
                Padding::default(),
                Rect::from_size(Size::new(icon_padded_size.width, bounds_size.height)),
                Size::default(),
            )
        }
        StartEndAlign::End => {
            text_input_style.padding.right += icon_padded_size.width;

            crate::layout::layout_inner_rect_with_min_size(
                Padding::default(),
                Rect::new(
                    Point::new(bounds_size.width - icon_padded_size.width, 0.0),
                    Size::new(icon_padded_size.width, bounds_size.height),
                ),
                Size::default(),
            )
        }
    };

    LayoutResult {
        icon_rect,
        text_input_style,
    }
}
