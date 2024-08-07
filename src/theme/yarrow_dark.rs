use crate::{
    elements::tooltip::TooltipStyle,
    prelude::*,
    style::{DEFAULT_ACCENT_COLOR, DEFAULT_ACCENT_HOVER_COLOR},
};

pub const TEXT_PADDING: Padding = Padding::new(6.0, 7.0, 6.0, 7.0);
pub const ICON_PADDING: Padding = Padding::new(4.0, 5.0, 4.0, 5.0);
pub const TEXT_ICON_SPACING: f32 = -8.0;

pub const TEXT_COLOR: RGBA8 = RGBA8::new(255, 255, 255, 200);
pub const TEXT_COLOR_BRIGHT: RGBA8 = RGBA8::new(255, 255, 255, 255);
pub const TEXT_COLOR_DIMMED: RGBA8 = RGBA8::new(255, 255, 255, 100);

pub const BUTTON_BG_COLOR: RGBA8 = RGBA8::new(42, 42, 42, 255);
pub const BUTTON_BG_HOVER_COLOR: RGBA8 = RGBA8::new(52, 52, 52, 255);
pub const BUTTON_BORDER_COLOR: RGBA8 = RGBA8::new(62, 62, 62, 255);
pub const BUTTON_BORDER_COLOR_HOVER: RGBA8 = RGBA8::new(82, 82, 82, 255);

pub const TOGGLE_OFF_BG_COLOR: RGBA8 = RGBA8::new(27, 27, 27, 255);
pub const TOGGLE_OFF_BG_COLOR_HOVER: RGBA8 = RGBA8::new(40, 40, 40, 255);

pub const TEXT_INPUT_BG_COLOR: RGBA8 = RGBA8::new(24, 24, 24, 255);
pub const DROPDOWN_BG_COLOR: RGBA8 = RGBA8::new(27, 27, 27, 255);
pub const DROPDOWN_BORDER_COLOR: RGBA8 = RGBA8::new(105, 105, 105, 255);

pub const TAB_OFF_COLOR_HOVER: RGBA8 = RGBA8::new(255, 255, 255, 5);
pub const TAB_TOGGLED_COLOR: RGBA8 = RGBA8::new(255, 255, 255, 6);
pub const TAB_TOGGLED_COLOR_HOVER: RGBA8 = RGBA8::new(255, 255, 255, 8);

pub const SCROLL_BAR_COLOR: RGBA8 = RGBA8::new(255, 255, 255, 33);
pub const SCROLL_BAR_COLOR_HOVER: RGBA8 = RGBA8::new(255, 255, 255, 70);

pub const SEPERATOR_COLOR: RGBA8 = RGBA8::new(202, 202, 202, 6);

pub const PANEL_BG_COLOR: RGBA8 = RGBA8::new(33, 33, 33, 255);

pub const BORDER_WIDTH: f32 = 1.0;
pub const BORDER_RADIUS: Radius = Radius {
    top_left: 4.0,
    top_right: 4.0,
    bottom_right: 4.0,
    bottom_left: 4.0,
};

pub fn button() -> ButtonStyle {
    ButtonStyle {
        text_padding: TEXT_PADDING,
        icon_padding: ICON_PADDING,
        text_icon_spacing: TEXT_ICON_SPACING,
        text_color: TEXT_COLOR,
        text_color_hover: Some(TEXT_COLOR_BRIGHT),
        back_bg: Background::Solid(BUTTON_BG_COLOR),
        back_bg_hover: Some(Background::Solid(BUTTON_BG_HOVER_COLOR)),
        back_border_color: BUTTON_BORDER_COLOR,
        back_border_color_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_width: BORDER_WIDTH,
        back_border_radius: BORDER_RADIUS,
        cursor_icon: Some(CursorIcon::Pointer),
        ..Default::default()
    }
}

pub fn menu_button() -> ButtonStyle {
    ButtonStyle {
        text_padding: TEXT_PADDING,
        icon_padding: ICON_PADDING,
        text_icon_spacing: TEXT_ICON_SPACING,
        text_color: TEXT_COLOR,
        text_color_hover: Some(TEXT_COLOR_BRIGHT),
        back_bg_hover: Some(Background::Solid(BUTTON_BG_HOVER_COLOR)),
        back_border_radius: BORDER_RADIUS,
        cursor_icon: Some(CursorIcon::Pointer),
        ..Default::default()
    }
}

