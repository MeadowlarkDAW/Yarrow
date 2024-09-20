use crate::math::{Point, Rect, Scale, Size, Vector};
use crate::{
    action::{Action, ActionSender},
    clipboard::Clipboard,
    style_system::ClassID,
    window::WindowID,
    CursorIcon, ResourceContext, ZIndex,
};
use crate::{ScissorRectID, TooltipData};

use super::mod_queue::{ElementModification, ElementModificationType, ModQueueSender};
use super::ElementID;

/// A context for this element instance.
pub struct ElementContext<'a, A: Action> {
    pub action_sender: &'a mut ActionSender<A>,
    /// The global resource context.
    pub res: &'a mut ResourceContext,
    /// The system clipboard.
    pub clipboard: &'a mut Box<dyn Clipboard>,
    /// The cursor icon. Mutate this to change the cursor icon.
    ///
    /// The icon is reset once the cursor moves.
    pub cursor_icon: CursorIcon,

    mod_queue: &'a mut ModQueueSender,

    rect: Rect,
    visible_rect: Option<Rect>,
    element_id: ElementID,
    window_id: WindowID,
    window_size: Size,
    scale_factor: f32,
    z_index: ZIndex,
    manually_hidden: bool,
    animating: bool,
    has_pointer_focus: bool,
    has_keyboard_focus: bool,
    pointer_locked: bool,
    class: ClassID,
}

