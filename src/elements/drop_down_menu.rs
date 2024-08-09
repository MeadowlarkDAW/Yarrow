use std::cell::RefCell;
use std::rc::Rc;

use rootvg::math::Point;
use rootvg::quad::{SolidQuadBuilder, SolidQuadPrimitive};
use rootvg::text::{CustomGlyphID, FontSystem, TextPrimitive, TextProperties};
use rootvg::PrimitiveGroup;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent};
use crate::layout::Padding;
use crate::math::{Rect, Size, ZIndex};
use crate::prelude::ElementStyle;
use crate::style::QuadStyle;
use crate::theme::DEFAULT_ICON_SIZE;
use crate::vg::color::{self, RGBA8};
use crate::view::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, RenderContext,
};
use crate::view::ScissorRectID;
use crate::window::WindowContext;
use crate::CursorIcon;

use super::label::{LabelInner, LabelPaddingInfo, LabelStyle};

// TODO: list of todos:
// * handle cases when the menu is too large to fit in the window, with
// two selectable behaviors:
//   * option A: use scroll wheel
//   * option B: stack horizontally
// * nested menus
// * keyboard navigation

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MenuEntry {
    Option {
        left_icon: Option<CustomGlyphID>,
        icon_scale: f32,
        left_text: String,
        right_text: Option<String>,
        unique_id: usize,
    },
    Divider,
    // TODO: Nested menus
}

impl MenuEntry {
    pub fn option(text: impl Into<String>, unique_id: usize) -> Self {
        Self::Option {
            left_icon: None,
            icon_scale: 1.0,
            left_text: text.into(),
            right_text: None,
            unique_id,
        }
    }

    pub fn option_with_right_text(
        left_text: impl Into<String>,
        right_text: Option<impl Into<String>>,
        unique_id: usize,
    ) -> Self {
        Self::Option {
            left_icon: None,
            icon_scale: 1.0,
            left_text: left_text.into(),
            right_text: right_text.map(|t| t.into()),
            unique_id,
        }
    }

    pub fn option_with_icon(
        text: impl Into<String>,
        icon_id: Option<impl Into<CustomGlyphID>>,
        icon_scale: f32,
        unique_id: usize,
    ) -> Self {
        Self::Option {
            left_icon: icon_id.map(|i| i.into()),
            icon_scale,
            left_text: text.into(),
            right_text: None,
            unique_id,
        }
    }
}

enum MenuEntryInner {
    Option {
        left_label: LabelInner,
        right_label: Option<LabelInner>,
        start_y: f32,
        end_y: f32,
        unique_id: usize,
    },
    Divider {
        y: f32,
    },
    // TODO: Nested menus
}

/// The style of a [`DropDownMenu`] element
#[derive(Debug, Clone, PartialEq)]
pub struct DropDownMenuStyle {
    pub text_properties: TextProperties,
    /// The properties of the right text.
    ///
    /// If this is `None`, then `text_properties` will be used.
    ///
    /// By default this is set to `None`.
    pub right_text_properties: Option<TextProperties>,

    /// The size of the icon in points.
    ///
    /// By default this is set to `20.0`.
    pub icon_size: f32,

    /// The color of the text
    ///
    /// By default this is set to `color::WHITE`.
    pub text_color: RGBA8,
    /// The color of the text when the entry is hovered.
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub text_color_hover: Option<RGBA8>,

    /// The color of the icon.
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub icon_color: Option<RGBA8>,
    /// The color of the icon when the entry is hovered.
    ///
    /// If this is `None`, then `icon_color` will be used.
    ///
    /// By default this is set to `None`.
    pub icon_color_hover: Option<RGBA8>,

    /// The color of the right text.
    ///
    /// If this is `None`, then `text_color` will be used.
    ///
    /// By default this is set to `None`.
    pub right_text_color: Option<RGBA8>,
    /// The color of the right text when the entry is hovered.
    ///
    /// If this is `None`, then `text_color_hover` will be used.
    ///
    /// By default this is set to `None`.
    pub right_text_color_hover: Option<RGBA8>,

