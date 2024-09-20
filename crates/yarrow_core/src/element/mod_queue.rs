use crate::{
    math::{Rect, Vector},
    style_system::ClassID,
    ScissorRectID, TooltipData, ZIndex,
};

use super::ElementID;

#[cfg(not(feature = "crossbeam"))]
pub type ModQueueSender = std::sync::mpsc::Sender<ElementModification>;
#[cfg(not(feature = "crossbeam"))]
pub type ModQueueReceiver = std::sync::mpsc::Receiver<ElementModification>;

#[cfg(feature = "crossbeam")]
pub type ModQueueSender = crossbeam_channel::Sender<ElementModification>;
#[cfg(feature = "crossbeam")]
pub type ModQueueReceiver = crossbeam_channel::Receiver<ElementModification>;

pub fn mod_queue_channel() -> (ModQueueSender, ModQueueReceiver) {
    #[cfg(not(feature = "crossbeam"))]
    return std::sync::mpsc::channel();

    #[cfg(feature = "crossbeam")]
    return crossbeam_channel::unbounded();
}

pub struct ElementModification {
    pub element_id: ElementID,
    pub type_: ElementModificationType,
}

pub enum ElementModificationType {
    Update,
    MarkDirty,
    RectChanged(Rect),
    ScissorRectChanged,
    ZIndexChanged(ZIndex),
    ExplicitlyHiddenChanged(bool),
    ClassChanged(ClassID),
    SetAnimating(bool),
    StealKeyboardFocus {
        temporary: bool,
    },
    ReleaseKeyboardFocus,
    StealPointerFocus,
    ReleasePointerFocus,
    HandleDropped,
    ListenToClickOff,
    StartHoverTimeout,
    StartScrollWheelTimeout,
    ShowTooltip {
        data: TooltipData,
        auto_hide: bool,
    },
    UpdateScissorRect {
        id: ScissorRectID,
        new_rect: Option<Rect>,
        new_scroll_offset: Option<Vector>,
    },
}
