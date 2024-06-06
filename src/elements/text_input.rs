use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use keyboard_types::{Code, CompositionState, KeyState, Modifiers};
use rootvg::quad::SolidQuadBuilder;
use rootvg::text::glyphon::cosmic_text::{Motion, Selection};
use rootvg::text::glyphon::{Action, Affinity, Cursor, Edit, FontSystem};
use rootvg::text::{
    EditorBorrowStatus, Family, RcTextBuffer, Shaping, TextPrimitive, TextProperties, Wrap,
};
use rootvg::PrimitiveGroup;
use unicode_segmentation::UnicodeSegmentation;

use crate::clipboard::{Clipboard, ClipboardKind};
use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::{Align, Align2, Padding};
use crate::math::{Point, Rect, Size, ZIndex};
use crate::style::{Background, BorderStyle, QuadStyle, DEFAULT_ACCENT_COLOR};
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, ElementTooltipInfo,
    RenderContext,
};
use crate::view::{ScissorRectID, MAIN_SCISSOR_RECT};
use crate::window::WindowContext;
use crate::CursorIcon;

// TODO: Scroll horizontally when text is longer than the bounds of the element.

/// The style of a [`TextInput`] element
#[derive(Debug, Clone, PartialEq)]
pub struct TextInputStyle {
    /// The text properties.
    pub properties: TextProperties,

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
            properties: TextProperties::default(),
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
            highlight_padding: Padding::new(3.0, 0.0, 1.0, 0.0),
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
    pub style: Rc<TextInputStyle>,
    pub z_index: ZIndex,
    pub bounding_rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect_id: ScissorRectID,
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
            style: Rc::clone(style),
            z_index: 0,
            bounding_rect: Rect::default(),
            manually_hidden: false,
            scissor_rect_id: MAIN_SCISSOR_RECT,
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

pub struct TextInputElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(String) -> A>>,
    right_click_action: Option<Box<dyn FnMut(Point) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
    active: bool,
    text_bounds_rect: Rect,
    prev_bounds_size: Size,
    cursor_x: f32,
    select_highlight_range: Option<(f32, f32)>,
    dragging: bool,
    queue_action: bool,
    cursor_blink_state_on: bool,
    cursor_blink_last_toggle_instant: Instant,
    select_all_when_focused: bool,
    password_buffer: Option<RcTextBuffer>,
    max_characters: usize,
    pointer_hovered: bool,
}

