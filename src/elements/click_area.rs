use keyboard_types::Modifiers;
use rootvg::math::Point;

use crate::event::{ElementEvent, EventCaptureStatus, PointerButton, PointerEvent, PointerType};
use crate::layout::Align2;
use crate::math::{Rect, ZIndex};
use crate::view::element::{Element, ElementBuilder, ElementContext, ElementFlags, ElementHandle};
use crate::view::ScissorRectID;
use crate::window::WindowContext;
use crate::CursorIcon;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickAreaInfo {
    pub element_bounds: Rect,
    pub click_position: Point,
}

pub struct ClickAreaBuilder<A: Clone + 'static> {
    pub click_action: Option<Box<dyn FnMut(ClickAreaInfo) -> A>>,
    pub tooltip_message: Option<String>,
    pub tooltip_align: Align2,
    pub button: PointerButton,
    pub modifiers: Option<Modifiers>,
    pub click_count: usize,
    pub cursor_icon: Option<CursorIcon>,
    pub pointer_type: Option<PointerType>,
    pub rect: Rect,
    pub z_index: Option<ZIndex>,
    pub disabled: bool,
    pub scissor_rect_id: Option<ScissorRectID>,
}

impl<A: Clone + 'static> ClickAreaBuilder<A> {
    pub fn new() -> Self {
        Self {
            click_action: None,
            tooltip_message: None,
            tooltip_align: Align2::TOP_CENTER,
            button: PointerButton::Primary,
            modifiers: None,
            click_count: 0,
            cursor_icon: None,
            pointer_type: None,
            rect: Rect::default(),
            z_index: None,
            disabled: false,
            scissor_rect_id: None,
        }
    }

    pub fn build(self, cx: &mut WindowContext<'_, A>) -> ClickArea {
        ClickAreaElement::create(self, cx)
    }

    pub fn on_clicked<F: FnMut(ClickAreaInfo) -> A + 'static>(mut self, f: F) -> Self {
        self.click_action = Some(Box::new(f));
        self
    }

    pub fn tooltip_message(mut self, message: impl Into<String>, align: Align2) -> Self {
        self.tooltip_message = Some(message.into());
        self.tooltip_align = align;
        self
    }

    pub const fn button(mut self, button: PointerButton) -> Self {
        self.button = button;
        self
    }

    pub const fn modifiers(mut self, modifiers: Option<Modifiers>) -> Self {
        self.modifiers = modifiers;
        self
    }

    pub const fn click_count(mut self, count: usize) -> Self {
        self.click_count = count;
        self
    }

    pub const fn cursor_icon(mut self, icon: CursorIcon) -> Self {
        self.cursor_icon = Some(icon);
        self
    }

    pub const fn pointer_type(mut self, pointer_type: PointerType) -> Self {
        self.pointer_type = Some(pointer_type);
        self
    }

    pub const fn z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = Some(z_index);
        self
    }

    pub const fn rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn scissor_rect(mut self, scissor_rect_id: ScissorRectID) -> Self {
        self.scissor_rect_id = Some(scissor_rect_id);
        self
    }
}

pub struct ClickAreaElement<A: Clone + 'static> {
    click_action: Option<Box<dyn FnMut(ClickAreaInfo) -> A>>,
    tooltip_message: Option<String>,
    tooltip_align: Align2,
    button: PointerButton,
    modifiers: Option<Modifiers>,
    click_count: usize,
    cursor_icon: Option<CursorIcon>,
    pointer_type: Option<PointerType>,
}

impl<A: Clone + 'static> ClickAreaElement<A> {
    pub fn create(builder: ClickAreaBuilder<A>, cx: &mut WindowContext<'_, A>) -> ClickArea {
        let ClickAreaBuilder {
            click_action,
            tooltip_message,
            tooltip_align,
            button,
            modifiers,
            click_count,
            cursor_icon,
            pointer_type,
            rect,
            z_index,
            disabled,
            scissor_rect_id,
        } = builder;

        let z_index = z_index.unwrap_or_else(|| cx.z_index());
        let scissor_rect_id = scissor_rect_id.unwrap_or_else(|| cx.scissor_rect_id());

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                click_action,
                tooltip_message,
                tooltip_align,
                button,
                modifiers,
                click_count,
                cursor_icon,
                pointer_type,
            }),
            z_index,
            rect,
            manually_hidden: disabled,
            scissor_rect_id,
            class: "",
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        ClickArea { el }
    }
}

impl<A: Clone + 'static> Element<A> for ClickAreaElement<A> {
    fn flags(&self) -> ElementFlags {
        ElementFlags::LISTENS_TO_POINTER_INSIDE_BOUNDS
    }

    fn on_event(
        &mut self,
        event: ElementEvent,
        cx: &mut ElementContext<'_, A>,
    ) -> EventCaptureStatus {
        match event {
            ElementEvent::Pointer(PointerEvent::Moved {
                just_entered,
                position,
                ..
            }) => {
                if just_entered && self.tooltip_message.is_some() {
                    cx.start_hover_timeout();
                }

                if let Some(icon) = self.cursor_icon {
                    if cx.rect().contains(position) {
                        cx.cursor_icon = icon;
                    }
                }
            }
            ElementEvent::Pointer(PointerEvent::ButtonJustPressed {
                position,
                button,
                click_count,
                modifiers,
                pointer_type,
            }) => {
                if button != self.button {
                    return EventCaptureStatus::NotCaptured;
                }

                if let Some(m) = self.modifiers {
                    if modifiers != m {
                        return EventCaptureStatus::NotCaptured;
                    }
                }

                if self.click_count != 0 && click_count != self.click_count {
                    return EventCaptureStatus::NotCaptured;
                }

                if let Some(t) = self.pointer_type {
                    if pointer_type != t {
                        return EventCaptureStatus::NotCaptured;
                    }
                }

                if let Some(f) = &mut self.click_action {
                    let element_bounds = cx.rect();
                    cx.send_action((f)(ClickAreaInfo {
                        element_bounds,
                        click_position: position,
                    }))
                    .unwrap();

                    return EventCaptureStatus::Captured;
                }
            }
            ElementEvent::Pointer(PointerEvent::HoverTimeout { .. }) => {
                if let Some(message) = &self.tooltip_message {
                    cx.show_tooltip(message.clone(), self.tooltip_align, true);
                }
            }
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }
}

pub struct ClickArea {
    pub el: ElementHandle,
}

impl ClickArea {
    pub fn builder<A: Clone + 'static>() -> ClickAreaBuilder<A> {
        ClickAreaBuilder::new()
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.el.set_hidden(disabled);
    }
}
