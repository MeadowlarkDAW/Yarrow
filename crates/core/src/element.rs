use std::any::Any;

pub mod context;
mod flags;
mod handle;
pub mod mod_queue;

pub use context::{ElementContext, RenderContext};
pub use flags::ElementFlags;
pub use handle::ElementHandle;

use crate::{
    action::{Action, ActionSender},
    event::{ElementEvent, EventCaptureStatus},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementID(pub thunderdome::Index);

pub trait Element<A: Action> {
    #[allow(unused)]
    fn on_event(&mut self, event: ElementEvent, cx: &mut ElementContext<A>) -> EventCaptureStatus {
        EventCaptureStatus::NotCaptured
    }

    #[allow(unused)]
    fn on_dropped(&mut self, action_sender: &mut ActionSender<A>) {}
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
