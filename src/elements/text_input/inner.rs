use std::time::{Duration, Instant};

use keyboard_types::{Code, CompositionEvent, KeyState, Modifiers};
use rootvg::quad::{QuadPrimitive, SolidQuadBuilder, SolidQuadPrimitive};
use rootvg::text::glyphon::cosmic_text::{Motion, Selection};
use rootvg::text::glyphon::{Action, Affinity, Cursor, Edit, FontSystem};
use rootvg::text::{
    Attrs, EditorBorrowStatus, Family, RcTextBuffer, Shaping, TextPrimitive, TextProperties, Wrap,
};
use smallvec::SmallVec;
use unicode_segmentation::UnicodeSegmentation;

use crate::clipboard::{Clipboard, ClipboardKind};
use crate::event::{EventCaptureStatus, KeyboardEvent, PointerButton};
use crate::layout::{Align, Padding};
use crate::math::{Point, Rect, Size};
use crate::style::{
    Background, BorderStyle, QuadStyle, DEFAULT_ACCENT_COLOR, DEFAULT_TEXT_ATTRIBUTES,
};
use crate::vg::color::{self, RGBA8};

/// The style of a [`TextInput`] element
#[derive(Debug, Clone, PartialEq)]
pub struct TextInputStyle {
    /// The text properties.
    pub properties: TextProperties,

    pub placeholder_text_attrs: Attrs<'static>,

    /// The color of the font
    ///
    /// By default this is set to `color::WHITE`.
    pub font_color: RGBA8,

    /// The color of the placeholder font
    ///
    /// By default this is set to `RGBA8::new(150, 150, 150, 255)`.
    pub font_color_placeholder: RGBA8,

    /// The color of the font when disabled
    ///
    /// By default this is set to `RGBA8::new(150, 150, 150, 255)`.
    pub font_color_disabled: RGBA8,

    /// The color of the font when highlighted
    ///
    /// By default this is set to `color::WHITE`.
    pub font_color_highlighted: RGBA8,

    /// The color of the font background when highlighted
    ///
    /// By default this is set to `RGBA8::new(30, 50, 200, 255)`.
    pub highlight_bg_color: RGBA8,

    /// The width of the text cursor
    ///
    /// By default this is set to `2.0`
    pub cursor_width: f32,

    /// The color of the text cursor
    ///
    /// By default this is set to `color::WHITE`
    pub cursor_color: RGBA8,

    /// The vertical alignment of the text.
    ///
    /// By default this is set to `Align::Center`.
    pub vertical_align: Align,

    /// The minimum size of the clipped text area.
    ///
    /// By default this is set to `Size::new(5.0, 5.0)`.
    pub min_clipped_size: Size,

    /// The padding between the text and the bounding rectangle.
    ///
    /// By default this is set to `Padding::new(6.0, 6.0, 6.0, 6.0)`.
    pub padding: Padding,

    /// The padding between the text and the highlight background.
    ///
    /// By default this is set to `Padding::new(3.0, 0.0, 1.0, 0.0)`.
    pub highlight_padding: Padding,

    /// The style of the padded background rectangle behind the text when
    /// the element does not have focus.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    pub back_quad_unfocused: QuadStyle,

    /// The style of the padded background rectangle behind the text when
    /// the element has focus.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    pub back_quad_focused: QuadStyle,

    /// The style of the padded background rectangle behind the text when
    /// disabled.
    ///
    /// Set to `QuadStyle::TRANSPARENT` for no background rectangle.
    pub back_quad_disabled: QuadStyle,