    pub back_quad: QuadStyle,
    pub entry_bg_quad_hover: QuadStyle,

    pub outer_padding: f32,

    /// The padding around the left text.
    ///
    /// By default this has all values set to `0.0`.
    pub left_text_padding: Padding,
    /// The padding around the left icon.
    ///
    /// By default this has all values set to `0.0`.
    pub left_icon_padding: Padding,
    /// Extra spacing between the left text and icon. (This can be negative to
    /// move them closer together).
    ///
    /// By default this set to `0.0`.
    pub left_text_icon_spacing: f32,

    /// The padding of the right text.
    pub right_text_padding: Padding,

    pub divider_color: RGBA8,
    pub divider_width: f32,
    pub divider_padding: f32,

    /// The cursor icon to show when the user hovers over a menu entry.
    ///
    /// If this is `None`, then the cursor icon will not be changed.
    ///
    /// By default this is set to `None`.
    pub cursor_icon: Option<CursorIcon>,
}

impl Default for DropDownMenuStyle {
    fn default() -> Self {
        Self {
            text_properties: Default::default(),
            right_text_properties: None,
            icon_size: DEFAULT_ICON_SIZE,
            text_color: color::WHITE,
            text_color_hover: None,
            icon_color: None,
            icon_color_hover: None,
            right_text_color: None,
            right_text_color_hover: None,
            back_quad: QuadStyle::TRANSPARENT,
            entry_bg_quad_hover: QuadStyle::TRANSPARENT,
            outer_padding: 0.0,
            left_icon_padding: Padding::default(),
            left_text_padding: Padding::default(),
            left_text_icon_spacing: 0.0,
            right_text_padding: Padding::default(),
            divider_color: color::TRANSPARENT,
            divider_width: 1.0,
            divider_padding: 0.0,
            cursor_icon: None,
        }
    }
}

impl DropDownMenuStyle {
    fn label_styles(&self, hovered: bool) -> (LabelStyle, LabelStyle) {
        (
            LabelStyle {
                text_properties: self.text_properties,
                icon_size: self.icon_size,
                text_color: if hovered {
                    self.text_color_hover.unwrap_or(self.text_color)
                } else {
                    self.text_color
                },
                icon_color: if hovered {
                    Some(
                        self.icon_color_hover.unwrap_or(
                            self.icon_color
                                .unwrap_or(self.text_color_hover.unwrap_or(self.text_color)),
                        ),
                    )
                } else {
                    Some(self.icon_color.unwrap_or(self.text_color))
                },
                icon_padding: self.left_icon_padding,
                text_padding: self.left_text_padding,
                text_icon_spacing: self.left_text_icon_spacing,
                ..Default::default()
            },
            LabelStyle {
                text_properties: self.right_text_properties.unwrap_or(self.text_properties),
                icon_size: self.icon_size,
                text_color: if hovered {
                    self.right_text_color_hover.unwrap_or(
                        self.right_text_color
                            .unwrap_or(self.text_color_hover.unwrap_or(self.text_color)),
                    )
                } else {
                    self.right_text_color.unwrap_or(self.text_color)
                },
                icon_color: None,
                icon_padding: Padding::zero(),
                text_padding: self.right_text_padding,
                ..Default::default()
            },
        )
    }

    fn text_row_height(&self) -> f32 {
        self.text_properties.metrics.line_height
            + self.left_text_padding.top
            + self.left_text_padding.bottom
    }

    fn left_padding_info(&self) -> LabelPaddingInfo {
        LabelPaddingInfo {
            icon_size: self.icon_size,
            text_padding: self.left_text_padding,
            icon_padding: self.left_icon_padding,
            text_icon_spacing: self.left_text_icon_spacing,
        }
    }

