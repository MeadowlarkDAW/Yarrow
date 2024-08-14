use rootvg::{quad::QuadFlags, text::Metrics};

use crate::{
    prelude::*,
    theme::{DEFAULT_ACCENT_COLOR, DEFAULT_ACCENT_HOVER_COLOR},
};

pub const TEXT_PADDING: Padding = padding_vh(6.0, 7.0);
pub const ICON_PADDING: Padding = padding_vh(4.0, 5.0);
pub const TEXT_ICON_SPACING: f32 = -8.0;

pub const TEXT_COLOR: RGBA8 = gray_a(255, 200);
pub const TEXT_COLOR_BRIGHT: RGBA8 = gray_a(255, 255);
pub const TEXT_COLOR_DIMMED: RGBA8 = gray_a(255, 100);

pub const BUTTON_BG_COLOR: RGBA8 = gray(42);
pub const BUTTON_BG_HOVER_COLOR: RGBA8 = gray(52);
pub const BUTTON_BORDER_COLOR: RGBA8 = gray(62);
pub const BUTTON_BORDER_COLOR_HOVER: RGBA8 = gray(82);

pub const KNOB_BG_COLOR: RGBA8 = gray(71);
pub const KNOB_BG_HOVER_COLOR: RGBA8 = gray(81);
pub const KNOB_BORDER_COLOR: RGBA8 = gray(91);
pub const KNOB_BORDER_HOVER_COLOR: RGBA8 = gray(101);
pub const KNOB_ARC_TRACK_COLOR: RGBA8 = gray(44);

pub const TOGGLE_OFF_BG_COLOR: RGBA8 = gray(27);
pub const TOGGLE_OFF_BG_COLOR_HOVER: RGBA8 = gray(40);

pub const TEXT_INPUT_BG_COLOR: RGBA8 = gray(24);
pub const DROPDOWN_BG_COLOR: RGBA8 = gray(27);
pub const DROPDOWN_BORDER_COLOR: RGBA8 = gray(105);

pub const TAB_OFF_COLOR_HOVER: RGBA8 = gray_a(255, 5);
pub const TAB_TOGGLED_COLOR: RGBA8 = gray_a(255, 6);
pub const TAB_TOGGLED_COLOR_HOVER: RGBA8 = gray_a(255, 8);

pub const SCROLL_BAR_COLOR: RGBA8 = gray_a(255, 33);
pub const SCROLL_BAR_COLOR_HOVER: RGBA8 = gray_a(255, 70);

pub const SEPERATOR_COLOR: RGBA8 = gray_a(202, 6);

pub const PANEL_BG_COLOR: RGBA8 = gray(33);

pub const BORDER_WIDTH: f32 = 1.0;
pub const BORDER_RADIUS: f32 = 4.0;

pub fn button(config: &Config) -> ButtonStyle {
    ButtonStyle {
        text_properties: TextProperties {
            metrics: config.text_metrics,
            attrs: config.text_attrs,
            ..Default::default()
        },
        text_padding: TEXT_PADDING,
        icon_padding: ICON_PADDING,
        text_icon_spacing: TEXT_ICON_SPACING,
        text_color: TEXT_COLOR,
        text_color_hover: Some(TEXT_COLOR_BRIGHT),
        back_bg: background(BUTTON_BG_COLOR),
        back_bg_hover: Some(background(BUTTON_BG_HOVER_COLOR)),
        back_border_color: BUTTON_BORDER_COLOR,
        back_border_color_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_width: BORDER_WIDTH,
        back_border_radius: config.radius.into(),
        cursor_icon: Some(CursorIcon::Pointer),
        ..Default::default()
    }
}

pub fn menu_button(config: &Config) -> ButtonStyle {
    ButtonStyle {
        text_properties: TextProperties {
            metrics: config.text_metrics,
            attrs: config.text_attrs,
            ..Default::default()
        },
        text_padding: TEXT_PADDING,
        icon_padding: ICON_PADDING,
        text_icon_spacing: TEXT_ICON_SPACING,
        text_color: TEXT_COLOR,
        text_color_hover: Some(TEXT_COLOR_BRIGHT),
        back_bg_hover: Some(background(BUTTON_BG_HOVER_COLOR)),
        back_border_radius: config.radius.into(),
        cursor_icon: Some(CursorIcon::Pointer),
        ..Default::default()
    }
}