    /// The interval at which the text cursor blinks.
    ///
    /// By default this is set to half a second.
    pub cursor_blink_interval: Duration,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self {
            properties: TextProperties {
                attrs: DEFAULT_TEXT_ATTRIBUTES,
                ..Default::default()
            },
            placeholder_text_attrs: Attrs {
                style: rootvg::text::Style::Italic,
                ..DEFAULT_TEXT_ATTRIBUTES
            },
            font_color: color::WHITE,
            font_color_placeholder: RGBA8::new(120, 120, 120, 255),
            font_color_disabled: RGBA8::new(120, 120, 120, 255),
            font_color_highlighted: color::WHITE,
            highlight_bg_color: DEFAULT_ACCENT_COLOR,
            cursor_width: 1.0,
            cursor_color: color::WHITE,
            vertical_align: Align::Center,
            min_clipped_size: Size::new(5.0, 5.0),
            padding: Padding::new(6.0, 6.0, 6.0, 6.0),
            highlight_padding: Padding::new(1.0, 0.0, 0.0, 0.0),
            back_quad_unfocused: QuadStyle {
                bg: Background::Solid(RGBA8::new(30, 30, 30, 255)),
                border: BorderStyle {
                    radius: 4.0.into(),
                    color: RGBA8::new(105, 105, 105, 255),
                    width: 1.0,
                    ..Default::default()
                },
            },
            back_quad_focused: QuadStyle {
                bg: Background::Solid(RGBA8::new(30, 30, 30, 255)),
                border: BorderStyle {
                    radius: 4.0.into(),
                    color: RGBA8::new(170, 170, 170, 255),
                    width: 1.0,
                    ..Default::default()
                },
            },
            back_quad_disabled: QuadStyle {
                bg: Background::Solid(RGBA8::new(30, 30, 30, 255)),
                border: BorderStyle {
                    radius: 4.0.into(),
                    color: RGBA8::new(65, 65, 65, 255),
                    width: 1.0,
                    ..Default::default()
                },
            },
            cursor_blink_interval: Duration::from_millis(500),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct TextInputUpdateResult {
    pub needs_repaint: bool,
    pub send_action: bool,
    pub right_clicked_at: Option<Point>,
    pub set_focus: Option<bool>,
    pub capture_status: EventCaptureStatus,
    pub set_cursor_icon: bool,
    pub start_hover_timeout: bool,
    pub listen_to_pointer_clicked_off: bool,
    pub set_animating: Option<bool>,
}

pub struct TextInputInner {
    pub show_password: bool,
    pub disabled: bool,
    pub has_tooltip_message: bool,

    buffer: RcTextBuffer,
    placeholder_buffer: Option<RcTextBuffer>,
    password_buffer: Option<RcTextBuffer>,
    text: String,
    placeholder_text: String,
    queued_actions: SmallVec<[TextInputAction; 4]>,
    max_characters: usize,
    focused: bool,
    do_send_action: bool,
    text_bounds_rect: Rect,
    prev_bounds_size: Size,
    cursor_x: f32,
    select_highlight_range: Option<(f32, f32)>,
    dragging: bool,
    cursor_blink_state_on: bool,
    cursor_blink_last_toggle_instant: Instant,
    pointer_hovered: bool,
    select_all_when_focused: bool,
}

impl TextInputInner {
    pub fn new(
        mut text: String,
        mut placeholder_text: String,
        password_mode: bool,
        max_characters: usize,
        bounds_size: Size,
        disabled: bool,
        has_tooltip_message: bool,
        select_all_when_focused: bool,
        style: &TextInputStyle,
        font_system: &mut FontSystem,
    ) -> Self {
        if text.len() > max_characters {
            text = String::from(&text[0..max_characters]);
        }
        if placeholder_text.len() > max_characters {
            placeholder_text = String::from(&placeholder_text[0..max_characters]);
        }

        let text_bounds_rect = layout_text_bounds(
            bounds_size,
            style.padding,
            style.min_clipped_size,
            style.vertical_align,
            style.properties.metrics.font_size,
            style.properties.metrics.line_height,
        );

        let mut properties = style.properties;
        properties.wrap = Wrap::None;
        properties.shaping = Shaping::Advanced;

        if password_mode {
            properties.attrs.family = Family::Monospace;
        }

        let buffer_size = Size::new(
            text_bounds_rect.width(),
            // Add some extra padding below so that text doesn't get clipped.
            text_bounds_rect.height() + 2.0,
        );

        let buffer = RcTextBuffer::new(&text, properties, buffer_size, true, font_system);

        let placeholder_buffer = if placeholder_text.is_empty() {
            None
        } else {
            let mut placeholder_properties = properties.clone();
            placeholder_properties.attrs = style.placeholder_text_attrs;

            Some(RcTextBuffer::new(
                &placeholder_text,
                placeholder_properties,
                buffer_size,
                true,
                font_system,
            ))
        };

        let password_buffer = if password_mode {
            Some(RcTextBuffer::new(
                &text_to_password_text(&buffer),
                properties,
                buffer_size,
                false,
                font_system,
            ))
        } else {
            None
        };

        Self {
            buffer,
            placeholder_buffer,
            password_buffer,
            text,
            placeholder_text,
            queued_actions: SmallVec::new(),
            show_password: false,
            max_characters,
            disabled,

            focused: false,
            do_send_action: false,
            text_bounds_rect,
            prev_bounds_size: bounds_size,
            cursor_x: 0.0,
            select_highlight_range: None,
            dragging: false,
            cursor_blink_state_on: false,
            cursor_blink_last_toggle_instant: Instant::now(),
            pointer_hovered: false,
            has_tooltip_message,
            select_all_when_focused,
        }
    }

    pub fn set_text(
        &mut self,
        text: &str,
        font_system: &mut FontSystem,
        select_all: bool,
    ) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if self.text == text {
            return res;
        }

        res.needs_repaint = true;

        self.text = String::from(&text[0..self.max_characters]);

        self.buffer.with_editor_mut(
            |editor, font_system| -> EditorBorrowStatus {
                editor.set_selection(Selection::Line(Cursor {
                    line: 0,
                    index: 0,
                    affinity: Affinity::Before,
                }));
                editor.delete_selection();
                editor.insert_string(text, None);
                editor.shape_as_needed(font_system, true);

                if select_all {
                    editor.set_selection(Selection::Line(Cursor {
                        line: 0,
                        index: 0,
                        affinity: Affinity::Before,
                    }));
                }

                EditorBorrowStatus {
                    text_changed: true,
                    has_text: !self.text.is_empty(),
                }
            },
            font_system,
        );

        self.layout_contents(font_system);

        res
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_placeholder_text(
        &mut self,
        mut text: &str,
        font_system: &mut FontSystem,
        style: &TextInputStyle,
    ) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if text.len() > self.max_characters {
            text = &text[0..self.max_characters];
        }

        if self.placeholder_text == text {
            return res;
        }

        self.placeholder_text = String::from(text);

        if let Some(buffer) = self.placeholder_buffer.as_mut() {
            buffer.set_text(text, font_system);
        } else {
            let mut placeholder_properties = style.properties.clone();
            placeholder_properties.attrs = style.placeholder_text_attrs;

            let buffer_size = Size::new(
                self.text_bounds_rect.width(),
                // Add some extra padding below so that text doesn't get clipped.
                self.text_bounds_rect.height() + 2.0,
            );

            self.placeholder_buffer = Some(RcTextBuffer::new(
                text,
                placeholder_properties,
                buffer_size,
                false,
                font_system,
            ));
        }

        res.needs_repaint = true;

        res
    }

    pub fn placeholder_text(&self) -> &str {
        &self.placeholder_text
    }

    pub fn max_characters(&self) -> usize {
        self.max_characters
    }

    pub fn set_style(&mut self, style: &TextInputStyle, font_system: &mut FontSystem) {
        let mut properties = style.properties;
        properties.wrap = Wrap::None;
        properties.shaping = Shaping::Advanced;

        if self.password_buffer.is_some() {
            properties.attrs.family = Family::Monospace;
        }

        self.buffer
            .set_text_and_props(&self.text, style.properties, font_system);

        if let Some(placeholder_buffer) = self.placeholder_buffer.as_mut() {
            let mut placeholder_properties = style.properties.clone();
            placeholder_properties.attrs = style.placeholder_text_attrs;
            placeholder_buffer.set_text_and_props(
                &self.placeholder_text,
                placeholder_properties,
                font_system,
            );
        }

        if let Some(password_buffer) = self.password_buffer.as_mut() {
            password_buffer.set_text_and_props(
                &text_to_password_text(&self.buffer),
                properties,
                font_system,
            );
        }
    }

    pub fn on_animation(&mut self, style: &TextInputStyle) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if !self.focused {
            return res;
        }

        if self.cursor_blink_last_toggle_instant.elapsed() >= style.cursor_blink_interval {
            self.cursor_blink_state_on = !self.cursor_blink_state_on;
            self.cursor_blink_last_toggle_instant = Instant::now();
            res.needs_repaint = true;
        }

        res
    }

