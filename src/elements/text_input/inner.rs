use smallvec::SmallVec;
use std::time::{Duration, Instant};
use unicode_segmentation::UnicodeSegmentation;

use crate::clipboard::{Clipboard, ClipboardKind};
use crate::prelude::*;
use crate::theme::DEFAULT_ACCENT_COLOR;
use crate::vg::quad::{QuadPrimitive, SolidQuadBuilder, SolidQuadPrimitive};
use crate::vg::text::glyphon::{
    cosmic_text::{Action, Affinity, Cursor, Motion, Selection},
    Edit,
};
use crate::vg::text::{EditorBorrowStatus, RcTextBuffer, TextPrimitive};

/// The style of a [`TextInput`] element
#[derive(Debug, Clone, PartialEq)]
pub struct TextInputStyle {
    /// The text properties.
    pub text_properties: TextProperties,

    /// The attritbutes of the placeholder text
    ///
    /// If this is `None`, then the attributes from `text_properties` will be used.
    ///
    /// By default this is set to `None`.
    pub placeholder_text_attrs: Option<Attrs<'static>>,

    /// The color of the font
    ///
    /// By default this is set to `color::WHITE`.
    pub text_color: RGBA8,
    /// The color of the placeholder font
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `RGBA8::new(150, 150, 150, 255)`.
    pub text_color_placeholder: Option<RGBA8>,
    /// The color of the font when hovered and not focused
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_hover: Option<RGBA8>,
    pub text_color_disabled: DisabledColor,
    /// The color of the placeholder font when hovered and not focused
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_placeholder_hover: Option<RGBA8>,
    pub text_color_placeholder_disabled: DisabledColor,
    /// The color of the font when focused
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_focused: Option<RGBA8>,
    /// The color of the placeholder font when focused
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_placeholder_focused: Option<RGBA8>,
    /// The color of the font when highlighted
    ///
    /// If this is `None`, then `text_color_focused` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_highlighted: Option<RGBA8>,

    /// The color of the font background when highlighted
    ///
    /// By default this is set to `RGBA8::new(30, 50, 200, 255)`.
    pub highlight_bg_color: RGBA8,

    /// The width of the text cursor
    ///
    /// By default this is set to `1.0`
    pub cursor_width: f32,

    /// The color of the text cursor
    ///
    /// If this is `None`, then `text_color_focused` will be used.
    ///
    /// By default this is set to `None`.
    pub cursor_color: Option<RGBA8>,

    /// The padding between the text and the bounding rectangle.
    ///
    /// By default this is set to `Padding::new(6.0, 6.0, 6.0, 6.0)`.
    pub padding: Padding,

    /// The padding between the text and the highlight background.
    ///
    /// By default this is set to `Padding::new(1.0, 0.0, 0.0, 0.0)`.
    pub highlight_padding: Padding,

    /// The background of the background quad.
    pub back_bg: Background,
    /// The background of the background quad when the element is hovered.
    ///
    /// If this is `None`, then `back_bg` will be used.
    ///
    /// By default this is set to `None`.
    pub back_bg_hover: Option<Background>,
    /// The background of the background quad when the button is focused.
    ///
    /// If this is `None`, then `back_bg` will be used.
    ///
    /// By default this is set to `None`.
    pub back_bg_focused: Option<Background>,
    pub back_bg_disabled: DisabledBackground,

    /// The color of the border on the background quad.
    pub back_border_color: RGBA8,
    /// The color of the border on the background quad when the element is hovered.
    ///
    /// If this is `None`, then `back_border_color` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_color_hover: Option<RGBA8>,
    /// The color of the border on the background quad when the element is focused.
    ///
    /// If this is `None`, then `back_border_color` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_color_focused: Option<RGBA8>,
    pub back_border_color_disabled: DisabledColor,

    /// The width of the border on the background quad.
    pub back_border_width: f32,
    /// The width of the border on the background quad when the element is hovered.
    ///
    /// If this is `None`, then `back_border_width` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_width_hover: Option<f32>,
    /// The width of the border on the background quad when the element is focused.
    ///
    /// If this is `None`, then `back_border_width` will be used.
    ///
    /// By default this is set to `None`.
    pub back_border_width_focused: Option<f32>,

    /// The border radius of the background quad.
    pub back_border_radius: Radius,

    /// The interval at which the text cursor blinks.
    ///
    /// By default this is set to half a second.
    pub cursor_blink_interval: Duration,