pub fn toggle_button(
    accent_color: Option<RGBA8>,
    accent_color_hover: Option<RGBA8>,
) -> ToggleButtonStyle {
    let accent_color = accent_color.unwrap_or(DEFAULT_ACCENT_COLOR);
    let accent_color_hover = accent_color_hover.unwrap_or(DEFAULT_ACCENT_HOVER_COLOR);

    ToggleButtonStyle {
        text_padding: TEXT_PADDING,
        icon_padding: ICON_PADDING,
        text_icon_spacing: TEXT_ICON_SPACING,
        text_color: TEXT_COLOR,
        text_color_on_hover: Some(TEXT_COLOR_BRIGHT),
        text_color_off_hover: Some(TEXT_COLOR_BRIGHT),
        back_bg: Background::Solid(TOGGLE_OFF_BG_COLOR),
        back_bg_on: Some(Background::Solid(accent_color)),
        back_bg_off_hover: Some(Background::Solid(TOGGLE_OFF_BG_COLOR_HOVER)),
        back_bg_on_hover: Some(Background::Solid(accent_color_hover)),
        back_border_color: BUTTON_BORDER_COLOR,
        back_border_color_off_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_color_on_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_width: BORDER_WIDTH,
        back_border_radius: BORDER_RADIUS,
        //cursor_icon: Some(CursorIcon::Pointer),
        ..Default::default()
    }
}

pub fn switch(accent_color: Option<RGBA8>, accent_color_hover: Option<RGBA8>) -> SwitchStyle {
    let accent_color = accent_color.unwrap_or(DEFAULT_ACCENT_COLOR);
    let accent_color_hover = accent_color_hover.unwrap_or(DEFAULT_ACCENT_HOVER_COLOR);

    SwitchStyle {
        outer_border_width: BORDER_WIDTH,
        outer_border_color_off: BUTTON_BORDER_COLOR,
        outer_border_color_off_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        off_bg: Background::Solid(TOGGLE_OFF_BG_COLOR),
        on_bg: Some(Background::Solid(accent_color)),
        off_bg_hover: Some(Background::Solid(TOGGLE_OFF_BG_COLOR_HOVER)),
        on_bg_hover: Some(Background::Solid(accent_color_hover)),
        slider_bg_off: Background::Solid(RGBA8::new(255, 255, 255, 180)),
        ..Default::default()
    }
}

pub fn radio_btn(
    accent_color: Option<RGBA8>,
    accent_color_hover: Option<RGBA8>,
) -> RadioButtonStyle {
    let accent_color = accent_color.unwrap_or(DEFAULT_ACCENT_COLOR);
    let accent_color_hover = accent_color_hover.unwrap_or(DEFAULT_ACCENT_HOVER_COLOR);

    RadioButtonStyle {
        outer_border_width: 1.0,
        outer_border_color_off: BUTTON_BORDER_COLOR,
        outer_border_color_off_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        off_bg: Background::Solid(TOGGLE_OFF_BG_COLOR),
        on_bg: Some(Background::Solid(accent_color)),
        off_bg_hover: Some(Background::Solid(TOGGLE_OFF_BG_COLOR_HOVER)),
        on_bg_hover: Some(Background::Solid(accent_color_hover)),
        dot_padding: 6.0,
        dot_bg: Background::Solid(TEXT_COLOR),
        dot_bg_hover: Some(Background::Solid(TEXT_COLOR_BRIGHT)),
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
        slider_bg_content_hover: Some(Background::Solid(SCROLL_BAR_COLOR)),
        slider_bg_slider_hover: Some(Background::Solid(SCROLL_BAR_COLOR_HOVER)),
        radius: 8.0.into(),
        ..Default::default()
    }
}

pub fn text_input(accent_color: Option<RGBA8>) -> TextInputStyle {
    let accent_color = accent_color.unwrap_or(DEFAULT_ACCENT_COLOR);

    TextInputStyle {
        placeholder_text_attrs: Some(Attrs::new().style(rootvg::text::Style::Italic)),
        text_color: TEXT_COLOR,
        text_color_placeholder: Some(TEXT_COLOR_DIMMED),
        text_color_focused: None,
        text_color_highlighted: Some(TEXT_COLOR_BRIGHT),
        highlight_bg_color: accent_color,
        padding: Padding::new(6.0, 6.0, 6.0, 6.0),
        highlight_padding: Padding::new(1.0, 0.0, 0.0, 0.0),
        back_bg: Background::Solid(TEXT_INPUT_BG_COLOR),
        back_border_color: BUTTON_BORDER_COLOR,
        back_border_color_hover: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_color_focused: Some(BUTTON_BORDER_COLOR_HOVER),
        back_border_width: 1.0,
        back_border_radius: BORDER_RADIUS,
        ..Default::default()
    }
}

pub fn icon_text_input(accent_color: Option<RGBA8>) -> IconTextInputStyle {
    IconTextInputStyle {
        text_input: text_input(accent_color),
        icon_padding: Padding::new(0.0, 0.0, 0.0, 5.0),
        ..Default::default()
    }
}