pub fn toggle_button(config: &Config) -> ToggleButtonStyle {
    ToggleButtonStyle {
        text_properties: TextProperties {
            metrics: config.text_metrics,
            attrs: config.text_attrs,
            ..Default::default()
        },
        text_padding: TEXT_PADDING,
        icon_padding: ICON_PADDING,
        text_icon_spacing: TEXT_ICON_SPACING,
        text_color: TEXT_COLOR,
        text_color_on_hover: Some(TEXT_COLOR_BRIGHT),
        text_color_off_hover: Some(TEXT_COLOR_BRIGHT),
        back_bg: background(TOGGLE_OFF_BG_COLOR),
        back_bg_on: Some(background(config.accent_color)),
        back_bg_off_hover: Some(background(TOGGLE_OFF_BG_COLOR_HOVER)),
        back_bg_on_hover: Some(background(config.accent_color_hover)),
        back_border_color: BUTTON_BORDER_COLOR,
        back_border_color_off_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_color_on_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_width: BORDER_WIDTH,
        back_border_radius: config.radius.into(),
        cursor_icon: Some(CursorIcon::Pointer),
        ..Default::default()
    }
}

pub fn switch(config: &Config) -> SwitchStyle {
    SwitchStyle {
        outer_border_width: BORDER_WIDTH,
        outer_border_color_off: BUTTON_BORDER_COLOR,
        outer_border_color_off_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        off_bg: background(TOGGLE_OFF_BG_COLOR),
        on_bg: Some(background(config.accent_color)),
        off_bg_hover: Some(background(TOGGLE_OFF_BG_COLOR_HOVER)),
        on_bg_hover: Some(background(config.accent_color_hover)),
        slider_bg_off: background(RGBA8::new(255, 255, 255, 180)),
        cursor_icon: Some(CursorIcon::Pointer),
        ..Default::default()
    }
}

pub fn radio_btn(config: &Config) -> RadioButtonStyle {
    RadioButtonStyle {
        outer_border_width: 1.0,
        outer_border_color_off: BUTTON_BORDER_COLOR,
        outer_border_color_off_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        off_bg: background(TOGGLE_OFF_BG_COLOR),
        on_bg: Some(background(config.accent_color)),
        off_bg_hover: Some(background(TOGGLE_OFF_BG_COLOR_HOVER)),
        on_bg_hover: Some(background(config.accent_color_hover)),
        dot_padding: 6.0,
        dot_bg: background(TEXT_COLOR),
        dot_bg_hover: Some(background(TEXT_COLOR_BRIGHT)),
        cursor_icon: Some(CursorIcon::Pointer),
        ..Default::default()
    }
}

pub fn resize_handle() -> ResizeHandleStyle {
    ResizeHandleStyle {
        drag_handle_color_hover: Some(SCROLL_BAR_COLOR_HOVER),
        drag_handle_width_hover: Some(3.0),
        ..Default::default()
    }
}

pub fn scroll_bar() -> ScrollBarStyle {
    ScrollBarStyle {
        slider_bg: Background::TRANSPARENT,
        slider_bg_content_hover: Some(background(SCROLL_BAR_COLOR)),
        slider_bg_slider_hover: Some(background(SCROLL_BAR_COLOR_HOVER)),
        radius: 8.0.into(),
        ..Default::default()
    }
}

pub fn text_input(config: &Config) -> TextInputStyle {
    TextInputStyle {
        text_properties: TextProperties {
            metrics: config.text_metrics,
            attrs: config.text_attrs,
            ..Default::default()
        },
        placeholder_text_attrs: Some(config.text_attrs.style(rootvg::text::Style::Italic)),
        text_color: TEXT_COLOR,
        text_color_placeholder: Some(TEXT_COLOR_DIMMED),
        text_color_focused: None,
        text_color_highlighted: Some(TEXT_COLOR_BRIGHT),
        highlight_bg_color: config.accent_color,
        padding: Padding::new(6.0, 6.0, 6.0, 6.0),
        highlight_padding: Padding::new(1.0, 0.0, 0.0, 0.0),
        back_bg: background(TEXT_INPUT_BG_COLOR),
        back_border_color: BUTTON_BORDER_COLOR,
        back_border_color_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_color_focused: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_width: 1.0,
        back_border_radius: config.radius.into(),
        ..Default::default()
    }
}

pub fn icon_text_input(config: &Config) -> IconTextInputStyle {
    IconTextInputStyle {
        text_input: text_input(config),
        icon_padding: padding(0.0, 0.0, 0.0, 5.0),
        ..Default::default()
    }
}

