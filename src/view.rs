// ---------------------------------------------------------------------------------
//
//    '%%' '%% '%%'
//    %'%\% | %/%'%     Yarrow GUI Library
//        \ | /
//         \|/          https://github.com/MeadowlarkDAW/Yarrow
//          |
//
//
// MIT License Copyright (c) 2024 Billy Messenger
// https://github.com/MeadowlarkDAW/Yarrow/blob/main/LICENSE
//
// ---------------------------------------------------------------------------------

use std::time::Duration;
use std::time::Instant;

use element::ElementRenderCache;
use keyboard_types::CompositionEvent;
use rootvg::color::PackedSrgb;
use rootvg::math::PhysicalSizeI32;
use rootvg::math::SizeI32;
use rootvg::math::Vector;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;
use thunderdome::Arena;

use crate::action_queue::ActionSender;
use crate::clipboard::Clipboard;
use crate::event::{CanvasEvent, ElementEvent, EventCaptureStatus, KeyboardEvent, PointerEvent};
use crate::layout::Align2;
use crate::math::{Point, PointI32, Rect, RectI32, ScaleFactor, Size, ZIndex};
use crate::prelude::{ClassID, ResourceCtx};
use crate::stmpsc_queue;
use crate::CursorIcon;
use crate::WindowID;

mod cache;
pub mod element;
mod scissor_rect;

use self::element::ChangeFocusRequest;
use self::element::RenderContext;
pub use self::scissor_rect::ScissorRectID;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    pub scroll_wheel_timeout_duration: Duration,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            clear_color: PackedSrgb::BLACK,
            preallocate_for_this_many_elements: 0,
            hover_timeout_duration: Duration::from_millis(500),
            scroll_wheel_timeout_duration: Duration::from_millis(250),
        }
    }
}

struct ViewContext<A: Clone + 'static> {
    current_focus_info: Option<FocusInfo>,
    prev_element_with_exclusive_focus: Option<ElementID>,
    mod_queue_sender: stmpsc_queue::Sender<ElementModification>,
    action_sender: ActionSender<A>,
    scale_factor: ScaleFactor,
    logical_size: Size,
    cursor_icon: CursorIcon,
    pointer_lock_request: Option<bool>,
    pointer_locked: bool,
    window_id: WindowID,
}

