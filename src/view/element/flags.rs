bitflags::bitflags! {
    /// The flags describing this element.
    ///
    /// Yarrow uses these flags to optimize elements.
    ///
    /// By default all these flags are disabled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ElementFlags: u16 {
        /// Whether or not this element paints anything to the screen.
        const PAINTS = 1 << 0;

        /// Whether or not this element paints to the screen using custom shaders.
        ///
        /// This has no effect if the `ElementFlags::Paints` flag is not set.
        const USES_CUSTOM_SHADERS = 1 << 1;

        /// Whether or not this element listens to pointer events when the pointer is
        /// within the assigned rectangular area of the element.
        const LISTENS_TO_POINTER_INSIDE_BOUNDS = 1 << 2;

        /// Whether or not this element should receive an event when it becomes hidden
        /// or when it becomes visible.
        ///
        /// An element is considered "hidden" if one or more of these scenarios is true:
        /// * The user has manually hidden the element through the element's handle.
        /// * The element lies outside of the screen bounds or its assigned scissoring
        /// rectangle.
        const LISTENS_TO_VISIBILITY_CHANGE = 1 << 3;

        /// Whether or not this element should receive an event when the size of the
        /// element's assigned rectangular area has changed.
        const LISTENS_TO_SIZE_CHANGE = 1 << 4;

        /// Whether or not this element should receive an event when its position
        /// changes.
        ///
        /// This can be useful for example making sure a drop-down menu stays within
        /// the bounds of the window.
        const LISTENS_TO_POSITION_CHANGE = 1 << 5;

        /// Whether or not this element should receive an event when it gets exclusive
        /// focus or when its exclusive focus is released.
        const LISTENS_TO_FOCUS_CHANGE = 1 << 6;

        /// Whether or not this element should receive an event when the z index is
        /// changed.
        const LISTENS_TO_Z_INDEX_CHANGE = 1 << 7;

        /// Whether or not `Element::on_dropped()` should be called when this element
        /// is dropped.
        const LISTENS_TO_ON_DROPPED = 1 << 8;

        /// Whether or not this element should receive pointer events when it has
        /// exclusive focus, even when the pointer is outside the assigned rectangular
        /// area of the element.
        const LISTENS_TO_POINTER_OUTSIDE_BOUNDS_WHEN_FOCUSED = 1 << 9;

        /// Whether or not this element should receive text composition events when it
        /// has exclusive focus.
        const LISTENS_TO_TEXT_COMPOSITION_WHEN_FOCUSED = 1 << 10;

        /// Whether or not this element should receive raw keyboard events when it has
        /// exclusive focus.
        const LISTENS_TO_KEYS_WHEN_FOCUSED = 1 << 11;

        /// Whether or not this element should receive an `init` event when it gets
        /// added to the view.
        const LISTENS_TO_INIT = 1 << 12;
    }
}
