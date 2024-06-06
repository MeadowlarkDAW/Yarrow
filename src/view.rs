// ---------------------------------------------------------------------------------
//
//    '%%' '%% '%%'
//    %'%\% | %/%'%     Yarrow GUI Library
//        \ | /
//         \|/          https://codeberg.org/BillyDM/Yarrow
//          |
//
//
// MIT License Copyright (c) 2024 Billy Messenger
// https://github.com/MeadowlarkDAW/Yarrow/blob/main/LICENSE
//
// ---------------------------------------------------------------------------------

use std::time::Duration;
use std::time::Instant;

use keyboard_types::CompositionEvent;
use rootvg::color::PackedSrgb;
use rootvg::math::PhysicalSizeI32;
use rootvg::math::SizeI32;
use rootvg::text::glyphon::FontSystem;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;
use thunderdome::Arena;

use crate::action_queue::ActionSender;
use crate::clipboard::Clipboard;
use crate::event::{CanvasEvent, ElementEvent, EventCaptureStatus, KeyboardEvent, PointerEvent};
use crate::layout::Align2;
use crate::math::{Point, PointI32, Rect, RectI32, ScaleFactor, Size, ZIndex};
use crate::stmpsc_queue;
use crate::CursorIcon;
use crate::WindowID;

mod cache;
pub mod element;
mod scissor_rect;

use self::element::ChangeFocusRequest;
use self::element::ElementTooltipInfo;
use self::element::RenderContext;
pub use self::scissor_rect::{ScissorRectID, MAIN_SCISSOR_RECT};

use self::cache::{
    sync_element_rect_cache, CachedElementPrimitives, CachedElementRectForPointerEvent,
};
use self::element::{
    Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle, ElementID,
    ElementModification, ElementModificationType,
};
use self::scissor_rect::ScissorRect;

/// The settings for a new `View`.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewConfig {
    /// The clear color.
    pub clear_color: PackedSrgb,

    /// An estimate for how many elements are expected to be in this view in a
    /// typical use case. This is used to pre-allocate capacity to improve slightly
    /// improve load-up times.
    ///
    /// By default this is set to `0` (no capacity will be pre-allocated).
    pub preallocate_for_this_many_elements: u32,

    /// The duration between when an element is first hovered and when it receives the
    /// `ElementEvent::Pointer(PointerEvent::HoverTimeout)` event.
    ///
    /// By default this is set to 0.5 seconds.
    pub hover_timeout_duration: Duration,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            clear_color: PackedSrgb::BLACK,
            preallocate_for_this_many_elements: 0,
            hover_timeout_duration: Duration::from_millis(500),
        }
    }
}

pub struct View<A: Clone + 'static> {
    pub clear_color: PackedSrgb,
    pub action_sender: ActionSender<A>,
    pub cursor_icon: CursorIcon,

    element_arena: Arena<ElementEntry<A>>,

    scissor_rects: Vec<ScissorRect>,

    mod_queue_sender: stmpsc_queue::Sender<ElementModification>,
    mod_queue_receiver: stmpsc_queue::Receiver<ElementModification>,

    hovered_elements: FxHashMap<ElementID, Option<Instant>>,
    animating_elements: Vec<ElementID>,
    current_focus_info: Option<FocusInfo>,
    prev_element_with_exclusive_focus: Option<ElementID>,

    elements_listening_to_pointer_event: Vec<CachedElementRectForPointerEvent>,
    elements_listening_to_pointer_event_need_sorted: bool,
    painted_elements: Vec<CachedElementPrimitives>,
    elements_listening_to_clicked_off: FxHashSet<ElementID>,
    element_with_active_tooltip: Option<(ElementID, Rect)>,

    physical_size: PhysicalSizeI32,
    logical_size: Size,
    scale_factor: ScaleFactor,
    hover_timeout_duration: Duration,
    window_id: WindowID,

    show_tooltip_action: Option<Box<dyn FnMut(TooltipInfo) -> A>>,
    hide_tooltip_action: Option<Box<dyn FnMut() -> A>>,

    view_needs_repaint: bool,
    window_visible: bool,
}