    pub fn on_custom_state_changed(
        &mut self,
        clipboard: &mut Clipboard,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        self.drain_actions(clipboard, font_system, &mut res);

        if res.needs_repaint {
            self.layout_contents(font_system);
        }

        if self.focused && self.disabled {
            self.focused = false;

            res.set_focus = Some(false);

            res.send_action = self.do_send_action;
            self.do_send_action = false;
        }

        res.needs_repaint = true;

        res
    }

    pub fn on_size_changed(
        &mut self,
        bounds_size: Size,
        style: &TextInputStyle,
        font_system: &mut FontSystem,
    ) {
        if self.prev_bounds_size == bounds_size {
            return;
        }
        self.prev_bounds_size = bounds_size;

        self.text_bounds_rect = layout_text_bounds(
            bounds_size,
            style.padding,
            style.min_clipped_size,
            style.vertical_align,
            style.properties.metrics.font_size,
            style.properties.metrics.line_height,
        );

        let buffer_size = Size::new(
            self.text_bounds_rect.width(),
            // Add some extra padding below so that text doesn't get clipped.
            self.text_bounds_rect.height() + 2.0,
        );

        self.buffer.set_bounds(buffer_size, font_system);

        if let Some(buffer) = self.placeholder_buffer.as_mut() {
            buffer.set_bounds(buffer_size, font_system);
        }

        if let Some(buffer) = self.password_buffer.as_mut() {
            buffer.set_bounds(buffer_size, font_system);
        }

        self.layout_contents(font_system);
    }

