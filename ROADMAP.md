# Yarrow Development Roadmap

*(The following list is in no particular order.)*

- [ ] Paragraph element
- [ ] Separator element
- [ ] Multi-window example in gallery demo
- [ ] "Virtual Slider" elements:
    - [ ] Knob element
    - [ ] Slider element
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
    - [ ] Spectrometer element
    - [ ] Waveform element
- [ ] ControlSpline element (i.e. for creating automation clips and ADSR controls)
- [ ] ModulationRangeSlider element
- [ ] Scrolling in long drop-down menus
- [ ] Helper to create a scrollable list of toggle buttons (i.e. for creating a preset selector)
- [ ] Logo

## Planned, but lower priority

- [ ] Progress bar element
- [ ] Audio visualization elements:
    - [ ] Spectrogram element
    - [ ] ScrollingWaveform element
    - [ ] ScrollingGraph element
    - [ ] Goniometer element
- [ ] ParametricEqControl element
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

## "Maybe" future ideas

*The following are features I would like to have, but are not strictly needed for Meadowlark, so they will only be supported if someone else implements them or if there is enough demand/financial support for them.*

- [ ] Official book
- [ ] C/C++ bindings. Yarrow does not rely on any Rust type system wizardry, so bindings should be feasible.
    - [ ] A Rust/C/C++/Zig compatible CLAP/VST3 plugin development framework using Yarrow for GUI. The idea would be to create a powerful MIT-licensed alternative to JUCE. This is actually an idea I've been thinking of for a while, so send me an email at billydm@anonaddy.me if you would be interested in sponsoring such a project.
- [ ] Multi-line text editor element
- [ ] A declarative data-driven wrapper around Yarrow (much like what the [Relm4](https://github.com/Relm4/Relm4) library does around the GTK4 library)