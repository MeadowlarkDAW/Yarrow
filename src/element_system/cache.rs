use rootvg::PrimitiveGroup;

use crate::math::{Rect, Vector, ZIndex};

use super::{ElementFlags, ElementID, EntryStackData};

pub(super) struct CachedElementRectForPointerEvent {
    pub z_index: ZIndex,
    pub element_id: ElementID,
    pub visible_rect: Option<Rect>,
}

#[derive(Debug)]
pub(super) struct CachedElementPrimitives {
    pub element_id: ElementID,
    pub offset: Vector,
    pub z_index: ZIndex,
    pub scissor_rect_index: usize,
    pub visible: bool,
    pub dirty: bool,
    pub primitives: PrimitiveGroup,
}

impl CachedElementPrimitives {
    pub fn new(
        element_id: ElementID,
        offset: Vector,
        z_index: ZIndex,
        scissor_rect_index: usize,
        visible: bool,
    ) -> Self {
        Self {
            element_id,
            offset,
            z_index,
            scissor_rect_index,
            visible,
            dirty: true,
            primitives: PrimitiveGroup::new(),
        }
    }
}

pub(super) fn sync_element_rect_cache(
    entry_stack_data: &EntryStackData,
    elements_listening_to_pointer_event: &mut Vec<CachedElementRectForPointerEvent>,
    painted_elements: &mut Vec<CachedElementPrimitives>,
    mark_dirty: bool,
) {
    if entry_stack_data
        .flags
        .contains(ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS)
    {
        elements_listening_to_pointer_event
            [entry_stack_data.index_in_pointer_event_list as usize]
            .visible_rect = entry_stack_data.visible_rect;
    }

    if entry_stack_data.flags.contains(ElementFlags::PAINTS) {
        let cache = &mut painted_elements[entry_stack_data.index_in_painted_list as usize];

        cache.offset = entry_stack_data.rect.origin.to_vector();
        cache.visible = entry_stack_data.visible();
        cache.dirty |= mark_dirty;
    }
}
