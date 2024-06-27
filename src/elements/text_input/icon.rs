use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::color::RGBA8;
use rootvg::math::Size;
use rootvg::text::glyphon::FontSystem;
use rootvg::text::{RcTextBuffer, TextPrimitive, TextProperties};
use rootvg::PrimitiveGroup;

use crate::elements::text_input::TextInputUpdateResult;
use crate::event::{ElementEvent, EventCaptureStatus, PointerEvent};
use crate::layout::{Align2, Padding, StartEndAlign};
use crate::math::{Point, Rect, ZIndex};
use crate::style::DEFAULT_TEXT_ATTRIBUTES;
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;
use crate::CursorIcon;

use super::{TextInputAction, TextInputInner, TextInputStyle};

/// The style of an [`IconTextInput`] element
#[derive(Debug, Clone, PartialEq)]
pub struct IconTextInputStyle {
    pub text_input: TextInputStyle,

    pub icon_properties: TextProperties,
    pub icon_color_idle: RGBA8,
    pub icon_color_focused: RGBA8,
    pub icon_color_disabled: RGBA8,
    pub icon_padding: Padding,
    pub icon_align: StartEndAlign,
}

impl Default for IconTextInputStyle {
    fn default() -> Self {
        Self {
            text_input: TextInputStyle::default(),
            icon_properties: TextProperties {
                attrs: DEFAULT_TEXT_ATTRIBUTES,
                ..Default::default()
            },
            icon_color_idle: RGBA8::new(255, 255, 255, 150),
            icon_color_focused: RGBA8::new(255, 255, 255, 255),
            icon_color_disabled: RGBA8::new(255, 255, 255, 100),
            icon_padding: Padding::new(6.0, 0.0, 6.0, 7.0),
            icon_align: StartEndAlign::Start,
        }
    }
}

pub struct IconTextInputBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(String) -> A>>,
    pub right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub placeholder_text: String,
    pub text: String,
    pub text_offset: Point,
    pub icon_text: String,
    pub icon_text_offset: Point,
    pub select_all_when_focused: bool,
    pub password_mode: bool,
    pub max_characters: usize,
    pub disabled: bool,
    pub style: Rc<IconTextInputStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
}

