use std::rc::Rc;
use yarrow::elements::icon::IconStyle;
use yarrow::elements::knob::KnobMarkersStyle;
use yarrow::prelude::*;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyIcon {
    Menu = 0,
    Dropdown,
    Info,
    Search,
    PowerOff,
    PowerOn,
    Save,
    Cut,
    Copy,
    Paste,
    Select,
}

impl MyIcon {
    const ALL: [Self; 11] = [
        Self::Menu,
        Self::Dropdown,
        Self::Info,
        Self::Search,
        Self::PowerOff,
        Self::PowerOn,
        Self::Save,
        Self::Cut,
        Self::Copy,
        Self::Paste,
        Self::Select,
    ];

    const fn source(&self) -> &'static [u8] {
        match self {
            Self::Menu => include_bytes!("../assets/menu.svg"),
            Self::Dropdown => include_bytes!("../assets/dropdown.svg"),
            Self::Info => include_bytes!("../assets/info.svg"),
            Self::Search => include_bytes!("../assets/search.svg"),
            Self::PowerOff => include_bytes!("../assets/power-off.svg"),
            Self::PowerOn => include_bytes!("../assets/power-on.svg"),
            Self::Save => include_bytes!("../assets/save.svg"),
            Self::Cut => include_bytes!("../assets/cut.svg"),
            Self::Copy => include_bytes!("../assets/copy.svg"),
            Self::Paste => include_bytes!("../assets/paste.svg"),
            Self::Select => include_bytes!("../assets/select.svg"),
        }
    }
}

impl Into<IconID> for MyIcon {
    fn into(self) -> IconID {
        self as IconID
    }
}

pub struct MyStyle {
    pub icon_style: Rc<IconStyle>,
    pub button_style: Rc<ButtonStyle>,
    pub toggle_btn_style: Rc<ToggleButtonStyle>,
    pub icon_btn_style: Rc<IconButtonStyle>,
    pub icon_toggle_btn_style: Rc<IconToggleButtonStyle>,
    pub icon_label_toggle_btn_style: Rc<IconLabelToggleButtonStyle>,
    pub switch_style: Rc<SwitchStyle>,
    pub label_style: Rc<LabelStyle>,
    pub icon_label_style: Rc<IconLabelStyle>,
    pub label_no_bg_style: Rc<LabelStyle>,
    pub radio_btn_style: Rc<RadioButtonStyle>,
    pub drop_down_btn_style: Rc<IconLabelButtonStyle>,
    pub menu_btn_style: Rc<IconButtonStyle>,
    pub menu_style: Rc<DropDownMenuStyle>,
    pub text_input_style: Rc<TextInputStyle>,
    pub icon_text_input_style: Rc<IconTextInputStyle>,
    pub panel_bg_style: Rc<QuadStyle>,
    pub panel_border_style: Rc<QuadStyle>,
    pub resize_handle_style: Rc<ResizeHandleStyle>,
    pub tooltip_style: Rc<LabelStyle>,
    pub tab_style: Rc<TabStyle>,
    pub scroll_bar_style: Rc<ScrollBarStyle>,
    pub paragraph_style: Rc<LabelStyle>,
    pub separator_style: Rc<SeparatorStyle>,
    pub knob_style_1: Rc<KnobStyle>,
    pub knob_style_2: Rc<KnobStyle>,
    pub slider_style_1: Rc<SliderStyle>,

    pub top_panel_height: f32,
    pub panel_border_width: f32,

    pub content_padding: f32,
    pub element_padding: f32,
    pub radio_group_row_padding: f32,
    pub radio_group_column_padding: f32,
    pub menu_btn_padding: f32,
    pub tab_group_padding: f32,
    pub tag_group_spacing: f32,
    pub param_label_padding: f32,
    pub knob_size: f32,
    pub param_spacing: f32,

    pub drop_down_btn_width: f32,
    pub text_input_size: Size,
    pub floating_text_input_size: Size,
    pub floating_text_input_align: Align2,
    pub floating_text_input_padding: Padding,
    pub separator_width: f32,

    pub clear_color: RGBA8,
}

