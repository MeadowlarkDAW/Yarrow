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

    pub fn load(&self, res: &mut ResourceCtx) {
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

        yarrow::theme::yarrow_dark::load(None, None, res);

        res.style_system.add(
            "fancy_icon",
            true,
            IconStyle {
                color: yarrow::style::DEFAULT_ACCENT_COLOR,
                size: 20.0,
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(42, 42, 42, 255)),
                    border: BorderStyle {
                        radius: 30.0.into(),
                        ..Default::default()
                    },
                },
                padding: Padding::new(4.0, 10.0, 4.0, 10.0),
                ..Default::default()
            },
        );

        res.style_system.add(
            "fancy_label",
            true,
            LabelStyle {
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(42, 42, 42, 255)),
                    border: BorderStyle {
                        radius: 30.0.into(),
                        ..Default::default()
                    },
                },
                text_color: RGBA8::new(255, 255, 255, 200),
                icon_color: Some(yarrow::style::DEFAULT_ACCENT_COLOR),
                text_padding: Padding::new(6.0, 12.0, 6.0, 12.0),
                icon_padding: Padding::new(4.0, 8.0, 4.0, 8.0),
                text_icon_spacing: -12.0,
                ..Default::default()
            },
        );

        res.style_system.add(
            "panel_border",
            true,
            QuadStyle {
                bg: Background::Solid(RGBA8::new(2, 2, 2, 255).into()),
                ..Default::default()
            },
        );

        res.style_system.add(
            "knob2",
            true,
            KnobStyle {
                notch: KnobNotchStyle::Line(Default::default()),
                markers: KnobMarkersStyle::Dots(Default::default()),
                ..Default::default()
            },
        );
    }
}
