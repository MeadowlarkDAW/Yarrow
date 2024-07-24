use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::PrimitiveGroup;

use crate::elements::text_input::TextInputUpdateResult;
use crate::event::{ElementEvent, EventCaptureStatus, PointerEvent};
use crate::layout::Align2;
use crate::math::{Point, Rect, ZIndex};
use crate::prelude::ResourceCtx;
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;
use crate::CursorIcon;

use super::{TextInputAction, TextInputInner, TextInputStyle};

pub struct TextInputBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(String) -> A>>,
    pub right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub placeholder_text: String,
    pub text: String,
    pub text_offset: Point,
    pub select_all_when_focused: bool,
    pub password_mode: bool,
    pub max_characters: usize,
    pub disabled: bool,
    pub style: Rc<TextInputStyle>,
    pub z_index: Option<ZIndex>,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> TextInputBuilder<A> {
    pub fn new(style: &Rc<TextInputStyle>) -> Self {
        Self {
            action: None,
            right_click_action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            placeholder_text: String::new(),
            text: String::new(),
            text_offset: Point::default(),
            select_all_when_focused: false,
            password_mode: false,
            max_characters: 256,
            disabled: false,
            style: Rc::clone(style),
            z_index: None,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: None,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> TextInput {
        TextInputElement::create(self, cx)
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

pub struct TextInputElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(String) -> A>>,
    right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
}

impl<A: Clone + 'static> TextInputElement<A> {
    pub fn create(builder: TextInputBuilder<A>, cx: &mut WindowContext<'_, A>) -> TextInput {
        let TextInputBuilder {
            action,
            right_click_action,
            tooltip_message,
            tooltip_align,
            placeholder_text,
            text,
            text_offset,
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

        let (z_index, scissor_rect_id) = cx.z_index_and_scissor_rect_id(z_index, scissor_rect_id);

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
                &style,
                &mut cx.res,
            ),
            style,
            text_offset,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                tooltip_message,
                tooltip_align,
                action,
                right_click_action,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        TextInput { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for TextInputElement<A> {
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
        } = &mut *shared_state;

        let res = match event {
            ElementEvent::Animation { .. } => inner.on_animation(style),
            ElementEvent::CustomStateChanged => {
                inner.on_custom_state_changed(cx.clipboard, &mut cx.res)
            }
            ElementEvent::SizeChanged => {
                inner.on_size_changed(cx.rect().size, style, &mut cx.res);
                TextInputUpdateResult::default()
            }
            ElementEvent::Pointer(PointerEvent::Moved { position, .. }) => {
                inner.on_pointer_moved(position, cx.rect(), &mut cx.res)
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
                &mut cx.res,
            ),
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                button, position, ..
            }) => inner.on_pointer_button_just_released(position, button, cx.rect()),
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                inner.on_pointer_left();
                TextInputUpdateResult::default()
            }
            ElementEvent::Keyboard(key_event) => {
                inner.on_keyboard_event(&key_event, cx.clipboard, &mut cx.res)
            }
            ElementEvent::TextComposition(comp_event) => {
                inner.on_text_composition_event(&comp_event, &mut cx.res)
            }
            ElementEvent::Focus(has_focus) => {
                inner.on_focus_changed(has_focus, cx.clipboard, &mut cx.res)
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
            &shared_state.style,
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
        if let Some(cursor) = p.cursor.take() {
            primitives.set_z_index(3);
            primitives.add_solid_quad(cursor);
        }
    }
}

struct SharedState {
    inner: TextInputInner,
    style: Rc<TextInputStyle>,
    text_offset: Point,
}

/// A handle to a [`TextInputElement`]
pub struct TextInput {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl TextInput {
    pub fn builder<A: Clone + 'static>(style: &Rc<TextInputStyle>) -> TextInputBuilder<A> {
        TextInputBuilder::new(style)
    }

    pub fn set_text(&mut self, text: &str, res: &mut ResourceCtx, select_all: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let result = shared_state.inner.set_text(text, res, select_all);
        if result.needs_repaint {
            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.inner.text())
    }

    pub fn set_placeholder_text(&mut self, text: &str, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState { inner, style, .. } = &mut *shared_state;

        let result = inner.set_placeholder_text(text, res, style);
        if result.needs_repaint {
            self.el.notify_custom_state_change();
        }
    }

    pub fn placeholder_text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| {
            s.inner.placeholder_text()
        })
    }

    pub fn set_style(&mut self, style: &Rc<TextInputStyle>, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState {
            inner,
            style: old_style,
            ..
        } = &mut *shared_state;

        if !Rc::ptr_eq(old_style, style) {
            *old_style = Rc::clone(style);
            inner.set_style(style, res);

            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<TextInputStyle> {
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

    /// Show/hide the password. This has no effect if the element wasn't created
    /// with password mode enabled.
    pub fn show_password(&mut self, show: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.inner.show_password != show {
            shared_state.inner.show_password = show;
            self.el.notify_custom_state_change();
        }
    }
}
