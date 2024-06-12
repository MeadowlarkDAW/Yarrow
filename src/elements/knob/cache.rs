//! Since it is very likely that there will be multiple knob instances that share
//! the same style, use a render cache to re-use expensive mesh primitives
//! across instances.

use std::{any::Any, hash::Hash, rc::Rc};

use rustc_hash::FxHashMap;

use crate::{prelude::KnobNotchStyle, view::element::ElementRenderCache};

use super::{KnobNotchLinePrimitives, KnobStyle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct KnobNotchLineCacheKey {
    style_ptr: usize,
    back_size: i32,
}

#[derive(Default)]
pub struct KnobRenderCacheInner {
    notch_line_meshes: FxHashMap<KnobNotchLineCacheKey, (KnobNotchLinePrimitives, bool)>,
}

impl KnobRenderCacheInner {
    pub fn pre_render(&mut self) {
        for entry in self.notch_line_meshes.values_mut() {
            entry.1 = false;
        }
    }

    pub fn post_render(&mut self) {
        self.notch_line_meshes.retain(|_, (_, active)| *active);
    }

    pub fn get_notch_line_mesh(
        &mut self,
        style: &Rc<KnobStyle>,
        back_size: f32,
    ) -> Option<&KnobNotchLinePrimitives> {
        let KnobNotchStyle::Line(notch_style) = &style.notch else {
            return None;
        };

        let key = KnobNotchLineCacheKey {
            style_ptr: Rc::as_ptr(style) as usize,
            back_size: back_size.round() as i32,
        };

        let entry = self
            .notch_line_meshes
            .entry(key)
            .or_insert_with(|| (KnobNotchLinePrimitives::new(notch_style, back_size), true));
        // Mark that this cache entry is active.
        entry.1 = true;

        Some(&entry.0)
    }
}

pub struct KnobRenderCache {
    cache: Box<dyn Any>,
}

impl KnobRenderCache {
    pub const ID: u32 = 2647228533;

    pub fn new() -> Self {
        Self {
            cache: Box::new(KnobRenderCacheInner::default()),
        }
    }
}

impl ElementRenderCache for KnobRenderCache {
    fn pre_render(&mut self) {
        if let Some(cache) = self.cache.downcast_mut::<KnobRenderCacheInner>() {
            cache.pre_render();
        }
    }

    fn post_render(&mut self) {
        if let Some(cache) = self.cache.downcast_mut::<KnobRenderCacheInner>() {
            cache.post_render();
        }
    }

    fn get_mut(&mut self) -> &mut Box<dyn std::any::Any> {
        &mut self.cache
    }
}
