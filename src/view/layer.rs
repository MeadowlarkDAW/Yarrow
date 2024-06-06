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
// https://codeberg.org/BillyDM/Yarrow/src/branch/main/LICENSE
//
// ---------------------------------------------------------------------------------

use super::arena::IdArena;
use super::{ElementEntry, ElementFlags, ElementID, ExclusiveFocusData};
use crate::action_queue::ActionSender;
use crate::element::{Element, ElementModification};
use crate::event::{Event, EventCaptureStatus, PointerEvent};
use crate::math::{Rect, ScaleFactor, ZIndex};
use crate::stmpsc_queue;

#[derive(Default, Clone)]
pub(super) struct ElementLayerData {
    // Keep these private to ensure that they can only be mutated by this module.
    z_index: ZIndex,
    index_in_pointer_event_list: u32,
    index_in_painted_list: u32,
}

impl ElementLayerData {
    pub fn z_index(&self) -> ZIndex {
        self.z_index
    }
}

pub(super) struct Layer {
    z_index_lists: Vec<ZIndexList>,
}

impl Layer {
    pub fn new(num_z_indexes: u8) -> Self {
        Self {
            z_index_lists: (0..num_z_indexes).map(|_| ZIndexList::new()).collect(),
        }
    }

    /// Add an element to this layer.
    pub fn add_element(
        &mut self,
        element_entry: &mut ElementEntry,
        element_id: ElementID,
        z_index: ZIndex,
    ) {
        // Clamp the z index so it is valid.
        let z_index = if usize::from(z_index) >= self.z_index_lists.len() {
            // TODO: Log warning to user.

            (self.z_index_lists.len() - 1) as ZIndex
        } else {
            z_index
        };

        element_entry.layer_data.z_index = z_index;

        let cached_rect = if element_entry
            .flags
            .contains(ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
        {
            Some(CachedElementRectForPointerEvent::new(
                element_id,
                element_entry.rect,
                element_entry.visible,
            ))
        } else {
            None
        };

        let cached_paint_commands = if element_entry.flags.contains(ElementFlags::PAINTS) {
            Some(CachedElementPrimitives::new(
                element_id,
                element_entry.visible,
            ))
        } else {
            None
        };

        self._add_element(element_entry, cached_rect, cached_paint_commands);
    }

    fn _add_element(
        &mut self,
        element_entry: &mut ElementEntry,
        cached_rect: Option<CachedElementRectForPointerEvent>,
        cached_paint_commands: Option<CachedElementPrimitives>,
    ) {
        if cached_rect.is_none() && cached_paint_commands.is_none() {
            return;
        }

        let z_index_list = &mut self.z_index_lists[usize::from(element_entry.layer_data.z_index)];

        if let Some(cached_rect) = cached_rect {
            element_entry.layer_data.index_in_pointer_event_list =
                z_index_list.elements_listening_to_pointer_event.len() as u32;

            z_index_list
                .elements_listening_to_pointer_event
                .push(cached_rect);
        }

        if let Some(cached_paint_commands) = cached_paint_commands {
            element_entry.layer_data.index_in_painted_list =
                z_index_list.painted_elements.len() as u32;

            z_index_list.painted_elements.push(cached_paint_commands);
        }
    }

    pub fn remove_element(
        &mut self,
        element_entry: &ElementEntry,
        element_arena: &mut IdArena<ElementEntry, Box<dyn Element>>,
    ) {
        let _ = self._remove_element(element_entry, element_arena);
    }

    fn _remove_element(
        &mut self,
        element_entry: &ElementEntry,
        element_arena: &mut IdArena<ElementEntry, Box<dyn Element>>,
    ) -> (
        Option<CachedElementRectForPointerEvent>,
        Option<CachedElementPrimitives>,
    ) {
        let index_in_pointer_event_list = if element_entry
            .flags
            .contains(ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
        {
            Some(element_entry.layer_data.index_in_pointer_event_list)
        } else {
            None
        };

        let index_in_painted_list = if element_entry.flags.contains(ElementFlags::PAINTS) {
            Some(element_entry.layer_data.index_in_painted_list)
        } else {
            None
        };

        let z_index_list = &mut self.z_index_lists[usize::from(element_entry.layer_data.z_index)];

        let mut cached_rect = None;
        if let Some(index_in_pointer_event_list) = index_in_pointer_event_list {
            cached_rect = Some(
                z_index_list
                    .elements_listening_to_pointer_event
                    .swap_remove(index_in_pointer_event_list as usize),
            );

            // Update the index in the element that was swapped.
            if let Some(swapped_element_cache) = z_index_list
                .elements_listening_to_pointer_event
                .get(index_in_pointer_event_list as usize)
            {
                element_arena
                    .get_stack_data_mut(swapped_element_cache.element_id.0)
                    .unwrap()
                    .layer_data
                    .index_in_pointer_event_list = index_in_pointer_event_list;
            }
        }

        let mut cached_paint_commands = None;
        if let Some(index_in_painted_list) = index_in_painted_list {
            cached_paint_commands = Some(
                z_index_list
                    .painted_elements
                    .swap_remove(index_in_painted_list as usize),
            );

            // Update the index in the element that was swapped.
            if let Some(swapped_element_cache) = z_index_list
                .painted_elements
                .get(index_in_painted_list as usize)
            {
                element_arena
                    .get_stack_data_mut(swapped_element_cache.element_id.0)
                    .unwrap()
                    .layer_data
                    .index_in_painted_list = index_in_painted_list;
            }
        }

        (cached_rect, cached_paint_commands)
    }

    // Returns `true` if the z index changed, false otherwise.
    pub fn sync_element_z_index(
        &mut self,
        element_id: ElementID,
        new_z_index: ZIndex,
        element_arena: &mut IdArena<ElementEntry, Box<dyn Element>>,
    ) -> bool {
        // Clamp the z index so it is valid.
        let new_z_index = if usize::from(new_z_index) >= self.z_index_lists.len() {
            // TODO: Log warning to user.

            (self.z_index_lists.len() - 1) as ZIndex
        } else {
            new_z_index
        };

        let element_entry = {
            let Some(element_entry) = element_arena.get_stack_data(element_id.0) else {
                return false;
            };

            if element_entry.layer_data.z_index == new_z_index {
                return false;
            }

            element_entry.clone()
        };

        let (cached_rect, cached_paint_commands) =
            self._remove_element(&element_entry, element_arena);

        let element_entry = element_arena.get_stack_data_mut(element_id.0).unwrap();

        element_entry.layer_data.z_index = new_z_index;

        self._add_element(element_entry, cached_rect, cached_paint_commands);

        true
    }

    pub fn sync_element_cache(&mut self, element_entry: &ElementEntry) {
        let z_index_list = &mut self.z_index_lists[usize::from(element_entry.layer_data.z_index)];

        if element_entry
            .flags
            .contains(ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
        {
            let cache = &mut z_index_list.elements_listening_to_pointer_event
                [element_entry.layer_data.index_in_pointer_event_list as usize];

            cache.rect = element_entry.rect;
            cache.visible = element_entry.visible;
        }

        if element_entry.flags.contains(ElementFlags::PAINTS) {
            let cache = &mut z_index_list.painted_elements
                [element_entry.layer_data.index_in_painted_list as usize];

            cache.visible = element_entry.visible;
            cache.dirty = cache.dirty || element_entry.visible;
        }
    }

    pub fn mark_element_dirty(&mut self, element_entry: &ElementEntry) {
        let z_index_list = &mut self.z_index_lists[usize::from(element_entry.layer_data.z_index)];

        if element_entry.flags.contains(ElementFlags::PAINTS) {
            let cache = &mut z_index_list.painted_elements
                [element_entry.layer_data.index_in_painted_list as usize];

            cache.dirty = true;
        }
    }

    pub fn handle_pointer_event(
        &mut self,
        event: &PointerEvent,
        element_arena: &mut IdArena<ElementEntry, Box<dyn Element>>,
        exclusive_focus_data: &Option<ExclusiveFocusData>,
        mod_queue_sender: &mut stmpsc_queue::Sender<ElementModification>,
        action_sender: &mut ActionSender,
        scale_factor: ScaleFactor,
    ) -> EventCaptureStatus {
        // Iterate z indexes from highest to lowest.
        for z_index_list in self.z_index_lists.iter_mut().rev() {
            for cached_rect in z_index_list.elements_listening_to_pointer_event.iter() {
                if !cached_rect.visible {
                    continue;
                }

                if !cached_rect.rect.contains(event.position) {
                    continue;
                }

                let Some((element_entry, element)) =
                    element_arena.get_mut(cached_rect.element_id.0)
                else {
                    continue;
                };

                if let EventCaptureStatus::Captured = super::send_event_to_element(
                    Event::Pointer(event.clone()),
                    element_entry,
                    element,
                    cached_rect.element_id,
                    exclusive_focus_data,
                    mod_queue_sender,
                    action_sender,
                    scale_factor,
                ) {
                    return EventCaptureStatus::Captured;
                }
            }
        }

        EventCaptureStatus::NotCaptured
    }

    pub fn render(&mut self, vg: &mut rootvg::CanvasCtx) {
        // TODO
    }
}

struct ZIndexList {
    elements_listening_to_pointer_event: Vec<CachedElementRectForPointerEvent>,
    painted_elements: Vec<CachedElementPrimitives>,
}

impl ZIndexList {
    fn new() -> Self {
        Self {
            elements_listening_to_pointer_event: Vec::new(),
            painted_elements: Vec::new(),
        }
    }
}

struct CachedElementRectForPointerEvent {
    element_id: ElementID,
    rect: Rect,
    visible: bool,
}

impl CachedElementRectForPointerEvent {
    pub fn new(element_id: ElementID, rect: Rect, visible: bool) -> Self {
        Self {
            element_id,
            rect,
            visible,
        }
    }
}

struct CachedElementPrimitives {
    element_id: ElementID,
    visible: bool,
    dirty: bool,
    //primitives: Primitives,
}

impl CachedElementPrimitives {
    pub fn new(element_id: ElementID, visible: bool) -> Self {
        Self {
            element_id,
            visible,
            dirty: true,
            //primitives: Primitives::new(),
        }
    }
}
