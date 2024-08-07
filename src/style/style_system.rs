use std::any::Any;
use std::rc::Rc;

use ahash::AHashMap;

use crate::view::element::ElementStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    element_type_id: &'static str,
    class: &'static str,
    is_dark_theme: bool,
}

pub struct StyleSystem {
    styles: AHashMap<Key, Rc<dyn Any>>,
    pub(crate) use_dark_theme: bool,
}

impl StyleSystem {
    pub fn new(use_dark_theme: bool) -> Self {
        Self {
            styles: AHashMap::default(),
            use_dark_theme,
        }
    }

    pub fn is_using_dark_theme(&self) -> bool {
        self.use_dark_theme
    }

    /// Insert a new style with the given class name for the given element type.
    ///
    /// Returns `true` if this style existed before and has been overwritten.
    pub fn add<T: ElementStyle>(
        &mut self,
        class: &'static str,
        is_dark_theme: bool,
        style: T,
    ) -> bool {
        self.styles
            .insert(
                Key {
                    element_type_id: T::ID,
                    class,
                    is_dark_theme,
                },
                Rc::new(style),
            )
            .is_some()
    }

    /// Remove a style from the system.
    ///
    /// Returns `true` if the style existed.
    pub fn remove<T: ElementStyle>(&mut self, class: &'static str, is_dark_theme: bool) -> bool {
        self.styles
            .remove(&Key {
                element_type_id: T::ID,
                class,
                is_dark_theme,
            })
            .is_some()
    }

    /// Get the style from the system.
    ///
    /// If the style doesn't exist in the system, the default style will be
    /// inserted and returned.
    pub fn get<T: ElementStyle>(&mut self, class: &'static str) -> &T {
        let key = Key {
            element_type_id: T::ID,
            class,
            is_dark_theme: self.use_dark_theme,
        };

        let entry = self.styles.entry(key);
        let entry = entry.or_insert_with(|| {
            Rc::new(if self.use_dark_theme {
                T::default_dark_style()
            } else {
                T::default_light_style()
            })
        });

        entry.downcast_ref().unwrap()
    }

    /// Get an Rc pointer to the style from the system.
    ///
    /// If there are many instances of this element type and this element type
    /// frequently updates/animates, using a cached Rc pointer can be more
    /// performant than frequently calling [`StyleSystem::get`].
    ///
    /// The returned value is gauranteed to be of type `T`.
    ///
    /// If the style doesn't exist in the system, the default style will be
    /// inserted and returned.
    pub fn get_rc<T: ElementStyle>(&mut self, class: &'static str) -> Rc<dyn Any> {
        let key = Key {
            element_type_id: T::ID,
            class,
            is_dark_theme: self.use_dark_theme,
        };

        let entry = self.styles.entry(key);
        let entry = entry.or_insert_with(|| {
            Rc::new(if self.use_dark_theme {
                T::default_dark_style()
            } else {
                T::default_light_style()
            })
        });

        Rc::clone(&entry)
    }
}