impl<A: Clone + 'static> IconTextInputBuilder<A> {
    pub fn new(style: &Rc<IconTextInputStyle>) -> Self {
        Self {
            action: None,
            right_click_action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            placeholder_text: String::new(),
            text: String::new(),
            text_offset: Point::default(),
            icon_text: String::new(),
            icon_text_offset: Point::default(),
            select_all_when_focused: false,
            password_mode: false,
            max_characters: 256,
            disabled: false,
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> IconTextInput {
        IconTextInputElement::create(self, cx)
    }

    pub fn on_changed<F: FnMut(String) -> A + 'static>(mut self, f: F) -> Self {
        self.action = Some(Box::new(f));
        self
    }

    pub fn on_right_click<F: FnMut(Point) -> A + 'static>(mut self, f: F) -> Self {
        self.right_click_action = Some(Box::new(f));
        self
    }

    pub fn tooltip_message(mut self, message: impl Into<String>, align: Align2) -> Self {
        self.tooltip_message = Some(message.into());
        self.tooltip_align = align;
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
    pub const fn text_offset(mut self, offset: Point) -> Self {
        self.text_offset = offset;
        self
    }

    pub fn icon_text(mut self, text: impl Into<String>) -> Self {
        self.icon_text = text.into();
        self
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub const fn icon_text_offset(mut self, offset: Point) -> Self {
        self.icon_text_offset = offset;
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

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The maximum characters that can be in this text input.
    ///
    /// By default this is set to `256`.
    pub const fn max_characters(mut self, max: usize) -> Self {
        self.max_characters = max;
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

    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = scissor_rect_id;
        self
    }
}

pub struct IconTextInputElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(String) -> A>>,
    right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
    icon_text_buffer: RcTextBuffer,
    icon_text_offset: Point,
    icon_text_rect: Rect,
    text_input_style: TextInputStyle,
}

impl<A: Clone + 'static> IconTextInputElement<A> {
    pub fn create(
        builder: IconTextInputBuilder<A>,
        cx: &mut WindowContext<'_, A>,
    ) -> IconTextInput {
        let IconTextInputBuilder {
            action,
            right_click_action,
            tooltip_message,
            tooltip_align,
            placeholder_text,
            text,
            text_offset,
            icon_text,
            icon_text_offset,
            select_all_when_focused,
            password_mode,
            max_characters,
            disabled,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let icon_text_buffer = RcTextBuffer::new(
            &icon_text,
            style.icon_properties,
            Size::new(1000.0, 200.0),
            false,
            cx.font_system,
        );

        let layout_res = layout(bounding_rect.size, &icon_text_buffer, &style);

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: TextInputInner::new(
                text,
                placeholder_text,
                password_mode,
                max_characters,
                bounding_rect.size,
                disabled,
                tooltip_message.is_some(),
                select_all_when_focused,
                &layout_res.text_input_style,
                cx.font_system,
            ),
            style,
            text_offset,
            style_changed: false,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                tooltip_message,
                tooltip_align,
                action,
                right_click_action,
                icon_text_buffer,
                icon_text_offset,
                icon_text_rect: layout_res.icon_text_rect,
                text_input_style: layout_res.text_input_style,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, cx.font_system, cx.clipboard);

        IconTextInput { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for IconTextInputElement<A> {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS
            | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
            | ElementFlags::LISTENS_TO_POINTER_OUTSIDE_BOUNDS_WHEN_FOCUSED
            | ElementFlags::LISTENS_TO_TEXT_COMPOSITION_WHEN_FOCUSED
            | ElementFlags::LISTENS_TO_KEYS_WHEN_FOCUSED
            | ElementFlags::LISTENS_TO_SIZE_CHANGE
            | ElementFlags::LISTENS_TO_FOCUS_CHANGE
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState {
            inner,
            style,
            text_offset: _,
            style_changed,
        } = &mut *shared_state;

        let res = match event {
            ElementEvent::Animation { .. } => inner.on_animation(&self.text_input_style),
            ElementEvent::CustomStateChanged => {
                let needs_repaint = *style_changed;
                if *style_changed {
                    *style_changed = false;

                    let layout_res = layout(cx.rect().size, &self.icon_text_buffer, style);

                    self.icon_text_rect = layout_res.icon_text_rect;
                    self.text_input_style = layout_res.text_input_style;

                    inner.set_style(&self.text_input_style, cx.font_system);
                    inner.on_size_changed(cx.rect().size, &self.text_input_style, cx.font_system);
                }

                let mut res = inner.on_custom_state_changed(cx.clipboard, cx.font_system);
                res.needs_repaint |= needs_repaint;
                res
            }
            ElementEvent::SizeChanged => {
                let layout_res = layout(cx.rect().size, &self.icon_text_buffer, style);

                self.icon_text_rect = layout_res.icon_text_rect;
                self.text_input_style = layout_res.text_input_style;

                inner.on_size_changed(cx.rect().size, &self.text_input_style, cx.font_system);

                TextInputUpdateResult::default()
            }
            ElementEvent::Pointer(PointerEvent::Moved { position, .. }) => {
                inner.on_pointer_moved(position, cx.rect(), cx.font_system)
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed {
                position,
                button,
                click_count,
                ..
            }) => inner.on_pointer_button_just_pressed(
                position,
                button,
                click_count,
                cx.rect(),
                cx.font_system,
            ),
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                button, position, ..
            }) => inner.on_pointer_button_just_released(position, button, cx.rect()),
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                inner.on_pointer_left();
                TextInputUpdateResult::default()
            }
            ElementEvent::Keyboard(key_event) => {
                inner.on_keyboard_event(&key_event, cx.clipboard, cx.font_system)
            }
            ElementEvent::TextComposition(comp_event) => {
                inner.on_text_composition_event(&comp_event, cx.font_system)
            }
            ElementEvent::Focus(has_focus) => {
                inner.on_focus_changed(has_focus, cx.clipboard, cx.font_system)
            }
            ElementEvent::ClickedOff => inner.on_clicked_off(),
            ElementEvent::Pointer(PointerEvent::HoverTimeout { .. }) => {
                if let Some(message) = &self.tooltip_message {
                    cx.show_tooltip(message.clone(), self.tooltip_align, true);
                }

                TextInputUpdateResult::default()
            }
            _ => TextInputUpdateResult::default(),
        };

        if res.needs_repaint {
            cx.request_repaint();
        }
        if res.send_action {
            if let Some(action) = self.action.as_mut() {
                cx.send_action((action)(String::from(inner.text())))
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
        if res.set_cursor_icon {
            cx.cursor_icon = CursorIcon::Text;
        }
        if res.start_hover_timeout {
            cx.start_hover_timeout();
        }
        if res.listen_to_pointer_clicked_off {
            cx.listen_to_pointer_clicked_off();
        }
        if let Some(animating) = res.set_animating {
            cx.set_animating(animating);
        }

        res.capture_status
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let shared_state = RefCell::borrow(&self.shared_state);

        let mut p = shared_state.inner.create_primitives(
            &self.text_input_style,
            Rect::from_size(cx.bounds_size),
            shared_state.text_offset,
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

        let icon_color = if shared_state.inner.disabled() {
            shared_state.style.icon_color_disabled
        } else if shared_state.inner.focused() {
            shared_state.style.icon_color_focused
        } else {
            shared_state.style.icon_color_idle
        };

        if icon_color.a != 0 {
            primitives.set_z_index(2);
            primitives.add_text(TextPrimitive::new(
                self.icon_text_buffer.clone(),
                self.icon_text_rect.origin + self.icon_text_offset.to_vector(),
                icon_color,
                None,
            ));
        }

        if let Some(cursor) = p.cursor.take() {
            primitives.set_z_index(3);
            primitives.add_solid_quad(cursor);
        }
    }
}

struct SharedState {
    inner: TextInputInner,
    style: Rc<IconTextInputStyle>,
    text_offset: Point,
    style_changed: bool,
}

/// A handle to a [`IconTextInputElement`]
pub struct IconTextInput {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl IconTextInput {
    pub fn builder<A: Clone + 'static>(style: &Rc<IconTextInputStyle>) -> IconTextInputBuilder<A> {
        IconTextInputBuilder::new(style)
    }

    pub fn set_text(&mut self, text: &str, font_system: &mut FontSystem, select_all: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let res = shared_state.inner.set_text(text, font_system, select_all);
        if res.needs_repaint {
            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn set_placeholder_text(&mut self, text: &str, font_system: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style, .. } = &mut *shared_state;

        let res = inner.set_placeholder_text(text, font_system, &style.text_input);
        if res.needs_repaint {
            self.el.notify_custom_state_change();
        }
    }

    pub fn placeholder_text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| {
            s.inner.placeholder_text()
        })
    }

    pub fn set_style(&mut self, style: &Rc<IconTextInputStyle>) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState {
            style: old_style,
            style_changed,
            ..
        } = &mut *shared_state;

        if !Rc::ptr_eq(old_style, style) {
            *old_style = Rc::clone(style);
            *style_changed = true;
            //inner.set_style(&style.text_input, font_system);

            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<IconTextInputStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.disabled != disabled {
            shared_state.inner.disabled = true;
            self.el.notify_custom_state_change();
        }
    }

    /// An offset that can be used mainly to correct the position of icon glyphs.
    /// This does not effect the position of the background quad.
    pub fn set_text_offset(&mut self, offset: Point) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.text_offset != offset {
            shared_state.text_offset = offset;
            self.el.notify_custom_state_change();
        }
    }

    pub fn max_characters(&self) -> usize {
        RefCell::borrow(&self.shared_state).inner.max_characters()
    }

    pub fn perform_action(&mut self, action: TextInputAction) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !shared_state.inner.disabled {
            shared_state.inner.queue_action(action);
            self.el.notify_custom_state_change();
        }
    }
}