    pub fn on_pointer_moved(
        &mut self,
        position: Point,
        bounds: Rect,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if self.disabled {
            return res;
        }

        let pointer_in_bounds = bounds.contains(position);

        if pointer_in_bounds && !self.pointer_hovered && self.has_tooltip_message {
            res.start_hover_timeout = true;
        }
        self.pointer_hovered = pointer_in_bounds;

        if self.focused && self.dragging {
            let (buf_x, buf_y) = pos_to_buffer_pos(position, bounds.origin, self.text_bounds_rect);

            self.buffer.with_editor_mut(
                |editor, font_system| -> EditorBorrowStatus {
                    editor.action(font_system, Action::Drag { x: buf_x, y: buf_y });

                    EditorBorrowStatus {
                        text_changed: false,
                        has_text: !self.text.is_empty(),
                    }
                },
                font_system,
            );

            res.set_cursor_icon = true;
            res.needs_repaint = true;
            res.capture_status = EventCaptureStatus::Captured;
        } else if pointer_in_bounds {
            res.set_cursor_icon = true;
            res.capture_status = EventCaptureStatus::Captured;
        }

        if res.needs_repaint {
            self.layout_contents(font_system);
        }

        res
    }

    pub fn on_pointer_button_just_pressed(
        &mut self,
        pointer_position: Point,
        button: PointerButton,
        click_count: usize,
        bounds: Rect,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if self.disabled || !bounds.contains(pointer_position) {
            return res;
        }

        if button == PointerButton::Secondary {
            res.send_action = self.do_send_action;
            self.do_send_action = false;
            res.capture_status = EventCaptureStatus::Captured;
            res.right_clicked_at = Some(pointer_position);

            if !self.focused {
                res.set_focus = Some(true);
            }

            return res;
        } else if button != PointerButton::Primary {
            res.capture_status = EventCaptureStatus::Captured;
            return res;
        }

        if !self.focused {
            res.set_focus = Some(true);
        }

        self.dragging = true;
        let (buf_x, buf_y) =
            pos_to_buffer_pos(pointer_position, bounds.origin, self.text_bounds_rect);

        let action = match click_count {
            2 => Action::DoubleClick { x: buf_x, y: buf_y },
            3 => Action::TripleClick { x: buf_x, y: buf_y },
            _ => Action::Click { x: buf_x, y: buf_y },
        };

        self.buffer.with_editor_mut(
            |editor, font_system| -> EditorBorrowStatus {
                editor.action(font_system, action);

                EditorBorrowStatus {
                    text_changed: false,
                    has_text: !self.text.is_empty(),
                }
            },
            font_system,
        );

        res.needs_repaint = true;
        self.layout_contents(font_system);

        res
    }

    pub fn on_pointer_button_just_released(
        &mut self,
        pointer_position: Point,
        button: PointerButton,
        bounds: Rect,
    ) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if button == PointerButton::Primary {
            self.dragging = false;
        }

        if !self.disabled && bounds.contains(pointer_position) {
            res.capture_status = EventCaptureStatus::Captured;
        }

