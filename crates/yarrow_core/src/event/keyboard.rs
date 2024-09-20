pub use keyboard_types::{Code, CompositionEvent, CompositionState, KeyState, Location, Modifiers};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeyboardEvent {
    /// Whether the key is pressed or released.
    pub state: KeyState,
    /// Physical key code.
    pub code: Code,
    /// The native key code if the physical code could not be determined.
    pub native_code: NativeKey,
    /// Location for keys with multiple instances on common keyboards.
    pub location: Location,
    /// Flags for pressed modifier keys.
    pub modifiers: Modifiers,
    /// True if the key is currently auto-repeated.
    pub repeat: bool,
    /// Events with this flag should be ignored in a text editor
    /// and instead composition events should be used.
    pub is_composing: bool,
}

/// Contains the platform-native logical key identifier
///
/// Exactly what that means differs from platform to platform, but the values are to some degree
/// tied to the currently active keyboard layout. The same key on the same keyboard may also report
/// different values on different platforms, which is one of the reasons this is a per-platform
/// enum.
///
/// This enum is primarily used to store raw keysym when Winit doesn't map a given native logical
/// key identifier to a meaningful [`Key`] variant. This lets you use [`Key`], and let the user
/// define keybinds which work in the presence of identifiers we haven't mapped for you yet.
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NativeKey {
    #[default]
    Unidentified,
    /// An Android "keycode", which is similar to a "virtual-key code" on Windows.
    Android(u32),
    /// A macOS "scancode". There does not appear to be any direct analogue to either keysyms or
    /// "virtual-key" codes in macOS, so we report the scancode instead.
    MacOS(u16),
    /// A Windows "virtual-key code".
    Windows(u16),
    /// An XKB "keysym".
    Xkb(u32),
}
