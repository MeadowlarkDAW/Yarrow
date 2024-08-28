mod context;
mod flags;
mod handle;

use std::any::Any;

use context::UpdateScissorRectRequest;
pub use context::{ElementContext, RenderContext};
pub use flags::ElementFlags;
pub use handle::ElementHandle;
use rootvg::math::Point;
use rootvg::PrimitiveGroup;

use super::ScissorRectID;
use crate::action_queue::ActionSender;
use crate::event::{ElementEvent, EventCaptureStatus};
use crate::math::{Rect, Size, ZIndex};
use crate::prelude::TooltipData;
use crate::stmpsc_queue;
use crate::style::ClassID;

pub(crate) use context::ChangeFocusRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ElementID(pub thunderdome::Index);

pub trait Element<A: Clone + 'static> {
    fn flags(&self) -> ElementFlags;

    #[allow(unused)]
    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        EventCaptureStatus::NotCaptured
    }

    #[allow(unused)]
    fn on_dropped(&mut self, action_sender: &mut ActionSender<A>) {}

    #[allow(unused)]
    fn render(&mut self, cx: RenderContext, primitives: &mut PrimitiveGroup) {}

    /// A unique identifier for the optional global render cache.
    ///
    /// All instances of this element type must return the same value.
    fn global_render_cache_id(&self) -> Option<u32> {
        None
    }

    /// An optional struct that is shared across all instances of this element type
    /// which can be used to cache rendering primitives.
    ///
    /// This will only be called once at the creation of the first instance of this
    /// element type.
    fn global_render_cache(&self) -> Option<Box<dyn ElementRenderCache>> {
        None
    }
}

pub trait ElementRenderCache {
    fn pre_render(&mut self) {}
    fn post_render(&mut self) {}

    fn get_mut(&mut self) -> &mut Box<dyn Any>;
}

pub struct ElementBuilder<A: Clone + 'static> {
    pub element: Box<dyn Element<A>>,
    pub z_index: ZIndex,
    pub rect: Rect,
    pub manually_hidden: bool,
    pub scissor_rect: ScissorRectID,
    pub class: ClassID,
}

impl<A: Clone + 'static> ElementBuilder<A> {
    pub const fn new(element: Box<dyn Element<A>>) -> Self {
        Self {
            element,
            z_index: 0,
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 0.0)),
            manually_hidden: false,
            scissor_rect: ScissorRectID::DEFAULT,
            class: 0,
        }
    }

    pub const fn class(mut self, class: ClassID) -> Self {
        self.class = class;
        self
    }

    pub const fn z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = z_index;
        self
    }

    pub const fn rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    pub const fn hidden(mut self, hidden: bool) -> Self {
        self.manually_hidden = hidden;
        self
    }

    pub const fn scissor_rect(mut self, scissor_rect: ScissorRectID) -> Self {
        self.scissor_rect = scissor_rect;
        self
    }
}

pub trait ElementStyle: Default + Any {
    const ID: &'static str;

    fn default_dark_style() -> Self {
        Self::default()
    }

    fn default_light_style() -> Self {
        Self::default()
    }
}

pub(super) struct ElementModification {
    pub element_id: ElementID,
    pub type_: ElementModificationType,
}

pub(super) enum ElementModificationType {
    CustomStateChanged,
    MarkDirty,
    RectChanged(Rect),
    ScissorRectChanged,
    ZIndexChanged(ZIndex),
    ExplicitlyHiddenChanged(bool),
    ClassChanged(ClassID),
    SetAnimating(bool),
    ChangeFocus(ChangeFocusRequest),
    HandleDropped,
    ListenToClickOff,
    StartHoverTimeout,
    StartScrollWheelTimeout,
    ShowTooltip { data: TooltipData, auto_hide: bool },
    UpdateScissorRect(UpdateScissorRectRequest),
}

// I get a warning about leaking `ElementID` if I make `ElementHandle::new()`
// have `public(crate)` visibility, so this is a workaround.
pub(super) fn new_element_handle(
    element_id: ElementID,
    mod_queue_sender: stmpsc_queue::Sender<ElementModification>,
    rect: Rect,
    z_index: ZIndex,
    manually_hidden: bool,
    class: ClassID,
) -> ElementHandle {
    ElementHandle::new(
        element_id,
        mod_queue_sender,
        rect,
        z_index,
        manually_hidden,
        class,
    )
}