impl<'a, A: Action> ElementContext<'a, A> {
    pub fn new(
        action_sender: &'a mut ActionSender<A>,
        res: &'a mut ResourceContext,
        clipboard: &'a mut Box<dyn Clipboard>,
        mod_queue: &'a mut ModQueueSender,
        rect: Rect,
        visible_rect: Option<Rect>,
        element_id: ElementID,
        window_id: WindowID,
        window_size: Size,
        scale_factor: f32,
        z_index: ZIndex,
        manually_hidden: bool,
        animating: bool,
        has_pointer_focus: bool,
        has_keyboard_focus: bool,
        pointer_locked: bool,
        class: ClassID,
        cursor_icon: CursorIcon,
    ) -> Self {
        Self {
            action_sender,
            res,
            clipboard,
            mod_queue,
            rect,
            visible_rect,
            element_id,
            window_id,
            window_size,
            scale_factor,
            z_index,
            manually_hidden,
            animating,
            has_pointer_focus,
            has_keyboard_focus,
            pointer_locked,
            class,
            cursor_icon,
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

    /// Returns whether or not this element is currently visible.
    ///
    /// If the element was manually hidden or if it lies outside the bounds of the
    /// scissoring rectangle, then this will return `false`.
    pub fn visible(&self) -> bool {
        self.visible_rect.is_some()
    }

    /// The z index of this element instance.
    pub fn z_index(&self) -> ZIndex {
        self.z_index
    }

    /// Whether or not the user manually set this element instance to be hidden
    /// via this element's handle.
    ///
    /// Note this differs from [`ElementContext::visible()`] in that this element
    /// may still be invisible due to it being outside of the render area.
    pub fn manually_hidden(&self) -> bool {
        self.manually_hidden
    }

    /// Whether or not this element instance is currently receiving the animation
    /// event.
    pub fn is_animating(&self) -> bool {
        self.animating
    }

    /// Returns `true` if this element currenly has exclusive pointer focus,
    /// `false` otherwise.
    pub fn has_pointer_focus(&self) -> bool {
        self.has_pointer_focus
    }

    /// Returns `true` if this element currenly has exclusive keyboard focus,
    /// `false` otherwise.
    pub fn has_keyboard_focus(&self) -> bool {
        self.has_keyboard_focus
    }

    /// The current scale factor in pixels per point.
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub fn is_point_within_visible_bounds(&self, point: Point) -> bool {
        self.visible_rect
            .map(|r| r.contains(point))
            .unwrap_or(false)
    }

    /// The ID of the window this element belongs to.
    pub fn window_id(&self) -> WindowID {
        self.window_id
    }

    /// Whether or not the pointer is currently locked in place.
    pub fn is_pointer_locked(&self) -> bool {
        self.pointer_locked
    }

    /// The current class ID.
    pub fn class(&self) -> ClassID {
        self.class
    }

    /// The current cursor icon.
    pub fn cursor_icon(&self) -> CursorIcon {
        self.cursor_icon
    }

    pub fn send_action(&mut self, action: impl Into<A>) -> Result<(), crate::action::SendError<A>> {
        self.action_sender.send(action)
    }

    /// Request to repaint this element this frame.
    ///
    /// This will also cause all child elements to be repainted.
    pub fn repaint(&mut self) {
        self.send_modification(ElementModificationType::MarkDirty);
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
        self.send_modification(ElementModificationType::SetAnimating(animating));
    }

    /// Request to steal keyboard focus.
    ///
    /// If another element instance has keyboard focus, then that element will
    /// automatically be unfocused.
    ///
    /// * `temporary` - If set to `true`, then the element that previously had
    /// keyboard focus will automatically regain focus after this element
    /// releases focus with [`ElementContext::relase_keyboard_focus`].
    ///
    /// By default every newly created element does not have keyboard focus.
    pub fn steal_keyboard_focus(&mut self, temporary: bool) {
        self.send_modification(ElementModificationType::StealKeyboardFocus { temporary });
    }

    /// Release keyboard focus.
    pub fn relase_keyboard_focus(&mut self) {
        self.send_modification(ElementModificationType::ReleaseKeyboardFocus);
    }

    /// Request to steal pointer focus.
    ///
    /// If another element instance has pointer focus, then that element will
    /// automatically be unfocused.
    pub fn steal_pointer_focus(&mut self) {
        self.send_modification(ElementModificationType::StealPointerFocus);
    }

    /// Release pointer focus.
    pub fn release_pointer_focus(&mut self) {
        self.send_modification(ElementModificationType::ReleasePointerFocus);
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
        self.send_modification(ElementModificationType::ListenToClickOff);
    }

    /// Request to lock/unlock the pointer in place and hide the cursor.
    ///
    /// The application and/or backend may choose to ignore this request.
    ///
    /// The pointer will automatically be unlocked when this element
    /// loses focus.
    pub fn request_pointer_lock(&mut self, _lock: bool) {
        todo!()
    }

    pub fn set_rect(&mut self, rect: Rect) {
        self.send_modification(ElementModificationType::RectChanged(rect));
    }

    pub fn start_hover_timeout(&mut self) {
        self.send_modification(ElementModificationType::StartHoverTimeout);
    }

    pub fn start_scroll_wheel_timeout(&mut self) {
        self.send_modification(ElementModificationType::StartScrollWheelTimeout);
    }

    pub fn show_tooltip(&mut self, data: TooltipData, auto_hide: bool) {
        self.send_modification(ElementModificationType::ShowTooltip { data, auto_hide });
    }

    /// Update the given scissoring rectangle with the given values.
    ///
    /// If `new_rect` or `new_scroll_offset` is `None`, then the
    /// current respecting value will not be changed.
    ///
    /// This will *NOT* trigger an update unless the value has changed,
    /// so this method is relatively cheap to call frequently.
    ///
    /// If a scissoring rectangle with the given ID does not exist, then
    /// one will be created.
    ///
    /// If `id == ScissorRectID::DEFAULT`, then this will do nothing.
    pub fn update_scissor_rect(
        &mut self,
        id: ScissorRectID,
        new_rect: Option<Rect>,
        new_scroll_offset: Option<Vector>,
    ) {
        self.send_modification(ElementModificationType::UpdateScissorRect {
            id,
            new_rect,
            new_scroll_offset,
        });
    }

    fn send_modification(&mut self, type_: ElementModificationType) {
        self.mod_queue
            .send(ElementModification {
                element_id: self.element_id,
                type_,
            })
            .unwrap();
    }
}

/// A context for this element instance for use in rendering primitives.
pub struct RenderContext<'a> {
    /// The global resource context
    pub res: &'a mut ResourceContext,
    /// The size of this element's bounding rectangle.
    pub bounds_size: Size,
    /// The origin of the element's bounding rectangle. This is normally not needed
    /// since the view automatically applies this offset to all primitives.
    pub bounds_origin: Point,
    /// The visible rectangular area, accounting for the scissoring rectangle that
    /// this element belongs to.
    pub visible_bounds: Rect,
    /// The scale factor.
    pub scale: Scale,
    /// The current class ID.
    pub class: ClassID,
    /// The size of the window. This can be useful to reposition/resize elements
    /// like drop-down menus to fit within the window.
    pub window_size: Size,
}
