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

use std::sync::mpsc;

use rootvg::math::{Point, Size};

use crate::action_queue::ActionSender;
use crate::clipboard::Clipboard;
use crate::layout::Align2;
use crate::math::{Rect, ScaleFactor, ZIndex};
use crate::prelude::{ClassID, ResourceCtx};
use crate::{CursorIcon, WindowID};

use super::ElementRenderCache;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChangeFocusRequest {
    StealFocus,
    StealTemporaryFocus,
    ReleaseFocus,
}

pub(crate) struct ShowTooltipRequest {
    pub message: String,
    pub align: Align2,
    pub auto_hide: bool,
}

/// A context for this element instance. This is used to request actions from the
/// UI library.
pub struct ElementContext<'a, A: Clone + 'static> {
    /// The cursor icon. Mutate this to change the cursor icon.
    ///
    /// The icon is reset once the cursor moves.
    pub cursor_icon: CursorIcon,
    /// A sender for the action queue.
    pub action_sender: &'a mut ActionSender<A>,
    /// The global resource context.
    pub res: &'a mut ResourceCtx,
    /// The system clipboard.
    pub clipboard: &'a mut Clipboard,

    pub(crate) listen_to_pointer_clicked_off: bool,
    pub(crate) requested_rect: Option<Rect>,
    pub(crate) requested_show_tooltip: Option<ShowTooltipRequest>,
    pub(crate) change_focus_request: Option<ChangeFocusRequest>,

    pub(crate) rect: Rect,
    pub(crate) visible_rect: Option<Rect>,
    pub(crate) window_size: Size,
    pub(crate) z_index: ZIndex,
    pub(crate) manually_hidden: bool,
    pub(crate) animating: bool,
    pub(crate) repaint_requested: bool,
    pub(crate) has_focus: bool,
    pub(crate) hover_timeout_requested: bool,
    pub(crate) scroll_wheel_timeout_requested: bool,
    pub(crate) scale_factor: ScaleFactor,
    pub(crate) window_id: WindowID,
    pub(crate) pointer_lock_request: Option<bool>,
    pointer_locked: bool,
    class: ClassID,
}