impl<A: Clone + 'static> View<A> {
    pub fn new(
        physical_size: PhysicalSizeI32,
        scale_factor: ScaleFactor,
        config: ViewConfig,
        action_sender: ActionSender<A>,
        window_id: WindowID,
    ) -> Self {
        let ViewConfig {
            clear_color,
            preallocate_for_this_many_elements,
            hover_timeout_duration,
        } = config;

        assert!(scale_factor.0 > 0.0);

        let logical_size = crate::math::to_logical_size_i32(physical_size, scale_factor);

        let view_rect = RectI32::new(
            PointI32::default(),
            SizeI32::new(
                logical_size.width.round() as i32,
                logical_size.height.round() as i32,
            ),
        );

        let scissor_rects = vec![ScissorRect::new(view_rect, Point::default())];

        let capacity = preallocate_for_this_many_elements as usize;

        let (mod_queue_sender, mod_queue_receiver) = stmpsc_queue::single_thread_mpsc_queue(
            // Give some wiggle-room since elements can be added to the queue more than once.
            capacity * 4,
        );

        Self {
            clear_color,
            action_sender,

            element_arena: Arena::with_capacity(capacity),

            scissor_rects,

            mod_queue_sender,
            mod_queue_receiver,

            hovered_elements: FxHashMap::default(),
            animating_elements: Vec::with_capacity(capacity),
            current_focus_info: None,
            prev_element_with_exclusive_focus: None,

            elements_listening_to_pointer_event: Vec::new(),
            elements_listening_to_pointer_event_need_sorted: false,
            painted_elements: Vec::new(),
            elements_listening_to_clicked_off: FxHashSet::default(),
            element_with_active_tooltip: None,

            physical_size,
            logical_size,
            scale_factor,
            hover_timeout_duration,
            window_id,

            view_needs_repaint: true,
            window_visible: true,

            show_tooltip_action: None,
            hide_tooltip_action: None,

            cursor_icon: CursorIcon::Default,
        }
    }

    pub fn size(&self) -> Size {
        self.logical_size
    }

    pub fn set_tooltip_actions<S, H>(&mut self, on_show_tooltip: S, on_hide_tooltip: H)
    where
        S: FnMut(TooltipInfo) -> A + 'static,
        H: FnMut() -> A + 'static,
    {
        self.show_tooltip_action = Some(Box::new(on_show_tooltip));
        self.hide_tooltip_action = Some(Box::new(on_hide_tooltip));
    }

    pub fn scissor_rect(&self, scissor_rect_id: ScissorRectID) -> Option<RectI32> {
        self.scissor_rects
            .get(usize::from(scissor_rect_id))
            .map(|c| c.rect())
    }

    pub fn scissor_rect_scroll_offset(&self, scissor_rect_id: ScissorRectID) -> Option<Point> {
        self.scissor_rects
            .get(usize::from(scissor_rect_id))
            .map(|c| c.scroll_offset())
    }

    pub fn set_num_additional_scissor_rects(&mut self, num: usize) {
        self.scissor_rects.resize_with(1 + num, || {
            ScissorRect::new(RectI32::default(), Point::default())
        })
    }

    // TODO: Custom error type.
    pub fn update_scissor_rect(
        &mut self,
        scissor_rect_id: ScissorRectID,
        new_rect: Option<Rect>,
        new_scroll_offset: Option<Point>,
    ) -> Result<(), ()> {
        if scissor_rect_id == MAIN_SCISSOR_RECT {
            return Err(());
        }

        let new_rect: Option<RectI32> = new_rect.map(|r| r.round().cast());

        if let Some(new_rect) = new_rect {
            let view_rect = self.scissor_rects[0].rect();
            if !view_rect.contains_rect(&new_rect) {
                // TODO: Log warning.
            }
        };

        let scissor_rect = self
            .scissor_rects
            .get_mut(usize::from(scissor_rect_id))
            .ok_or(())?;

        scissor_rect.update(new_rect, new_scroll_offset, &mut self.mod_queue_sender);

        Ok(())
    }

    pub fn add_element(
        &mut self,
        element_builder: ElementBuilder<A>,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) -> ElementHandle {
        let ElementBuilder {
            element,
            z_index,
            bounding_rect,
            manually_hidden,
            scissor_rect_id,
        } = element_builder;

        let flags = element.flags();

        // If the scissoring rectangle ID is invalid, default to the view scissoring rectangle.
        let scissor_rect_id = if usize::from(scissor_rect_id) >= self.scissor_rects.len() {
            // TODO: Log warning.

            MAIN_SCISSOR_RECT
        } else {
            scissor_rect_id
        };

        let mut stack_data = EntryStackData {
            rect: bounding_rect,
            visible_rect: None,
            pos_relative_to_scissor_rect: bounding_rect.origin,
            scissor_rect_id,
            z_index,
            flags,
            manually_hidden,
            animating: false,
            index_in_painted_list: 0,
            index_in_pointer_event_list: 0,
            index_in_animating_list: 0,
            index_in_scissor_rect_list: 0,
        };

        stack_data.update_layout(&self.scissor_rects);
        stack_data.update_visibility(&self.scissor_rects, self.window_visible);

        if stack_data.visible() && stack_data.flags.contains(ElementFlags::PAINTS) {
            self.view_needs_repaint = true;
        }

        let element_id = ElementID(self.element_arena.insert(ElementEntry {
            stack_data,
            element,
        }));

        let element_entry = self.element_arena.get_mut(element_id.0).unwrap();

        self.scissor_rects[usize::from(scissor_rect_id)]
            .add_element(&mut element_entry.stack_data, element_id);

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
        {
            element_entry.stack_data.index_in_pointer_event_list =
                self.elements_listening_to_pointer_event.len() as u32;

            self.elements_listening_to_pointer_event
                .push(CachedElementRectForPointerEvent {
                    z_index: element_entry.stack_data.z_index,
                    element_id,
                    visible_rect: element_entry.stack_data.visible_rect,
                });
            self.elements_listening_to_pointer_event_need_sorted = true;
        }

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::PAINTS)
        {
            element_entry.stack_data.index_in_painted_list = self.painted_elements.len() as u32;
            self.painted_elements.push(CachedElementPrimitives::new(
                element_id,
                element_entry.stack_data.rect.origin,
                element_entry.stack_data.z_index,
                element_entry.stack_data.scissor_rect_id,
                element_entry.stack_data.visible(),
            ));
        }

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::LISTENS_TO_INIT)
        {
            send_event_to_element(
                ElementEvent::Init,
                element_entry,
                element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        self::element::new_element_handle(
            element_id,
            self.mod_queue_sender.clone(),
            bounding_rect,
            z_index,
            manually_hidden,
        )
    }

    /// Returns the bounding rectangle of the given element, accounting for scroll offset.
    ///
    /// If the element has been dropped, then this will return `None`.
    pub fn element_rect(&self, handle: &ElementHandle) -> Option<Rect> {
        self.element_arena
            .get(handle.id().0)
            .map(|entry| entry.stack_data.rect)
    }

    pub(crate) fn resize(&mut self, physical_size: PhysicalSizeI32, scale_factor: ScaleFactor) {
        self.physical_size = physical_size;
        self.scale_factor = scale_factor;
        self.logical_size = crate::math::to_logical_size_i32(physical_size, scale_factor);

        self.scissor_rects[0].update(
            Some(RectI32::new(
                PointI32::default(),
                SizeI32::new(
                    self.logical_size.width.round() as i32,
                    self.logical_size.height.round() as i32,
                ),
            )),
            None,
            &mut self.mod_queue_sender,
        );

        self.view_needs_repaint = true;
    }

    pub(crate) fn handle_event(
        &mut self,
        event: &CanvasEvent,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) -> EventCaptureStatus {
        match event {
            CanvasEvent::Animation {
                delta_seconds,
                pointer_position,
            } => {
                self.handle_animation_event(
                    *delta_seconds,
                    *pointer_position,
                    font_system,
                    clipboard,
                );

                // Capture status is not relavant for this event.
                EventCaptureStatus::NotCaptured
            }
            CanvasEvent::Pointer(pointer_event) => {
                self.handle_pointer_event(pointer_event, font_system, clipboard)
            }
            CanvasEvent::Keyboard(keyboard_event) => {
                self.handle_keyboard_event(keyboard_event, font_system, clipboard)
            }
            CanvasEvent::TextComposition(text_composition_event) => {
                self.handle_text_composition_event(text_composition_event, font_system, clipboard)
            }
            CanvasEvent::WindowHidden => {
                self.handle_window_hidden(font_system, clipboard);
                EventCaptureStatus::NotCaptured
            }
            CanvasEvent::WindowShown => {
                self.handle_window_shown(font_system, clipboard);
                EventCaptureStatus::NotCaptured
            }
            CanvasEvent::WindowFocused => {
                // TODO
                EventCaptureStatus::NotCaptured
            }
            CanvasEvent::WindowUnfocused => {
                self.handle_window_unfocused(font_system, clipboard);
                EventCaptureStatus::NotCaptured
            }
        }
    }

    fn handle_window_shown(&mut self, font_system: &mut FontSystem, clipboard: &mut Clipboard) {
        if self.window_visible {
            return;
        }
        self.window_visible = true;

        let painted_elements: Vec<ElementID> =
            self.painted_elements.iter().map(|e| e.element_id).collect();
        for element_id in painted_elements.iter() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                element_entry
                    .stack_data
                    .update_visibility(&self.scissor_rects, self.window_visible);

                if element_entry.stack_data.visible() {
                    sync_element_rect_cache(
                        &mut element_entry.stack_data,
                        &mut self.elements_listening_to_pointer_event,
                        &mut self.painted_elements,
                        false,
                    );

                    if element_entry
                        .stack_data
                        .flags
                        .contains(ElementFlags::LISTENS_TO_VISIBILITY_CHANGE)
                    {
                        send_event_to_element(
                            ElementEvent::Shown,
                            element_entry,
                            *element_id,
                            &self.current_focus_info,
                            &mut self.mod_queue_sender,
                            &mut self.action_sender,
                            self.scale_factor,
                            self.logical_size,
                            &mut self.cursor_icon,
                            font_system,
                            clipboard,
                        );
                    }
                }
            }
        }
    }

    fn handle_window_hidden(&mut self, font_system: &mut FontSystem, clipboard: &mut Clipboard) {
        if !self.window_visible {
            return;
        }
        self.window_visible = false;

        let mut visible_elements: Vec<ElementID> = Vec::new();
        for painted_element in self.painted_elements.iter() {
            if painted_element.visible {
                visible_elements.push(painted_element.element_id);
            }
        }

        for element_id in visible_elements.iter() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                element_entry
                    .stack_data
                    .update_visibility(&self.scissor_rects, self.window_visible);

                sync_element_rect_cache(
                    &mut element_entry.stack_data,
                    &mut self.elements_listening_to_pointer_event,
                    &mut self.painted_elements,
                    false,
                );

                if element_entry
                    .stack_data
                    .flags
                    .contains(ElementFlags::LISTENS_TO_VISIBILITY_CHANGE)
                {
                    send_event_to_element(
                        ElementEvent::Hidden,
                        element_entry,
                        *element_id,
                        &self.current_focus_info,
                        &mut self.mod_queue_sender,
                        &mut self.action_sender,
                        self.scale_factor,
                        self.logical_size,
                        &mut self.cursor_icon,
                        font_system,
                        clipboard,
                    );
                }
            }
        }

        if let Some(_) = self.element_with_active_tooltip.take() {
            if let Some(action) = self.hide_tooltip_action.as_mut() {
                self.action_sender.send((action)()).unwrap();
            }
        }
    }

    fn handle_window_unfocused(&mut self, font_system: &mut FontSystem, clipboard: &mut Clipboard) {
        for (element_id, _) in self.hovered_elements.iter() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                send_event_to_element(
                    ElementEvent::Pointer(PointerEvent::PointerLeft),
                    element_entry,
                    *element_id,
                    &self.current_focus_info,
                    &mut self.mod_queue_sender,
                    &mut self.action_sender,
                    self.scale_factor,
                    self.logical_size,
                    &mut self.cursor_icon,
                    font_system,
                    clipboard,
                );
            }
        }
        self.hovered_elements.clear();

        for element_id in self.elements_listening_to_clicked_off.iter() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                send_event_to_element(
                    ElementEvent::ClickedOff,
                    element_entry,
                    *element_id,
                    &self.current_focus_info,
                    &mut self.mod_queue_sender,
                    &mut self.action_sender,
                    self.scale_factor,
                    self.logical_size,
                    &mut self.cursor_icon,
                    font_system,
                    clipboard,
                );
            }
        }
        self.elements_listening_to_clicked_off.clear();

        if let Some(_) = self.element_with_active_tooltip.take() {
            if let Some(action) = self.hide_tooltip_action.as_mut() {
                self.action_sender.send((action)()).unwrap();
            }
        }

        // TODO: Release exclusive focus if the pointer is locked.
    }

    fn handle_animation_event(
        &mut self,
        delta_seconds: f64,
        pointer_position: Option<Point>,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) {
        for element_id in self.animating_elements.iter() {
            let element_entry = self.element_arena.get_mut(element_id.0).unwrap();

            let _ = send_event_to_element(
                ElementEvent::Animation { delta_seconds },
                element_entry,
                *element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        let pos = pointer_position.unwrap_or_default();
        for (element_id, hover_start_instant) in self.hovered_elements.iter_mut() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                if let Some(visible_rect) = element_entry.stack_data.visible_rect {
                    if visible_rect.contains(pos) {
                        if let Some(instant) = hover_start_instant.take() {
                            if instant.elapsed() >= self.hover_timeout_duration {
                                send_event_to_element(
                                    ElementEvent::Pointer(PointerEvent::HoverTimeout {
                                        position: pos,
                                    }),
                                    element_entry,
                                    *element_id,
                                    &self.current_focus_info,
                                    &mut self.mod_queue_sender,
                                    &mut self.action_sender,
                                    self.scale_factor,
                                    self.logical_size,
                                    &mut self.cursor_icon,
                                    font_system,
                                    clipboard,
                                );
                            } else {
                                *hover_start_instant = Some(instant)
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_pointer_event(
        &mut self,
        event: &PointerEvent,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) -> EventCaptureStatus {
        let pos = event.position();

        if let PointerEvent::Moved { .. } = event {
            self.cursor_icon = CursorIcon::Default;

            if let Some((element_id, bounds)) = self.element_with_active_tooltip.take() {
                if !bounds.contains(pos) {
                    if let Some(action) = self.hide_tooltip_action.as_mut() {
                        self.action_sender.send((action)()).unwrap();
                    }
                } else {
                    self.element_with_active_tooltip = Some((element_id, bounds));
                }
            }
        } else if let PointerEvent::PointerLeft = event {
            for (element_id, _) in self.hovered_elements.iter_mut() {
                if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                    send_event_to_element(
                        ElementEvent::Pointer(PointerEvent::PointerLeft),
                        element_entry,
                        *element_id,
                        &self.current_focus_info,
                        &mut self.mod_queue_sender,
                        &mut self.action_sender,
                        self.scale_factor,
                        self.logical_size,
                        &mut self.cursor_icon,
                        font_system,
                        clipboard,
                    );
                }
            }
            self.hovered_elements.clear();

            if let Some(_) = self.element_with_active_tooltip.take() {
                if let Some(action) = self.hide_tooltip_action.as_mut() {
                    self.action_sender.send((action)()).unwrap();
                }
            }

            return EventCaptureStatus::NotCaptured;
        }

        let mut unhovered_elements: SmallVec<[ElementID; 4]> = SmallVec::new();
        for (element_id, hover_start_instant) in self.hovered_elements.iter_mut() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                let unhovered = if let Some(visible_rect) = element_entry.stack_data.visible_rect {
                    !visible_rect.contains(pos)
                } else {
                    true
                };

                if unhovered {
                    unhovered_elements.push(*element_id);

                    send_event_to_element(
                        ElementEvent::Pointer(PointerEvent::PointerLeft),
                        element_entry,
                        *element_id,
                        &self.current_focus_info,
                        &mut self.mod_queue_sender,
                        &mut self.action_sender,
                        self.scale_factor,
                        self.logical_size,
                        &mut self.cursor_icon,
                        font_system,
                        clipboard,
                    );
                } else if let Some(instant) = hover_start_instant.take() {
                    if instant.elapsed() >= self.hover_timeout_duration {
                        send_event_to_element(
                            ElementEvent::Pointer(PointerEvent::HoverTimeout { position: pos }),
                            element_entry,
                            *element_id,
                            &self.current_focus_info,
                            &mut self.mod_queue_sender,
                            &mut self.action_sender,
                            self.scale_factor,
                            self.logical_size,
                            &mut self.cursor_icon,
                            font_system,
                            clipboard,
                        );
                    } else {
                        *hover_start_instant = Some(instant)
                    }
                }
            } else {
                unhovered_elements.push(*element_id);
            }
        }
        for element_id in unhovered_elements.iter() {
            self.hovered_elements.remove(element_id);
        }

        if let PointerEvent::ButtonJustPressed { .. } = event {
            let mut clicked_off_elements: SmallVec<[ElementID; 4]> = SmallVec::new();
            for element_id in self.elements_listening_to_clicked_off.iter() {
                if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                    let clicked_off =
                        if let Some(visible_rect) = element_entry.stack_data.visible_rect {
                            !visible_rect.contains(pos)
                        } else {
                            true
                        };

                    if clicked_off {
                        clicked_off_elements.push(*element_id);

                        let _ = send_event_to_element(
                            ElementEvent::ClickedOff,
                            element_entry,
                            *element_id,
                            &self.current_focus_info,
                            &mut self.mod_queue_sender,
                            &mut self.action_sender,
                            self.scale_factor,
                            self.logical_size,
                            &mut self.cursor_icon,
                            font_system,
                            clipboard,
                        );
                    }
                }
            }
            for element_id in clicked_off_elements.iter() {
                self.elements_listening_to_clicked_off.remove(element_id);
            }
        }

        let mut send_pointer_event = |element_entry: &mut ElementEntry<A>,
                                      element_id: ElementID,
                                      event: PointerEvent,
                                      did_just_enter: bool|
         -> EventCaptureStatus {
            let mut event = event.clone();
            if let PointerEvent::Moved { just_entered, .. } = &mut event {
                *just_entered = did_just_enter;
            }

            send_event_to_element(
                ElementEvent::Pointer(event),
                element_entry,
                element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            )
        };

        // Focused elements get first priority.
        if let Some(focused_data) = &self.current_focus_info {
            if focused_data.listens_to_pointer_inside_bounds
                || focused_data.listens_to_pointer_outside_bounds
            {
                let element_entry = self
                    .element_arena
                    .get_mut(focused_data.element_id.0)
                    .unwrap();

                if let Some(visible_rect) = element_entry.stack_data.visible_rect {
                    let in_bounds = visible_rect.contains(pos);

                    let send_event = if focused_data.listens_to_pointer_outside_bounds {
                        true
                    } else {
                        in_bounds
                    };

                    let mut did_just_enter = false;
                    if in_bounds {
                        self.hovered_elements
                            .entry(focused_data.element_id)
                            .or_insert_with(|| {
                                did_just_enter = false;
                                None
                            });
                    }

                    if send_event {
                        let capture_status = send_pointer_event(
                            element_entry,
                            focused_data.element_id,
                            event.clone(),
                            did_just_enter,
                        );

                        if let EventCaptureStatus::Captured = capture_status {
                            return EventCaptureStatus::Captured;
                        }
                    }
                }
            }
        }

        if self.elements_listening_to_pointer_event_need_sorted {
            self.elements_listening_to_pointer_event_need_sorted = false;
            self.elements_listening_to_pointer_event
                .sort_unstable_by(|a, b| a.z_index.cmp(&b.z_index));

            for (i, cache) in self.elements_listening_to_pointer_event.iter().enumerate() {
                if let Some(element_entry) = self.element_arena.get_mut(cache.element_id.0) {
                    element_entry.stack_data.index_in_pointer_event_list = i as u32;
                }
            }
        }

        // Iterate z indexes from highest to lowest.
        for cached_rect in self.elements_listening_to_pointer_event.iter().rev() {
            if let Some(visible_rect) = &cached_rect.visible_rect {
                if !visible_rect.contains(pos) {
                    continue;
                }

                let Some(element_entry) = self.element_arena.get_mut(cached_rect.element_id.0)
                else {
                    continue;
                };

                let mut did_just_enter = false;
                self.hovered_elements
                    .entry(cached_rect.element_id)
                    .or_insert_with(|| {
                        did_just_enter = true;
                        None
                    });

                let capture_status = send_pointer_event(
                    element_entry,
                    cached_rect.element_id,
                    event.clone(),
                    did_just_enter,
                );

                if let EventCaptureStatus::Captured = capture_status {
                    return EventCaptureStatus::Captured;
                }
            }
        }

        EventCaptureStatus::NotCaptured
    }

    fn handle_keyboard_event(
        &mut self,
        event: &KeyboardEvent,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) -> EventCaptureStatus {
        if let Some(focused_data) = &self.current_focus_info {
            if focused_data.listens_to_keys {
                let element_entry = self
                    .element_arena
                    .get_mut(focused_data.element_id.0)
                    .unwrap();

                let capture_satus = send_event_to_element(
                    ElementEvent::Keyboard(event.clone()),
                    element_entry,
                    focused_data.element_id,
                    &self.current_focus_info,
                    &mut self.mod_queue_sender,
                    &mut self.action_sender,
                    self.scale_factor,
                    self.logical_size,
                    &mut self.cursor_icon,
                    font_system,
                    clipboard,
                );

                if let EventCaptureStatus::Captured = capture_satus {
                    return EventCaptureStatus::Captured;
                }
            }
        }

        EventCaptureStatus::NotCaptured
    }

    fn handle_text_composition_event(
        &mut self,
        event: &CompositionEvent,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) -> EventCaptureStatus {
        if let Some(focused_data) = &self.current_focus_info {
            if focused_data.listens_to_text_composition {
                let element_entry = self
                    .element_arena
                    .get_mut(focused_data.element_id.0)
                    .unwrap();

                let capture_satus = send_event_to_element(
                    ElementEvent::TextComposition(event.clone()),
                    element_entry,
                    focused_data.element_id,
                    &self.current_focus_info,
                    &mut self.mod_queue_sender,
                    &mut self.action_sender,
                    self.scale_factor,
                    self.logical_size,
                    &mut self.cursor_icon,
                    font_system,
                    clipboard,
                );

                if let EventCaptureStatus::Captured = capture_satus {
                    return EventCaptureStatus::Captured;
                }
            }
        }

        EventCaptureStatus::NotCaptured
    }

    /// Returns `true` if any updates were processed.
    pub fn process_updates(
        &mut self,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) -> bool {
        let mut processed_update = false;
        while let Some(modification) = self.mod_queue_receiver.try_recv() {
            processed_update = true;
            match modification.type_ {
                ElementModificationType::CustomStateChanged => {
                    self.handle_element_custom_state_changed(
                        modification.element_id,
                        font_system,
                        clipboard,
                    );
                }
                ElementModificationType::MarkDirty => {
                    self.mark_element_dirty(modification.element_id);
                }
                ElementModificationType::RectChanged(new_rect) => {
                    self.update_element_rect(
                        modification.element_id,
                        new_rect,
                        font_system,
                        clipboard,
                    );
                }
                ElementModificationType::ScissorRectChanged => {
                    self.handle_scissor_rect_changed_for_element(
                        modification.element_id,
                        font_system,
                        clipboard,
                    );
                }
                ElementModificationType::ZIndexChanged(new_z_index) => {
                    self.update_element_z_index(
                        modification.element_id,
                        new_z_index,
                        font_system,
                        clipboard,
                    );
                }
                ElementModificationType::ExplicitlyHiddenChanged(manually_hidden) => {
                    self.update_element_manually_hidden(
                        modification.element_id,
                        manually_hidden,
                        font_system,
                        clipboard,
                    );
                }
                ElementModificationType::SetAnimating(animating) => {
                    self.set_element_animating(modification.element_id, animating);
                }
                ElementModificationType::ChangeFocus(req) => match req {
                    ChangeFocusRequest::StealExclusiveFocus => {
                        self.element_steal_focus(
                            modification.element_id,
                            false,
                            font_system,
                            clipboard,
                        );
                    }
                    ChangeFocusRequest::StealTemporaryFocus => {
                        self.element_steal_focus(
                            modification.element_id,
                            true,
                            font_system,
                            clipboard,
                        );
                    }
                    ChangeFocusRequest::ReleaseFocus => {
                        self.element_release_focus(modification.element_id, font_system, clipboard);
                    }
                },
                ElementModificationType::HandleDropped => {
                    self.drop_element(modification.element_id, font_system, clipboard);
                }
                ElementModificationType::ListenToClickOff => {
                    self.handle_element_listen_to_click_off(modification.element_id);
                }
                ElementModificationType::StartHoverTimeout => {
                    self.handle_element_start_hover_timeout(modification.element_id);
                }
                ElementModificationType::ShowTooltip(info) => {
                    self.handle_element_show_tooltip(modification.element_id, info);
                }
            }
        }

        processed_update
    }

    pub fn view_needs_repaint(&self) -> bool {
        self.view_needs_repaint
    }

    fn handle_element_listen_to_click_off(&mut self, element_id: ElementID) {
        if self.element_arena.contains(element_id.0) {
            self.elements_listening_to_clicked_off.insert(element_id);
        }
    }

    fn handle_element_start_hover_timeout(&mut self, element_id: ElementID) {
        if self.element_arena.contains(element_id.0) {
            if let Some(hover_start_instant) = self.hovered_elements.get_mut(&element_id) {
                *hover_start_instant = Some(Instant::now());
            }
        }
    }

    fn handle_element_show_tooltip(&mut self, element_id: ElementID, info: ElementTooltipInfo) {
        if self.element_arena.get(element_id.0).is_none() {
            // Element has been dropped. Do nothing and return.
            return;
        };

        self.element_with_active_tooltip = Some((element_id, info.element_bounds));

        if let Some(action) = self.show_tooltip_action.as_mut() {
            let info = TooltipInfo {
                message: info.message,
                element_bounds: info.element_bounds,
                align: info.align,
                window_id: self.window_id,
            };

            self.action_sender.send((action)(info)).unwrap();
        }
    }

    fn handle_element_custom_state_changed(
        &mut self,
        element_id: ElementID,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        send_event_to_element(
            ElementEvent::CustomStateChanged,
            element_entry,
            element_id,
            &self.current_focus_info,
            &mut self.mod_queue_sender,
            &mut self.action_sender,
            self.scale_factor,
            self.logical_size,
            &mut self.cursor_icon,
            font_system,
            clipboard,
        );
    }

    fn mark_element_dirty(&mut self, element_id: ElementID) {
        let Some(element_entry) = self.element_arena.get(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        if !element_entry
            .stack_data
            .flags
            .contains(ElementFlags::PAINTS)
            || !element_entry.stack_data.visible()
        {
            return;
        }

        self.painted_elements[element_entry.stack_data.index_in_painted_list as usize].dirty = true;

        self.view_needs_repaint = true;
    }

    fn update_element_rect(
        &mut self,
        element_id: ElementID,
        new_rect: Rect,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        let pos_changed = element_entry.stack_data.pos_relative_to_scissor_rect != new_rect.origin;
        let size_changed = element_entry.stack_data.rect.size != new_rect.size;

        if !(pos_changed || size_changed) {
            return;
        }

        element_entry.stack_data.pos_relative_to_scissor_rect = new_rect.origin;
        element_entry.stack_data.rect.size = new_rect.size;
        element_entry.stack_data.update_layout(&self.scissor_rects);

        let old_visibility = element_entry.stack_data.visible();
        element_entry
            .stack_data
            .update_visibility(&self.scissor_rects, self.window_visible);
        let visibility_changed = element_entry.stack_data.visible() != old_visibility;

        if visibility_changed && !element_entry.stack_data.visible() {
            release_focus_for_element(
                element_id,
                element_entry,
                &mut self.current_focus_info,
                &mut self.prev_element_with_exclusive_focus,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        if size_changed
            && element_entry
                .stack_data
                .flags
                .contains(ElementFlags::LISTENS_TO_SIZE_CHANGE)
        {
            send_event_to_element(
                ElementEvent::SizeChanged,
                element_entry,
                element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        if pos_changed
            && element_entry
                .stack_data
                .flags
                .contains(ElementFlags::LISTENS_TO_POSITION_CHANGE)
        {
            send_event_to_element(
                ElementEvent::PositionChanged,
                element_entry,
                element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        if visibility_changed
            && element_entry
                .stack_data
                .flags
                .contains(ElementFlags::LISTENS_TO_VISIBILITY_CHANGE)
        {
            let event = if element_entry.stack_data.visible() {
                ElementEvent::Shown
            } else {
                ElementEvent::Hidden
            };

            send_event_to_element(
                event,
                element_entry,
                element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        let mark_dirty = (visibility_changed && element_entry.stack_data.visible()) || size_changed;

        sync_element_rect_cache(
            &element_entry.stack_data,
            &mut self.elements_listening_to_pointer_event,
            &mut self.painted_elements,
            mark_dirty,
        );

        if element_entry.stack_data.visible() || visibility_changed {
            self.view_needs_repaint = true;
        }
    }

    fn handle_scissor_rect_changed_for_element(
        &mut self,
        element_id: ElementID,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        element_entry.stack_data.update_layout(&self.scissor_rects);

        let old_visibility = element_entry.stack_data.visible();
        element_entry
            .stack_data
            .update_visibility(&self.scissor_rects, self.window_visible);
        let visibility_changed = element_entry.stack_data.visible() != old_visibility;

        let mark_dirty = visibility_changed && element_entry.stack_data.visible();

        sync_element_rect_cache(
            &element_entry.stack_data,
            &mut self.elements_listening_to_pointer_event,
            &mut self.painted_elements,
            mark_dirty,
        );

        if visibility_changed && !element_entry.stack_data.visible() {
            release_focus_for_element(
                element_id,
                element_entry,
                &mut self.current_focus_info,
                &mut self.prev_element_with_exclusive_focus,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        if visibility_changed
            && element_entry
                .stack_data
                .flags
                .contains(ElementFlags::LISTENS_TO_VISIBILITY_CHANGE)
        {
            let event = if element_entry.stack_data.visible() {
                ElementEvent::Shown
            } else {
                ElementEvent::Hidden
            };

            send_event_to_element(
                event,
                element_entry,
                element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        if element_entry.stack_data.visible() || visibility_changed {
            self.view_needs_repaint = true;
        }
    }

    fn update_element_z_index(
        &mut self,
        element_id: ElementID,
        new_z_index: ZIndex,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        if element_entry.stack_data.z_index == new_z_index {
            return;
        }
        element_entry.stack_data.z_index = new_z_index;

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
        {
            self.elements_listening_to_pointer_event
                [element_entry.stack_data.index_in_pointer_event_list as usize]
                .z_index = new_z_index;
            self.elements_listening_to_pointer_event_need_sorted = true;
        }

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::PAINTS)
        {
            self.painted_elements[element_entry.stack_data.index_in_painted_list as usize]
                .z_index = new_z_index;
        }

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::LISTENS_TO_Z_INDEX_CHANGE)
        {
            send_event_to_element(
                ElementEvent::ZIndexChanged,
                element_entry,
                element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        // Detecting if a z index change requires a repaint or not would be very tricky,
        // so just repaint regardless if the element is visible.
        if element_entry.stack_data.visible() {
            self.view_needs_repaint = true;
        }
    }

    fn update_element_manually_hidden(
        &mut self,
        element_id: ElementID,
        manually_hidden: bool,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        if element_entry.stack_data.manually_hidden == manually_hidden {
            return;
        }

        element_entry.stack_data.manually_hidden = manually_hidden;

        let old_visibility = element_entry.stack_data.visible();
        element_entry
            .stack_data
            .update_visibility(&self.scissor_rects, self.window_visible);
        let visibility_changed = element_entry.stack_data.visible() != old_visibility;

        if !visibility_changed {
            return;
        }

        let mark_dirty = visibility_changed && element_entry.stack_data.visible();

        sync_element_rect_cache(
            &element_entry.stack_data,
            &mut self.elements_listening_to_pointer_event,
            &mut self.painted_elements,
            mark_dirty,
        );

        if visibility_changed && !element_entry.stack_data.visible() {
            release_focus_for_element(
                element_id,
                element_entry,
                &mut self.current_focus_info,
                &mut self.prev_element_with_exclusive_focus,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::LISTENS_TO_VISIBILITY_CHANGE)
        {
            let event = if element_entry.stack_data.visible() {
                ElementEvent::Shown
            } else {
                ElementEvent::Hidden
            };

            send_event_to_element(
                event,
                element_entry,
                element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }

        self.view_needs_repaint = true;
    }

    fn set_element_animating(&mut self, element_id: ElementID, animating: bool) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        if element_entry.stack_data.animating == animating {
            return;
        }

        element_entry.stack_data.animating = animating;

        if animating {
            element_entry.stack_data.index_in_animating_list = self.animating_elements.len() as u32;
            self.animating_elements.push(element_id);
        } else {
            let _ = self
                .animating_elements
                .swap_remove(element_entry.stack_data.index_in_animating_list as usize);

            // Update the index on the element that was swapped.
            if let Some(swapped_element_id) = self
                .animating_elements
                .get(element_entry.stack_data.index_in_animating_list as usize)
                .copied()
            {
                self.element_arena
                    .get_mut(swapped_element_id.0)
                    .as_mut()
                    .unwrap()
                    .stack_data
                    .index_in_animating_list = element_entry.stack_data.index_in_animating_list;
            }
        }
    }

    fn element_steal_focus(
        &mut self,
        element_id: ElementID,
        is_temporary: bool,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) {
        if self.element_arena.get(element_id.0).is_none() {
            // Element has been dropped. Do nothing and return.
            return;
        }

        if let Some(focus_info) = &self.current_focus_info {
            if focus_info.element_id == element_id {
                // The element is already focused.
                return;
            }
        }

        for id in self.elements_listening_to_clicked_off.iter() {
            if let Some(element_entry) = self.element_arena.get_mut(id.0) {
                send_event_to_element(
                    ElementEvent::ClickedOff,
                    element_entry,
                    *id,
                    &self.current_focus_info,
                    &mut self.mod_queue_sender,
                    &mut self.action_sender,
                    self.scale_factor,
                    self.logical_size,
                    &mut self.cursor_icon,
                    font_system,
                    clipboard,
                );
            }
        }
        self.elements_listening_to_clicked_off.clear();

        let prev_element_with_exclusive_focus = self.prev_element_with_exclusive_focus.take();

        // Release focus from the previously focused element.
        if let Some(info) = &self.current_focus_info {
            self.element_release_focus(info.element_id, font_system, clipboard);
        }

        let element_entry = self.element_arena.get_mut(element_id.0).unwrap();

        self.current_focus_info = Some(FocusInfo {
            element_id,
            listens_to_pointer_inside_bounds: element_entry
                .stack_data
                .flags
                .contains(ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS),
            listens_to_pointer_outside_bounds: element_entry
                .stack_data
                .flags
                .contains(ElementFlags::LISTENS_TO_POINTER_OUTSIDE_BOUNDS_WHEN_FOCUSED),
            listens_to_text_composition: element_entry
                .stack_data
                .flags
                .contains(ElementFlags::LISTENS_TO_TEXT_COMPOSITION_WHEN_FOCUSED),
            listens_to_keys: element_entry
                .stack_data
                .flags
                .contains(ElementFlags::LISTENS_TO_KEYS_WHEN_FOCUSED),
        });

        self.prev_element_with_exclusive_focus = if is_temporary {
            prev_element_with_exclusive_focus
        } else {
            Some(element_id)
        };

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::LISTENS_TO_FOCUS_CHANGE)
        {
            send_event_to_element(
                ElementEvent::ExclusiveFocus(true),
                element_entry,
                element_id,
                &self.current_focus_info,
                &mut self.mod_queue_sender,
                &mut self.action_sender,
                self.scale_factor,
                self.logical_size,
                &mut self.cursor_icon,
                font_system,
                clipboard,
            );
        }
    }

    fn element_release_focus(
        &mut self,
        element_id: ElementID,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        release_focus_for_element(
            element_id,
            element_entry,
            &mut self.current_focus_info,
            &mut self.prev_element_with_exclusive_focus,
            &mut self.mod_queue_sender,
            &mut self.action_sender,
            self.scale_factor,
            self.logical_size,
            &mut self.cursor_icon,
            font_system,
            clipboard,
        );
    }

    fn drop_element(
        &mut self,
        element_id: ElementID,
        font_system: &mut FontSystem,
        clipboard: &mut Clipboard,
    ) {
        if let Some(focus_info) = &self.current_focus_info {
            if focus_info.element_id == element_id {
                self.element_release_focus(element_id, font_system, clipboard);
            }
        }

        let Some(mut element_entry) = self.element_arena.remove(element_id.0) else {
            // Element has already been dropped. Do nothing and return.
            return;
        };

        release_focus_for_element(
            element_id,
            &mut element_entry,
            &mut self.current_focus_info,
            &mut self.prev_element_with_exclusive_focus,
            &mut self.mod_queue_sender,
            &mut self.action_sender,
            self.scale_factor,
            self.logical_size,
            &mut self.cursor_icon,
            font_system,
            clipboard,
        );

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::LISTENS_TO_ON_DROPPED)
        {
            element_entry.element.on_dropped(&mut self.action_sender);
        }

        if element_entry.stack_data.animating {
            let _ = self
                .animating_elements
                .swap_remove(element_entry.stack_data.index_in_animating_list as usize);

            // Update the index on the element that was swapped.
            if let Some(swapped_element_id) = self
                .animating_elements
                .get(element_entry.stack_data.index_in_animating_list as usize)
                .copied()
            {
                self.element_arena
                    .get_mut(swapped_element_id.0)
                    .as_mut()
                    .unwrap()
                    .stack_data
                    .index_in_animating_list = element_entry.stack_data.index_in_animating_list;
            }
        }

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
        {
            let _ = self
                .elements_listening_to_pointer_event
                .swap_remove(element_entry.stack_data.index_in_pointer_event_list as usize);

            // Update the index on the element that was swapped.
            if let Some(swapped_element_id) = self
                .elements_listening_to_pointer_event
                .get(element_entry.stack_data.index_in_pointer_event_list as usize)
                .map(|cache| cache.element_id)
            {
                self.element_arena
                    .get_mut(swapped_element_id.0)
                    .as_mut()
                    .unwrap()
                    .stack_data
                    .index_in_pointer_event_list =
                    element_entry.stack_data.index_in_pointer_event_list;
            }

            self.elements_listening_to_pointer_event_need_sorted = true;
        }

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::PAINTS)
        {
            let _ = self
                .painted_elements
                .swap_remove(element_entry.stack_data.index_in_painted_list as usize);

            // Update the index on the element that was swapped.
            if let Some(swapped_element_id) = self
                .painted_elements
                .get(element_entry.stack_data.index_in_painted_list as usize)
                .map(|cache| cache.element_id)
            {
                self.element_arena
                    .get_mut(swapped_element_id.0)
                    .as_mut()
                    .unwrap()
                    .stack_data
                    .index_in_painted_list = element_entry.stack_data.index_in_painted_list;
            }
        }

        self.scissor_rects[usize::from(element_entry.stack_data.scissor_rect_id)]
            .remove_element(&element_entry.stack_data, &mut self.element_arena);

        if element_entry.stack_data.visible() {
            self.view_needs_repaint = true;
        }
    }

    pub fn render<P: FnOnce()>(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vg: &mut rootvg::Canvas,
        pre_present_notify: P,
        font_system: &mut FontSystem,
    ) -> Result<(), wgpu::SurfaceError> {
        if !self.view_needs_repaint {
            return Ok(());
        }

        // Set up the frame and wgpu encoder.
        let frame = surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut vg = vg.begin(self.physical_size, self.scale_factor);

            for cache in self.painted_elements.iter_mut() {
                if !cache.visible {
                    continue;
                }

                if cache.dirty {
                    cache.dirty = false;

                    cache.primitives.clear();

                    let element_entry = self.element_arena.get_mut(cache.element_id.0).unwrap();

                    element_entry.element.render_primitives(
                        RenderContext {
                            font_system,
                            bounds_size: element_entry.stack_data.rect.size,
                            bounds_origin: element_entry.stack_data.rect.origin,
                            visible_bounds: element_entry.stack_data.visible_rect.unwrap(),
                            scale: self.scale_factor,
                            window_size: self.logical_size,
                        },
                        &mut cache.primitives,
                    );
                }

                vg.set_z_index(cache.z_index);
                vg.set_scissor_rect(self.scissor_rects[cache.scissor_rect_id as usize].rect());
                vg.add_group_with_offset(&cache.primitives, cache.offset);
            }
        }

        // Render the view to the target texture.
        vg.render_to_target(
            Some(self.clear_color),
            device,
            queue,
            &mut encoder,
            &view,
            self.physical_size,
            font_system,
        )
        .unwrap(); // TODO: handle this error properly.

        pre_present_notify();

        // Submit the commands and present the frame.
        queue.submit(Some(encoder.finish()));
        frame.present();

        self.view_needs_repaint = false;

        Ok(())
    }
}

struct ElementEntry<A: Clone + 'static> {
    pub stack_data: EntryStackData,
    pub element: Box<dyn Element<A>>,
}

// Ideally the size of this struct should be as small as possible to
// maximize cache locality when accessing entries at random from the
// arena.
//
// Since an application is probably never going to have more than 4
// billion elements in the same layer anyway, I've opted to use u32
// for indexes.
#[derive(Clone)]
struct EntryStackData {
    rect: Rect,
    visible_rect: Option<Rect>,
    pos_relative_to_scissor_rect: Point,

    scissor_rect_id: ScissorRectID,
    z_index: ZIndex,

    flags: ElementFlags,
    manually_hidden: bool,
    animating: bool,

    index_in_pointer_event_list: u32,
    index_in_painted_list: u32,
    index_in_animating_list: u32,
    index_in_scissor_rect_list: u32,
}

impl EntryStackData {
    #[inline]
    fn update_layout(&mut self, scissor_rects: &[ScissorRect]) {
        let scissor_rect = &scissor_rects[self.scissor_rect_id as usize];
        let scissor_rect_origin: Point = scissor_rect.origin().cast();

        self.rect.origin = scissor_rect_origin + self.pos_relative_to_scissor_rect.to_vector()
            - scissor_rect.scroll_offset().to_vector();
    }

    fn update_visibility(&mut self, scissor_rects: &[ScissorRect], window_visible: bool) {
        self.visible_rect = if self.manually_hidden
            || self.rect.size.width <= 0.0
            || self.rect.size.height <= 0.0
            || !window_visible
        {
            None
        } else {
            let scissor_rect: Rect = scissor_rects[self.scissor_rect_id as usize].rect().cast();
            scissor_rect.intersection(&self.rect)
        };
    }

    fn visible(&self) -> bool {
        self.visible_rect.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TooltipInfo {
    pub message: String,
    pub element_bounds: Rect,
    pub align: Align2,
    pub window_id: WindowID,
}

struct FocusInfo {
    element_id: ElementID,
    listens_to_pointer_inside_bounds: bool,
    listens_to_pointer_outside_bounds: bool,
    listens_to_text_composition: bool,
    listens_to_keys: bool,
}

fn send_event_to_element<A: Clone + 'static>(
    event: ElementEvent,
    element_entry: &mut ElementEntry<A>,
    element_id: ElementID,
    focus_info: &Option<FocusInfo>,
    mod_queue_sender: &mut stmpsc_queue::Sender<ElementModification>,
    action_sender: &mut ActionSender<A>,
    scale_factor: ScaleFactor,
    window_size: Size,
    cursor_icon: &mut CursorIcon,
    font_system: &mut FontSystem,
    clipboard: &mut Clipboard,
) -> EventCaptureStatus {
    let has_focus = focus_info
        .as_ref()
        .map(|d| d.element_id == element_id)
        .unwrap_or(false);

    let mut cx = ElementContext::new(
        element_entry.stack_data.rect,
        element_entry.stack_data.visible_rect,
        window_size,
        element_entry.stack_data.z_index,
        element_entry.stack_data.manually_hidden,
        element_entry.stack_data.animating,
        has_focus,
        scale_factor,
        *cursor_icon,
        action_sender,
        font_system,
        clipboard,
    );

    let capture_status = element_entry.element.on_event(event, &mut cx);

    *cursor_icon = cx.cursor_icon;

    if cx.listen_to_pointer_clicked_off {
        mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::ListenToClickOff,
        });
    }

    if let Some(req) = cx.change_focus_request {
        let do_send = match req {
            ChangeFocusRequest::ReleaseFocus => has_focus,
            _ => !has_focus,
        };

        if do_send {
            mod_queue_sender.send_to_front(ElementModification {
                element_id,
                type_: ElementModificationType::ChangeFocus(req),
            });
        }
    }

    if cx.is_animating() != element_entry.stack_data.animating {
        mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::SetAnimating(cx.is_animating()),
        });
    }

    if let Some(new_rect) = cx.requested_rect {
        mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::RectChanged(new_rect),
        });
    }

    if cx.repaint_requested() {
        mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::MarkDirty,
        });
    }

    if cx.hover_timeout_requested() {
        mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::StartHoverTimeout,
        });
    }

    if let Some(info) = cx.requested_show_tooltip {
        mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::ShowTooltip(info),
        });
    }

    capture_status
}