    fn right_padding_info(&self) -> LabelPaddingInfo {
        LabelPaddingInfo {
            icon_size: 0.0,
            text_padding: self.right_text_padding,
            icon_padding: Padding::zero(),
            text_icon_spacing: 0.0,
        }
    }

    fn measure(&self, entries: &mut [MenuEntryInner]) -> Size {
        if entries.is_empty() {
            return Size::default();
        }

        let text_row_height = self.text_row_height();

        let mut max_width: f32 = 0.0;
        let mut total_height: f32 = self.outer_padding;
        for entry in entries.iter_mut() {
            match entry {
                MenuEntryInner::Option {
                    left_label,
                    right_label,
                    start_y,
                    end_y,
                    ..
                } => {
                    let left_size = left_label.desired_size(|| self.left_padding_info());
                    let right_size = right_label
                        .as_mut()
                        .map(|l| l.desired_size(|| self.right_padding_info()))
                        .unwrap_or(Size::zero());

                    let total_width = left_size.width + right_size.width;

                    max_width = max_width.max(total_width);

                    *start_y = total_height;
                    total_height += text_row_height;
                    *end_y = total_height;
                }
                MenuEntryInner::Divider { y } => {
                    *y = total_height + self.divider_padding;

                    total_height +=
                        self.divider_width + self.divider_padding + self.divider_padding;
                }
            }
        }

        Size::new(
            max_width.ceil() + (self.outer_padding * 2.0),
            total_height + self.outer_padding,
        )
    }
}

impl ElementStyle for DropDownMenuStyle {
    const ID: &'static str = "ddmenu";

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

pub struct DropDownMenuBuilder<A: Clone + 'static> {
    pub action: Option<Box<dyn FnMut(usize) -> A>>,
    pub entries: Vec<MenuEntry>,
    pub class: Option<&'static str>,
    pub z_index: Option<ZIndex>,
    pub position: Point,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> DropDownMenuBuilder<A> {
    pub fn new() -> Self {
        Self {
            action: None,
            entries: Vec::new(),
            class: None,
            z_index: None,
            position: Point::default(),
            scissor_rect_id: None,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> DropDownMenu {
        DropDownMenuElement::create(self, cx)
    }

    pub fn on_entry_selected<F: FnMut(usize) -> A + 'static>(mut self, f: F) -> Self {
        self.action = Some(Box::new(f));
        self
    }

    pub fn entries(mut self, entries: Vec<MenuEntry>) -> Self {
        self.entries = entries.into();
        self
    }

    /// The style class name
    ///
    /// If this method is not used, then the current class from the window context will
    /// be used.
    pub const fn class(mut self, class: &'static str) -> Self {
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

    /// The ID of the scissoring rectangle this element belongs to.
    ///
    /// If this method is not used, then the current scissoring rectangle ID from the
    /// window context will be used.
    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = Some(scissor_rect_id);
        self
    }
}

pub struct DropDownMenuElement<A: Clone + 'static> {
    shared_state: Rc<RefCell<SharedState>>,
    action: Option<Box<dyn FnMut(usize) -> A>>,
    entries: Vec<MenuEntryInner>,
    size: Size,
    active: bool,
    hovered_entry_index: Option<usize>,
    cursor_icon: Option<CursorIcon>,
}

impl<A: Clone + 'static> DropDownMenuElement<A> {
    pub fn create(builder: DropDownMenuBuilder<A>, cx: &mut WindowContext<'_, A>) -> DropDownMenu {
        let DropDownMenuBuilder {
            action,
            entries,
            class,
            z_index,
            position,
            scissor_rect_id,
        } = builder;

        let (z_index, scissor_rect_id, class) = cx.builder_values(z_index, scissor_rect_id, class);

        let shared_state = Rc::new(RefCell::new(SharedState {
            new_entries: None,
            open_requested: false,
        }));

        let style = cx.res.style_system.get::<DropDownMenuStyle>(class);
        let cursor_icon = style.cursor_icon;

        let mut entries = build_entries(entries, &style, &mut cx.res.font_system);

        let size = style.measure(&mut entries);

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                shared_state: Rc::clone(&shared_state),
                action,
                entries,
                size,
                active: false,
                hovered_entry_index: None,
                cursor_icon,
            }),
            z_index,
            bounding_rect: Rect::new(position, Size::zero()),
            manually_hidden: false,
            scissor_rect_id,
            class,
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        DropDownMenu { el, shared_state }
    }
}

