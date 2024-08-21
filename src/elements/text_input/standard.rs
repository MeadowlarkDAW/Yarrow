use std::cell::{Ref, RefCell};
use std::rc::Rc;

use rootvg::PrimitiveGroup;

use crate::elements::text_input::TextInputUpdateResult;
use crate::event::{ElementEvent, EventCaptureStatus, PointerEvent};
use crate::layout::Align2;
use crate::math::{Point, Rect, Vector, ZIndex};
use crate::prelude::{ClassID, ResourceCtx};
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
    pub text_offset: Vector,
    pub select_all_when_focused: bool,
    pub password_mode: bool,
    pub max_characters: usize,
    pub disabled: bool,
    pub class: Option<ClassID>,
    pub z_index: Option<ZIndex>,
    pub rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> TextInputBuilder<A> {
    pub fn new() -> Self {
        Self {
            action: None,
            right_click_action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            placeholder_text: String::new(),
            text: String::new(),
            text_offset: Vector::default(),
            select_all_when_focused: false,
            password_mode: false,
            max_characters: 256,
            disabled: false,
            class: None,
            z_index: None,
            rect: Rect::default(),
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
    pub const fn text_offset(mut self, offset: Vector) -> Self {
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

    /// The maximum characters that can be in this text input.
    ///
    /// By default this is set to `256`.
    pub const fn max_characters(mut self, max: usize) -> Self {
        self.max_characters = max;
        self
    }

    /// The style class ID
    ///
    /// If this method is not used, then the current class from the window context will
    /// be used.
    pub const fn class(mut self, class: ClassID) -> Self {
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
    pub const fn rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
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

pub struct TextInputElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(String) -> A>>,
    right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
    hovered: bool,
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
            class,
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);
        let style = cx.res.style_system.get(cx.class());

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: TextInputInner::new(
                text,
                placeholder_text,
                password_mode,
                max_characters,
                rect.size,
                disabled,
                tooltip_message.is_some(),
                select_all_when_focused,
                &style,
                &mut cx.res.font_system,
            ),
            text_offset,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                tooltip_message,
                tooltip_align,
                action,
                right_click_action,
                hovered: false,
            }),
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
            class,
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

        let res = match event {
            ElementEvent::Animation { .. } => shared_state.inner.on_animation(),
            ElementEvent::CustomStateChanged => shared_state
                .inner
                .on_custom_state_changed(cx.clipboard, &mut cx.res.font_system),
            ElementEvent::SizeChanged => {
                let bounds_size = cx.rect().size;
                let style = cx.res.style_system.get(cx.class());
                shared_state
                    .inner
                    .on_size_changed(bounds_size, style, &mut cx.res.font_system);
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
        let style: &TextInputStyle = cx.res.style_system.get(cx.class);

        let mut p = shared_state.inner.create_primitives(
            style,
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
        if let Some(cursor) = p.cursor.take() {
            primitives.set_z_index(3);
            primitives.add_solid_quad(cursor);
        }
    }
}

struct SharedState {
    inner: TextInputInner,
    text_offset: Vector,
}

/// A handle to a [`TextInputElement`]
pub struct TextInput {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl TextInput {
    pub fn builder<A: Clone + 'static>() -> TextInputBuilder<A> {
        TextInputBuilder::new()
    }

    /// Set the text.
    ///
    /// Returns `true` if the text has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently. However, this method still
    /// involves a string comparison so you may want to call this method
    /// sparingly.
    pub fn set_text(&mut self, text: &str, res: &mut ResourceCtx, select_all: bool) -> bool {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let result = shared_state
            .inner
            .set_text(text, &mut res.font_system, select_all);
        if result.needs_repaint {
            self.el._notify_custom_state_change();
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
    /// Note, this will *always* cause an element update even if
    /// the placeholder text has not changed, so prefer to use this method sparingly.
    pub fn set_placeholder_text(&mut self, text: &str, res: &mut ResourceCtx) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let result = shared_state
            .inner
            .set_placeholder_text(text, &mut res.font_system, || {
                res.style_system
                    .get::<TextInputStyle>(self.el.class())
                    .clone()
            });
        if result.needs_repaint {
            self.el._notify_custom_state_change();
        }
    }

    pub fn placeholder_text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| {
            s.inner.placeholder_text()
        })
    }

    /// Set the class of the element.
    ///
    /// Returns `true` if the class has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// and the class ID is cached in the handle itself, so this is very
    /// cheap to call frequently.
    pub fn set_class(&mut self, class: ClassID, res: &mut ResourceCtx) -> bool {
        if self.el.class() != class {
            RefCell::borrow_mut(&self.shared_state)
                .inner
                .sync_new_style(res.style_system.get(class), &mut res.font_system);

            self.el._notify_class_change(class);
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

        if shared_state.inner.disabled != disabled {
            shared_state.inner.disabled = true;
            self.el._notify_custom_state_change();
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
            self.el._notify_custom_state_change();
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
            self.el._notify_custom_state_change();
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
            self.el._notify_custom_state_change();
            true
        } else {
            false
        }
    }
}
