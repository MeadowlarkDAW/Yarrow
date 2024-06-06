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

use super::ElementModificationType;
use crate::math::{Point, Rect, Size, ZIndex};
use crate::stmpsc_queue;
use crate::view::{ElementID, ElementModification};

pub struct ElementHandle {
    element_id: ElementID,
    mod_queue_sender: stmpsc_queue::Sender<ElementModification>,

    rect: Rect,
    z_index: ZIndex,
    manually_hidden: bool,
}

impl ElementHandle {
    pub(super) fn new(
        element_id: ElementID,
        mod_queue_sender: stmpsc_queue::Sender<ElementModification>,
        rect: Rect,
        z_index: ZIndex,
        manually_hidden: bool,
    ) -> Self {
        Self {
            element_id,
            mod_queue_sender,
            rect,
            z_index,
            manually_hidden,
        }
    }

    /// Get the rectangular area assigned to this element instance.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Get the z index of this element instance.
    pub fn z_index(&self) -> ZIndex {
        self.z_index
    }

    /// Returns `true` if the element instance has been manually hidden.
    ///
    /// Note that even if this returns `true`, the element may still be hidden
    /// due to it being outside of the render area.
    pub fn manually_hidden(&self) -> bool {
        self.manually_hidden
    }

    /// Set the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    pub fn set_rect(&mut self, rect: Rect) {
        if self.rect != rect {
            self.rect = rect;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(rect),
            });
        }
    }

    /// Set the position of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_rect()` than
    /// to set the position and size separately.
    pub fn set_pos(&mut self, pos: Point) {
        if self.rect.origin != pos {
            self.rect.origin = pos;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
        }
    }

    /// Set the size of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_rect()` than
    /// to set the position and size separately.
    pub fn set_size(&mut self, size: Size) {
        if self.rect.size != size || true {
            self.rect.size = size;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
        }
    }

    /// Set the x position of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_pos()` or
    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
    /// separately.
    pub fn set_x(&mut self, x: f32) {
        if self.rect.origin.x != x {
            self.rect.origin.x = x;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
        }
    }

    /// Set the y position of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_pos()` or
    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
    /// separately.
    pub fn set_y(&mut self, y: f32) {
        if self.rect.origin.y != y {
            self.rect.origin.y = y;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
        }
    }

    /// Set the width of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_size()` or
    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
    /// separately.
    pub fn set_width(&mut self, width: f32) {
        if self.rect.size.width != width {
            self.rect.size.width = width;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
        }
    }

    /// Set the height of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_size()` or
    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
    /// separately.
    pub fn set_height(&mut self, height: f32) {
        if self.rect.size.height != height {
            self.rect.size.height = height;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
        }
    }

    /// Set the z index of this element instance.
    ///
    /// An update will only be sent to the view if the z index has changed.
    pub fn set_z_index(&mut self, z_index: ZIndex) {
        if self.z_index != z_index {
            self.z_index = z_index;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::ZIndexChanged(z_index),
            });
        }
    }

    /// Set to hide or show this element instance.
    ///
    /// Note, there is no need to hide elements just because they appear outside
    /// of the render area. The view already handles that for you.
    ///
    /// An update will only be sent to the view if the visibility request
    /// has changed since the previous call.
    pub fn set_hidden(&mut self, hidden: bool) {
        if self.manually_hidden != hidden {
            self.manually_hidden = hidden;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::ExplicitlyHiddenChanged(hidden),
            });
        }
    }

    /// Notify the element that its custom state has changed.
    ///
    /// This is meant to be used by element implementations, not by the end-user.
    pub fn notify_custom_state_change(&mut self) {
        self.mod_queue_sender.send(ElementModification {
            element_id: self.element_id,
            type_: ElementModificationType::CustomStateChanged,
        });
    }

    pub(crate) fn id(&self) -> ElementID {
        self.element_id
    }
}

impl Drop for ElementHandle {
    fn drop(&mut self) {
        self.mod_queue_sender.send(ElementModification {
            element_id: self.element_id,
            type_: ElementModificationType::HandleDropped,
        });
    }
}