struct LayoutResult {
    icon_text_rect: Rect,
    text_input_style: TextInputStyle,
}

fn layout(
    bounds_size: Size,
    icon_text_buffer: &RcTextBuffer,
    style: &IconTextInputStyle,
) -> LayoutResult {
    let icon_unclipped_size = icon_text_buffer.measure();

    let icon_padded_size = Size::new(
        icon_unclipped_size.width + style.icon_padding.left + style.icon_padding.right,
        icon_unclipped_size.height + style.icon_padding.top + style.icon_padding.bottom,
    );

    let mut text_input_style = style.text_input.clone();

    let mut icon_text_rect = match style.icon_align {
        StartEndAlign::Start => {
            text_input_style.padding.left += icon_padded_size.width;

            crate::layout::layout_inner_rect_with_min_size(
                style.icon_padding,
                Rect::from_size(Size::new(icon_padded_size.width, bounds_size.height)),
                Size::default(),
            )
        }
        StartEndAlign::End => {
            text_input_style.padding.right += icon_padded_size.width;

            crate::layout::layout_inner_rect_with_min_size(
                style.icon_padding,
                Rect::new(
                    Point::new(bounds_size.width - icon_padded_size.width, 0.0),
                    Size::new(icon_padded_size.width, bounds_size.height),
                ),
                Size::default(),
            )
        }
    };

    // We need to vertically align the text ourselves as rootvg/glyphon does not do this.
    icon_text_rect.origin.y =
        icon_text_rect.min_y() + ((icon_text_rect.height() - icon_unclipped_size.height) * 0.5);

    LayoutResult {
        icon_text_rect,
        text_input_style,
    }
}
