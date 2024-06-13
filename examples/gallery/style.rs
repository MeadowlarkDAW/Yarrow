use std::rc::Rc;
use std::sync::Arc;
use yarrow::prelude::*;

const ICON_FONT: &'static [u8] =
    include_bytes!("../assets/Font Awesome 6 Free-Solid-900.otf").as_slice();

pub struct MyStyle {
    pub button_style: Rc<ButtonStyle>,
    pub toggle_btn_style: Rc<ToggleButtonStyle>,
    pub dual_toggle_btn_style: Rc<DualToggleButtonStyle>,
    pub switch_style: Rc<SwitchStyle>,
    pub label_style: Rc<LabelStyle>,
    pub dual_label_style: Rc<DualLabelStyle>,
    pub label_no_bg_style: Rc<LabelStyle>,
    pub radio_btn_style: Rc<RadioButtonStyle>,
    pub drop_down_btn_style: Rc<DualButtonStyle>,
    pub menu_btn_style: Rc<ButtonStyle>,
    pub menu_style: Rc<DropDownMenuStyle>,
    pub text_input_style: Rc<TextInputStyle>,
    pub panel_bg_style: Rc<QuadStyle>,
    pub panel_border_style: Rc<QuadStyle>,
    pub resize_handle_style: Rc<ResizeHandleStyle>,
    pub tooltip_style: Rc<LabelStyle>,
    pub tab_style: Rc<TabStyle>,
    pub scroll_bar_style: Rc<ScrollBarStyle>,
    pub knob_style_1: Rc<KnobStyle>,

    pub top_panel_height: f32,
    pub panel_border_width: f32,

    pub content_padding: f32,
    pub element_padding: f32,
    pub radio_group_row_padding: f32,
    pub radio_group_column_padding: f32,
    pub menu_btn_padding: f32,
    pub tab_group_padding: f32,
    pub tag_group_spacing: f32,

    pub drop_down_btn_width: f32,
    pub text_input_size: Size,
    pub floating_text_input_size: Size,
    pub floating_text_input_align: Align2,
    pub floating_text_input_padding: Padding,

    pub clear_color: RGBA8,
}

impl MyStyle {
    pub fn new() -> Self {
        let icon_attrs = Attrs::new().family(Family::Fantasy).weight(Weight::BLACK);

        let mut menu_btn_style = ButtonStyle::default_menu_style();
        menu_btn_style.properties.attrs = icon_attrs;

        Self {
            button_style: Rc::new(ButtonStyle::default()),
            toggle_btn_style: Rc::new(ToggleButtonStyle::default()),
            dual_toggle_btn_style: Rc::new(DualToggleButtonStyle {
                left_properties: TextProperties {
                    attrs: icon_attrs,
                    ..Default::default()
                },
                ..Default::default()
            }),
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
            dual_label_style: Rc::new(DualLabelStyle {
                left_properties: TextProperties {
                    attrs: icon_attrs,
                    ..Default::default()
                },
                left_font_color: DEFAULT_ACCENT_COLOR,
                back_quad: QuadStyle {
                    bg: Background::Solid(RGBA8::new(40, 40, 40, 255)),
                    border: BorderStyle {
                        radius: 30.0.into(),
                        ..Default::default()
                    },
                },
                left_padding: Padding::new(5.0, 10.0, 5.0, 10.0),
                right_padding: Padding::new(5.0, 10.0, 5.0, 0.0),
                ..Default::default()
            }),
            label_no_bg_style: Rc::new(LabelStyle::default()),
            radio_btn_style: Rc::new(RadioButtonStyle::default()),
            drop_down_btn_style: Rc::new(DualButtonStyle {
                right_properties: TextProperties {
                    attrs: icon_attrs,
                    ..Default::default()
                },
                layout: DualLabelLayout::LeftAndRightAlign,
                right_padding: Padding::new(0.0, 10.0, 0.0, 0.0),
                ..DualButtonStyle::default()
            }),
            menu_btn_style: Rc::new(menu_btn_style),
            text_input_style: Rc::new(TextInputStyle::default()),
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
            knob_style_1: Rc::new(KnobStyle::default()),

            top_panel_height: 30.0,
            panel_border_width: 1.0,

            content_padding: 20.0,
            element_padding: 15.0,
            radio_group_row_padding: 7.0,
            radio_group_column_padding: 10.0,
            menu_btn_padding: 3.0,
            tab_group_padding: 1.0,
            tag_group_spacing: 1.0,

            drop_down_btn_width: 100.0,
            text_input_size: Size::new(240.0, 30.0),
            floating_text_input_size: Size::new(100.0, 30.0),
            floating_text_input_align: Align2::BOTTOM_CENTER,
            floating_text_input_padding: Padding::new(5.0, 5.0, 5.0, 5.0),

            clear_color: RGBA8::new(15, 15, 15, 255),
        }
    }

    pub fn load_fonts(&self, font_system: &mut FontSystem) {
        let _ = font_system
            .db_mut()
            .load_font_source(FontSource::Binary(Arc::new(ICON_FONT)));
        // Since the fantasy font is never used, replace it with our icon
        // font for simplicity.
        font_system
            .db_mut()
            .set_fantasy_family("Font Awesome 6 Free");
    }
}