    /// Additional flags for the quad primitives.
    ///
    /// By default this is set to `QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL`.
    pub quad_flags: QuadFlags,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self {
            text_properties: Default::default(),
            placeholder_text_attrs: None,
            text_color: color::WHITE,
            text_color_placeholder: None,
            text_color_hover: None,
            text_color_placeholder_hover: None,
            text_color_disabled: Default::default(),
            text_color_placeholder_disabled: Default::default(),
            text_color_focused: None,
            text_color_placeholder_focused: None,
            text_color_highlighted: None,
            highlight_bg_color: DEFAULT_ACCENT_COLOR,
            cursor_width: 1.0,
            cursor_color: None,
            padding: Padding::default(),
            highlight_padding: Padding::default(),
            back_bg: Background::TRANSPARENT,
            back_bg_hover: None,
            back_bg_focused: None,
            back_bg_disabled: Default::default(),
            back_border_color: color::TRANSPARENT,
            back_border_color_hover: None,
            back_border_color_focused: None,
            back_border_color_disabled: Default::default(),
            back_border_width: 0.0,
            back_border_width_hover: None,
            back_border_width_focused: None,
            back_border_radius: Radius::default(),
            cursor_blink_interval: Duration::from_millis(500),
            quad_flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        }
    }
}

impl ElementStyle for TextInputStyle {
    const ID: &'static str = "txtinpt";

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self {
            text_color: color::BLACK,
            ..Default::default()
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
    pub hovered: bool,
    pub listen_to_pointer_clicked_off: bool,
    pub set_animating: Option<bool>,
    pub enter_key_pressed: bool,
    pub escape_key_pressed: bool,
}

pub struct TextInputInner {
    pub show_password: bool,
    pub disabled: bool,

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
    cursor_blink_interval: Duration,
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
            style.text_properties.metrics.line_height,
        );

        let mut text_properties = style.text_properties;
        text_properties.wrap = Wrap::None;
        text_properties.shaping = Shaping::Advanced;

        if password_mode {
            text_properties.attrs.family = Family::Monospace;
        }

        let buffer = RcTextBuffer::new(
            &text,
            text_properties,
            Some(text_bounds_rect.width()),
            None,
            true,
            font_system,
        );

        let placeholder_buffer = if placeholder_text.is_empty() {
            None
        } else {
            let mut placeholder_properties = text_properties.clone();
            placeholder_properties.attrs = style
                .placeholder_text_attrs
                .unwrap_or(text_properties.attrs);

            Some(RcTextBuffer::new(
                &placeholder_text,
                placeholder_properties,
                Some(text_bounds_rect.width()),
                None,
                false,
                font_system,
            ))
        };

