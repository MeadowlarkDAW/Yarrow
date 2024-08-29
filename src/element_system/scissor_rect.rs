use std::u32;

use thunderdome::Arena;

use super::{ElementEntry, ElementID, EntryStackData};
use crate::element_system::element::{ElementModification, ElementModificationType};
use crate::math::{PointI32, RectI32, Vector};
use crate::stmpsc_queue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScissorRectID(pub u32);

impl ScissorRectID {
    /// `ScissorRectID` of `u32::MAX` means to use use the main ElementSystem itself as the
    /// scissoring rectangle.
    pub const DEFAULT: Self = Self(u32::MAX);
}

impl Default for ScissorRectID {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub(super) struct ScissorRect {
    rect: RectI32,
    scroll_offset: Vector,
    assigned_elements: Vec<ElementID>,
}

impl ScissorRect {
    pub fn new(mut rect: RectI32, scroll_offset: Vector) -> Self {
        rect.size.width = rect.size.width.max(0);
        rect.size.height = rect.size.height.max(0);

        Self {
            rect,
            scroll_offset,
            assigned_elements: Vec::new(),
        }
    }

    pub fn rect(&self) -> RectI32 {
        self.rect
    }

    /// Returns `true` if the rect changed, `false` otherwise.
    ///
    /// # Panics
    /// This will panic if the width or the height of the rectangle is less than or
    /// equal to 0.
    pub fn update(
        &mut self,
        mut new_rect: Option<RectI32>,
        new_scroll_offset: Option<Vector>,
        mod_queue_sender: &mut stmpsc_queue::Sender<ElementModification>,
    ) -> bool {
        let mut changed = false;

        if let Some(new_rect) = &mut new_rect {
            new_rect.size.width = new_rect.size.width.max(0);
            new_rect.size.height = new_rect.size.height.max(0);

            if self.rect != *new_rect {
                self.rect = *new_rect;
                changed = true;
            }
        }

        if let Some(new_scroll_offset) = new_scroll_offset {
            if self.scroll_offset != new_scroll_offset {
                self.scroll_offset = new_scroll_offset;
                changed = true;
            }
        }

        if changed {
            for element_id in self.assigned_elements.iter() {
                mod_queue_sender.send(ElementModification {
                    element_id: *element_id,
                    type_: ElementModificationType::ScissorRectChanged,
                });
            }
        }

        changed
    }

    pub fn origin(&self) -> PointI32 {
        self.rect.origin
    }

    pub fn scroll_offset(&self) -> Vector {
        self.scroll_offset
    }

    pub fn add_element(&mut self, entry_stack_data: &mut EntryStackData, element_id: ElementID) {
        entry_stack_data.index_in_scissor_rect_list = self.assigned_elements.len() as u32;

        self.assigned_elements.push(element_id);
    }

    pub fn remove_element<A: Clone + 'static>(
        &mut self,
        entry_stack_data: &EntryStackData,
        element_arena: &mut Arena<ElementEntry<A>>,
    ) {
        let _ = self
            .assigned_elements
            .swap_remove(entry_stack_data.index_in_scissor_rect_list as usize);

        // Update the index in the element that was swapped.
        if let Some(swapped_element_id) = self
            .assigned_elements
            .get(entry_stack_data.index_in_scissor_rect_list as usize)
        {
            if let Some(swapped_element) = element_arena.get_mut(swapped_element_id.0) {
                swapped_element.stack_data.index_in_scissor_rect_list =
                    entry_stack_data.index_in_scissor_rect_list;
            }
        }
    }
}
