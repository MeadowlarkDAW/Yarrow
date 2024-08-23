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
    pub const CLASS_PANEL_BORDER: ClassID = 1;
    pub const CLASS_FANCY_LABEL: ClassID = 2;
    pub const CLASS_KNOB_2: ClassID = 3;

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
            text_input_size: size(240.0, 30.0),
            floating_text_input_size: size(100.0, 30.0),
            floating_text_input_align: Align2::BOTTOM_CENTER,
            floating_text_input_padding: padding_all_same(5.0),
            separator_width: 1.0,

            clear_color: rgb(15, 15, 15),
        }
    }

    pub fn load(&self, res: &mut ResourceCtx) {
        const LABEL_BG_COLOR: RGBA8 = gray(42);

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

        yarrow::theme::yarrow_dark::load(Default::default(), res);

        res.style_system.add(
            Self::CLASS_FANCY_LABEL,
            true,
            IconStyle {
                color: yarrow::theme::DEFAULT_ACCENT_COLOR,
                size: 20.0,
                back_quad: quad_style(background(LABEL_BG_COLOR), border_radius_only(radius(30.0))),
                padding: padding_vh(4.0, 10.0),
                ..Default::default()
            },
        );

        res.style_system.add(
            Self::CLASS_FANCY_LABEL,
            true,
            LabelStyle {
                back_quad: quad_style(background(LABEL_BG_COLOR), border_radius_only(radius(30.0))),
                text_color: gray_a(255, 240),
                icon_color: Some(yarrow::theme::DEFAULT_ACCENT_COLOR),
                text_padding: padding_vh(6.0, 12.0),
                icon_padding: padding_vh(4.0, 8.0),
                text_icon_spacing: -12.0,
                ..Default::default()
            },
        );

        res.style_system.add(
            Self::CLASS_PANEL_BORDER,
            true,
            QuadStyle {
                bg: background_gray(2),
                ..Default::default()
            },
        );

        res.style_system.add(
            Self::CLASS_KNOB_2,
            true,
            yarrow::theme::yarrow_dark::knob_style(
                yarrow::theme::DEFAULT_ACCENT_COLOR,
                yarrow::theme::DEFAULT_ACCENT_HOVER_COLOR,
                true,
                true,
            ),
        );
    }
}