impl<A: Clone + 'static> TextInputElement<A> {
    pub fn create(builder: TextInputBuilder<A>, cx: &mut WindowContext<'_, A>) -> TextInput {
        let TextInputBuilder {
            action,
            right_click_action,
            tooltip_message,
            tooltip_align,
            mut placeholder_text,
            mut text,
            text_offset,
            select_all_when_focused,
            password_mode,
            max_characters,
            style,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = builder;

        if text.len() > max_characters {
            text = String::from(&text[0..max_characters]);
        }
        if placeholder_text.len() > max_characters {
            placeholder_text = String::from(&placeholder_text[0..max_characters]);
        }

        let text_bounds_rect = layout_text_bounds(
            bounding_rect.size,
            style.padding,
            style.min_clipped_size,
            style.vertical_align,
            style.properties.metrics.font_size,
            style.properties.metrics.line_height,
        );

        let mut properties = style.properties;
        properties.wrap = Wrap::None;
        properties.shaping = Shaping::Advanced;
        let placeholder_properties = properties.clone();

        if password_mode {
            properties.attrs.family = Family::Monospace;
        }

        let buffer = RcTextBuffer::new(
            &text,
            properties,
            text_bounds_rect.size,
            true,
            cx.font_system,
        );

        let placeholder_buffer = RcTextBuffer::new(
            &placeholder_text,
            placeholder_properties,
            text_bounds_rect.size,
            true,
            cx.font_system,
        );

        let password_buffer = if password_mode {
            Some(RcTextBuffer::new(
                &text_to_password_text(&buffer),
                properties,
                text_bounds_rect.size,
                false,
                cx.font_system,
            ))
        } else {
            None
        };

        let shared_state = Rc::new(RefCell::new(SharedState {
            buffer,
            placeholder_buffer,
            placeholder_text,
            text,
            style,
            text_offset,
            disabled: false,
            queued_actions: Vec::new(),
            show_password: false,
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                tooltip_message,
                tooltip_align,
                action,
                right_click_action,
                active: false,
                text_bounds_rect,
                prev_bounds_size: bounding_rect.size,
                cursor_x: 0.0,
                select_highlight_range: None,
                dragging: false,
                queue_action: false,
                cursor_blink_state_on: false,
                cursor_blink_last_toggle_instant: Instant::now(),
                select_all_when_focused,
                password_buffer,
                max_characters,
                pointer_hovered: false,
            }),
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        };

        let el = cx
            .view
            .add_element(element_builder, cx.font_system, cx.clipboard);

        TextInput {
            el,
            shared_state,
            max_characters,
        }
    }

    fn layout_contents(&mut self) {
        self.cursor_x = 0.0;
        self.select_highlight_range = None;

        let shared_state = RefCell::borrow(&self.shared_state);

        if self.active {
            let cursor = shared_state.buffer.buffer().editor().unwrap().cursor();
            let selection_bounds = shared_state
                .buffer
                .buffer()
                .editor()
                .unwrap()
                .selection_bounds();

            for run in shared_state.buffer.raw_buffer().layout_runs() {
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
        let mut editor_state_changed = false;

        let cut_action = |text: &mut String,
                          buffer: &mut RcTextBuffer,
                          clipboard: &mut Clipboard,
                          font_system: &mut FontSystem,
                          queue_action: &mut bool,
                          editor_state_changed: &mut bool| {
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

                        *editor_state_changed = true;
                        *queue_action = true;
                    }

                    EditorBorrowStatus {
                        text_changed,
                        has_text: !text.is_empty(),
                    }
                },
                font_system,
            );
        };

        let copy_action = |text: &mut String,
                           buffer: &mut RcTextBuffer,
                           clipboard: &mut Clipboard,
                           font_system: &mut FontSystem| {
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
        };

        let paste_action = |text: &mut String,
                            buffer: &mut RcTextBuffer,
                            clipboard: &mut Clipboard,
                            font_system: &mut FontSystem,
                            queue_action: &mut bool,
                            editor_state_changed: &mut bool| {
            if text.len() < self.max_characters {
                if let Some(content) = clipboard.read(ClipboardKind::Standard) {
                    let mut text_changed = false;

                    let content = if text.len() + content.len() > self.max_characters {
                        &content[0..self.max_characters - text.len()]
                    } else {
                        &content
                    };

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
                        *editor_state_changed = true;
                        *queue_action = true;
                    }
                }
            }
        };

        let select_all_action = |text: &mut String,
                                 buffer: &mut RcTextBuffer,
                                 font_system: &mut FontSystem,
                                 editor_state_changed: &mut bool| {
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

            *editor_state_changed = true;
        };

        match event {
            ElementEvent::Animation { .. } => {
                if !self.active {
                    return EventCaptureStatus::NotCaptured;
                }

                if self.cursor_blink_last_toggle_instant.elapsed()
                    >= RefCell::borrow(&self.shared_state)
                        .style
                        .cursor_blink_interval
                {
                    self.cursor_blink_state_on = !self.cursor_blink_state_on;
                    self.cursor_blink_last_toggle_instant = Instant::now();

                    cx.request_repaint();
                }
            }
            ElementEvent::CustomStateChanged => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState {
                    buffer,
                    text,
                    disabled,
                    queued_actions,
                    ..
                } = &mut *shared_state;

                if cx.has_focus() && *disabled {
                    cx.release_focus();
                }

                for action in queued_actions.drain(..) {
                    match action {
                        TextInputAction::Cut => {
                            cut_action(
                                text,
                                buffer,
                                cx.clipboard,
                                cx.font_system,
                                &mut self.queue_action,
                                &mut editor_state_changed,
                            );
                        }
                        TextInputAction::Copy => {
                            copy_action(text, buffer, cx.clipboard, cx.font_system);
                        }
                        TextInputAction::Paste => {
                            paste_action(
                                text,
                                buffer,
                                cx.clipboard,
                                cx.font_system,
                                &mut self.queue_action,
                                &mut editor_state_changed,
                            );
                        }
                        TextInputAction::SelectAll => {
                            select_all_action(
                                text,
                                buffer,
                                cx.font_system,
                                &mut editor_state_changed,
                            );
                        }
                    }
                }

                editor_state_changed = true;
            }
            ElementEvent::SizeChanged => {
                let new_size = cx.rect().size;
                if self.prev_bounds_size != new_size {
                    self.prev_bounds_size = new_size;

                    let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                    self.text_bounds_rect = layout_text_bounds(
                        new_size,
                        shared_state.style.padding,
                        shared_state.style.min_clipped_size,
                        shared_state.style.vertical_align,
                        shared_state.style.properties.metrics.font_size,
                        shared_state.style.properties.metrics.line_height,
                    );

                    shared_state
                        .buffer
                        .set_bounds(self.text_bounds_rect.size, cx.font_system);

                    shared_state
                        .placeholder_buffer
                        .set_bounds(self.text_bounds_rect.size, cx.font_system);

                    editor_state_changed = true;
                }
            }
            ElementEvent::Pointer(PointerEvent::Moved { position, .. }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState {
                    buffer,
                    text,
                    disabled,
                    ..
                } = &mut *shared_state;

                if *disabled {
                    return EventCaptureStatus::NotCaptured;
                }

                let pointer_in_bounds = cx.rect().contains(position);

                if pointer_in_bounds && !self.pointer_hovered && self.tooltip_message.is_some() {
                    cx.start_hover_timeout();
                }
                self.pointer_hovered = pointer_in_bounds;

                if self.active && self.dragging {
                    let (buf_x, buf_y) =
                        pos_to_buffer_pos(position, cx.rect().origin, self.text_bounds_rect);

                    buffer.with_editor_mut(
                        |editor, font_system| -> EditorBorrowStatus {
                            editor.action(font_system, Action::Drag { x: buf_x, y: buf_y });

                            EditorBorrowStatus {
                                text_changed: false,
                                has_text: !text.is_empty(),
                            }
                        },
                        cx.font_system,
                    );

                    cx.cursor_icon = CursorIcon::Text;
                    editor_state_changed = true;
                } else if pointer_in_bounds {
                    cx.cursor_icon = CursorIcon::Text;
                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed {
                position,
                button,
                click_count,
                ..
            }) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState {
                    buffer,
                    text,
                    disabled,
                    ..
                } = &mut *shared_state;

                if *disabled || !cx.rect().contains(position) {
                    return EventCaptureStatus::NotCaptured;
                }

                if button == PointerButton::Secondary {
                    if let Some(action) = self.right_click_action.as_mut() {
                        cx.send_action((action)(position)).unwrap();
                    }

                    if !cx.has_focus() {
                        cx.steal_focus();
                    }

                    return EventCaptureStatus::Captured;
                } else if button != PointerButton::Primary {
                    return EventCaptureStatus::Captured;
                }

                if !cx.has_focus() {
                    cx.steal_focus();
                }

                self.dragging = true;
                let (buf_x, buf_y) =
                    pos_to_buffer_pos(position, cx.rect().origin, self.text_bounds_rect);

                let action = match click_count {
                    2 => Action::DoubleClick { x: buf_x, y: buf_y },
                    3 => Action::TripleClick { x: buf_x, y: buf_y },
                    _ => Action::Click { x: buf_x, y: buf_y },
                };

                buffer.with_editor_mut(
                    |editor, font_system| -> EditorBorrowStatus {
                        editor.action(font_system, action);

                        EditorBorrowStatus {
                            text_changed: false,
                            has_text: !text.is_empty(),
                        }
                    },
                    cx.font_system,
                );

                editor_state_changed = true;
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustReleased {
                button, position, ..
            }) => {
                if button == PointerButton::Primary {
                    self.dragging = false;
                }

                if !RefCell::borrow(&self.shared_state).disabled && cx.rect().contains(position) {
                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                self.pointer_hovered = false;
            }
            ElementEvent::Keyboard(key_event) => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                let SharedState {
                    buffer,
                    text,
                    disabled,
                    ..
                } = &mut *shared_state;

                if *disabled || key_event.state == KeyState::Up || !self.active {
                    return EventCaptureStatus::NotCaptured;
                }

                match key_event.code {
                    Code::Backspace => {
                        let mut text_changed = false;

                        buffer.with_editor_mut(
                            |editor, font_system| -> EditorBorrowStatus {
                                editor.action(font_system, Action::Backspace);
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
                            cx.font_system,
                        );

                        if text_changed {
                            editor_state_changed = true;
                            self.queue_action = true;
                        }
                    }
                    Code::Escape => {
                        buffer.with_editor_mut(
                            |editor, font_system| -> EditorBorrowStatus {
                                editor.action(font_system, Action::Escape);

                                EditorBorrowStatus {
                                    text_changed: false,
                                    has_text: !text.is_empty(),
                                }
                            },
                            cx.font_system,
                        );

                        editor_state_changed = true;
                    }
                    Code::Delete => {
                        let mut text_changed = false;

                        buffer.with_editor_mut(
                            |editor, font_system| -> EditorBorrowStatus {
                                editor.action(font_system, Action::Delete);
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
                            cx.font_system,
                        );

                        if text_changed {
                            editor_state_changed = true;
                            self.queue_action = true;
                        }
                    }
                    Code::ArrowLeft => {
                        buffer.with_editor_mut(
                            |editor, font_system| -> EditorBorrowStatus {
                                if editor.selection() != Selection::None {
                                    editor.set_selection(Selection::None);
                                }

                                editor.action(font_system, Action::Motion(Motion::Left));

                                EditorBorrowStatus {
                                    text_changed: false,
                                    has_text: !text.is_empty(),
                                }
                            },
                            cx.font_system,
                        );

                        editor_state_changed = true;
                    }
                    Code::ArrowRight => {
                        buffer.with_editor_mut(
                            |editor, font_system| -> EditorBorrowStatus {
                                if editor.selection() != Selection::None {
                                    editor.set_selection(Selection::None);
                                }

                                editor.action(font_system, Action::Motion(Motion::Right));

                                EditorBorrowStatus {
                                    text_changed: false,
                                    has_text: !text.is_empty(),
                                }
                            },
                            cx.font_system,
                        );

                        editor_state_changed = true;
                    }
                    Code::Enter => {
                        if self.queue_action {
                            self.queue_action = false;

                            if let Some(action) = self.action.as_mut() {
                                cx.send_action((action)(text.clone())).unwrap();
                            }
                        }
                    }
                    // TODO: Make this keyboard shortcut configurable.
                    Code::KeyA => {
                        if key_event.modifiers.contains(Modifiers::CONTROL) {
                            select_all_action(
                                text,
                                buffer,
                                cx.font_system,
                                &mut editor_state_changed,
                            );
                        }
                    }
                    // TODO: Make this keyboard shortcut configurable.
                    Code::KeyX => {
                        if key_event.modifiers.contains(Modifiers::CONTROL) {
                            cut_action(
                                text,
                                buffer,
                                cx.clipboard,
                                cx.font_system,
                                &mut self.queue_action,
                                &mut editor_state_changed,
                            );
                        }
                    }
                    // TODO: Make this keyboard shortcut configurable.
                    Code::KeyC => {
                        if key_event.modifiers.contains(Modifiers::CONTROL) {
                            copy_action(text, buffer, cx.clipboard, cx.font_system);
                        }
                    }
                    // TODO: Make this keyboard shortcut configurable.
                    Code::KeyV => {
                        if key_event.modifiers.contains(Modifiers::CONTROL) {
                            paste_action(
                                text,
                                buffer,
                                cx.clipboard,
                                cx.font_system,
                                &mut self.queue_action,
                                &mut editor_state_changed,
                            );
                        }
                    }
                    _ => {}
                }
            }
            ElementEvent::TextComposition(comp_event) => {
                if let CompositionState::End = comp_event.state {
                    let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                    let SharedState { buffer, text, .. } = &mut *shared_state;

                    if comp_event.data.is_empty() || text.len() >= self.max_characters {
                        return EventCaptureStatus::Captured;
                    }

                    let contents = if text.len() + comp_event.data.len() > self.max_characters {
                        &comp_event.data[0..self.max_characters - text.len()]
                    } else {
                        &comp_event.data
                    };

                    let mut text_changed = false;

                    buffer.with_editor_mut(
                        |editor, font_system| -> EditorBorrowStatus {
                            editor.insert_string(contents, None);
                            editor.shape_as_needed(font_system, false);

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
                        cx.font_system,
                    );

                    if text_changed {
                        self.queue_action = true;
                        editor_state_changed = true;
                    }
                }
            }
            ElementEvent::ExclusiveFocus(has_focus) => {
                if has_focus {
                    cx.listen_to_pointer_clicked_off();
                    self.cursor_blink_state_on = true;
                    self.cursor_blink_last_toggle_instant = Instant::now();
                    editor_state_changed = true;

                    let mut shared_state = RefCell::borrow_mut(&self.shared_state);
                    let SharedState { buffer, text, .. } = &mut *shared_state;

                    if self.select_all_when_focused && !text.is_empty() {
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
                            cx.font_system,
                        );
                    }
                } else {
                    self.dragging = false;

                    if self.queue_action {
                        self.queue_action = false;

                        if let Some(action) = self.action.as_mut() {
                            cx.send_action((action)(
                                RefCell::borrow(&self.shared_state).text.clone(),
                            ))
                            .unwrap();
                        }
                    }
                }

                self.active = has_focus;
                cx.set_animating(has_focus);
                cx.request_repaint();
            }
            ElementEvent::ClickedOff => {
                if cx.has_focus() {
                    cx.release_focus();
                }
                self.dragging = false;
            }
            ElementEvent::Pointer(PointerEvent::HoverTimeout { .. }) => {
                if let Some(message) = &self.tooltip_message {
                    cx.show_tooltip(ElementTooltipInfo {
                        message: message.clone(),
                        element_bounds: cx.rect(),
                        align: self.tooltip_align,
                    });
                }
            }
            _ => {}
        }

        if editor_state_changed {
            self.cursor_blink_state_on = true;
            self.cursor_blink_last_toggle_instant = Instant::now();
            self.layout_contents();
            cx.request_repaint();

            if let Some(password_buffer) = self.password_buffer.as_mut() {
                let shared_state = RefCell::borrow(&self.shared_state);
                password_buffer
                    .set_text(&text_to_password_text(&shared_state.buffer), cx.font_system);
            }

            EventCaptureStatus::Captured
        } else {
            EventCaptureStatus::NotCaptured
        }
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let shared_state = RefCell::borrow(&self.shared_state);

        if shared_state.disabled {
            primitives.add(
                shared_state
                    .style
                    .back_quad_disabled
                    .create_primitive(Rect::from_size(cx.bounds_size)),
            );
        } else if self.active {
            primitives.add(
                shared_state
                    .style
                    .back_quad_focused
                    .create_primitive(Rect::from_size(cx.bounds_size)),
            );
        } else {
            primitives.add(
                shared_state
                    .style
                    .back_quad_unfocused
                    .create_primitive(Rect::from_size(cx.bounds_size)),
            );
        }

        let highlight_height = shared_state.style.properties.metrics.font_size
            + shared_state.style.highlight_padding.top
            + shared_state.style.highlight_padding.bottom;
        let highlight_y = self.text_bounds_rect.min_y() - shared_state.style.highlight_padding.top;

        let scroll_x = if self.active {
            let cursor_max_x = self.cursor_x
                + (shared_state.style.cursor_width * 0.5)
                + shared_state.style.padding.left;
            if cursor_max_x >= self.text_bounds_rect.max_x() {
                cursor_max_x - self.text_bounds_rect.max_x()
            } else {
                0.0
            }
        } else {
            0.0
        };

        if self.active {
            if let Some((start_x, end_x)) = self.select_highlight_range {
                let start_x = (start_x + shared_state.style.padding.left - scroll_x)
                    .clamp(self.text_bounds_rect.min_x(), self.text_bounds_rect.max_x());
                let end_x = (end_x + shared_state.style.padding.left - scroll_x)
                    .clamp(self.text_bounds_rect.min_x(), self.text_bounds_rect.max_x());

                if start_x < end_x {
                    primitives.set_z_index(1);

                    primitives.add_solid_quad(
                        SolidQuadBuilder::new(Size::new(end_x - start_x, highlight_height))
                            .position(Point::new(
                                start_x - (shared_state.style.cursor_width * 0.5),
                                highlight_y,
                            ))
                            .bg_color(shared_state.style.highlight_bg_color),
                    );
                }
            }
        }

        if !shared_state.text.is_empty() {
            let color = if shared_state.disabled {
                shared_state.style.font_color_disabled
            } else {
                shared_state.style.font_color
            };

            primitives.set_z_index(2);

            let buffer = if let Some(password_buffer) = &self.password_buffer {
                if shared_state.show_password {
                    shared_state.buffer.clone()
                } else {
                    password_buffer.clone()
                }
            } else {
                shared_state.buffer.clone()
            };

            primitives.add_text(TextPrimitive {
                buffer,
                pos: self.text_bounds_rect.origin + shared_state.text_offset.to_vector()
                    - Point::new(scroll_x, 0.0).to_vector(),
                color,
                clipping_bounds: Rect::new(Point::new(scroll_x, 0.0), self.text_bounds_rect.size),
            })
        } else if !shared_state.placeholder_text.is_empty() {
            primitives.set_z_index(2);

            primitives.add_text(TextPrimitive {
                buffer: shared_state.placeholder_buffer.clone(),
                pos: self.text_bounds_rect.origin + shared_state.text_offset.to_vector(),
                color: shared_state.style.font_color_placeholder,
                clipping_bounds: Rect::from_size(self.text_bounds_rect.size),
            })
        }

        if self.active && self.cursor_blink_state_on {
            primitives.set_z_index(3);

            primitives.add_solid_quad(
                SolidQuadBuilder::new(Size::new(shared_state.style.cursor_width, highlight_height))
                    .position(Point::new(
                        (self.text_bounds_rect.min_x() + self.cursor_x
                            - (shared_state.style.cursor_width * 0.5)
                            - scroll_x)
                            .round(),
                        highlight_y,
                    ))
                    .bg_color(shared_state.style.cursor_color),
            );
        }
    }
}