pub fn tab(config: &Config) -> TabStyle {
    TabStyle {
        toggle_btn_style: ToggleButtonStyle {
            text_properties: TextProperties {
                metrics: config.text_metrics,
                attrs: config.text_attrs,
                ..Default::default()
            },
            text_padding: TEXT_PADDING,
            icon_padding: ICON_PADDING,
            text_icon_spacing: TEXT_ICON_SPACING,
            text_color: TEXT_COLOR,
            text_color_on_hover: Some(TEXT_COLOR_BRIGHT),
            text_color_off_hover: Some(TEXT_COLOR_BRIGHT),
            back_bg_on: Some(background(TAB_TOGGLED_COLOR)),
            back_bg_off_hover: Some(background(TAB_OFF_COLOR_HOVER)),
            back_bg_on_hover: Some(background(TAB_TOGGLED_COLOR_HOVER)),
            cursor_icon: Some(CursorIcon::Pointer),
            ..Default::default()
        },
        on_indicator_line_style: QuadStyle {
            bg: background(config.accent_color),
            border: border_radius_only(config.radius.into()),
            flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        },
        on_indicator_line_width: 3.0,
        ..Default::default()
    }
}

pub fn tooltip(config: &Config) -> TooltipStyle {
    TooltipStyle {
        text_properties: TextProperties {
            metrics: config.text_metrics,
            attrs: config.text_attrs,
            ..Default::default()
        },
        text_color: TEXT_COLOR,
        text_padding: TEXT_PADDING,
        back_quad: QuadStyle {
            bg: background(DROPDOWN_BG_COLOR),
            border: border(DROPDOWN_BORDER_COLOR, 1.0, config.radius.into()),
            flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        },
        ..Default::default()
    }
}

pub fn separator() -> SeparatorStyle {
    SeparatorStyle {
        quad_style: QuadStyle {
            bg: background(SEPERATOR_COLOR),
            border: BorderStyle::default(),
            flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        },
        ..Default::default()
    }
}

pub fn dropdown_menu(config: &Config) -> DropDownMenuStyle {
    DropDownMenuStyle {
        text_properties: TextProperties {
            metrics: config.text_metrics,
            attrs: config.text_attrs,
            ..Default::default()
        },
        text_color: TEXT_COLOR,
        text_color_hover: Some(TEXT_COLOR_BRIGHT),
        back_quad: QuadStyle {
            bg: background(DROPDOWN_BG_COLOR),
            border: border(DROPDOWN_BORDER_COLOR, 1.0, config.radius.into()),
            flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        },
        entry_bg_quad_hover: QuadStyle {
            bg: background(BUTTON_BG_HOVER_COLOR),
            border: border(BUTTON_BORDER_COLOR, 1.0, config.radius.into()),
            flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
        },
        outer_padding: 2.0,
        left_icon_padding: padding_vh(0.0, 4.0),
        left_text_padding: padding_vh(5.0, 10.0),
        left_text_icon_spacing: TEXT_ICON_SPACING,
        right_text_padding: padding(0.0, 10.0, 0.0, 30.0),
        divider_color: SEPERATOR_COLOR,
        divider_width: 1.0,
        divider_padding: 1.0,
        cursor_icon: Some(CursorIcon::Pointer),
        ..Default::default()
    }
}

pub fn label(config: &Config) -> LabelStyle {
    LabelStyle {
        text_properties: TextProperties {
            metrics: config.text_metrics,
            attrs: config.text_attrs,
            ..Default::default()
        },
        text_color: TEXT_COLOR,
        ..Default::default()
    }
}

pub fn panel() -> QuadStyle {
    QuadStyle {
        bg: background(PANEL_BG_COLOR),
        border: Default::default(),
        flags: QuadFlags::SNAP_ALL_TO_NEAREST_PIXEL,
    }
}

pub fn slider_style_modern(
    accent_color: RGBA8,
    accent_color_hover: RGBA8,
    radius: f32,
) -> SliderStyleModern {
    SliderStyleModern {
        back_bg: background(TEXT_INPUT_BG_COLOR),
        back_border_color: BUTTON_BORDER_COLOR,
        back_border_color_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_width: 1.0,
        back_border_radius: radius.into(),
        handle_bg: background(TEXT_COLOR),
        handle_bg_hover: Some(background(TEXT_COLOR_BRIGHT)),
        handle_border_radius: radius.into(),
        handle_border_color: TEXT_INPUT_BG_COLOR,
        handle_border_width: 1.0,
        fill_bg: background(accent_color),
        fill_bg_hover: Some(background(accent_color_hover)),
        handle_height: SizeType::FixedPoints(8.0),
        handle_padding: Padding::new(2.0, 2.0, 2.0, 2.0),
        fill_padding: Padding::new(3.0, 5.0, 3.0, 5.0),
        ..Default::default()
    }
}