        res
    }

    pub fn on_pointer_left(&mut self) {
        self.pointer_hovered = false;
    }

    pub fn on_keyboard_event(
        &mut self,
        event: &KeyboardEvent,
        clipboard: &mut Clipboard,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if self.disabled || event.state == KeyState::Up || !self.focused {
            return res;
        }

        match event.code {
            Code::Backspace => {
                let mut text_changed = false;

                self.buffer.with_editor_mut(
                    |editor, font_system| -> EditorBorrowStatus {
                        editor.action(font_system, Action::Backspace);
                        editor.shape_as_needed(font_system, true);

                        editor.with_buffer(|buffer| {
                            if let Some(run) = buffer.layout_runs().next() {
                                if self.text != run.text {
                                    self.text = run.text.into();
                                    text_changed = true;
                                }
                            } else if !self.text.is_empty() {
                                self.text.clear();
                                text_changed = true;
                            }
                        });

                        EditorBorrowStatus {
                            text_changed,
                            has_text: !self.text.is_empty(),
                        }
                    },
                    font_system,
                );

                if text_changed {
                    res.needs_repaint = true;
                    self.do_send_action = true;
                }
            }
            Code::Escape => {
                self.buffer.with_editor_mut(
                    |editor, font_system| -> EditorBorrowStatus {
                        editor.action(font_system, Action::Escape);

                        EditorBorrowStatus {
                            text_changed: false,
                            has_text: !self.text.is_empty(),
                        }
                    },
                    font_system,
                );

                res.needs_repaint = true;
            }
            Code::Delete => {
                let mut text_changed = false;

                self.buffer.with_editor_mut(
                    |editor, font_system| -> EditorBorrowStatus {
                        editor.action(font_system, Action::Delete);
                        editor.shape_as_needed(font_system, true);

                        editor.with_buffer(|buffer| {
                            if let Some(run) = buffer.layout_runs().next() {
                                if self.text != run.text {
                                    self.text = run.text.into();
                                    text_changed = true;
                                }
                            } else if !self.text.is_empty() {
                                self.text.clear();
                                text_changed = true;
                            }
                        });

                        EditorBorrowStatus {
                            text_changed,
                            has_text: !self.text.is_empty(),
                        }
                    },
                    font_system,
                );

                if text_changed {
                    res.needs_repaint = true;
                    self.do_send_action = true;
                }
            }
            Code::ArrowLeft => {
                self.buffer.with_editor_mut(
                    |editor, font_system| -> EditorBorrowStatus {
                        if editor.selection() != Selection::None {
                            editor.set_selection(Selection::None);
                        }

                        editor.action(font_system, Action::Motion(Motion::Left));

                        EditorBorrowStatus {
                            text_changed: false,
                            has_text: !self.text.is_empty(),
                        }
                    },
                    font_system,
                );

                res.needs_repaint = true;
            }
            Code::ArrowRight => {
                self.buffer.with_editor_mut(
                    |editor, font_system| -> EditorBorrowStatus {
                        if editor.selection() != Selection::None {
                            editor.set_selection(Selection::None);
                        }

                        editor.action(font_system, Action::Motion(Motion::Right));

                        EditorBorrowStatus {
                            text_changed: false,
                            has_text: !self.text.is_empty(),
                        }
                    },
                    font_system,
                );

                res.needs_repaint = true;
            }
            Code::Enter => {
                if self.do_send_action {
                    self.do_send_action = false;
                    res.send_action = true;
                }
            }
            // TODO: Make this keyboard shortcut configurable.
            Code::KeyA => {
                if event.modifiers.contains(Modifiers::CONTROL) {
                    self.queue_action(TextInputAction::SelectAll);
                }
            }
            // TODO: Make this keyboard shortcut configurable.
            Code::KeyX => {
                if event.modifiers.contains(Modifiers::CONTROL) {
                    self.queue_action(TextInputAction::Cut);
                }
            }
            // TODO: Make this keyboard shortcut configurable.
            Code::KeyC => {
                if event.modifiers.contains(Modifiers::CONTROL) {
                    self.queue_action(TextInputAction::Copy);
                }
            }
            // TODO: Make this keyboard shortcut configurable.
            Code::KeyV => {
                if event.modifiers.contains(Modifiers::CONTROL) {
                    self.queue_action(TextInputAction::Paste);
                }
            }
            _ => {}
        }

        self.drain_actions(clipboard, font_system, &mut res);

        if res.needs_repaint {
            self.layout_contents(font_system);
        }

        res
    }

    pub fn on_text_composition_event(
        &mut self,
        event: &CompositionEvent,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if !self.focused || self.disabled {
            return res;
        }

        res.capture_status = EventCaptureStatus::Captured;

        if event.data.is_empty() || self.text.len() >= self.max_characters {
            return res;
        }

        let contents = if self.text.len() + event.data.len() > self.max_characters {
            &event.data[0..self.max_characters - self.text.len()]
        } else {
            &event.data
        };

        let mut text_changed = false;

        self.buffer.with_editor_mut(
            |editor, font_system| -> EditorBorrowStatus {
                editor.insert_string(contents, None);
                editor.shape_as_needed(font_system, false);

                editor.with_buffer(|buffer| {
                    if let Some(run) = buffer.layout_runs().next() {
                        if self.text != run.text {
                            self.text = run.text.into();
                            text_changed = true;
                        }
                    } else if !self.text.is_empty() {
                        self.text.clear();
                        text_changed = true;
                    }
                });

                EditorBorrowStatus {
                    text_changed,
                    has_text: !self.text.is_empty(),
                }
            },
            font_system,
        );

        if text_changed {
            self.do_send_action = true;
            res.needs_repaint = true;

            self.layout_contents(font_system);
        }

        res
    }

    pub fn on_focus_changed(
        &mut self,
        has_focus: bool,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if has_focus {
            res.listen_to_pointer_clicked_off = true;
            self.cursor_blink_state_on = true;
            self.cursor_blink_last_toggle_instant = Instant::now();

            if self.select_all_when_focused && !self.text.is_empty() {
                self.buffer.with_editor_mut(
                    |editor, _| -> EditorBorrowStatus {
                        editor.set_selection(Selection::Line(Cursor {
                            line: 0,
                            index: 0,
                            affinity: Affinity::Before,
                        }));

                        EditorBorrowStatus {
                            text_changed: false,
                            has_text: !self.text.is_empty(),
                        }
                    },
                    font_system,
                );
            }

            self.layout_contents(font_system);
        } else {
            self.dragging = false;

            if self.do_send_action {
                self.do_send_action = false;
                res.send_action = true;
            }
        }

        self.focused = has_focus;
        res.set_animating = Some(has_focus);
        res.needs_repaint = true;

        res
    }

    pub fn on_clicked_off(&mut self) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if self.focused {
            res.set_focus = Some(false);
        }
        self.dragging = false;

        res
    }

    pub fn queue_action(&mut self, action: TextInputAction) {
        self.queued_actions.push(action);
    }

    fn drain_actions(
        &mut self,
        clipboard: &mut Clipboard,
        font_system: &mut FontSystem,
        res: &mut TextInputUpdateResult,
    ) {
        for action in self.queued_actions.drain(..) {
            perform_action(
                action,
                self.max_characters,
                &mut self.text,
                &mut self.buffer,
                clipboard,
                font_system,
                res,
                &mut self.do_send_action,
            );
        }
    }

    fn layout_contents(&mut self, font_system: &mut FontSystem) {
        self.cursor_x = 0.0;
        self.select_highlight_range = None;

        if self.focused {
            self.cursor_blink_state_on = true;
            self.cursor_blink_last_toggle_instant = Instant::now();
        }

        if let Some(password_buffer) = self.password_buffer.as_mut() {
            password_buffer.set_text(&text_to_password_text(&self.buffer), font_system);
        }

        if self.focused {
            let cursor = self.buffer.buffer().editor().unwrap().cursor();
            let selection_bounds = self.buffer.buffer().editor().unwrap().selection_bounds();

            for run in self.buffer.raw_buffer().layout_runs() {
                let cursor_to_x = |cursor: &Cursor| -> f32 {
                    let mut found_glyph = None;

                    for (glyph_i, glyph) in run.glyphs.iter().enumerate() {
                        if cursor.index == glyph.start {
                            found_glyph = Some((glyph_i, 0.0));
                            break;
                        } else if cursor.index > glyph.start && cursor.index < glyph.end {
                            // Guess x offset based on characters
                            let mut before = 0;
                            let mut total = 0;

                            let cluster = &run.text[glyph.start..glyph.end];
                            for (i, _) in cluster.grapheme_indices(true) {
                                if glyph.start + i < cursor.index {
                                    before += 1;
                                }
                                total += 1;
                            }

                            let offset = glyph.w * (before as f32) / (total as f32);

                            found_glyph = Some((glyph_i, offset));
                            break;
                        }
                    }

                    let found_glyph = found_glyph.unwrap_or_else(|| match run.glyphs.last() {
                        Some(_) => (run.glyphs.len(), 0.0),
                        None => (0, 0.0),
                    });

                    match run.glyphs.get(found_glyph.0) {
                        Some(glyph) => {
                            // Start of detected glyph
                            if glyph.level.is_rtl() {
                                glyph.x + glyph.w - found_glyph.1
                            } else {
                                glyph.x + found_glyph.1
                            }
                        }
                        None => match run.glyphs.last() {
                            Some(glyph) => {
                                // End of last glyph
                                if glyph.level.is_rtl() {
                                    glyph.x
                                } else {
                                    glyph.x + glyph.w
                                }
                            }
                            None => {
                                // Start of empty line
                                0.0
                            }
                        },
                    }
                };

                if let Some((start, end)) = selection_bounds {
                    if run.line_i == start.line && run.line_i == end.line {
                        let start_x = cursor_to_x(&start);
                        let end_x = cursor_to_x(&end);

                        self.select_highlight_range = if end_x == start_x {
                            None
                        } else if end_x >= start_x {
                            Some((start_x, end_x))
                        } else {
                            Some((end_x, start_x))
                        };
                    }
                }

                if run.line_i == cursor.line {
                    self.cursor_x = cursor_to_x(&cursor);
                }
            }
        }
    }

    pub fn create_primitives(
        &self,
        style: &TextInputStyle,
        bounds: Rect,
        text_offset: Point,
    ) -> TextInputPrimitives {
        let mut primitives = TextInputPrimitives {
            back_quad: None,
            highlight_range: None,
            text: None,
            cursor: None,
        };

        if self.disabled {
            if !style.back_quad_disabled.is_transparent() {
                primitives.back_quad = Some(style.back_quad_disabled.create_primitive(bounds));
            }
        } else if self.focused {
            if !style.back_quad_focused.is_transparent() {
                primitives.back_quad = Some(style.back_quad_focused.create_primitive(bounds));
            }
        } else {
            if !style.back_quad_unfocused.is_transparent() {
                primitives.back_quad = Some(style.back_quad_unfocused.create_primitive(bounds));
            }
        }

        let highlight_height = self.text_bounds_rect.height()
            + style.highlight_padding.top
            + style.highlight_padding.bottom;
        let highlight_y = self.text_bounds_rect.min_y() - style.highlight_padding.top;

        let scroll_x = if self.focused {
            let cursor_max_x = self.cursor_x + (style.cursor_width * 0.5) + style.padding.left;
            if cursor_max_x >= self.text_bounds_rect.max_x() {
                cursor_max_x - self.text_bounds_rect.max_x()
            } else {
                0.0
            }
        } else {
            0.0
        };

        if self.focused {
            if let Some((start_x, end_x)) = self.select_highlight_range {
                let start_x = (start_x + style.padding.left - scroll_x)
                    .clamp(self.text_bounds_rect.min_x(), self.text_bounds_rect.max_x());
                let end_x = (end_x + style.padding.left - scroll_x)
                    .clamp(self.text_bounds_rect.min_x(), self.text_bounds_rect.max_x());

                if start_x < end_x {
                    primitives.highlight_range = Some(
                        SolidQuadBuilder::new(Size::new(end_x - start_x, highlight_height))
                            .position(Point::new(
                                start_x - (style.cursor_width * 0.5) + bounds.min_x(),
                                highlight_y + bounds.min_y(),
                            ))
                            .bg_color(style.highlight_bg_color)
                            .into(),
                    );
                }
            }
        }

        if !self.text.is_empty() {
            let color = if self.disabled {
                style.font_color_disabled
            } else {
                style.font_color
            };

            let buffer = if let Some(password_buffer) = &self.password_buffer {
                if self.show_password {
                    self.buffer.clone()
                } else {
                    password_buffer.clone()
                }
            } else {
                self.buffer.clone()
            };

            primitives.text = Some(TextPrimitive {
                buffer,
                pos: self.text_bounds_rect.origin + text_offset.to_vector()
                    - Point::new(scroll_x, 0.0).to_vector()
                    + bounds.origin.to_vector(),
                color,
                clipping_bounds: Rect::new(
                    Point::new(scroll_x, 0.0) + bounds.origin.to_vector(),
                    self.text_bounds_rect.size,
                ),
            });
        } else if !self.placeholder_text.is_empty() {
            if let Some(placeholder_buffer) = &self.placeholder_buffer {
                primitives.text = Some(TextPrimitive {
                    buffer: placeholder_buffer.clone(),
                    pos: self.text_bounds_rect.origin
                        + text_offset.to_vector()
                        + bounds.origin.to_vector(),
                    color: style.font_color_placeholder,
                    clipping_bounds: Rect::new(bounds.origin, self.text_bounds_rect.size),
                });
            }
        }

        if self.focused && self.cursor_blink_state_on {
            primitives.cursor = Some(
                SolidQuadBuilder::new(Size::new(style.cursor_width, highlight_height))
                    .position(Point::new(
                        (self.text_bounds_rect.min_x() + self.cursor_x
                            - (style.cursor_width * 0.5)
                            - scroll_x
                            + bounds.min_x())
                        .round(),
                        highlight_y + bounds.min_y(),
                    ))
                    .bg_color(style.cursor_color)
                    .into(),
            );
        }

        primitives
    }
}