impl MyStyle {
    pub fn new() -> Self {
        Self {
            icon_style: Rc::new(IconStyle {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    border: BorderStyle {
                        radius: 30.0.into(),
                        ..Default::default()
                    },
                },
                padding: Padding::new(2.0, 10.0, 2.0, 10.0),
                ..Default::default()
            }),
            button_style: Rc::new(ButtonStyle::default()),
            toggle_btn_style: Rc::new(ToggleButtonStyle::default()),
            icon_btn_style: Rc::new(IconButtonStyle::default()),
            icon_toggle_btn_style: Rc::new(IconToggleButtonStyle::default()),
            icon_label_toggle_btn_style: Rc::new(IconLabelToggleButtonStyle::default()),
            switch_style: Rc::new(SwitchStyle::default()),
            label_style: Rc::new(LabelStyle {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    border: BorderStyle {
                        radius: 30.0.into(),
                        ..Default::default()
                    },
                },
                padding: Padding::new(5.0, 10.0, 5.0, 10.0),
                ..Default::default()
            }),
            icon_label_style: Rc::new(IconLabelStyle {
                icon_color: DEFAULT_ACCENT_COLOR,
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    border: BorderStyle {
                        radius: 30.0.into(),
                        ..Default::default()
                    },
                },
                text_padding: Padding::new(5.0, 10.0, 5.0, 10.0),
                icon_padding: Padding::new(0.0, 0.0, 0.0, 5.0),
                ..Default::default()
            }),
            label_no_bg_style: Rc::new(LabelStyle::default()),
            radio_btn_style: Rc::new(RadioButtonStyle::default()),
            drop_down_btn_style: Rc::new(IconLabelButtonStyle::default_dropdown_style()),
            menu_btn_style: Rc::new(IconButtonStyle::default_menu_style()),
            text_input_style: Rc::new(TextInputStyle::default()),
            icon_text_input_style: Rc::new(IconTextInputStyle::default()),
            panel_bg_style: Rc::new(QuadStyle {
                bg: Background::Solid(RGBA8::new(40, 40, 40, 255).into()),
                ..Default::default()
            }),
            panel_border_style: Rc::new(QuadStyle {
                bg: Background::Solid(RGBA8::new(65, 65, 65, 255).into()),
                ..Default::default()
            }),
            resize_handle_style: Rc::new(ResizeHandleStyle {
                ..Default::default()
            }),
            menu_style: Rc::new(DropDownMenuStyle::default()),
            tooltip_style: Rc::new(LabelStyle::default_tooltip_style()),
            tab_style: Rc::new(TabStyle {
                toggle_btn_style: ToggleButtonStyle {
                    properties: TextProperties {
                        attrs: Attrs::new().weight(Weight::NORMAL),
                        align: Some(TextAlign::Left),
                        ..Default::default()
                    },
                    padding: Padding::new(6.0, 12.0, 6.0, 12.0),
                    ..TabStyle::default().toggle_btn_style
                },
                ..Default::default()
            }),
            scroll_bar_style: Rc::new(ScrollBarStyle::default()),
            paragraph_style: Rc::new(LabelStyle::default_paragraph_style()),
            separator_style: Rc::new(SeparatorStyle::default()),
            knob_style_1: Rc::new(KnobStyle::default()),
            knob_style_2: Rc::new(KnobStyle {
                notch: KnobNotchStyle::Line(Default::default()),
                markers: KnobMarkersStyle::Dots(Default::default()),
                ..Default::default()
            }),
            slider_style_1: Rc::new(SliderStyle::default()),

            top_panel_height: 30.0,
            panel_border_width: 1.0,

            content_padding: 20.0,
            element_padding: 15.0,
            radio_group_row_padding: 7.0,
            radio_group_column_padding: 10.0,
            menu_btn_padding: 3.0,
            tab_group_padding: 1.0,
            tag_group_spacing: 1.0,
            param_label_padding: 5.0,
            knob_size: 40.0,
            param_spacing: 30.0,

            drop_down_btn_width: 100.0,
            text_input_size: Size::new(240.0, 30.0),
            floating_text_input_size: Size::new(100.0, 30.0),
            floating_text_input_align: Align2::BOTTOM_CENTER,
            floating_text_input_padding: Padding::new(5.0, 5.0, 5.0, 5.0),
            separator_width: 1.0,

            clear_color: RGBA8::new(15, 15, 15, 255),
        }
    }

    pub fn load_resources(&self, res: &mut ResourceCtx) {
        for icon in MyIcon::ALL {
            res.svg_icon_system
                .add_from_bytes(
                    icon,
                    icon.source(),
                    &Default::default(),
                    IconContentType::Mask,
                )
                .unwrap();
        }
    }
}
