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

use super::ElementModificationType;
use crate::layout::Align2;
use crate::math::{Point, Rect, Size, Vector, ZIndex};
use crate::stmpsc_queue;
use crate::view::{ElementID, ElementModification};

pub struct ElementHandle {
    element_id: ElementID,
    mod_queue_sender: stmpsc_queue::Sender<ElementModification>,

    rect: Rect,
    z_index: ZIndex,
    manually_hidden: bool,
    class: &'static str,
}

impl ElementHandle {
    pub(super) fn new(
        element_id: ElementID,
        mod_queue_sender: stmpsc_queue::Sender<ElementModification>,
        rect: Rect,
        z_index: ZIndex,
        manually_hidden: bool,
        class: &'static str,
    ) -> Self {
        Self {
            element_id,
            mod_queue_sender,
            rect,
            z_index,
            manually_hidden,
            class,
        }
    }

    /// Get the bounding rectangle of this element instance.
    ///
    /// This is cached directly in the handle so this is very cheap to call.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Get the z index of this element instance.
    ///
    /// This is cached directly in the handle so this is very cheap to call.
    pub fn z_index(&self) -> ZIndex {
        self.z_index
    }

    /// Returns `true` if the element instance has been manually hidden.
    ///
    /// Note that even if this returns `true`, the element may still be hidden
    /// due to it being outside of the render area.
    ///
    /// This is cached directly in the handle so this is very cheap to call.
    pub fn manually_hidden(&self) -> bool {
        self.manually_hidden
    }

    /// Set the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Returns `true` if the rectangle has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call.
    pub fn set_rect(&mut self, rect: Rect) -> bool {
        if self.rect != rect {
            self.rect = rect;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(rect),
            });
            true
        } else {
            false
        }
    }

    /// Set the position of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_rect()` than
    /// to set the position and size separately.
    ///
    /// Returns `true` if the position has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call.
    pub fn set_pos(&mut self, pos: Point) -> bool {
        if self.rect.origin != pos {
            self.rect.origin = pos;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
            true
        } else {
            false
        }
    }

    /// Set the size of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_rect()` than
    /// to set the position and size separately.
    ///
    /// Returns `true` if the size has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call.
    pub fn set_size(&mut self, size: Size) -> bool {
        if self.rect.size != size || true {
            self.rect.size = size;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
            true
        } else {
            false
        }
    }

    /// Set the x position of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_pos()` or
    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
    /// separately.
    ///
    /// Returns `true` if the x position has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call.
    pub fn set_x(&mut self, x: f32) -> bool {
        if self.rect.origin.x != x {
            self.rect.origin.x = x;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
            true
        } else {
            false
        }
    }

    /// Set the y position of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_pos()` or
    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
    /// separately.
    ///
    /// Returns `true` if the y position has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call.
    pub fn set_y(&mut self, y: f32) -> bool {
        if self.rect.origin.y != y {
            self.rect.origin.y = y;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
            true
        } else {
            false
        }
    }

    /// Set the width of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_size()` or
    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
    /// separately.
    ///
    /// Returns `true` if the width has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call.
    pub fn set_width(&mut self, width: f32) -> bool {
        if self.rect.size.width != width {
            self.rect.size.width = width;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
            true
        } else {
            false
        }
    }

    /// Set the height of the rectangular area of this element instance.
    ///
    /// An update will only be sent to the view if the rectangle has changed.
    ///
    /// Note, it is more efficient to use `ElementHandle::set_size()` or
    /// `ElementHandle::set_rect()` than to set the fields of the rectangle
    /// separately.
    ///
    /// Returns `true` if the height has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call.
    pub fn set_height(&mut self, height: f32) -> bool {
        if self.rect.size.height != height {
            self.rect.size.height = height;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            });
            true
        } else {
            false
        }
    }

    /// Offset the element's rectangular area.
    ///
    /// Note, this will *always* cause an element update even if the offset
    /// is zero, so prefer to call this method sparingly.
    pub fn offset_pos(&mut self, offset: Vector) {
        self.rect.origin += offset;
        self.mod_queue_sender.send(ElementModification {
            element_id: self.element_id,
            type_: ElementModificationType::RectChanged(self.rect),
        });
    }

    /// Set the z index of this element instance.
    ///
    /// An update will only be sent to the view if the z index has changed.
    ///
    /// Returns `true` if the z index has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call.
    pub fn set_z_index(&mut self, z_index: ZIndex) -> bool {
        if self.z_index != z_index {
            self.z_index = z_index;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::ZIndexChanged(z_index),
            });
            true
        } else {
            false
        }
    }

    /// Set to hide or show this element instance.
    ///
    /// Note, there is no need to hide elements just because they appear outside
    /// of the render area. The view already handles that for you.
    ///
    /// An update will only be sent to the view if the visibility request
    /// has changed since the previous call.
    ///
    /// Returns `true` if the hidden state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call.
    pub fn set_hidden(&mut self, hidden: bool) -> bool {
        if self.manually_hidden != hidden {
            self.manually_hidden = hidden;
            self.mod_queue_sender.send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::ExplicitlyHiddenChanged(hidden),
            });
            true
        } else {
            false
        }
    }

    /// Show a tooltip on the element.
    pub fn show_tooltip(&mut self, message: String, align: Align2, auto_hide: bool) {
        self.mod_queue_sender.send(ElementModification {
            element_id: self.element_id,
            type_: ElementModificationType::ShowTooltip {
                message,
                align,
                auto_hide,
            },
        })
    }

    /// The current style class of the element.
    ///
    /// This is cached directly in the handle so this is very cheap to call.
    pub fn class(&self) -> &'static str {
        self.class
    }

    /// Notify the element that its custom state has changed.
    ///
    /// This is meant to be used by element implementations, not by the end-user.
    ///
    /// Note, this will *always* cause an element update, so prefer to call this
    /// method sparingly.
    pub fn _notify_custom_state_change(&mut self) {
        self.mod_queue_sender.send(ElementModification {
            element_id: self.element_id,
            type_: ElementModificationType::CustomStateChanged,
        });
    }

    /// Notify the element that its class name has changed.
    ///
    /// This is meant to be used by element implementations, not by the end-user.
    /// Using this method directly instead of the element's provided `set_class`
    /// method may lead to de-synced state and unexpected results.
    ///
    /// Note, this will *always* cause an element update, so prefer to call this
    /// method sparingly.
    pub fn _notify_class_change(&mut self, new_class: &'static str) {
        self.class = new_class;
        self.mod_queue_sender.send(ElementModification {
            element_id: self.element_id,
            type_: ElementModificationType::ClassChanged(new_class),
        })
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