pub struct TextInputPrimitives {
    pub back_quad: Option<QuadPrimitive>,
    pub highlight_range: Option<SolidQuadPrimitive>,
    pub text: Option<TextPrimitive>,
    pub cursor: Option<SolidQuadPrimitive>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputAction {
    Cut,
    Copy,
    Paste,
    SelectAll,
}

fn perform_action(
    action: TextInputAction,
    max_characters: usize,
    text: &mut String,
    buffer: &mut RcTextBuffer,
    clipboard: &mut Clipboard,
    font_system: &mut FontSystem,
    res: &mut TextInputUpdateResult,
    do_send_action: &mut bool,
) {
    match action {
        TextInputAction::Cut => {
            buffer.with_editor_mut(
                |editor, font_system| -> EditorBorrowStatus {
                    let text_changed = if let Some(contents) = editor.copy_selection() {
                        clipboard.write(ClipboardKind::Standard, contents);
                        editor.delete_selection();
                        editor.shape_as_needed(font_system, true);
                        true
                    } else {
                        false
                    };

                    if text_changed {
                        editor.with_buffer(|buffer| {
                            if let Some(run) = buffer.layout_runs().next() {
                                if text != run.text {
                                    *text = run.text.into();
                                }
                            } else {
                                text.clear();
                            }
                        });

                        *do_send_action = true;
                        res.needs_repaint = true;
                    }

                    EditorBorrowStatus {
                        text_changed,
                        has_text: !text.is_empty(),
                    }
                },
                font_system,
            );
        }
        TextInputAction::Copy => {
            buffer.with_editor_mut(
                |editor, _| -> EditorBorrowStatus {
                    if let Some(contents) = editor.copy_selection() {
                        clipboard.write(ClipboardKind::Standard, contents);
                    }

                    EditorBorrowStatus {
                        text_changed: false,
                        has_text: !text.is_empty(),
                    }
                },
                font_system,
            );
        }
        TextInputAction::Paste => {
            if text.len() < max_characters {
                if let Some(content) = clipboard.read(ClipboardKind::Standard) {
                    let content = if text.len() + content.len() > max_characters {
                        &content[0..max_characters - text.len()]
                    } else {
                        &content
                    };

                    let mut text_changed = false;

                    buffer.with_editor_mut(
                        |editor, font_system| -> EditorBorrowStatus {
                            editor.insert_string(&content, None);
                            editor.shape_as_needed(font_system, true);

                            editor.with_buffer(|buffer| {
                                if let Some(run) = buffer.layout_runs().next() {
                                    if text != run.text {
                                        *text = run.text.into();
                                        text_changed = true;
                                    }
                                } else if !text.is_empty() {
                                    text.clear();
                                    text_changed = true;
                                }
                            });

                            EditorBorrowStatus {
                                text_changed,
                                has_text: !text.is_empty(),
                            }
                        },
                        font_system,
                    );

                    if text_changed {
                        *do_send_action = true;
                        res.needs_repaint = true;
                    }
                }
            }
        }
        TextInputAction::SelectAll => {
            buffer.with_editor_mut(
                |editor, _| -> EditorBorrowStatus {
                    editor.set_selection(Selection::Line(Cursor {
                        line: 0,
                        index: 0,
                        affinity: Affinity::Before,
                    }));

                    EditorBorrowStatus {
                        text_changed: false,
                        has_text: !text.is_empty(),
                    }
                },
                font_system,
            );

            res.needs_repaint = true;
        }
    }
}

fn layout_text_bounds(
    bounds_size: Size,
    padding: Padding,
    min_clipped_size: Size,
    vertical_align: Align,
    font_size: f32,
    line_height: f32,
) -> Rect {
    let content_rect = crate::layout::layout_inner_rect_with_min_size(
        padding,
        Rect::from_size(bounds_size),
        min_clipped_size,
    );

    // We need to vertically align the text ourselves as rootvg/glyphon does not do this.
    let text_bounds_y = match vertical_align {
        Align::Start => content_rect.min_y(),
        Align::Center => content_rect.min_y() + ((content_rect.height() - line_height) / 2.0),
        //Align::Center => content_rect.min_y() + ((content_rect.height() - font_size) / 2.0) + 1.0,
        Align::End => content_rect.max_y() - font_size,
    };

    Rect::new(
        Point::new(content_rect.min_x(), text_bounds_y),
        content_rect.size,
    )
}

fn pos_to_buffer_pos(pos: Point, bounds_origin: Point, text_bounds: Rect) -> (i32, i32) {
    let p = pos - (bounds_origin.to_vector() + text_bounds.origin.to_vector());
    let x = p.x.round() as i32;

    // Because this is a single-line input only, it is fine to always set
    // y to be 0.
    let y = 0;

    (x, y)
}

fn text_to_password_text(buffer: &RcTextBuffer) -> String {
    if let Some(run) = buffer.raw_buffer().layout_runs().next() {
        run.glyphs.iter().map(|_| '\u{2022}').collect()
    } else {
        String::new()
    }
}
