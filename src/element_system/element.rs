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
use crate::style::ClassID;
use crate::{stmpsc_queue, WindowContext};

pub(crate) use context::ChangeFocusRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ElementID(pub thunderdome::Index);

pub trait Element<A: Clone + 'static> {
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
    pub flags: ElementFlags,
}

impl<A: Clone + 'static> ElementBuilder<A> {
    pub fn new(element: impl Element<A> + 'static) -> Self {
        Self {
            element: Box::new(element),
            z_index: 0,
            rect: Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 0.0)),
            manually_hidden: false,
            scissor_rect: ScissorRectID::DEFAULT,
            class: 0,
            flags: ElementFlags::empty(),
        }
    }

    pub fn builder_values(
        mut self,
        z_index: Option<ZIndex>,
        scissor_rect: Option<ScissorRectID>,
        class: Option<ClassID>,
        window_cx: &mut WindowContext<A>,
    ) -> Self {
        self.z_index = z_index.unwrap_or_else(|| window_cx.z_index());
        self.scissor_rect = scissor_rect.unwrap_or_else(|| window_cx.scissor_rect());
        self.class = class.unwrap_or_else(|| window_cx.class());
        self
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

    pub const fn flags(mut self, flags: ElementFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn build(self, window_cx: &mut WindowContext<A>) -> ElementHandle {
        window_cx.add_element(self)
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