impl<'a, A: Clone + 'static> ElementContext<'a, A> {
    pub(crate) fn new(
        rect: Rect,
        visible_rect: Option<Rect>,
        window_size: Size,
        z_index: ZIndex,
        manually_hidden: bool,
        animating: bool,
        has_focus: bool,
        scale_factor: ScaleFactor,
        cursor_icon: CursorIcon,
        window_id: WindowID,
        pointer_locked: bool,
        class: ClassID,
        action_sender: &'a mut ActionSender<A>,
        res: &'a mut ResourceCtx,
        clipboard: &'a mut Clipboard,
    ) -> Self {
        Self {
            cursor_icon,
            action_sender,
            res,
            rect,
            visible_rect,
            window_size,
            z_index,
            manually_hidden,
            animating,
            repaint_requested: false,
            has_focus,
            scale_factor,
            window_id,
            pointer_lock_request: None,
            pointer_locked,
            listen_to_pointer_clicked_off: false,
            hover_timeout_requested: false,
            scroll_wheel_timeout_requested: false,
            requested_rect: None,
            requested_show_tooltip: None,
            change_focus_request: None,
            class,
            clipboard,
        }
    }

    /// The rectangular area assigned to this element instance.
    ///
    /// Note, the rectangle may have a position and size of zero if the element
    /// has yet to be laid out.
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// The visible rectangular area, accounting for the scissoring rectangle that
    /// this element belongs to.
    ///
    /// If the element was manually hidden or if it lies outside the bounds of the
    /// scissoring rectangle, this will return `None`.
    pub fn visible_rect(&self) -> Option<Rect> {
        self.visible_rect
    }

    /// The size of the window. This can be useful to reposition/resize elements
    /// like drop-down menus to fit within the window.
    pub fn window_size(&self) -> Size {
        self.window_size
    }

    /// If the element was manually hidden or if it lies outside the bounds of the
    /// scissoring rectangle, then this will return `false`.
    pub fn visible(&self) -> bool {
        self.visible_rect.is_some()
    }

    /// Returns whether or not this element is currently visible.

    /// The z index of this element instance.
    pub fn z_index(&self) -> ZIndex {
        self.z_index
    }

    /// Whether or not the user manually set this element instance to be hidden
    /// via this element's handle.
    ///
    /// Note this differs from `ElementContext::is_visible()` in that this element
    /// may still be invisible due to it being outside of the render area.
    pub fn manually_hidden(&self) -> bool {
        self.manually_hidden
    }

    /// Whether or not this element instance is currently receiving the animation
    /// event.
    pub fn is_animating(&self) -> bool {
        self.animating
    }

    /// Returns `true` if this element currenly has focus, `false` otherwise.
    pub fn has_focus(&self) -> bool {
        self.has_focus
    }

    /// Request to repaint this element this frame.
    ///
    /// This will also cause all child elements to be repainted.
    pub fn request_repaint(&mut self) {
        self.repaint_requested = true;
    }

    /// Set/unset whether this element should receive the animation event. The
    /// animation event is sent every frame just before rendering begins.
    ///
    /// Once the element instance is done animating, prefer to unset this to save on
    /// system resources.
    ///
    /// By default every newly created element instance does not listen to this
    /// event.
    pub fn set_animating(&mut self, animating: bool) {
        self.animating = animating;
    }

    /// Request to steal focus.
    ///
    /// If another element instance has focus, then that element will
    /// automatically be unfocused.
    ///
    /// By default every newly created element does not have focus.
    pub fn steal_focus(&mut self) {
        self.change_focus_request = Some(ChangeFocusRequest::StealFocus);
    }

    /// Request to temporarily steal focus.
    ///
    /// This is similar to `ElementContext::steal_focus()`, except that
    /// when this element has its focused released, the last element that had
    /// focus will be given its focus back.
    ///
    /// This can be useful, for example, a drop-down menu element or a scrollbar
    /// element to return focus back to a previously-focused text input.
    pub fn steal_temporary_focus(&mut self) {
        self.change_focus_request = Some(ChangeFocusRequest::StealTemporaryFocus);
    }

    /// Request to release focus.
    pub fn release_focus(&mut self) {
        self.change_focus_request = Some(ChangeFocusRequest::ReleaseFocus);
    }

    /// The current scale factor.
    pub fn scale_factor(&self) -> ScaleFactor {
        self.scale_factor
    }

    /// Schedule this element to recieve an `ElementEvent::ClickedOff` event when
    /// one of the following happens:
    /// * The user clicks outside the bounds of this element.
    /// * An element steals focus.
    /// * The window is unfocused.
    ///
    /// This is useful, for example, hiding a drop-down menu.
    ///
    /// Note, for performance reasons, only call this method once whenever this
    /// needs to be used, i.e. when the drop-down menu is shown.
    pub fn listen_to_pointer_clicked_off(&mut self) {
        self.listen_to_pointer_clicked_off = true;
    }

    pub fn is_point_within_visible_bounds(&self, point: Point) -> bool {
        self.visible_rect
            .map(|r| r.contains(point))
            .unwrap_or(false)
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.requested_rect = Some(rect);
    }

    pub fn send_action(&mut self, action: impl Into<A>) -> Result<(), mpsc::SendError<A>> {
        self.action_sender.send(action)
    }

    pub fn start_hover_timeout(&mut self) {
        self.hover_timeout_requested = true;
    }

    pub fn start_scroll_wheel_timeout(&mut self) {
        self.scroll_wheel_timeout_requested = true;
    }

    pub fn show_tooltip(&mut self, message: String, align: Align2, auto_hide: bool) {
        self.requested_show_tooltip = Some(ShowTooltipRequest {
            message,
            align,
            auto_hide,
        });
    }

    /// The ID of the window this element belongs to.
    pub fn window_id(&self) -> WindowID {
        self.window_id
    }

    /// Request to lock/unlock the pointer in place and hide the cursor.
    ///
    /// The application and/or backend may choose to ignore this request.
    ///
    /// The pointer will automatically be unlocked when this element
    /// loses focus.
    pub fn request_pointer_lock(&mut self, lock: bool) {
        self.pointer_lock_request = Some(lock);
    }

    /// Whether or not the pointer is currently locked in place.
    pub fn is_pointer_locked(&self) -> bool {
        self.pointer_locked
    }

    /// The current class ID.
    pub fn class(&self) -> ClassID {
        self.class
    }
}

/// A context for this element instance for use in rendering primitives.
pub struct RenderContext<'a> {
    /// The font system.
    pub res: &'a mut ResourceCtx,
    /// The size of this element's bounding rectangle.
    pub bounds_size: Size,
    /// The origin of the element's bounding rectangle. This is normally not needed
    /// since the view automatically applies this offset to all primitives.
    pub bounds_origin: Point,
    /// The visible rectangular area, accounting for the scissoring rectangle that
    /// this element belongs to.
    pub visible_bounds: Rect,
    /// The scale factor.
    pub scale: ScaleFactor,
    /// The current class ID.
    pub class: ClassID,
    /// The size of the window. This can be useful to reposition/resize elements
    /// like drop-down menus to fit within the window.
    pub window_size: Size,
    /// The optional global render cache.
    pub render_cache: Option<&'a mut Box<dyn ElementRenderCache>>,
}
