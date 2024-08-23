use crate::style::MyStyle;
use crate::{MyAction, OVERLAY_Z_INDEX, RIGHT_CLICK_AREA_Z_INDEX};
use yarrow::elements::progress_bar::ProgressBar;
use yarrow::prelude::*;

const SCROLL_AREA_SRECT: ScissorRectID = ScissorRectID(1);

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ParamUpdate(ParamUpdate),
    ShowParamTooltip(ParamElementTooltipInfo),
    OpenTextInput(ParamOpenTextEntryInfo),
    FloatingTextInput(Option<String>),
}

pub struct Elements {
    knob_0: Knob,
    knob_0_label: Label,

    knob_1: Knob,
    knob_1_label: Label,

    knob_2: Knob,
    knob_2_label: Label,

    slider_3: Slider,
    slider_4: Slider,
    slider_5: Slider,
    slider_6: Slider,

    progress_bar: ProgressBar,

    separator: Separator,

    scroll_area: ScrollArea,
    floating_text_input: FloatingTextInput,

    text_input_param_id: Option<u32>,
}

impl Elements {
    pub fn new(style: &MyStyle, cx: &mut WindowContext<'_, MyAction>) -> Self {
        let scroll_area = ScrollArea::builder()
            .control_scissor_rect(SCROLL_AREA_SRECT)
            .z_index(RIGHT_CLICK_AREA_Z_INDEX)
            .build(cx);

        let floating_text_input = FloatingTextInput::builder()
            .on_result(|new_text| Action::FloatingTextInput(new_text).into())
            .rect(Rect::from_size(style.floating_text_input_size))
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        cx.with_scissor_rect(SCROLL_AREA_SRECT, |cx| Self {
            knob_0: Knob::builder(0)
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .build(cx),
            knob_0_label: Label::builder().text("Normal").build(cx),

            knob_1: Knob::builder(1)
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .bipolar(true)
                .normal_value(0.5)
                .default_normal(0.5)
                .build(cx),
            knob_1_label: Label::builder().text("Bipolar").build(cx),

            knob_2: Knob::builder(2)
                .class(MyStyle::CLASS_KNOB_2)
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .num_quantized_steps(Some(5))
                .build(cx),
            knob_2_label: Label::builder().text("Stepped").build(cx),

            slider_3: Slider::builder(3)
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .build(cx),

            slider_4: Slider::builder(4)
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .bipolar(true)
                .default_normal(0.5)
                .normal_value(0.5)
                .build(cx),

            slider_5: Slider::builder(5)
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .horizontal(true)
                .drag_horizontally(true)
                .build(cx),

            slider_6: Slider::builder(6)
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .horizontal(true)
                .drag_horizontally(true)
                .bipolar(true)
                .default_normal(0.5)
                .normal_value(0.5)
                .build(cx),

            separator: Separator::builder().build(cx),

            progress_bar: ProgressBar::builder().build(cx),

            scroll_area,
            floating_text_input,

            text_input_param_id: None,
        })
    }

