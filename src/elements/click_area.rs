use derive_where::derive_where;
use std::cell::RefCell;
use std::rc::Rc;

use crate::derive::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickAreaInfo {
    pub element_bounds: Rect,
    pub click_position: Point,
}

#[element_builder]
#[element_builder_rect]
#[element_builder_disabled]
#[element_builder_tooltip]
#[derive_where(Default)]
pub struct ClickAreaBuilder<A: Clone + 'static> {
    pub click_action: Option<Box<dyn FnMut(ClickAreaInfo) -> A>>,
    pub button: PointerButton,
    pub modifiers: Option<Modifiers>,
    pub click_count: usize,
    pub cursor_icon: Option<CursorIcon>,
    pub pointer_type: Option<PointerType>,
}

impl<A: Clone + 'static> ClickAreaBuilder<A> {
    pub fn build(self, cx: &mut WindowContext<'_, A>) -> ClickArea {
        ClickAreaElement::create(self, cx)
    }

    pub fn on_clicked<F: FnMut(ClickAreaInfo) -> A + 'static>(mut self, f: F) -> Self {
        self.click_action = Some(Box::new(f));
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
}

pub struct ClickAreaElement<A: Clone + 'static> {
    click_action: Option<Box<dyn FnMut(ClickAreaInfo) -> A>>,
    button: PointerButton,
    modifiers: Option<Modifiers>,
    click_count: usize,
    cursor_icon: Option<CursorIcon>,
    pointer_type: Option<PointerType>,
    shared_state: Rc<RefCell<SharedState>>,
}

impl<A: Clone + 'static> ClickAreaElement<A> {
    pub fn create(builder: ClickAreaBuilder<A>, cx: &mut WindowContext<'_, A>) -> ClickArea {
        let ClickAreaBuilder {
            click_action,
            tooltip_data,
            button,
            modifiers,
            click_count,
            cursor_icon,
            pointer_type,
            rect,
            z_index,
            disabled,
            scissor_rect,
        } = builder;

        let z_index = z_index.unwrap_or_else(|| cx.z_index());
        let scissor_rect = scissor_rect.unwrap_or_else(|| cx.scissor_rect());

        let shared_state = Rc::new(RefCell::new(SharedState {
            tooltip_inner: TooltipInner::new(tooltip_data),
        }));

        let element_builder = ElementBuilder {
            element: Box::new(Self {
                click_action,
                button,
                modifiers,
                click_count,
                cursor_icon,
                pointer_type,
                shared_state: Rc::clone(&shared_state),
            }),
            z_index,
            rect,
            manually_hidden: disabled,
            scissor_rect,
            class: Default::default(),
        };

        let el = cx
            .view
            .add_element(element_builder, &mut cx.res, cx.clipboard);

        ClickArea { el, shared_state }
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
        RefCell::borrow(&self.shared_state)
            .tooltip_inner
            .handle_event(&event, false, cx);

        match event {
            ElementEvent::Pointer(PointerEvent::Moved { position, .. }) => {
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
            _ => {}
        }

        EventCaptureStatus::NotCaptured
    }
}

struct SharedState {
    tooltip_inner: TooltipInner,
}

#[element_handle]
#[element_handle_set_rect]
#[element_handle_set_tooltip]
pub struct ClickArea {
    shared_state: Rc<RefCell<SharedState>>,
}

impl ClickArea {
    pub fn builder<A: Clone + 'static>() -> ClickAreaBuilder<A> {
        ClickAreaBuilder::default()
    }

    /// Set the disabled state of this element.
    ///
    /// Returns `true` if the disabled state has changed.
    ///
    /// This will *NOT* trigger an element update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    pub fn set_disabled(&mut self, disabled: bool) -> bool {
        self.el.set_hidden(disabled)
    }
}
