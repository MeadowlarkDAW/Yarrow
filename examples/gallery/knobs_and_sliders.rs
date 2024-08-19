use crate::style::MyStyle;
use crate::{MyAction, OVERLAY_Z_INDEX, RIGHT_CLICK_AREA_Z_INDEX};
use yarrow::prelude::*;

const SCROLL_AREA_SRECT: ScissorRectID = ScissorRectID(1);

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ParamUpdate(ParamUpdate),
    ShowParamTooltip(ParamElementTooltipInfo),
    OpenTextInput(ParamOpenTextEntryInfo),
    FloatingTextInput(Option<String>),
    ScrollOffsetChanged(Vector),
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

    text_input_param_id: Option<u32>,
}

impl Elements {
    pub fn new(style: &MyStyle, cx: &mut WindowContext<'_, MyAction>) -> Self {
        let scroll_area = ScrollArea::builder()
            .on_scrolled(|scroll_offset| Action::ScrollOffsetChanged(scroll_offset).into())
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
                self.show_param_tooltip(info.param_info, info.is_gesturing(), cx);

                if !info.is_gesturing() {
                    // Set the tooltip to auto-hide when gesturing is finished.
                    cx.view.auto_hide_tooltip();
                }
            }
            Action::ShowParamTooltip(info) => self.show_param_tooltip(info.param_info, false, cx),
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
            Action::ScrollOffsetChanged(scroll_offset) => {
                cx.view
                    .update_scissor_rect(SCROLL_AREA_SRECT, None, Some(scroll_offset));
            }
        }

        needs_layout
    }

    fn show_param_tooltip(
        &mut self,
        param_info: ParamInfo,
        is_gesturing: bool,
        cx: &WindowContext<'_, MyAction>,
    ) {
        let el = match param_info.id {
            0 => &mut self.knob_0.el,
            1 => &mut self.knob_1.el,
            2 => &mut self.knob_2.el,
            3 => &mut self.slider_3.el,
            4 => &mut self.slider_4.el,
            5 => &mut self.slider_5.el,
            6 => &mut self.slider_6.el,
            _ => return,
        };

        // Don't show if the element is not being gestured and
        // it is not currently hovered.
        if !is_gesturing {
            if !cx.view.element_is_hovered(el) {
                return;
            }
        }

        let message = match param_info.value() {
            ParamValue::Normal(n) => format!("{:.4}", n),
            ParamValue::Stepped(s) => format!("{}", s),
        };

        el.show_tooltip(
            message,
            Align2::TOP_CENTER,
            // Don't auto-hide the tooltip when gesturing, otherwise
            // the tooltip may flicker.
            !is_gesturing,
        );
    }

    pub fn layout(
        &mut self,
        content_rect: Rect,
        style: &MyStyle,
        cx: &mut WindowContext<'_, MyAction>,
    ) {
        self.scroll_area.el.set_rect(content_rect);
        cx.view
            .update_scissor_rect(SCROLL_AREA_SRECT, Some(self.scroll_area.el.rect()), None);

        let start_pos = Point::new(style.content_padding, style.content_padding);

        self.knob_0.el.set_rect(Rect::new(
            start_pos,
            Size::new(style.knob_size, style.knob_size),
        ));
        self.knob_0_label.layout_aligned(
            Point::new(
                self.knob_0.el.rect().center().x,
                self.knob_0.el.rect().max_y() + style.param_label_padding,
            ),
            Align2::TOP_CENTER,
            cx.res,
        );

        self.knob_1.el.set_rect(Rect::new(
            Point::new(
                self.knob_0.el.rect().max_x() + style.param_spacing,
                start_pos.y,
            ),
            Size::new(style.knob_size, style.knob_size),
        ));
        self.knob_1_label.layout_aligned(
            Point::new(
                self.knob_1.el.rect().center().x,
                self.knob_1.el.rect().max_y() + style.param_label_padding,
            ),
            Align2::TOP_CENTER,
            cx.res,
        );

        self.knob_2.el.set_rect(Rect::new(
            Point::new(
                self.knob_1.el.rect().max_x() + style.param_spacing,
                start_pos.y,
            ),
            Size::new(style.knob_size, style.knob_size),
        ));
        self.knob_2_label.layout_aligned(
            Point::new(
                self.knob_2.el.rect().center().x,
                self.knob_2.el.rect().max_y() + style.param_label_padding,
            ),
            Align2::TOP_CENTER,
            cx.res,
        );

        self.separator.el.set_rect(Rect::new(
            Point::new(
                start_pos.x,
                self.knob_2_label.el.rect().max_y() + style.element_padding,
            ),
            Size::new(
                content_rect.width() - style.content_padding - style.content_padding,
                style.separator_width,
            ),
        ));

        self.slider_3.el.set_rect(Rect::new(
            Point::new(
                start_pos.x,
                self.separator.el.rect().max_y() + style.element_padding,
            ),
            Size::new(22.0, 100.0),
        ));

        self.slider_4.el.set_rect(Rect::new(
            Point::new(
                self.slider_3.el.rect().max_x() + style.param_spacing,
                self.slider_3.el.rect().min_y(),
            ),
            Size::new(22.0, 100.0),
        ));

        self.slider_5.el.set_rect(Rect::new(
            Point::new(
                self.slider_4.el.rect().max_x() + style.param_spacing,
                self.slider_4.el.rect().min_y(),
            ),
            Size::new(100.0, 22.0),
        ));

        self.slider_6.el.set_rect(Rect::new(
            Point::new(
                self.slider_5.el.rect().min_x(),
                self.slider_5.el.rect().max_y() + style.element_padding,
            ),
            Size::new(100.0, 22.0),
        ));

        self.scroll_area.set_content_size(Size::new(
            self.slider_6.el.rect().max_x() + style.content_padding,
            self.slider_4.el.rect().max_y() + style.content_padding,
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

        knob_0.el.set_hidden(hidden);
        knob_0_label.el.set_hidden(hidden);
        knob_1.el.set_hidden(hidden);
        knob_1_label.el.set_hidden(hidden);
        knob_2.el.set_hidden(hidden);
        knob_2_label.el.set_hidden(hidden);
        slider_3.el.set_hidden(hidden);
        slider_4.el.set_hidden(hidden);
        slider_5.el.set_hidden(hidden);
        slider_6.el.set_hidden(hidden);
        scroll_area.el.set_hidden(hidden);
        separator.el.set_hidden(hidden);
        floating_text_input.hide();
    }
}