pub struct View<A: Clone + 'static> {
    pub clear_color: PackedSrgb,

    context: ViewContext<A>,

    element_arena: Arena<ElementEntry<A>>,
    scissor_rect_id_to_index_map: FxHashMap<ScissorRectID, usize>,
    scissor_rects: Vec<ScissorRect>,

    mod_queue_receiver: stmpsc_queue::Receiver<ElementModification>,

    hovered_elements: FxHashMap<ElementID, Option<Instant>>,
    elements_with_scroll_wheel_timeout: FxHashMap<ElementID, Option<Instant>>,
    animating_elements: Vec<ElementID>,

    elements_listening_to_pointer_event: Vec<CachedElementRectForPointerEvent>,
    elements_listening_to_pointer_event_need_sorted: bool,
    painted_elements: Vec<CachedElementPrimitives>,
    elements_listening_to_clicked_off: FxHashSet<ElementID>,
    element_with_active_tooltip: Option<ActiveTooltipInfo>,

    physical_size: PhysicalSizeI32,
    hover_timeout_duration: Duration,
    scroll_wheel_timeout_duration: Duration,
    prev_pointer_pos: Option<Point>,

    show_tooltip_action: Option<Box<dyn FnMut(TooltipInfo) -> A>>,
    hide_tooltip_action: Option<Box<dyn FnMut() -> A>>,

    view_needs_repaint: bool,
    window_visible: bool,

    render_caches: FxHashMap<u32, Box<dyn ElementRenderCache>>,
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
            scroll_wheel_timeout_duration,
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

        let scissor_rects = vec![ScissorRect::new(view_rect, Vector::default())];
        let mut scissor_rect_id_to_index_map = FxHashMap::<ScissorRectID, usize>::default();
        scissor_rect_id_to_index_map.insert(ScissorRectID::DEFAULT, 0);

        let capacity = preallocate_for_this_many_elements as usize;

        let (mod_queue_sender, mod_queue_receiver) = stmpsc_queue::single_thread_mpsc_queue(
            // Give some wiggle-room since elements can be added to the queue more than once.
            capacity * 4,
        );

        Self {
            clear_color,

            context: ViewContext {
                current_focus_info: None,
                prev_element_with_exclusive_focus: None,
                mod_queue_sender,
                action_sender,
                scale_factor,
                logical_size,
                cursor_icon: CursorIcon::Default,
                pointer_lock_request: None,
                pointer_locked: false,
                window_id,
            },

            element_arena: Arena::with_capacity(capacity),
            scissor_rect_id_to_index_map,
            scissor_rects,

            mod_queue_receiver,

            hovered_elements: FxHashMap::default(),
            elements_with_scroll_wheel_timeout: FxHashMap::default(),
            animating_elements: Vec::with_capacity(capacity),

            elements_listening_to_pointer_event: Vec::new(),
            elements_listening_to_pointer_event_need_sorted: false,
            painted_elements: Vec::new(),
            elements_listening_to_clicked_off: FxHashSet::default(),
            element_with_active_tooltip: None,

            physical_size,
            hover_timeout_duration,
            scroll_wheel_timeout_duration,
            prev_pointer_pos: None,

            view_needs_repaint: true,
            window_visible: true,

            show_tooltip_action: None,
            hide_tooltip_action: None,

            render_caches: FxHashMap::default(),
        }
    }

    pub fn size(&self) -> Size {
        self.context.logical_size
    }

    pub fn set_tooltip_actions<S, H>(&mut self, on_show_tooltip: S, on_hide_tooltip: H)
    where
        S: FnMut(TooltipInfo) -> A + 'static,
        H: FnMut() -> A + 'static,
    {
        self.show_tooltip_action = Some(Box::new(on_show_tooltip));
        self.hide_tooltip_action = Some(Box::new(on_hide_tooltip));
    }

    /// Get the current rectangle of the given scissoring rectangle.
    ///
    /// If a scissoring rectangle with the given ID does not exist, then
    /// one will be created.
    pub fn scissor_rect(&mut self, scissor_rect_id: ScissorRectID) -> RectI32 {
        let i = self.get_scissor_rect_index(scissor_rect_id);
        self.scissor_rects[i].rect()
    }

    /// Get the current scroll offset vector of the given scissoring rectangle.
    ///
    /// If a scissoring rectangle with the given ID does not exist, then
    /// one will be created.
    pub fn scissor_rect_scroll_offset(&mut self, scissor_rect_id: ScissorRectID) -> Vector {
        let i = self.get_scissor_rect_index(scissor_rect_id);
        self.scissor_rects[i].scroll_offset()
    }

    /// Update the given scissoring rectangle with the given values.
    ///
    /// If `new_rect` or `new_scroll_offset` is `None`, then the
    /// current respecting value will not be changed.
    ///
    /// This will *NOT* trigger an update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    ///
    /// If a scissoring rectangle with the given ID does not exist, then
    /// one will be created.
    ///
    /// If `scissor_rect_id == ScissorRectID::DEFAULT`, then this
    /// will do nothing.
    pub fn update_scissor_rect(
        &mut self,
        scissor_rect_id: ScissorRectID,
        new_rect: Option<Rect>,
        new_scroll_offset: Option<Vector>,
    ) {
        if scissor_rect_id == ScissorRectID::DEFAULT {
            return;
        }

        let new_rect: Option<RectI32> = new_rect.map(|r| r.round().cast());

        let i = self.get_scissor_rect_index(scissor_rect_id);

        self.scissor_rects[i].update(
            new_rect,
            new_scroll_offset,
            &mut self.context.mod_queue_sender,
        );
    }

    pub fn add_element(
        &mut self,
        element_builder: ElementBuilder<A>,
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) -> ElementHandle {
        let ElementBuilder {
            element,
            z_index,
            rect,
            manually_hidden,
            scissor_rect_id,
            class,
        } = element_builder;

        let flags = element.flags();

        let scissor_rect_index = self.get_scissor_rect_index(scissor_rect_id);

        let mut stack_data = EntryStackData {
            rect,
            visible_rect: None,
            offset_from_scissor_rect_origin: rect.origin.to_vector(),
            scissor_rect_index,
            z_index,
            flags,
            manually_hidden,
            class,
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

        self.scissor_rects[scissor_rect_index]
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
                element_entry.stack_data.rect.origin.to_vector(),
                element_entry.stack_data.z_index,
                element_entry.stack_data.scissor_rect_index,
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
                &mut self.context,
                res,
                clipboard,
            );
        }

        if let Some(render_cache_id) = element_entry.element.global_render_cache_id() {
            if !self.render_caches.contains_key(&render_cache_id) {
                if let Some(render_cache) = element_entry.element.global_render_cache() {
                    self.render_caches.insert(render_cache_id, render_cache);
                }
            }
        }

        self::element::new_element_handle(
            element_id,
            self.context.mod_queue_sender.clone(),
            rect,
            z_index,
            manually_hidden,
            class,
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

    pub fn auto_hide_tooltip(&mut self) {
        if let Some(info) = &mut self.element_with_active_tooltip {
            info.auto_hide = true;
        }
    }

    pub(crate) fn on_pointer_locked(&mut self, locked: bool) {
        self.context.pointer_locked = locked;
        self.context.pointer_lock_request = None;
    }

    pub(crate) fn resize(&mut self, physical_size: PhysicalSizeI32, scale_factor: ScaleFactor) {
        self.physical_size = physical_size;
        self.context.scale_factor = scale_factor;
        self.context.logical_size = crate::math::to_logical_size_i32(physical_size, scale_factor);

        self.scissor_rects[0].update(
            Some(RectI32::new(
                PointI32::default(),
                SizeI32::new(
                    self.context.logical_size.width.round() as i32,
                    self.context.logical_size.height.round() as i32,
                ),
            )),
            None,
            &mut self.context.mod_queue_sender,
        );

        self.view_needs_repaint = true;
    }

    pub(crate) fn on_theme_changed(&mut self, res: &mut ResourceCtx, clipboard: &mut Clipboard) {
        let mut element_ids = Vec::new();
        for (element_id, element_entry) in self.element_arena.iter_mut() {
            let element_id = ElementID(element_id);

            send_event_to_element(
                ElementEvent::StyleChanged,
                element_entry,
                element_id,
                &mut self.context,
                res,
                clipboard,
            );

            element_ids.push(element_id);
        }

        for element_id in element_ids.iter().copied() {
            self.mark_element_dirty(element_id);
        }
    }

    pub(crate) fn handle_event(
        &mut self,
        event: &CanvasEvent,
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) -> EventCaptureStatus {
        match event {
            CanvasEvent::Animation {
                delta_seconds,
                pointer_position,
            } => {
                self.handle_animation_event(*delta_seconds, *pointer_position, res, clipboard);

                // Capture status is not relavant for this event.
                EventCaptureStatus::NotCaptured
            }
            CanvasEvent::Pointer(pointer_event) => {
                self.handle_pointer_event(pointer_event, res, clipboard)
            }
            CanvasEvent::Keyboard(keyboard_event) => {
                self.handle_keyboard_event(keyboard_event, res, clipboard)
            }
            CanvasEvent::TextComposition(text_composition_event) => {
                self.handle_text_composition_event(text_composition_event, res, clipboard)
            }
            CanvasEvent::WindowHidden => {
                self.handle_window_hidden(res, clipboard);
                EventCaptureStatus::NotCaptured
            }
            CanvasEvent::WindowShown => {
                self.handle_window_shown(res, clipboard);
                EventCaptureStatus::NotCaptured
            }
            CanvasEvent::WindowFocused => {
                // TODO
                EventCaptureStatus::NotCaptured
            }
            CanvasEvent::WindowUnfocused => {
                self.handle_window_unfocused(res, clipboard);
                EventCaptureStatus::NotCaptured
            }
        }
    }

    fn get_scissor_rect_index(&mut self, scissor_rect_id: ScissorRectID) -> usize {
        *self
            .scissor_rect_id_to_index_map
            .entry(scissor_rect_id)
            .or_insert_with(|| {
                let i = self.scissor_rects.len();
                self.scissor_rects
                    .push(ScissorRect::new(RectI32::default(), Vector::default()));
                i
            })
    }

    fn handle_window_shown(&mut self, res: &mut ResourceCtx, clipboard: &mut Clipboard) {
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
                            &mut self.context,
                            res,
                            clipboard,
                        );
                    }
                }
            }
        }
    }

    fn handle_window_hidden(&mut self, res: &mut ResourceCtx, clipboard: &mut Clipboard) {
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
                        &mut self.context,
                        res,
                        clipboard,
                    );
                }
            }
        }

        if let Some(_) = self.element_with_active_tooltip.take() {
            if let Some(action) = self.hide_tooltip_action.as_mut() {
                self.context.action_sender.send((action)()).unwrap();
            }
        }
    }

    fn handle_window_unfocused(&mut self, res: &mut ResourceCtx, clipboard: &mut Clipboard) {
        for (element_id, _) in self.hovered_elements.iter() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                send_event_to_element(
                    ElementEvent::Pointer(PointerEvent::PointerLeft),
                    element_entry,
                    *element_id,
                    &mut self.context,
                    res,
                    clipboard,
                );
            }
        }
        self.hovered_elements.clear();

        for (element_id, _) in self.elements_with_scroll_wheel_timeout.iter_mut() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                send_event_to_element(
                    ElementEvent::Pointer(PointerEvent::ScrollWheelTimeout),
                    element_entry,
                    *element_id,
                    &mut self.context,
                    res,
                    clipboard,
                );
            }
        }
        self.elements_with_scroll_wheel_timeout.clear();

        for element_id in self.elements_listening_to_clicked_off.iter() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                send_event_to_element(
                    ElementEvent::ClickedOff,
                    element_entry,
                    *element_id,
                    &mut self.context,
                    res,
                    clipboard,
                );
            }
        }
        self.elements_listening_to_clicked_off.clear();

        if let Some(_) = self.element_with_active_tooltip.take() {
            if let Some(action) = self.hide_tooltip_action.as_mut() {
                self.context.action_sender.send((action)()).unwrap();
            }
        }

        self.prev_pointer_pos = None;

        // TODO: Release exclusive focus if the pointer is locked.
    }

    fn handle_animation_event(
        &mut self,
        delta_seconds: f64,
        pointer_position: Option<Point>,
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) {
        for element_id in self.animating_elements.iter() {
            let element_entry = self.element_arena.get_mut(element_id.0).unwrap();

            let _ = send_event_to_element(
                ElementEvent::Animation { delta_seconds },
                element_entry,
                *element_id,
                &mut self.context,
                res,
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
                                    &mut self.context,
                                    res,
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

        for (element_id, start_instant) in self.elements_with_scroll_wheel_timeout.iter_mut() {
            if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                if let Some(instant) = start_instant.take() {
                    if instant.elapsed() >= self.scroll_wheel_timeout_duration {
                        send_event_to_element(
                            ElementEvent::Pointer(PointerEvent::ScrollWheelTimeout),
                            element_entry,
                            *element_id,
                            &mut self.context,
                            res,
                            clipboard,
                        );
                    } else {
                        *start_instant = Some(instant);
                    }
                }
            }
        }

        if let Some(info) = self.element_with_active_tooltip {
            let mut hide_tooltip = true;

            if info.auto_hide {
                if let Some(element_entry) = self.element_arena.get(info.element_id.0) {
                    if let Some(pos) = self.prev_pointer_pos {
                        if let Some(visible_rect) = &element_entry.stack_data.visible_rect {
                            hide_tooltip = !visible_rect.contains(pos);
                        }
                    }
                }
            } else {
                hide_tooltip = false;
            }

            if hide_tooltip {
                self.element_with_active_tooltip = None;
                if let Some(action) = self.hide_tooltip_action.as_mut() {
                    self.context.action_sender.send((action)()).unwrap();
                }
            }
        }
    }

    fn handle_pointer_event(
        &mut self,
        event: &PointerEvent,
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) -> EventCaptureStatus {
        let pos = event.position();

        match event {
            PointerEvent::Moved { .. } => {
                self.context.cursor_icon = CursorIcon::Default;

                if let Some(info) = self.element_with_active_tooltip {
                    if info.auto_hide {
                        let mut hide_tooltip = true;

                        if let Some(element_entry) = self.element_arena.get(info.element_id.0) {
                            if let Some(visible_rect) = element_entry.stack_data.visible_rect {
                                hide_tooltip = !visible_rect.contains(pos);
                            }
                        }

                        if hide_tooltip {
                            self.element_with_active_tooltip = None;
                            if let Some(action) = self.hide_tooltip_action.as_mut() {
                                self.context.action_sender.send((action)()).unwrap();
                            }
                        }
                    }
                }

                self.prev_pointer_pos = Some(pos);
            }
            PointerEvent::PointerLeft => {
                for (element_id, _) in self.hovered_elements.iter_mut() {
                    if let Some(element_entry) = self.element_arena.get_mut(element_id.0) {
                        send_event_to_element(
                            ElementEvent::Pointer(PointerEvent::PointerLeft),
                            element_entry,
                            *element_id,
                            &mut self.context,
                            res,
                            clipboard,
                        );
                    }
                }
                self.hovered_elements.clear();

                if let Some(_) = self.element_with_active_tooltip.take() {
                    if let Some(action) = self.hide_tooltip_action.as_mut() {
                        self.context.action_sender.send((action)()).unwrap();
                    }
                }

                self.prev_pointer_pos = None;

                return EventCaptureStatus::NotCaptured;
            }
            _ => {}
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
                        &mut self.context,
                        res,
                        clipboard,
                    );
                } else if let Some(instant) = hover_start_instant.take() {
                    if instant.elapsed() >= self.hover_timeout_duration {
                        send_event_to_element(
                            ElementEvent::Pointer(PointerEvent::HoverTimeout { position: pos }),
                            element_entry,
                            *element_id,
                            &mut self.context,
                            res,
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
                            &mut self.context,
                            res,
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
                                      did_just_enter: bool,
                                      view_cx: &mut ViewContext<A>|
         -> EventCaptureStatus {
            let mut event = event.clone();

            match &mut event {
                PointerEvent::Moved { just_entered, .. } => {
                    *just_entered = did_just_enter;
                }
                PointerEvent::ScrollWheel { .. } => {
                    if let Some(Some(start_instant)) =
                        self.elements_with_scroll_wheel_timeout.get_mut(&element_id)
                    {
                        *start_instant = Instant::now();
                    }
                }
                _ => {}
            }

            send_event_to_element(
                ElementEvent::Pointer(event),
                element_entry,
                element_id,
                view_cx,
                res,
                clipboard,
            )
        };

        // Focused elements get first priority.
        if let Some(focused_data) = &self.context.current_focus_info {
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
                            &mut self.context,
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
                    &mut self.context,
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
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) -> EventCaptureStatus {
        if let Some(focused_data) = &self.context.current_focus_info {
            if focused_data.listens_to_keys {
                let element_entry = self
                    .element_arena
                    .get_mut(focused_data.element_id.0)
                    .unwrap();

                let capture_satus = send_event_to_element(
                    ElementEvent::Keyboard(event.clone()),
                    element_entry,
                    focused_data.element_id,
                    &mut self.context,
                    res,
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
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) -> EventCaptureStatus {
        if let Some(focused_data) = &self.context.current_focus_info {
            if focused_data.listens_to_text_composition {
                let element_entry = self
                    .element_arena
                    .get_mut(focused_data.element_id.0)
                    .unwrap();

                let capture_satus = send_event_to_element(
                    ElementEvent::TextComposition(event.clone()),
                    element_entry,
                    focused_data.element_id,
                    &mut self.context,
                    res,
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
    pub fn process_updates(&mut self, res: &mut ResourceCtx, clipboard: &mut Clipboard) -> bool {
        let mut processed_update = false;
        while let Some(modification) = self.mod_queue_receiver.try_recv() {
            processed_update = true;
            match modification.type_ {
                ElementModificationType::CustomStateChanged => {
                    self.handle_element_custom_state_changed(
                        modification.element_id,
                        res,
                        clipboard,
                    );
                }
                ElementModificationType::MarkDirty => {
                    self.mark_element_dirty(modification.element_id);
                }
                ElementModificationType::RectChanged(new_rect) => {
                    self.update_element_rect(modification.element_id, new_rect, res, clipboard);
                }
                ElementModificationType::ScissorRectChanged => {
                    self.handle_scissor_rect_changed_for_element(
                        modification.element_id,
                        res,
                        clipboard,
                    );
                }
                ElementModificationType::ZIndexChanged(new_z_index) => {
                    self.update_element_z_index(
                        modification.element_id,
                        new_z_index,
                        res,
                        clipboard,
                    );
                }
                ElementModificationType::ExplicitlyHiddenChanged(manually_hidden) => {
                    self.update_element_manually_hidden(
                        modification.element_id,
                        manually_hidden,
                        res,
                        clipboard,
                    );
                }
                ElementModificationType::ClassChanged(new_class) => {
                    self.handle_element_class_changed(
                        modification.element_id,
                        new_class,
                        res,
                        clipboard,
                    );
                }
                ElementModificationType::SetAnimating(animating) => {
                    self.set_element_animating(modification.element_id, animating);
                }
                ElementModificationType::ChangeFocus(req) => match req {
                    ChangeFocusRequest::StealFocus => {
                        self.element_steal_focus(modification.element_id, false, res, clipboard);
                    }
                    ChangeFocusRequest::StealTemporaryFocus => {
                        self.element_steal_focus(modification.element_id, true, res, clipboard);
                    }
                    ChangeFocusRequest::ReleaseFocus => {
                        self.element_release_focus(modification.element_id, res, clipboard);
                    }
                },
                ElementModificationType::HandleDropped => {
                    self.drop_element(modification.element_id, res, clipboard);
                }
                ElementModificationType::ListenToClickOff => {
                    self.handle_element_listen_to_click_off(modification.element_id);
                }
                ElementModificationType::StartHoverTimeout => {
                    self.handle_element_start_hover_timeout(modification.element_id);
                }
                ElementModificationType::StartScrollWheelTimeout => {
                    self.handle_element_start_scroll_wheel_timeout(modification.element_id);
                }
                ElementModificationType::ShowTooltip {
                    message,
                    align,
                    auto_hide,
                } => {
                    self.handle_element_show_tooltip(
                        modification.element_id,
                        message,
                        align,
                        auto_hide,
                    );
                }
                ElementModificationType::UpdateScissorRect(req) => {
                    self.update_scissor_rect(
                        req.scissor_rect_id,
                        req.new_rect,
                        req.new_scroll_offset,
                    );
                }
            }
        }

        processed_update
    }

    pub fn view_needs_repaint(&self) -> bool {
        self.view_needs_repaint
    }

    pub fn element_is_hovered(&self, element: &ElementHandle) -> bool {
        let Some(element_entry) = self.element_arena.get(element.id().0) else {
            return false;
        };

        if let Some(pos) = self.prev_pointer_pos {
            if let Some(visible_rect) = &element_entry.stack_data.visible_rect {
                return visible_rect.contains(pos);
            }
        }

        false
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

    fn handle_element_start_scroll_wheel_timeout(&mut self, element_id: ElementID) {
        if self.element_arena.contains(element_id.0) {
            self.elements_with_scroll_wheel_timeout
                .insert(element_id, Some(Instant::now()));
        }
    }

    fn handle_element_show_tooltip(
        &mut self,
        element_id: ElementID,
        message: String,
        align: Align2,
        auto_hide: bool,
    ) {
        let Some(element_entry) = self.element_arena.get(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        self.element_with_active_tooltip = Some(ActiveTooltipInfo {
            element_id,
            auto_hide,
        });

        if let Some(action) = self.show_tooltip_action.as_mut() {
            let info = TooltipInfo {
                message,
                element_bounds: element_entry.stack_data.rect,
                align,
                window_id: self.context.window_id,
            };

            self.context.action_sender.send((action)(info)).unwrap();
        }
    }

    fn handle_element_class_changed(
        &mut self,
        element_id: ElementID,
        new_class: ClassID,
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        element_entry.stack_data.class = new_class;

        send_event_to_element(
            ElementEvent::StyleChanged,
            element_entry,
            element_id,
            &mut self.context,
            res,
            clipboard,
        );

        self.mark_element_dirty(element_id);
    }

    fn handle_element_custom_state_changed(
        &mut self,
        element_id: ElementID,
        res: &mut ResourceCtx,
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
            &mut self.context,
            res,
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
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        let pos_changed =
            element_entry.stack_data.offset_from_scissor_rect_origin != new_rect.origin.to_vector();
        let size_changed = element_entry.stack_data.rect.size != new_rect.size;

        if !(pos_changed || size_changed) {
            return;
        }

        element_entry.stack_data.offset_from_scissor_rect_origin = new_rect.origin.to_vector();
        element_entry.stack_data.rect.size = new_rect.size;
        element_entry.stack_data.update_layout(&self.scissor_rects);

        let old_visibility = element_entry.stack_data.visible();
        element_entry
            .stack_data
            .update_visibility(&self.scissor_rects, self.window_visible);
        let visibility_changed = element_entry.stack_data.visible() != old_visibility;

        if visibility_changed && !element_entry.stack_data.visible() {
            release_focus_for_element(element_id, element_entry, &mut self.context, res, clipboard);
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
                &mut self.context,
                res,
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
                &mut self.context,
                res,
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
                &mut self.context,
                res,
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
        res: &mut ResourceCtx,
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
            release_focus_for_element(element_id, element_entry, &mut self.context, res, clipboard);
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
                &mut self.context,
                res,
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
        res: &mut ResourceCtx,
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
                &mut self.context,
                res,
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
        res: &mut ResourceCtx,
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
            release_focus_for_element(element_id, element_entry, &mut self.context, res, clipboard);
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
                &mut self.context,
                res,
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
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) {
        if self.element_arena.get(element_id.0).is_none() {
            // Element has been dropped. Do nothing and return.
            return;
        }

        if let Some(focus_info) = &self.context.current_focus_info {
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
                    &mut self.context,
                    res,
                    clipboard,
                );
            }
        }
        self.elements_listening_to_clicked_off.clear();

        let prev_element_with_exclusive_focus =
            self.context.prev_element_with_exclusive_focus.take();

        // Release focus from the previously focused element.
        if let Some(info) = &self.context.current_focus_info {
            self.element_release_focus(info.element_id, res, clipboard);
        }

        let element_entry = self.element_arena.get_mut(element_id.0).unwrap();

        self.context.current_focus_info = Some(FocusInfo {
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

        self.context.prev_element_with_exclusive_focus = if is_temporary {
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
                ElementEvent::Focus(true),
                element_entry,
                element_id,
                &mut self.context,
                res,
                clipboard,
            );
        }
    }

    fn element_release_focus(
        &mut self,
        element_id: ElementID,
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) {
        let Some(element_entry) = self.element_arena.get_mut(element_id.0) else {
            // Element has been dropped. Do nothing and return.
            return;
        };

        release_focus_for_element(element_id, element_entry, &mut self.context, res, clipboard);
    }

    fn drop_element(
        &mut self,
        element_id: ElementID,
        res: &mut ResourceCtx,
        clipboard: &mut Clipboard,
    ) {
        if let Some(focus_info) = &self.context.current_focus_info {
            if focus_info.element_id == element_id {
                self.element_release_focus(element_id, res, clipboard);
            }
        }

        let Some(mut element_entry) = self.element_arena.remove(element_id.0) else {
            // Element has already been dropped. Do nothing and return.
            return;
        };

        release_focus_for_element(
            element_id,
            &mut element_entry,
            &mut self.context,
            res,
            clipboard,
        );

        if element_entry
            .stack_data
            .flags
            .contains(ElementFlags::LISTENS_TO_ON_DROPPED)
        {
            element_entry
                .element
                .on_dropped(&mut self.context.action_sender);
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

        self.scissor_rects[usize::from(element_entry.stack_data.scissor_rect_index)]
            .remove_element(&element_entry.stack_data, &mut self.element_arena);

        if let Some(info) = &self.element_with_active_tooltip {
            if element_id == info.element_id {
                self.element_with_active_tooltip = None;

                if let Some(action) = self.hide_tooltip_action.as_mut() {
                    self.context.action_sender.send((action)()).unwrap();
                }
            }
        }

        self.hovered_elements.remove(&element_id);
        self.elements_with_scroll_wheel_timeout.remove(&element_id);

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
        res: &mut ResourceCtx,
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

        for render_cache in self.render_caches.values_mut() {
            render_cache.pre_render();
        }

        {
            let mut vg = vg.begin(self.physical_size, self.context.scale_factor);

            for cache in self.painted_elements.iter_mut() {
                if !cache.visible {
                    continue;
                }

                if cache.dirty {
                    cache.dirty = false;

                    cache.primitives.clear();

                    let element_entry = self.element_arena.get_mut(cache.element_id.0).unwrap();

                    let render_cache = if let Some(render_cache_id) =
                        element_entry.element.global_render_cache_id()
                    {
                        self.render_caches.get_mut(&render_cache_id)
                    } else {
                        None
                    };

                    element_entry.element.render_primitives(
                        RenderContext {
                            res,
                            bounds_size: element_entry.stack_data.rect.size,
                            bounds_origin: element_entry.stack_data.rect.origin,
                            visible_bounds: element_entry.stack_data.visible_rect.unwrap(),
                            scale: self.context.scale_factor,
                            window_size: self.context.logical_size,
                            render_cache,
                            class: element_entry.stack_data.class,
                        },
                        &mut cache.primitives,
                    );
                }

                vg.set_z_index(cache.z_index);
                vg.set_scissor_rect(self.scissor_rects[cache.scissor_rect_index].rect());
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
            &mut res.font_system,
            &mut res.svg_icon_system,
        )
        .unwrap(); // TODO: handle this error properly.

        for render_cache in self.render_caches.values_mut() {
            render_cache.post_render();
        }

        pre_present_notify();

        // Submit the commands and present the frame.
        queue.submit(Some(encoder.finish()));
        frame.present();

        self.view_needs_repaint = false;

        Ok(())
    }

    pub(crate) fn cursor_icon(&self) -> CursorIcon {
        self.context.cursor_icon
    }

    pub(crate) fn pointer_lock_request(&mut self) -> Option<bool> {
        self.context.pointer_lock_request.take()
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
    offset_from_scissor_rect_origin: Vector,

    scissor_rect_index: usize,
    z_index: ZIndex,

    class: ClassID,

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
        let scissor_rect = &scissor_rects[self.scissor_rect_index];
        let scissor_rect_origin: Point = scissor_rect.origin().cast();

        self.rect.origin = scissor_rect_origin + self.offset_from_scissor_rect_origin
            - scissor_rect.scroll_offset();
    }

    fn update_visibility(&mut self, scissor_rects: &[ScissorRect], window_visible: bool) {
        self.visible_rect = if self.manually_hidden
            || self.rect.size.width <= 0.0
            || self.rect.size.height <= 0.0
            || !window_visible
        {
            None
        } else {
            let scissor_rect: Rect = scissor_rects[self.scissor_rect_index].rect().cast();
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

#[derive(Clone, Copy)]
struct FocusInfo {
    element_id: ElementID,
    listens_to_pointer_inside_bounds: bool,
    listens_to_pointer_outside_bounds: bool,
    listens_to_text_composition: bool,
    listens_to_keys: bool,
}

#[derive(Clone, Copy)]
struct ActiveTooltipInfo {
    element_id: ElementID,
    auto_hide: bool,
}

fn send_event_to_element<A: Clone + 'static>(
    event: ElementEvent,
    element_entry: &mut ElementEntry<A>,
    element_id: ElementID,
    view_cx: &mut ViewContext<A>,
    res: &mut ResourceCtx,
    clipboard: &mut Clipboard,
) -> EventCaptureStatus {
    let has_focus = view_cx
        .current_focus_info
        .as_ref()
        .map(|d| d.element_id == element_id)
        .unwrap_or(false);

    let mut el_cx = ElementContext::new(
        element_entry.stack_data.rect,
        element_entry.stack_data.visible_rect,
        view_cx.logical_size,
        element_entry.stack_data.z_index,
        element_entry.stack_data.manually_hidden,
        element_entry.stack_data.animating,
        has_focus,
        view_cx.scale_factor,
        view_cx.cursor_icon,
        view_cx.window_id,
        view_cx.pointer_locked,
        element_entry.stack_data.class,
        &mut view_cx.action_sender,
        res,
        clipboard,
    );

    let capture_status = element_entry.element.on_event(event, &mut el_cx);

    view_cx.cursor_icon = el_cx.cursor_icon;

    if let Some(req) = el_cx.pointer_lock_request {
        view_cx.pointer_lock_request = Some(req);
    }

    if el_cx.listen_to_pointer_clicked_off {
        view_cx.mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::ListenToClickOff,
        });
    }

    if let Some(req) = el_cx.change_focus_request {
        let do_send = match req {
            ChangeFocusRequest::ReleaseFocus => has_focus,
            _ => !has_focus,
        };

        if do_send {
            view_cx.mod_queue_sender.send_to_front(ElementModification {
                element_id,
                type_: ElementModificationType::ChangeFocus(req),
            });
        }
    }

    if el_cx.is_animating() != element_entry.stack_data.animating {
        view_cx.mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::SetAnimating(el_cx.is_animating()),
        });
    }

    if let Some(new_rect) = el_cx.requested_rect {
        view_cx.mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::RectChanged(new_rect),
        });
    }

    if el_cx.repaint_requested {
        view_cx.mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::MarkDirty,
        });
    }

    if el_cx.hover_timeout_requested {
        view_cx.mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::StartHoverTimeout,
        });
    }

    if el_cx.scroll_wheel_timeout_requested {
        view_cx.mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::StartScrollWheelTimeout,
        });
    }

    if let Some(req) = el_cx.requested_show_tooltip {
        view_cx.mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::ShowTooltip {
                message: req.message,
                align: req.align,
                auto_hide: req.auto_hide,
            },
        });
    }

    if let Some(req) = el_cx.update_scissor_rect_req {
        view_cx.mod_queue_sender.send_to_front(ElementModification {
            element_id,
            type_: ElementModificationType::UpdateScissorRect(req),
        });
    }

    capture_status
}

fn release_focus_for_element<A: Clone + 'static>(
    element_id: ElementID,
    element_entry: &mut ElementEntry<A>,
    cx: &mut ViewContext<A>,
    res: &mut ResourceCtx,
    clipboard: &mut Clipboard,
) {
    if let Some(info) = &cx.current_focus_info {
        if info.element_id != element_id {
            return;
        }
    } else {
        return;
    };

    cx.current_focus_info = None;

    // Make sure the pointer does not stay locked.
    if let Some(lock) = &mut cx.pointer_lock_request {
        *lock = false;
    }

    if element_entry
        .stack_data
        .flags
        .contains(ElementFlags::LISTENS_TO_FOCUS_CHANGE)
    {
        send_event_to_element(
            ElementEvent::Focus(false),
            element_entry,
            element_id,
            cx,
            res,
            clipboard,
        );
    }

    if let Some(prev_element_id) = cx.prev_element_with_exclusive_focus.take() {
        if prev_element_id != element_id {
            cx.mod_queue_sender.send_to_front(ElementModification {
                element_id: prev_element_id,
                type_: ElementModificationType::ChangeFocus(ChangeFocusRequest::StealFocus),
            });
        }
    }
}