struct SharedState {
    buffer: RcTextBuffer,
    placeholder_buffer: RcTextBuffer,
    placeholder_text: String,
    text: String,
    style: Rc<TextInputStyle>,
    text_offset: Point,
    disabled: bool,
    queued_actions: Vec<TextInputAction>,
    show_password: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TextInputAction {
    Cut,
    Copy,
    Paste,
    SelectAll,
}

/// A handle to a [`TextInputElement`]
pub struct TextInput {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
    max_characters: usize,
}

impl TextInput {
    pub fn builder<A: Clone + 'static>(style: &Rc<TextInputStyle>) -> TextInputBuilder<A> {
        TextInputBuilder::new(style)
    }

    pub fn set_text(&mut self, text: &str, font_system: &mut FontSystem, select_all: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState {
            buffer,
            text: old_text,
            ..
        } = &mut *shared_state;

        if old_text != text {
            *old_text = String::from(&text[0..self.max_characters]);

            buffer.with_editor_mut(
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
                        has_text: !old_text.is_empty(),
                    }
                },
                font_system,
            );

            self.el.notify_custom_state_change();
        }
    }

    pub fn text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| s.text.as_str())
    }

    pub fn set_placeholder_text(&mut self, text: &str, font_system: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.placeholder_text != text {
            shared_state.placeholder_text = String::from(&text[0..self.max_characters]);
            shared_state.placeholder_buffer.set_text(text, font_system);
            self.el.notify_custom_state_change();
        }
    }

    pub fn placeholder_text<'a>(&'a self) -> Ref<'a, str> {
        Ref::map(RefCell::borrow(&self.shared_state), |s| {
            s.placeholder_text.as_str()
        })
    }

    pub fn set_style(&mut self, style: &Rc<TextInputStyle>, font_system: &mut FontSystem) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);
        let SharedState {
            buffer,
            placeholder_buffer,
            style: old_style,
            text,
            placeholder_text,
            ..
        } = &mut *shared_state;

        if !Rc::ptr_eq(old_style, style) {
            *old_style = Rc::clone(style);
            buffer.set_text_and_props(text, style.properties, font_system);
            placeholder_buffer.set_text_and_props(placeholder_text, style.properties, font_system);
            self.el.notify_custom_state_change();
        }
    }

    pub fn style(&self) -> Rc<TextInputStyle> {
        Rc::clone(&RefCell::borrow(&self.shared_state).style)
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.disabled != disabled {
            shared_state.disabled = true;
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
        self.max_characters
    }

    pub fn perform_cut_action(&mut self) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !shared_state.disabled {
            shared_state.queued_actions.push(TextInputAction::Cut);
            self.el.notify_custom_state_change();
        }
    }

    pub fn perform_copy_action(&mut self) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !shared_state.disabled {
            shared_state.queued_actions.push(TextInputAction::Copy);
            self.el.notify_custom_state_change();
        }
    }

    pub fn perform_paste_action(&mut self) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !shared_state.disabled {
            shared_state.queued_actions.push(TextInputAction::Paste);
            self.el.notify_custom_state_change();
        }
    }

    pub fn perform_select_all_action(&mut self) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if !shared_state.disabled {
            shared_state.queued_actions.push(TextInputAction::SelectAll);
            self.el.notify_custom_state_change();
        }
    }

    /// Show/hide the password. This has no effect if the element wasn't created
    /// with password mode enabled.
    pub fn show_password(&mut self, show: bool) {
        let mut shared_state = RefCell::borrow_mut(&self.shared_state);

        if shared_state.show_password != show {
            shared_state.show_password = show;
            self.el.notify_custom_state_change();
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
        Align::Center => content_rect.min_y() + ((content_rect.height() - font_size) / 2.0) + 1.0,
        Align::End => content_rect.max_y() - font_size,
    };

    Rect::new(
        Point::new(content_rect.min_x(), text_bounds_y),
        Size::new(content_rect.width(), line_height),
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
