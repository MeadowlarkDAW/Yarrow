use crate::{
    layout::Align2,
    math::{Point, Rect, Size, Vector},
    style_system::ClassID,
    TooltipData, ZIndex,
};

use super::{
    mod_queue::{ElementModification, ElementModificationType, ModQueueSender},
    ElementID,
};

pub struct ElementHandle {
    element_id: ElementID,
    mod_queue_sender: ModQueueSender,

    rect: Rect,
    z_index: ZIndex,
    manually_hidden: bool,
    class: ClassID,
}

impl ElementHandle {
    pub fn new(
        element_id: ElementID,
        mod_queue_sender: ModQueueSender,
        rect: Rect,
        z_index: ZIndex,
        manually_hidden: bool,
        class: ClassID,
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
    /// This is cached directly in the handle so this is very cheap to call frequently.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Get the z index of this element instance.
    ///
    /// This is cached directly in the handle so this is very cheap to call frequently.
    pub fn z_index(&self) -> ZIndex {
        self.z_index
    }

    /// Returns `true` if the element instance has been manually hidden.
    ///
    /// Note that even if this returns `true`, the element may still be hidden
    /// due to it being outside of the render area.
    ///
    /// This is cached directly in the handle so this is very cheap to call frequently.
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
    /// so this method is very cheap to call frequently.
    pub fn set_rect(&mut self, rect: Rect) -> bool {
        if self.rect != rect {
            self.rect = rect;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::RectChanged(rect),
                })
                .unwrap();
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
    /// so this method is very cheap to call frequently.
    pub fn set_pos(&mut self, pos: Point) -> bool {
        if self.rect.origin != pos {
            self.rect.origin = pos;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::RectChanged(self.rect),
                })
                .unwrap();
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
    /// so this method is very cheap to call frequently.
    pub fn set_size(&mut self, size: Size) -> bool {
        if self.rect.size != size {
            self.rect.size = size;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::RectChanged(self.rect),
                })
                .unwrap();
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
    /// so this method is very cheap to call frequently.
    pub fn set_x(&mut self, x: f32) -> bool {
        if self.rect.origin.x != x {
            self.rect.origin.x = x;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::RectChanged(self.rect),
                })
                .unwrap();
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
    /// so this method is very cheap to call frequently.
    pub fn set_y(&mut self, y: f32) -> bool {
        if self.rect.origin.y != y {
            self.rect.origin.y = y;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::RectChanged(self.rect),
                })
                .unwrap();
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
    /// so this method is very cheap to call frequently.
    pub fn set_width(&mut self, width: f32) -> bool {
        if self.rect.size.width != width {
            self.rect.size.width = width;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::RectChanged(self.rect),
                })
                .unwrap();
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
    /// so this method is very cheap to call frequently.
    pub fn set_height(&mut self, height: f32) -> bool {
        if self.rect.size.height != height {
            self.rect.size.height = height;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::RectChanged(self.rect),
                })
                .unwrap();
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
        self.mod_queue_sender
            .send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::RectChanged(self.rect),
            })
            .unwrap();
    }

    /// Set the z index of this element instance.
    ///
    /// An update will only be sent to the view if the z index has changed.
    ///
    /// Returns `true` if the z index has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is very cheap to call frequently.
    pub fn set_z_index(&mut self, z_index: ZIndex) -> bool {
        if self.z_index != z_index {
            self.z_index = z_index;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::ZIndexChanged(z_index),
                })
                .unwrap();
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
    /// so this method is very cheap to call frequently.
    pub fn set_hidden(&mut self, hidden: bool) -> bool {
        if self.manually_hidden != hidden {
            self.manually_hidden = hidden;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::ExplicitlyHiddenChanged(hidden),
                })
                .unwrap();
            true
        } else {
            false
        }
    }

    /// Show a tooltip on the element
    ///
    /// * `text` - The tooltip text
    /// * `align` - Where to align the tooltip relative to this element
    /// * `auto_hide` - Whether or not the tooltip should automatically hide when
    /// the mouse pointer is no longer over the element.
    pub fn show_tooltip(&mut self, text: impl Into<String>, align: Align2, auto_hide: bool) {
        self.mod_queue_sender
            .send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::ShowTooltip {
                    data: TooltipData {
                        text: text.into(),
                        align,
                    },
                    auto_hide,
                },
            })
            .unwrap();
    }

    /// The current style class of the element.
    ///
    /// This is cached directly in the handle so this is very cheap to call frequently.
    pub fn class(&self) -> ClassID {
        self.class
    }

    /// Notify the system that data which this element is bound to has changed.
    ///
    /// Note, this will *always* cause an element update, so prefer to call this
    /// method sparingly.
    pub fn notify_update(&mut self) {
        self.mod_queue_sender
            .send(ElementModification {
                element_id: self.element_id,
                type_: ElementModificationType::Update,
            })
            .unwrap();
    }

    /// Set the class of this element instance.
    ///
    /// An update will only be sent to the view if the class has changed.
    ///
    /// Returns `true` if the class has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently (although a
    /// string comparison is performed).
    pub fn set_class(&mut self, new_class: ClassID) -> bool {
        if self.class != new_class {
            self.class = new_class;
            self.mod_queue_sender
                .send(ElementModification {
                    element_id: self.element_id,
                    type_: ElementModificationType::ClassChanged(new_class),
                })
                .unwrap();
            true
        } else {
            false
        }
    }

    pub fn id(&self) -> ElementID {
        self.element_id
    }
}

impl Drop for ElementHandle {
    fn drop(&mut self) {
        let _ = self.mod_queue_sender.send(ElementModification {
            element_id: self.element_id,
            type_: ElementModificationType::HandleDropped,
        });
    }
}
