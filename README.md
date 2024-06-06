[![Documentation](https://docs.rs/yarrow/badge.svg)](https://docs.rs/yarrow)
[![Crates.io](https://img.shields.io/crates/v/yarrow.svg)](https://crates.io/crates/yarrow)
[![License](https://img.shields.io/crates/l/yarrow.svg)](https://github.com/MeadowlarkDAW/Yarrow/blob/main/LICENSE)

<div align="center">

# Yarrow

> **WORK IN PROGRESS. This library is currently in an unstable pre-alpha state and is currently missing features. Check the [roadmap] for more details.**

```
'%%' '%% '%%'
%'%\% | %/%'%
    \ | /    
     \|/     
      |      
```

**A non-declarative GUI library in Rust with extreme performance and control, geared towards audio software.**

![gallery screenshot](screenshots/gallery-basic-elements.png)

</div>

# What to Expect

The goal of Yarrow is different from most other modern GUI libraries. Instead of aiming for an "elegant" declarative API, Yarrow aims to provide a powerful yet easy-to-use retained-mode API with lots of control over how elements are styled, laid-out, interacted with, and rendered *(\*easy compared to other retained-mode frameworks)*. This library is not optimized for quick prototyping, and instead works best for applications which already have a design mocked up.

Yarrow does not aim to be "general purpose" GUI library. Only features that are needed for the [Meadowlark DAW Project](https://github.com/MeadowlarkDAW/Meadowlark) and its audio plugins are planned.

# Features

* Cross-platform (Linux, Mac, and Windows)
* Native and lightweight
* Hardware-accelerated rendering in [wgpu] with support for text, vector graphics, textures, and/or custom shaders
* Extreme performance (you are in control of how your elements are updated)
* Can be used for both standalone applications and audio plugins
* Designed from the ground-up to support multi-windowed applications
* Scaling support (with built-in support for hi-dpi texture assets)
* [Pointer locking](https://developer.mozilla.org/en-US/docs/Web/API/Pointer_Lock_API) for knob and slider elements
* Accessibility support*
* [Permissive MIT license](./LICENSE)

> \*Yarrow does not automatically set up accessibility for you. You must manually provide Yarrow with information on how accessibility tools should navigate your program.

# How it Works

* There is no "widget tree", and widgets cannot contain other widgets. There are only elements (widgets), z-indexes, and scissoring rectangles. Each element has a manually-defined z index and bounding rectangle, and each element belongs to a single scissoring rectangle.
* Because there are no "parent widgets", there is no system for composing elements out of multiple simpler elements. Elements in Yarrow tend to be "monolithic" (i.e. a `DropDownMenu` is a single element.)
* Yarrow is not declarative. Elements are added and removed dynamically at runtime. Each newly created element returns a handle to that element.
* Yarrow uses an event-driven update system. Elements (widgets) cannot mutate application state, they can only send actions (events). All actions are sent to a single monolithic user-defined action handler method. Inside this method, the user manually mutates the state and updates elements accordingly.
* There is no layout system. Elements and scissoring rectangles are laid out manually with user-defined functions. Elements are only updated if their position/layout has changed, allowing for a sort of immediate-mode style layout with little performance overhead.
* There is no "cascading" styling systems, instead each element defines its own custom style struct.
* Rendering is done in [wgpu] using the [RootVG](https://github.com/MeadowlarkDAW/rootvg) library. Elements can also be rendered using custom wgpu shaders.
* Scissoring rectangles have an additional "offset" vector which can be used to create scrolling regions.

# Get Started

To get started, read the [book] (TODO).

> This repository only houses the GUI library. For examples and guides on how to use Yarrow for audio plugin development, see (TODO).

# Contributing

(TODO)

[wgpu]: https://wgpu.rs
[roadmap]: ROADMAP.md
