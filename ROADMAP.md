# Yarrow Development Roadmap

*(The following list is in no particular order.)*

- [x] Paragraph element
- [ ] Separator element
- [ ] Progress bar element
- [x] Multi-window example in gallery demo
- [ ] Implement [pointer-locking](https://developer.mozilla.org/en-US/docs/Web/API/Pointer_Lock_API) support
- [ ] "Virtual Slider" elements:
    - [x] Knob element
    - [x] Slider element (modern style)
    - [ ] Slider element (classic style)
    - [ ] TextureKnob element
    - [ ] TextureSlider element
    - [ ] Number spinner element
    - [ ] Ramp element
    - [ ] XYPad element
    - [ ] ModulationArc element
    - [ ] ModulationLine element
- [ ] Custom icon in TextInput element (to create a search bar)
- [ ] Drag-n-drop source and target elements
- [ ] TextureQuad element
- [ ] TextureButton element
- [ ] TextureToggleButton element
- [ ] Audio visualization elements:
    - [ ] DBMeter element
- [ ] Scrolling in long drop-down menus
- [ ] Helper to create a scrollable list of toggle buttons (i.e. for creating a preset selector)
- [ ] Logo
- [ ] [baseview](https://github.com/RustAudio/baseview) backend for audio plugins
- [ ] Audio plugin example

## Planned, but lower priority

- [ ] ControlSpline element (i.e. for creating automation clips and ADSR controls)
- [ ] Audio visualization elements:
    - [ ] Spectrometer element
    - [ ] Waveform element
    - [ ] Spectrogram element
    - [ ] ScrollingWaveform element
    - [ ] ScrollingGraph element
    - [ ] Goniometer element
- [ ] ParametricEqControl element
- [ ] RangeSlider element
- [ ] Nested drop-down menus
- [ ] Ellipses on text that is cut off
- [ ] Custom element example
- [ ] Custom shader example
- [ ] Drop shadow element
- [ ] PianoKeys element
- [ ] ModWheel element (an element that emulates a pitch wheel/modulation wheel on a MIDI keyboard)
- [ ] Better documentation
- [ ] Keyboard navigation
- [ ] Accessibility support
- [ ] Transition animations in common elements

## "Maybe" future ideas

*The following are features I would like to have, but are not strictly needed for Meadowlark, so they will only be supported if someone else implements them or if there is enough demand/financial support for them.*

- [ ] Official book
- [ ] C/C++ bindings. Yarrow does not rely on any Rust type system wizardry, so bindings should be feasible.
    - [ ] A Rust/C/C++/Zig compatible CLAP/AU/VST3 plugin development framework using Yarrow for GUI. The idea would be to create an MIT-licensed alternative to JUCE. This is actually an idea I've been thinking of for a while, although for now I'd rather focus on my [Meadowlark](https://github.com/MeadowlarkDAW/Meadowlark) DAW project. That being said, if you are a company that would be willing to sponsor this JUCE-alternative idea, let me know on Discord or email me at billydm@anonaddy.me and I'll consider going through with it.
- [ ] Multi-line text editor element
- [ ] TreeMenu element
- [ ] A declarative data-driven wrapper around Yarrow (much like what the [Relm4](https://github.com/Relm4/Relm4) library does around the GTK4 library)