impl<A: Clone + 'static> Element<A> for DropDownMenuElement<A> {
    fn flags(&self) -> ElementFlags {
        ElementFlags::PAINTS
            | ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
            | ElementFlags::LISTENS_TO_FOCUS_CHANGE
            | ElementFlags::LISTENS_TO_POINTER_OUTSIDE_BOUNDS_WHEN_FOCUSED
            | ElementFlags::LISTENS_TO_POSITION_CHANGE
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        match event {
            ElementEvent::CustomStateChanged => {
                let mut shared_state = RefCell::borrow_mut(&self.shared_state);

                let mut show = false;
                let mut request_focus = false;
                if shared_state.open_requested && !self.active {
                    self.active = true;
                    show = true;
                    request_focus = true;
                }
                shared_state.open_requested = false;

                if let Some(new_entries) = shared_state.new_entries.take() {
                    let style = cx.res.style_system.get(cx.class());

                    self.entries = build_entries(new_entries, style, &mut cx.res.font_system);

                    self.size = style.measure(&mut self.entries);

                    if self.active {
                        let rect = Rect::new(cx.rect().origin, self.size);
                        let layout_info = layout(rect, cx.window_size());
                        if let Some(new_bounds) = layout_info.new_bounds {
                            cx.set_bounding_rect(new_bounds);
                        } else {
                            cx.set_bounding_rect(rect)
                        }
                        cx.request_repaint();
                    } else {
                        cx.set_bounding_rect(Rect::new(cx.rect().origin, Size::zero()));
                    }

                    show = false;
                }

                if show {
                    let rect = Rect::new(cx.rect().origin, self.size);
                    let layout_info = layout(rect, cx.window_size());
                    if let Some(new_bounds) = layout_info.new_bounds {
                        cx.set_bounding_rect(new_bounds);
                    } else {
                        cx.set_bounding_rect(rect)
                    }
                }

                if request_focus {
                    cx.steal_temporary_focus();
                    cx.listen_to_pointer_clicked_off();
                }
            }
            ElementEvent::StyleChanged => {
                let style = cx.res.style_system.get::<DropDownMenuStyle>(cx.class());
                self.cursor_icon = style.cursor_icon;
            }
            ElementEvent::ClickedOff => {
                cx.release_focus();
            }
            ElementEvent::Focus(false) => {
                self.active = false;
                self.hovered_entry_index = None;
                cx.set_bounding_rect(Rect::new(cx.rect().origin, Size::zero()));
            }
            ElementEvent::Pointer(PointerEvent::Moved { position, .. }) => {
                if !self.active {
                    return EventCaptureStatus::NotCaptured;
                }

                let mut new_hovered_entry_index = None;
                let pointer_y = position.y - cx.rect().min_y();
                if cx.rect().contains(position) {
                    for (i, entry) in self.entries.iter().enumerate() {
                        match entry {
                            MenuEntryInner::Option { start_y, end_y, .. } => {
                                if pointer_y >= *start_y && pointer_y < *end_y {
                                    new_hovered_entry_index = Some(i);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if self.hovered_entry_index != new_hovered_entry_index {
                    self.hovered_entry_index = new_hovered_entry_index;
                    cx.request_repaint();
                }

                if let Some(cursor_icon) = self.cursor_icon {
                    if self.hovered_entry_index.is_some() {
                        cx.cursor_icon = cursor_icon;
                    }
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::PointerLeft) => {
                if !self.active {
                    return EventCaptureStatus::NotCaptured;
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed {
                button, position, ..
            }) => {
                if !self.active {
                    return EventCaptureStatus::NotCaptured;
                }

                if button == PointerButton::Primary && cx.rect().contains(position) {
                    let mut selected_entry_id = None;
                    let pointer_y = position.y - cx.rect().min_y();
                    for entry in self.entries.iter() {
                        match entry {
                            MenuEntryInner::Option {
                                start_y,
                                end_y,
                                unique_id,
                                ..
                            } => {
                                if pointer_y >= *start_y && pointer_y < *end_y {
                                    selected_entry_id = Some(*unique_id);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    if let Some(id) = selected_entry_id {
                        if let Some(action) = &mut self.action {
                            cx.send_action((action)(id)).unwrap();
                        }

                        cx.release_focus();
                        cx.cursor_icon = CursorIcon::Default;
                    }
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::Pointer(..) => {
                if !self.active {
                    return EventCaptureStatus::NotCaptured;
                }

                return EventCaptureStatus::Captured;
            }
            ElementEvent::PositionChanged => {
                if !self.active {
                    return EventCaptureStatus::NotCaptured;
                }

                let layout_info = layout(cx.rect(), cx.window_size());
                if let Some(new_bounds) = layout_info.new_bounds {
                    cx.set_bounding_rect(new_bounds);
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }

    fn render_primitives(&mut self, cx: RenderContext<'_>, primitives: &mut PrimitiveGroup) {
        let style: &DropDownMenuStyle = cx.res.style_system.get(cx.class);

        let (left_style_idle, right_style_idle) = style.label_styles(false);
        let (left_style_hover, right_style_hover) = style.label_styles(true);

        let label_size = Size::new(
            self.size.width - (style.outer_padding * 2.0),
            style.text_row_height(),
        );

        let mut text_primitives: Vec<TextPrimitive> = Vec::with_capacity(self.entries.len() * 3);
        let mut divider_primitives: Vec<SolidQuadPrimitive> =
            Vec::with_capacity(self.entries.len());

        primitives.add(
            style
                .back_quad
                .create_primitive(Rect::from_size(cx.bounds_size)),
        );

        for (i, entry) in self.entries.iter_mut().enumerate() {
            match entry {
                MenuEntryInner::Option {
                    left_label,
                    right_label,
                    start_y,
                    ..
                } => {
                    let hovered = if let Some(hovered_index) = self.hovered_entry_index {
                        i == hovered_index
                    } else {
                        false
                    };

                    if hovered {
                        primitives.set_z_index(1);
                        primitives.add(style.entry_bg_quad_hover.create_primitive(Rect::new(
                            Point::new(style.outer_padding, *start_y),
                            label_size,
                        )));
                    }

                    let left_primitives = left_label.render_primitives(
                        Rect::new(Point::new(style.outer_padding, *start_y), label_size),
                        if hovered {
                            &left_style_hover
                        } else {
                            &left_style_idle
                        },
                        &mut cx.res.font_system,
                    );

                    if let Some(p) = left_primitives.icon {
                        text_primitives.push(p);
                    }
                    if let Some(p) = left_primitives.text {
                        text_primitives.push(p);
                    }

                    if let Some(right_label) = right_label {
                        let right_style = if hovered {
                            &right_style_hover
                        } else {
                            &right_style_idle
                        };

                        let right_x = cx.bounds_size.width
                            - style.outer_padding
                            - right_label
                                .desired_size(|| style.right_padding_info())
                                .width;

                        let right_primitives = right_label.render_primitives(
                            Rect::new(Point::new(right_x, *start_y), label_size),
                            right_style,
                            &mut cx.res.font_system,
                        );

                        if let Some(p) = right_primitives.text {
                            text_primitives.push(p);
                        }
                    }
                }
                MenuEntryInner::Divider { y } => divider_primitives.push(
                    SolidQuadBuilder::new(Size::new(label_size.width, style.divider_width))
                        .bg_color(style.divider_color)
                        .position(Point::new(style.outer_padding, *y))
                        .into(),
                ),
            }
        }

        primitives.set_z_index(2);

        // It is more efficient to batch primitives together.
        primitives.add_text_batch(text_primitives);
        primitives.add_solid_quad_batch(divider_primitives);
    }
}

struct SharedState {
    new_entries: Option<Vec<MenuEntry>>,
    open_requested: bool,
}

/// A handle to a [`DropDownMenuElement`].
pub struct DropDownMenu {
    pub el: ElementHandle,
    shared_state: Rc<RefCell<SharedState>>,
}

impl DropDownMenu {
    pub fn builder<A: Clone + 'static>() -> DropDownMenuBuilder<A> {
        DropDownMenuBuilder::new()
    }

    pub fn set_class(&mut self, class: &'static str) {
        if self.el.class() != class {
            self.el._notify_class_change(class);
        }
    }

    pub fn set_position(&mut self, pos: Point) {
        self.el.set_pos(pos)
    }

    pub fn set_entries(&mut self, entries: Vec<MenuEntry>) {
        RefCell::borrow_mut(&self.shared_state).new_entries = Some(entries);
        self.el._notify_custom_state_change();
    }

    pub fn open(&mut self, position: Option<Point>) {
        if let Some(pos) = position {
            self.set_position(pos);
        }

        RefCell::borrow_mut(&self.shared_state).open_requested = true;
        self.el._notify_custom_state_change();
    }
}

fn build_entries(
    entries: Vec<MenuEntry>,
    style: &DropDownMenuStyle,
    font_system: &mut FontSystem,
) -> Vec<MenuEntryInner> {
    let (left_style, right_style) = style.label_styles(false);

    entries
        .into_iter()
        .map(|entry| match entry {
            MenuEntry::Option {
                left_icon,
                icon_scale,
                left_text,
                right_text,
                unique_id,
            } => MenuEntryInner::Option {
                left_label: LabelInner::new(
                    Some(left_text),
                    left_icon,
                    Point::default(),
                    Point::default(),
                    icon_scale,
                    Default::default(),
                    &left_style,
                    font_system,
                ),
                right_label: right_text.map(|text| {
                    LabelInner::new(
                        Some(text),
                        None,
                        Point::default(),
                        Point::default(),
                        1.0,
                        Default::default(),
                        &right_style,
                        font_system,
                    )
                }),
                start_y: 0.0,
                end_y: 0.0,
                unique_id,
            },
            MenuEntry::Divider => MenuEntryInner::Divider { y: 0.0 },
        })
        .collect()
}

fn layout(current_bounds: Rect, window_size: Size) -> LayoutInfo {
    let window_rect = Rect::from_size(window_size);

    let (width, width_clipped) = if current_bounds.width() > window_rect.width() {
        (window_rect.width(), true)
    } else {
        (current_bounds.width(), false)
    };
    let (height, height_clipped) = if current_bounds.height() > window_rect.height() {
        (window_rect.height(), true)
    } else {
        (current_bounds.height(), false)
    };

    let (x, x_repositioned) = if current_bounds.min_x() <= 0.0 {
        (0.0, true)
    } else if current_bounds.min_x() + width > window_size.width {
        (window_size.width - width, true)
    } else {
        (current_bounds.min_x(), false)
    };
    let (y, y_repositioned) = if current_bounds.min_y() <= 0.0 {
        (0.0, true)
    } else if current_bounds.min_y() + height > window_size.height {
        (window_size.height - height, true)
    } else {
        (current_bounds.min_y(), false)
    };

    let new_bounds = if width_clipped || height_clipped || x_repositioned || y_repositioned {
        Some(Rect::new(Point::new(x, y), Size::new(width, height)))
    } else {
        None
    };

    LayoutInfo {
        new_bounds,
        width_clipped,
        height_clipped,
    }
}

struct LayoutInfo {
    new_bounds: Option<Rect>,
    // TODO: use these
    #[allow(unused)]
    width_clipped: bool,
    #[allow(unused)]
    height_clipped: bool,
}