fn release_focus_for_element<A: Clone + 'static>(
    element_id: ElementID,
    element_entry: &mut ElementEntry<A>,
    current_focus_info: &mut Option<FocusInfo>,
    prev_element_with_exclusive_focus: &mut Option<ElementID>,
    mod_queue_sender: &mut stmpsc_queue::Sender<ElementModification>,
    action_sender: &mut ActionSender<A>,
    scale_factor: ScaleFactor,
    window_size: Size,
    cursor_icon: &mut CursorIcon,
    font_system: &mut FontSystem,
    clipboard: &mut Clipboard,
) {
    if let Some(info) = &current_focus_info {
        if info.element_id != element_id {
            return;
        }
    } else {
        return;
    };

    *current_focus_info = None;

    if element_entry
        .stack_data
        .flags
        .contains(ElementFlags::LISTENS_TO_FOCUS_CHANGE)
    {
        send_event_to_element(
            ElementEvent::ExclusiveFocus(false),
            element_entry,
            element_id,
            current_focus_info,
            mod_queue_sender,
            action_sender,
            scale_factor,
            window_size,
            cursor_icon,
            font_system,
            clipboard,
        );
    }

    if let Some(prev_element_id) = prev_element_with_exclusive_focus.take() {
        if prev_element_id != element_id {
            mod_queue_sender.send_to_front(ElementModification {
                element_id: prev_element_id,
                type_: ElementModificationType::ChangeFocus(
                    ChangeFocusRequest::StealExclusiveFocus,
                ),
            });
        }
    }
}
