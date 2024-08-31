use std::cell::{Ref, RefCell};
use std::rc::Rc;

use crate::derive::*;
use crate::prelude::*;

use super::{TextInputAction, TextInputInner, TextInputStyle, TextInputUpdateResult};

#[element_builder]
#[element_builder_class]
#[element_builder_rect]
pub struct FloatingTextInputBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(Option<String>) -> A>>,
    pub right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    pub placeholder_text: String,
    pub text: String,
    pub text_offset: Vector,
    pub select_all_when_focused: bool,
    pub max_characters: usize,
}

impl<A: Clone + 'static> FloatingTextInputBuilder<A> {
    pub fn new() -> Self {
        Self {
            action: None,
            right_click_action: None,
            placeholder_text: String::new(),
            text: String::new(),
            text_offset: Vector::default(),
            select_all_when_focused: true,
            max_characters: 256,
            z_index: Default::default(),
            scissor_rect: Default::default(),
            class: Default::default(),
            rect: Default::default(),
        }
    }

    pub fn on_result<F: FnMut(Option<String>) -> A + 'static>(mut self, f: F) -> Self {
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

    /// If set to `true`, then all text will be selected whenever the element is
    /// focused.
    pub const fn select_all_when_focused(mut self, do_select_all: bool) -> Self {
        self.select_all_when_focused = do_select_all;
        self
    }

    /// The maximum characters that can be in this text input.
    ///
    /// By default this is set to `256`.
    pub const fn max_characters(mut self, max: usize) -> Self {
        self.max_characters = max;
        self
    }

    pub fn build(self, window_cx: &mut WindowContext<'_, A>) -> FloatingTextInput {
        let FloatingTextInputBuilder {
            action,
            right_click_action,
            placeholder_text,
            text,
            text_offset,
            select_all_when_focused,
            max_characters,
            class,
            z_index,
            rect,
            scissor_rect,
        } = self;

        let style = window_cx
            .res
            .style_system
            .get(window_cx.builder_class(class));

        let shared_state = Rc::new(RefCell::new(SharedState {
            inner: TextInputInner::new(
                text,
                placeholder_text,
                false,
                max_characters,
                rect.size,
                false,
                select_all_when_focused,
                &style,
                &mut window_cx.res.font_system,
            ),
            text_offset,
            show_with_info: None,
        }));

        let el = ElementBuilder::new(FloatingTextInputElement {
            shared_state: Rc::clone(&shared_state),
            action,
            right_click_action,
            start_text: String::new(),
            size: rect.size,
            canceled: false,
            hovered: false,
        })
        .builder_values(z_index, scissor_rect, class, window_cx)
        .rect(rect)
        .hidden(true)
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

        FloatingTextInput { el, shared_state }
    }
}

struct FloatingTextInputElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(Option<String>) -> A>>,
    right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    start_text: String,
    size: Size,
    canceled: bool,
    hovered: bool,
}

impl<A: Clone + 'static> Element<A> for FloatingTextInputElement<A> {
    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        let res = match event {
            ElementEvent::Animation { .. } => shared_state.inner.on_animation(),
            ElementEvent::CustomStateChanged => {
                if let Some((element_rect, align, padding)) = shared_state.show_with_info.take() {
                    self.start_text = String::from(shared_state.inner.text());

                    let origin = align.align_floating_element(element_rect, self.size, padding);

                    let mut rect = Rect::new(origin, self.size);
                    let window_rect = Rect::from_size(cx.window_size());

                    if rect.min_x() < window_rect.min_x() {
                        rect.origin.x = 0.0;
                    }
                    if rect.max_x() > window_rect.max_x() {
                        rect.origin.x = window_rect.max_x() - rect.size.width;
                    }
                    if rect.min_y() < window_rect.min_y() {
                        rect.origin.y = 0.0;
                    }
                    if rect.max_y() > window_rect.max_y() {
                        rect.origin.y = window_rect.max_y() - rect.size.height;
                    }

                    cx.set_rect(rect);
                    cx.steal_temporary_focus();
                    cx.listen_to_pointer_clicked_off();
                }

                shared_state
                    .inner
                    .on_custom_state_changed(cx.clipboard, &mut cx.res.font_system)
            }
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
            ElementEvent::Focus(has_focus) => {
                if !has_focus {
                    cx.set_rect(Rect::new(cx.rect().origin, Size::zero()));

                    if let Some(action) = self.action.as_mut() {
                        let new_text =
                            if &self.start_text == shared_state.inner.text() || self.canceled {
                                self.canceled = false;
                                None
                            } else {
                                Some(String::from(shared_state.inner.text()))
                            };

                        cx.send_action((action)(new_text)).unwrap();
                    }
                }

                shared_state.inner.on_focus_changed(
                    has_focus,
                    cx.clipboard,
                    &mut cx.res.font_system,
                )
            }
            ElementEvent::ClickedOff => {
                cx.release_focus();

                shared_state.inner.on_clicked_off()
            }
            _ => TextInputUpdateResult::default(),
        };

        if res.needs_repaint {
            cx.request_repaint();
        }
        if let Some(pos) = res.right_clicked_at {
            if let Some(action) = self.right_click_action.as_mut() {
                cx.send_action((action)(pos)).unwrap();
            }
        }
        if res.hovered {
            self.hovered = true;
            cx.cursor_icon = CursorIcon::Text;
        } else {
            self.hovered = false;
        }
        if let Some(animating) = res.set_animating {
            cx.set_animating(animating);
        }

        if res.enter_key_pressed {
            cx.release_focus();
        } else if res.escape_key_pressed {
            cx.release_focus();
            self.canceled = true;
        }

        res.capture_status
    }

    fn render(&mut self, cx: RenderContext, primitives: &mut PrimitiveGroup) {
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
    show_with_info: Option<(Rect, Align2, Padding)>,
}

/// A handle to a [`FloatingTextInputElement`]
#[element_handle]
#[element_handle_class]
pub struct FloatingTextInput {
    shared_state: Rc<RefCell<SharedState>>,
}

impl FloatingTextInput {
    pub fn builder<A: Clone + 'static>() -> FloatingTextInputBuilder<A> {
        FloatingTextInputBuilder::new()
    }

    pub fn show(
        &mut self,
        text: Option<&str>,
        placeholder_text: Option<&str>,
        element_bounds: Rect,
        align: Align2,
        padding: Padding,
        res: &mut ResourceCtx,
    ) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if let Some(text) = placeholder_text {
            shared_state
                .inner
                .set_placeholder_text(text, &mut res.font_system, || {
                    res.style_system
                        .get::<TextInputStyle>(self.el.class())
                        .clone()
                });
        }

        if let Some(text) = text {
            shared_state
                .inner
                .set_text(text, &mut res.font_system, false);
        }

        shared_state.show_with_info = Some((element_bounds, align, padding));

        self.el.notify_custom_state_change();
        self.el.set_hidden(false);
    }

    pub fn hide(&mut self) {
        RefCell::borrow_mut(&self.shared_state).show_with_info = None;

        self.el.set_hidden(true);
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
}