pub fn knob_style(
    accent_color: RGBA8,
    accent_color_hover: RGBA8,
    use_line_notch: bool,
    use_dot_markers: bool,
) -> KnobStyle {
    KnobStyle {
        back: KnobBackStyle::Quad(KnobBackStyleQuad {
            bg: background(KNOB_BG_COLOR),
            bg_hover: Some(background(KNOB_BG_HOVER_COLOR)),
            border_color: KNOB_BORDER_COLOR,
            border_color_hover: Some(KNOB_BORDER_HOVER_COLOR),
            border_width: 1.0,
            size: SizeType::Scale(0.7),
            ..Default::default()
        }),
        notch: if use_line_notch {
            KnobNotchStyle::Line(KnobNotchStyleLine {
                bg: KnobNotchStyleLineBg::Solid {
                    idle: TEXT_COLOR,
                    hovered: Some(TEXT_COLOR_BRIGHT),
                    gesturing: None,
                    disabled: Default::default(),
                },
                ..Default::default()
            })
        } else {
            KnobNotchStyle::Quad(KnobNotchStyleQuad {
                bg: background(TEXT_COLOR),
                bg_hover: Some(background(TEXT_COLOR_BRIGHT)),
                ..Default::default()
            })
        },
        markers: if use_dot_markers {
            KnobMarkersStyle::Dots(KnobMarkersDotStyle {
                primary_quad_style: QuadStyle {
                    bg: background(TEXT_COLOR_DIMMED),
                    border: border_radius_only(Radius::CIRCLE),
                    flags: QuadFlags::empty(),
                },
                ..Default::default()
            })
        } else {
            KnobMarkersStyle::Arc(KnobMarkersArcStyle {
                fill_bg: background(accent_color),
                fill_bg_hover: Some(background(accent_color_hover)),
                back_bg: background(KNOB_ARC_TRACK_COLOR),
                ..Default::default()
            })
        },
        ..Default::default()
    }
}

pub struct Config {
    pub accent_color: RGBA8,
    pub accent_color_hover: RGBA8,
    pub radius: f32,
    pub text_metrics: Metrics,
    pub text_attrs: Attrs<'static>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            accent_color: DEFAULT_ACCENT_COLOR,
            accent_color_hover: DEFAULT_ACCENT_HOVER_COLOR,
            radius: BORDER_RADIUS,
            text_metrics: Metrics {
                font_size: 14.0,
                line_height: 16.0,
            },
            text_attrs: Attrs::new(),
        }
    }
}

pub fn load(config: Config, res: &mut ResourceCtx) {
    res.style_system
        .add(ClassID::default(), true, button(&config));
    res.style_system
        .add(ClassID::default(), true, toggle_button(&config));
    res.style_system
        .add(ClassID::default(), true, switch(&config));
    res.style_system
        .add(ClassID::default(), true, radio_btn(&config));
    res.style_system
        .add(ClassID::default(), true, resize_handle());
    res.style_system.add(ClassID::default(), true, scroll_bar());
    res.style_system
        .add(ClassID::default(), true, text_input(&config));
    res.style_system
        .add(ClassID::default(), true, icon_text_input(&config));
    res.style_system.add(ClassID::default(), true, tab(&config));
    res.style_system
        .add(ClassID::default(), true, tooltip(&config));
    res.style_system.add(ClassID::default(), true, separator());
    res.style_system
        .add(ClassID::default(), true, dropdown_menu(&config));
    res.style_system
        .add(ClassID::default(), true, label(&config));
    res.style_system.add(CLASS_PANEL, true, panel());
    res.style_system.add(CLASS_MENU, true, menu_button(&config));
    res.style_system.add(
        ClassID::default(),
        true,
        SliderStyle::Modern(slider_style_modern(
            config.accent_color,
            config.accent_color_hover,
            config.radius,
        )),
    );
    res.style_system.add(
        ClassID::default(),
        true,
        knob_style(config.accent_color, config.accent_color_hover, false, false),
    );
}