        let password_buffer = if password_mode {
            Some(RcTextBuffer::new(
                &text_to_password_text(&buffer),
                text_properties,
                Some(text_bounds_rect.width()),
                None,
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
            cursor_blink_interval: style.cursor_blink_interval,
            pointer_hovered: false,
            select_all_when_focused,
        }
    }

    pub fn set_text<T: AsRef<str> + Into<String>>(
        &mut self,
        text: T,
        font_system: &mut FontSystem,
        select_all: bool,
    ) -> TextInputUpdateResult {
        let mut result = TextInputUpdateResult::default();

        if self.text.as_str() == text.as_ref() {
            if select_all {
                self.queue_action(TextInputAction::SelectAll);
            }

            return result;
        }

        result.needs_repaint = true;

        self.text = text.into();
        if self.text.len() > self.max_characters {
            self.text = String::from(&self.text[0..self.max_characters])
        };

        self.buffer.with_editor_mut(
            |editor, font_system| -> EditorBorrowStatus {
                editor.set_selection(Selection::Line(Cursor {
                    line: 0,
                    index: 0,
                    affinity: Affinity::Before,
                }));
                editor.delete_selection();

                editor.insert_string(&self.text, None);
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

        result
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_placeholder_text<T: AsRef<str> + Into<String>, F: FnOnce() -> TextInputStyle>(
        &mut self,
        text: T,
        font_system: &mut FontSystem,
        get_style: F,
    ) -> TextInputUpdateResult {
        let mut result = TextInputUpdateResult::default();

        if self.placeholder_text.as_str() == text.as_ref() {
            return result;
        }

        self.placeholder_text = text.into();
        if self.placeholder_text.len() > self.max_characters {
            self.placeholder_text = String::from(&self.placeholder_text[0..self.max_characters]);
        }

        if let Some(buffer) = self.placeholder_buffer.as_mut() {
            buffer.set_text(&self.placeholder_text, font_system);
        } else {
            let style = (get_style)();

            let mut placeholder_properties = style.text_properties.clone();
            placeholder_properties.attrs = style
                .placeholder_text_attrs
                .unwrap_or(placeholder_properties.attrs);

            self.placeholder_buffer = Some(RcTextBuffer::new(
                &self.placeholder_text,
                placeholder_properties,
                Some(self.text_bounds_rect.width()),
                None,
                false,
                font_system,
            ));
        }

        result.needs_repaint = true;

        result
    }

    pub fn placeholder_text(&self) -> &str {
        &self.placeholder_text
    }

    pub fn max_characters(&self) -> usize {
        self.max_characters
    }

    pub fn sync_new_style(&mut self, style: &TextInputStyle, font_system: &mut FontSystem) {
        let mut text_properties = style.text_properties;
        text_properties.wrap = Wrap::None;
        text_properties.shaping = Shaping::Advanced;

        if self.password_buffer.is_some() {
            text_properties.attrs.family = Family::Monospace;
        }

        self.buffer
            .set_text_and_props(&self.text, text_properties, font_system);

        if let Some(placeholder_buffer) = self.placeholder_buffer.as_mut() {
            let mut placeholder_properties = text_properties.clone();
            placeholder_properties.attrs = style
                .placeholder_text_attrs
                .unwrap_or(placeholder_properties.attrs);
            placeholder_buffer.set_text_and_props(
                &self.placeholder_text,
                placeholder_properties,
                font_system,
            );
        }

        if let Some(password_buffer) = self.password_buffer.as_mut() {
            password_buffer.set_text_and_props(
                &text_to_password_text(&self.buffer),
                text_properties,
                font_system,
            );
        }

        self.cursor_blink_interval = style.cursor_blink_interval;
    }

    pub fn on_animation(&mut self) -> TextInputUpdateResult {
        let mut res = TextInputUpdateResult::default();

        if !self.focused {
            return res;
        }

        if self.cursor_blink_last_toggle_instant.elapsed() >= self.cursor_blink_interval {
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
        let mut result = TextInputUpdateResult::default();

        self.drain_actions(clipboard, font_system, &mut result);

        if result.needs_repaint {
            self.layout_contents(font_system);
        }

        if self.focused && self.disabled {
            self.focused = false;

            result.set_focus = Some(false);

            result.send_action = self.do_send_action;
            self.do_send_action = false;
        }

        result.needs_repaint = true;

        result
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
            style.text_properties.metrics.line_height,
        );

        self.buffer
            .set_bounds(Some(self.text_bounds_rect.width()), None, font_system);

        if let Some(buffer) = self.placeholder_buffer.as_mut() {
            buffer.set_bounds(Some(self.text_bounds_rect.width()), None, font_system);
        }

        if let Some(buffer) = self.password_buffer.as_mut() {
            buffer.set_bounds(Some(self.text_bounds_rect.width()), None, font_system);
        }

        self.layout_contents(font_system);
    }

    pub fn on_pointer_moved(
        &mut self,
        position: Point,
        bounds: Rect,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut result = TextInputUpdateResult::default();

        if self.disabled {
            return result;
        }

        let pointer_in_bounds = bounds.contains(position);

        if !self.pointer_hovered && pointer_in_bounds {
            result.needs_repaint = true;
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

            result.hovered = true;
            result.needs_repaint = true;
            result.capture_status = EventCaptureStatus::Captured;
        } else if pointer_in_bounds {
            result.hovered = true;
            result.capture_status = EventCaptureStatus::Captured;
        }

        if result.needs_repaint {
            self.layout_contents(font_system);
        }

        result
    }

    pub fn on_pointer_button_just_pressed(
        &mut self,
        pointer_position: Point,
        button: PointerButton,
        click_count: usize,
        bounds: Rect,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut result = TextInputUpdateResult::default();

        if self.disabled || !bounds.contains(pointer_position) {
            return result;
        }

        if button == PointerButton::Secondary {
            result.send_action = self.do_send_action;
            self.do_send_action = false;
            result.capture_status = EventCaptureStatus::Captured;
            result.right_clicked_at = Some(pointer_position);

            if !self.focused {
                result.set_focus = Some(true);
            }

            return result;
        } else if button != PointerButton::Primary {
            return result;
        }

        result.capture_status = EventCaptureStatus::Captured;

        if !self.focused {
            result.set_focus = Some(true);
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

        result.needs_repaint = true;
        self.layout_contents(font_system);

        result
    }

    pub fn on_pointer_button_just_released(
        &mut self,
        pointer_position: Point,
        button: PointerButton,
        bounds: Rect,
    ) -> TextInputUpdateResult {
        let mut result = TextInputUpdateResult::default();

        if button == PointerButton::Primary {
            self.dragging = false;
        }

        if !self.disabled && bounds.contains(pointer_position) {
            result.capture_status = EventCaptureStatus::Captured;
        }

        result
    }

    pub fn on_pointer_left(&mut self) -> TextInputUpdateResult {
        self.pointer_hovered = false;
        TextInputUpdateResult {
            hovered: false,
            needs_repaint: true,
            ..Default::default()
        }
    }

    pub fn on_keyboard_event(
        &mut self,
        event: &KeyboardEvent,
        clipboard: &mut Clipboard,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut result = TextInputUpdateResult::default();

        if self.disabled || event.state == KeyState::Up || !self.focused {
            return result;
        }

        match event.code {
            Code::Backspace => {
                result.capture_status = EventCaptureStatus::Captured;

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
                    result.needs_repaint = true;
                    self.do_send_action = true;
                }
            }
            Code::Escape => {
                result.capture_status = EventCaptureStatus::Captured;
                result.escape_key_pressed = true;

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

                result.needs_repaint = true;
            }
            Code::Delete => {
                result.capture_status = EventCaptureStatus::Captured;

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
                    result.needs_repaint = true;
                    self.do_send_action = true;
                }
            }
            Code::ArrowLeft => {
                result.capture_status = EventCaptureStatus::Captured;

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

                result.needs_repaint = true;
            }
            Code::ArrowRight => {
                result.capture_status = EventCaptureStatus::Captured;

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

                result.needs_repaint = true;
            }
            Code::Enter | Code::NumpadEnter => {
                result.capture_status = EventCaptureStatus::Captured;

                result.enter_key_pressed = true;

                if self.do_send_action {
                    self.do_send_action = false;
                    result.send_action = true;
                }
            }
            // TODO: Make this keyboard shortcut configurable.
            Code::KeyA => {
                if event.modifiers.contains(Modifiers::CONTROL) {
                    result.capture_status = EventCaptureStatus::Captured;
                    self.queue_action(TextInputAction::SelectAll);
                }
            }
            // TODO: Make this keyboard shortcut configurable.
            Code::KeyX => {
                if event.modifiers.contains(Modifiers::CONTROL) {
                    result.capture_status = EventCaptureStatus::Captured;
                    self.queue_action(TextInputAction::Cut);
                }
            }
            // TODO: Make this keyboard shortcut configurable.
            Code::KeyC => {
                if event.modifiers.contains(Modifiers::CONTROL) {
                    result.capture_status = EventCaptureStatus::Captured;
                    self.queue_action(TextInputAction::Copy);
                }
            }
            // TODO: Make this keyboard shortcut configurable.
            Code::KeyV => {
                if event.modifiers.contains(Modifiers::CONTROL) {
                    result.capture_status = EventCaptureStatus::Captured;
                    self.queue_action(TextInputAction::Paste);
                }
            }
            _ => {}
        }

        self.drain_actions(clipboard, font_system, &mut result);

        if result.needs_repaint {
            self.layout_contents(font_system);
        }

        result
    }

    pub fn on_text_composition_event(
        &mut self,
        event: &CompositionEvent,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut result = TextInputUpdateResult::default();

        if !self.focused || self.disabled {
            return result;
        }

        result.capture_status = EventCaptureStatus::Captured;

        if event.data.is_empty() || self.text.len() >= self.max_characters {
            return result;
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
            result.needs_repaint = true;

            self.layout_contents(font_system);
        }

        result
    }

    pub fn on_focus_changed(
        &mut self,
        has_focus: bool,
        clipboard: &mut Clipboard,
        font_system: &mut FontSystem,
    ) -> TextInputUpdateResult {
        let mut result = TextInputUpdateResult::default();

        if has_focus {
            result.listen_to_pointer_clicked_off = true;
            self.cursor_blink_state_on = true;
            self.cursor_blink_last_toggle_instant = Instant::now();
            self.focused = true;

            if self.select_all_when_focused && !self.text.is_empty() {
                self.queue_action(TextInputAction::SelectAll);
            }

            self.drain_actions(clipboard, font_system, &mut result);

            if result.needs_repaint {
                self.layout_contents(font_system);
            }
        } else {
            self.focused = false;
            self.dragging = false;

            if self.do_send_action {
                self.do_send_action = false;
                result.send_action = true;
            }
        }

        result.set_animating = Some(has_focus);
        result.needs_repaint = true;

        result
    }

    pub fn on_clicked_off(&mut self) -> TextInputUpdateResult {
        let mut result = TextInputUpdateResult::default();

        if self.focused {
            result.set_focus = Some(false);
        }
        self.dragging = false;

        result
    }

    pub fn queue_action(&mut self, action: TextInputAction) {
        self.queued_actions.push(action);
    }

    fn drain_actions(
        &mut self,
        clipboard: &mut Clipboard,
        font_system: &mut FontSystem,
        result: &mut TextInputUpdateResult,
    ) {
        for action in self.queued_actions.drain(..) {
            match action {
                TextInputAction::Cut => {
                    self.buffer.with_editor_mut(
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
                                        if self.text != run.text {
                                            self.text = run.text.into();
                                        }
                                    } else {
                                        self.text.clear();
                                    }
                                });

                                self.do_send_action = true;
                                result.needs_repaint = true;
                            }

                            EditorBorrowStatus {
                                text_changed,
                                has_text: !self.text.is_empty(),
                            }
                        },
                        font_system,
                    );
                }
                TextInputAction::Copy => {
                    self.buffer.with_editor_mut(
                        |editor, _| -> EditorBorrowStatus {
                            if let Some(contents) = editor.copy_selection() {
                                clipboard.write(ClipboardKind::Standard, contents);
                            }

                            EditorBorrowStatus {
                                text_changed: false,
                                has_text: !self.text.is_empty(),
                            }
                        },
                        font_system,
                    );
                }
                TextInputAction::Paste => {
                    if self.text.len() < self.max_characters {
                        if let Some(content) = clipboard.read(ClipboardKind::Standard) {
                            let content = if self.text.len() + content.len() > self.max_characters {
                                &content[0..self.max_characters - self.text.len()]
                            } else {
                                &content
                            };

                            let mut text_changed = false;

                            self.buffer.with_editor_mut(
                                |editor, font_system| -> EditorBorrowStatus {
                                    editor.insert_string(&content, None);
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
                                self.do_send_action = true;
                                result.needs_repaint = true;
                            }
                        }
                    }
                }
                TextInputAction::SelectAll => {
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

                    result.needs_repaint = true;
                }
            }
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
        text_offset: Vector,
        hovered: bool,
    ) -> TextInputPrimitives {
        let mut primitives = TextInputPrimitives {
            back_quad: None,
            highlight_range: None,
            text: None,
            cursor: None,
        };

        if self.disabled {
            let quad_style = QuadStyle {
                bg: style.back_bg_disabled.get(style.back_bg),
                border: BorderStyle {
                    color: style
                        .back_border_color_disabled
                        .get(style.back_border_color),
                    width: style.back_border_width,
                    radius: style.back_border_radius,
                },
                flags: style.quad_flags,
            };

            if !quad_style.is_transparent() {
                primitives.back_quad = Some(quad_style.create_primitive(bounds));
            }
        } else if self.focused {
            let bg = style.back_bg_focused.unwrap_or(style.back_bg);
            let border_width = style
                .back_border_width_focused
                .unwrap_or(style.back_border_width);

            if !(bg.is_transparent() && border_width == 0.0) {
                primitives.back_quad = Some(
                    QuadStyle {
                        bg,
                        border: BorderStyle {
                            color: style
                                .back_border_color_focused
                                .unwrap_or(style.back_border_color),
                            width: border_width,
                            radius: style.back_border_radius,
                        },
                        flags: style.quad_flags,
                    }
                    .create_primitive(bounds),
                );
            }
        } else if hovered {
            let bg = style.back_bg_hover.unwrap_or(style.back_bg);
            let border_width = style
                .back_border_width_hover
                .unwrap_or(style.back_border_width);

            if !(bg.is_transparent() && border_width == 0.0) {
                primitives.back_quad = Some(
                    QuadStyle {
                        bg,
                        border: BorderStyle {
                            color: style
                                .back_border_color_hover
                                .unwrap_or(style.back_border_color),
                            width: border_width,
                            radius: style.back_border_radius,
                        },
                        flags: style.quad_flags,
                    }
                    .create_primitive(bounds),
                );
            }
        } else {
            if !(style.back_bg.is_transparent() && style.back_border_width == 0.0) {
                primitives.back_quad = Some(
                    QuadStyle {
                        bg: style.back_bg,
                        border: BorderStyle {
                            color: style.back_border_color,
                            width: style.back_border_width,
                            radius: style.back_border_radius,
                        },
                        flags: style.quad_flags,
                    }
                    .create_primitive(bounds),
                );
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
                let start_x = (start_x + self.text_bounds_rect.min_x() - scroll_x)
                    .clamp(self.text_bounds_rect.min_x(), self.text_bounds_rect.max_x());
                let end_x = (end_x + self.text_bounds_rect.min_x() - scroll_x)
                    .clamp(self.text_bounds_rect.min_x(), self.text_bounds_rect.max_x());

                if start_x < end_x {
                    primitives.highlight_range = Some(
                        SolidQuadBuilder::new(Size::new(end_x - start_x, highlight_height))
                            .position(Point::new(
                                start_x - (style.cursor_width * 0.5) + bounds.min_x(),
                                highlight_y + bounds.min_y(),
                            ))
                            .bg_color(style.highlight_bg_color)
                            .flags(style.quad_flags)
                            .into(),
                    );
                }
            }
        }

        if !self.text.is_empty() {
            let color = if self.disabled {
                style.text_color_disabled.get(style.text_color)
            } else if self.focused {
                style.text_color_focused.unwrap_or(style.text_color)
            } else if self.pointer_hovered {
                style.text_color_hover.unwrap_or(style.text_color)
            } else {
                style.text_color
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
                buffer: Some(buffer),
                pos: self.text_bounds_rect.origin + text_offset
                    - Point::new(scroll_x, 0.0).to_vector()
                    + bounds.origin.to_vector(),
                color,
                clipping_bounds: Some(Rect::new(
                    Point::new(scroll_x, 0.0) + bounds.origin.to_vector(),
                    self.text_bounds_rect.size,
                )),
                #[cfg(feature = "svg-icons")]
                icons: SmallVec::new(),
            });
        } else if !self.placeholder_text.is_empty() {
            if let Some(placeholder_buffer) = &self.placeholder_buffer {
                let color = if self.disabled {
                    style.text_color_placeholder_disabled.get(style.text_color)
                } else if self.focused {
                    style.text_color_placeholder_focused.unwrap_or(
                        style
                            .text_color_placeholder
                            .unwrap_or(style.text_color_focused.unwrap_or(style.text_color)),
                    )
                } else if self.pointer_hovered {
                    style
                        .text_color_placeholder_hover
                        .unwrap_or(style.text_color_placeholder.unwrap_or(style.text_color))
                } else {
                    style.text_color_placeholder.unwrap_or(style.text_color)
                };

                primitives.text = Some(TextPrimitive {
                    buffer: Some(placeholder_buffer.clone()),
                    pos: self.text_bounds_rect.origin + text_offset + bounds.origin.to_vector(),
                    color,
                    clipping_bounds: Some(Rect::new(bounds.origin, self.text_bounds_rect.size)),
                    #[cfg(feature = "svg-icons")]
                    icons: SmallVec::new(),
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
                    .bg_color(
                        style
                            .cursor_color
                            .unwrap_or(style.text_color_focused.unwrap_or(style.text_color)),
                    )
                    .flags(style.quad_flags)
                    .into(),
            );
        }

        primitives
    }

    pub fn disabled(&self) -> bool {
        self.disabled
    }

    pub fn focused(&self) -> bool {
        self.focused
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

fn layout_text_bounds(bounds_size: Size, padding: Padding, line_height: f32) -> Rect {
    let content_rect = crate::layout::layout_inner_rect_with_min_size(
        padding,
        Rect::from_size(bounds_size),
        Size::default(),
    );

    // We need to vertically align the text ourselves as rootvg/glyphon does not do this.
    let text_bounds_y = content_rect.min_y() + ((content_rect.height() - line_height) / 2.0);

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