pub fn tab(accent_color: Option<RGBA8>) -> TabStyle {
    let accent_color = accent_color.unwrap_or(DEFAULT_ACCENT_COLOR);

    TabStyle {
        toggle_btn_style: ToggleButtonStyle {
            text_properties: TextProperties {
                attrs: Attrs::new().weight(Weight::NORMAL),
                align: Some(TextAlign::Left),
                ..Default::default()
            },
            text_padding: TEXT_PADDING,
            icon_padding: ICON_PADDING,
            text_icon_spacing: TEXT_ICON_SPACING,
            text_color: TEXT_COLOR,
            text_color_on_hover: Some(TEXT_COLOR_BRIGHT),
            text_color_off_hover: Some(TEXT_COLOR_BRIGHT),
            back_bg_on: Some(Background::Solid(TAB_TOGGLED_COLOR)),
            back_bg_off_hover: Some(Background::Solid(TAB_OFF_COLOR_HOVER)),
            back_bg_on_hover: Some(Background::Solid(TAB_TOGGLED_COLOR_HOVER)),
            ..Default::default()
        },
        on_indicator_line_style: QuadStyle {
            bg: Background::Solid(accent_color),
            border: BorderStyle {
                radius: BORDER_RADIUS,
                ..Default::default()
            },
        },
        on_indicator_line_width: 3.0,
        ..Default::default()
    }
}

pub fn tooltip() -> TooltipStyle {
    TooltipStyle {
        text_color: TEXT_COLOR,
        text_padding: TEXT_PADDING,
        back_quad: QuadStyle {
            bg: Background::Solid(DROPDOWN_BG_COLOR),
            border: BorderStyle {
                color: DROPDOWN_BORDER_COLOR,
                width: 1.0,
                radius: BORDER_RADIUS,
            },
        },
        ..Default::default()
    }
}

pub fn separator() -> SeparatorStyle {
    SeparatorStyle {
        quad_style: QuadStyle {
            bg: Background::Solid(SEPERATOR_COLOR),
            border: BorderStyle::default(),
        },
        ..Default::default()
    }
}

pub fn dropdown_menu() -> DropDownMenuStyle {
    DropDownMenuStyle {
        text_color: TEXT_COLOR,
        text_color_hover: Some(TEXT_COLOR_BRIGHT),
        back_quad: QuadStyle {
            bg: Background::Solid(DROPDOWN_BG_COLOR),
            border: BorderStyle {
                radius: BORDER_RADIUS,
                color: DROPDOWN_BORDER_COLOR,
                width: 1.0,
                ..Default::default()
            },
        },
        entry_bg_quad_hover: QuadStyle {
            bg: Background::Solid(BUTTON_BG_HOVER_COLOR),
            border: BorderStyle {
                radius: BORDER_RADIUS,
                color: BUTTON_BORDER_COLOR,
                width: 1.0,
                ..Default::default()
            },
        },
        outer_padding: 2.0,
        left_icon_padding: Padding::new(0.0, 4.0, 0.0, 4.0),
        left_text_padding: Padding::new(5.0, 10.0, 5.0, 10.0),
        left_text_icon_spacing: TEXT_ICON_SPACING,
        right_text_padding: Padding::new(0.0, 10.0, 0.0, 30.0),
        divider_color: SEPERATOR_COLOR,
        divider_width: 1.0,
        divider_padding: 1.0,
        ..Default::default()
    }
}

pub fn label() -> LabelStyle {
    LabelStyle {
        text_color: TEXT_COLOR,
        ..Default::default()
    }
}

pub fn panel() -> QuadStyle {
    QuadStyle {
        bg: Background::Solid(PANEL_BG_COLOR),
        border: Default::default(),
    }
}

pub fn load(accent_color: Option<RGBA8>, accent_color_hover: Option<RGBA8>, res: &mut ResourceCtx) {
    res.style_system.add("", true, button());
    res.style_system
        .add("", true, toggle_button(accent_color, accent_color_hover));
    res.style_system
        .add("", true, switch(accent_color, accent_color_hover));
    res.style_system
        .add("", true, radio_btn(accent_color, accent_color_hover));
    res.style_system.add("", true, resize_handle());
    res.style_system.add("", true, scroll_bar());
    res.style_system.add("", true, text_input(accent_color));
    res.style_system
        .add("", true, icon_text_input(accent_color));
    res.style_system.add("", true, tab(accent_color));
    res.style_system.add("", true, tooltip());
    res.style_system.add("", true, separator());
    res.style_system.add("", true, dropdown_menu());
    res.style_system.add("", true, label());
    res.style_system.add("panel", true, panel());
    res.style_system.add("menu", true, menu_button());
}