    /// Returns `true` if the the contents need to be laid out.
    pub fn handle_action(
        &mut self,
        action: Action,
        style: &MyStyle,
        cx: &mut WindowContext<'_, MyAction>,
    ) -> bool {
        let needs_layout = false;

        match action {
            Action::ParamUpdate(info) => {
                self.show_param_tooltip(info.param_info, cx);

                if info.param_info.id == 0 {
                    self.progress_bar
                        .set_percent(info.param_info.normal_value as f32);
                }

                if !info.is_gesturing() {
                    // Set the tooltip to auto-hide when gesturing is finished.
                    cx.view.auto_hide_tooltip();
                }
            }
            Action::ShowParamTooltip(info) => self.show_param_tooltip(info.param_info, cx),
            Action::OpenTextInput(info) => {
                self.text_input_param_id = Some(info.param_info.id);
                let text = match info.param_info.value() {
                    ParamValue::Normal(n) => format!("{:.4}", n),
                    ParamValue::Stepped(s) => format!("{}", s),
                };

                self.floating_text_input.show(
                    Some(&text),
                    None,
                    info.bounds,
                    style.floating_text_input_align,
                    style.floating_text_input_padding,
                    &mut cx.res,
                );
            }
            Action::FloatingTextInput(new_text) => {
                if let Some(param_id) = self.text_input_param_id.take() {
                    if let Some(new_text) = new_text {
                        match param_id {
                            0 => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.knob_0.set_normal_value(v);
                                }
                            }
                            1 => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.knob_1.set_normal_value(v);
                                }
                            }
                            2 => {
                                if let Ok(v) = new_text.parse::<u32>() {
                                    self.knob_2.set_stepped_value(v);
                                }
                            }
                            3 => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.slider_3.set_normal_value(v);
                                }
                            }
                            4 => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.slider_4.set_normal_value(v);
                                }
                            }
                            5 => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.slider_5.set_normal_value(v);
                                }
                            }
                            6 => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.slider_6.set_normal_value(v);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        needs_layout
    }

    fn show_param_tooltip(&mut self, param_info: ParamInfo, cx: &WindowContext<'_, MyAction>) {
        let get_text = || match param_info.value() {
            ParamValue::Normal(n) => format!("{:.4}", n),
            ParamValue::Stepped(s) => format!("{}", s),
        };

        match param_info.id {
            0 => self.knob_0.show_tooltip(get_text, Align2::TOP_CENTER, cx),
            1 => self.knob_1.show_tooltip(get_text, Align2::TOP_CENTER, cx),
            2 => self.knob_2.show_tooltip(get_text, Align2::TOP_CENTER, cx),
            3 => self.slider_3.show_tooltip(get_text, Align2::TOP_CENTER, cx),
            4 => self.slider_4.show_tooltip(get_text, Align2::TOP_CENTER, cx),
            5 => self.slider_5.show_tooltip(get_text, Align2::TOP_CENTER, cx),
            6 => self.slider_6.show_tooltip(get_text, Align2::TOP_CENTER, cx),
            _ => return,
        };
    }

    pub fn layout(
        &mut self,
        content_rect: Rect,
        style: &MyStyle,
        cx: &mut WindowContext<'_, MyAction>,
    ) {
        self.scroll_area.set_rect(content_rect);

        let start_pos = point(style.content_padding, style.content_padding);

        self.knob_0.set_rect(rect(
            start_pos.x,
            start_pos.y,
            style.knob_size,
            style.knob_size,
        ));
        self.knob_0_label.layout_aligned(
            point(
                self.knob_0.center().x,
                self.knob_0.max_y() + style.param_label_padding,
            ),
            Align2::TOP_CENTER,
            cx.res,
        );

        self.knob_1.set_rect(rect(
            self.knob_0.max_x() + style.param_spacing,
            start_pos.y,
            style.knob_size,
            style.knob_size,
        ));
        self.knob_1_label.layout_aligned(
            point(
                self.knob_1.center().x,
                self.knob_1.max_y() + style.param_label_padding,
            ),
            Align2::TOP_CENTER,
            cx.res,
        );

        self.knob_2.set_rect(rect(
            self.knob_1.max_x() + style.param_spacing,
            start_pos.y,
            style.knob_size,
            style.knob_size,
        ));
        self.knob_2_label.layout_aligned(
            point(
                self.knob_2.center().x,
                self.knob_2.max_y() + style.param_label_padding,
            ),
            Align2::TOP_CENTER,
            cx.res,
        );

        self.separator.set_rect(rect(
            start_pos.x,
            self.knob_2_label.max_y() + style.element_padding,
            content_rect.width() - style.content_padding - style.content_padding,
            style.separator_width,
        ));

        self.slider_3.set_rect(rect(
            start_pos.x,
            self.separator.max_y() + style.element_padding,
            22.0,
            100.0,
        ));

        self.slider_4.set_rect(rect(
            self.slider_3.max_x() + style.param_spacing,
            self.slider_3.min_y(),
            22.0,
            100.0,
        ));

        self.slider_5.set_rect(rect(
            self.slider_4.max_x() + style.param_spacing,
            self.slider_4.min_y(),
            100.0,
            22.0,
        ));

        self.slider_6.set_rect(rect(
            self.slider_5.min_x(),
            self.slider_5.max_y() + style.element_padding,
            100.0,
            22.0,
        ));

        self.progress_bar.set_rect(Rect::new(
            Point::new(
                start_pos.x,
                self.slider_4.rect().max_y() + style.element_padding,
            ),
            Size::new(200.0, 22.0),
        ));

        self.scroll_area.set_content_size(Size::new(
            self.slider_6.rect().max_x() + style.content_padding,
            self.progress_bar.rect().max_y() + style.content_padding,
        ));
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        // Destructuring helps to make sure you didn't miss any elements.
        let Self {
            knob_0,
            knob_0_label,
            knob_1,
            knob_1_label,
            knob_2,
            knob_2_label,
            slider_3,
            slider_4,
            slider_5,
            slider_6,
            scroll_area,
            floating_text_input,
            separator,
            progress_bar,
            text_input_param_id: _,
        } = self;

        knob_0.set_hidden(hidden);
        knob_0_label.set_hidden(hidden);
        knob_1.set_hidden(hidden);
        knob_1_label.set_hidden(hidden);
        knob_2.set_hidden(hidden);
        knob_2_label.set_hidden(hidden);
        slider_3.set_hidden(hidden);
        slider_4.set_hidden(hidden);
        slider_5.set_hidden(hidden);
        slider_6.set_hidden(hidden);
        scroll_area.set_hidden(hidden);
        separator.set_hidden(hidden);
        progress_bar.set_hidden(hidden);
        floating_text_input.hide();
    }
}
