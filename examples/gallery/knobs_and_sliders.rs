use crate::style::MyStyle;
use crate::{MyAction, OVERLAY_Z_INDEX, RIGHT_CLICK_AREA_Z_INDEX};
use smol_str::SmolStr;
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

    separator: Separator,

    scroll_area: ScrollArea,
    floating_text_input: FloatingTextInput,

    text_input_param_id: Option<SmolStr>,
}

impl Elements {
    pub fn new(style: &MyStyle, window_cx: &mut WindowContext<MyAction>) -> Self {
        let scroll_area = ScrollArea::builder()
            .control_scissor_rect(SCROLL_AREA_SRECT)
            .z_index(RIGHT_CLICK_AREA_Z_INDEX)
            .build(window_cx);

        let floating_text_input = FloatingTextInput::builder()
            .on_result(|new_text| Action::FloatingTextInput(new_text).into())
            .rect(Rect::from_size(style.floating_text_input_size))
            .z_index(OVERLAY_Z_INDEX)
            .build(window_cx);

        window_cx.with_scissor_rect(SCROLL_AREA_SRECT, |window_cx| Self {
            knob_0: Knob::builder("knob_0")
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .build(window_cx),
            knob_0_label: Label::builder().text("Normal").build(window_cx),

            knob_1: Knob::builder("knob_1")
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .bipolar(true)
                .normal_value(0.5)
                .default_normal(0.5)
                .build(window_cx),
            knob_1_label: Label::builder().text("Bipolar").build(window_cx),

            knob_2: Knob::builder("knob_2")
                .class(MyStyle::CLASS_KNOB_2)
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .num_quantized_steps(Some(5))
                .build(window_cx),
            knob_2_label: Label::builder().text("Stepped").build(window_cx),

            slider_3: Slider::builder("slider_3")
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .build(window_cx),

            slider_4: Slider::builder("slider_4")
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .bipolar(true)
                .default_normal(0.5)
                .normal_value(0.5)
                .build(window_cx),

            slider_5: Slider::builder("slider_5")
                .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
                .on_open_text_entry(|info| Action::OpenTextInput(info).into())
                .on_tooltip_request(
                    |info| Action::ShowParamTooltip(info).into(),
                    Align2::TOP_CENTER,
                )
                .horizontal(true)
                .drag_horizontally(true)
                .build(window_cx),

            slider_6: Slider::builder("slider_6")
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
                .build(window_cx),

            separator: Separator::builder().build(window_cx),

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
        window_cx: &mut WindowContext<MyAction>,
    ) -> bool {
        let needs_layout = false;

        match action {
            Action::ParamUpdate(info) => {
                self.show_param_tooltip(&info.param_info, window_cx);

                if !info.is_gesturing() {
                    // Set the tooltip to auto-hide when gesturing is finished.
                    window_cx.auto_hide_tooltip();
                }
            }
            Action::ShowParamTooltip(info) => self.show_param_tooltip(&info.param_info, window_cx),
            Action::OpenTextInput(info) => {
                self.text_input_param_id = Some(info.param_info.id.clone());
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
                    window_cx.res,
                );
            }
            Action::FloatingTextInput(new_text) => {
                if let Some(param_id) = self.text_input_param_id.take() {
                    if let Some(new_text) = new_text {
                        match param_id.as_str() {
                            "knob_0" => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.knob_0.set_normal_value(v);
                                }
                            }
                            "knob_1" => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.knob_1.set_normal_value(v);
                                }
                            }
                            "knob_2" => {
                                if let Ok(v) = new_text.parse::<u32>() {
                                    self.knob_2.set_stepped_value(v);
                                }
                            }
                            "slider_3" => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.slider_3.set_normal_value(v);
                                }
                            }
                            "slider_4" => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.slider_4.set_normal_value(v);
                                }
                            }
                            "slider_5" => {
                                if let Ok(v) = new_text.parse::<f64>() {
                                    self.slider_5.set_normal_value(v);
                                }
                            }
                            "slider_6" => {
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

    fn show_param_tooltip(&mut self, param_info: &ParamInfo, window_cx: &WindowContext<MyAction>) {
        let get_text = || match param_info.value() {
            ParamValue::Normal(n) => format!("{:.4}", n),
            ParamValue::Stepped(s) => format!("{}", s),
        };

        match param_info.id.as_str() {
            "knob_0" => self
                .knob_0
                .show_tooltip(get_text, Align2::TOP_CENTER, window_cx),
            "knob_1" => self
                .knob_1
                .show_tooltip(get_text, Align2::TOP_CENTER, window_cx),
            "knob_2" => self
                .knob_2
                .show_tooltip(get_text, Align2::TOP_CENTER, window_cx),
            "slider_3" => self
                .slider_3
                .show_tooltip(get_text, Align2::TOP_CENTER, window_cx),
            "slider_4" => self
                .slider_4
                .show_tooltip(get_text, Align2::TOP_CENTER, window_cx),
            "slider_5" => self
                .slider_5
                .show_tooltip(get_text, Align2::TOP_CENTER, window_cx),
            "slider_6" => self
                .slider_6
                .show_tooltip(get_text, Align2::TOP_CENTER, window_cx),
            _ => return,
        };
    }

    pub fn layout(
        &mut self,
        content_rect: Rect,
        style: &MyStyle,
        window_cx: &mut WindowContext<MyAction>,
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
            window_cx.res,
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
            window_cx.res,
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
            window_cx.res,
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

        self.scroll_area.set_content_size(size(
            self.slider_6.max_x() + style.content_padding,
            self.slider_4.max_y() + style.content_padding,
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
        floating_text_input.hide();
    }
}
