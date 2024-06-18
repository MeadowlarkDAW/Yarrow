use crate::style::MyStyle;
use crate::{MyAction, MAIN_Z_INDEX};
use yarrow::prelude::*;

pub const SCROLL_AREA_SCISSOR_RECT: ScissorRectID = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ParamUpdate(ParamUpdate),
    ShowParamTooltip(ParamElementTooltipInfo),
    OpenTextInput(ParamOpenTextEntryInfo),
    FloatingTextInput(Option<String>),
    ScrollOffsetChanged(Point),
}

pub struct Elements {
    knob_0: Knob,
    knob_0_label: Label,

    knob_1: Knob,
    knob_1_label: Label,

    knob_2: Knob,
    knob_2_label: Label,

    scroll_area: ScrollArea,
    floating_text_input: FloatingTextInput,

    text_input_param_id: Option<u32>,
}

impl Elements {
    pub fn new(style: &MyStyle, cx: &mut WindowContext<'_, MyAction>) -> Self {
        let knob_0 = Knob::builder(0, &style.knob_style_1)
            .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
            .on_open_text_entry(|info| Action::OpenTextInput(info).into())
            .on_tooltip_request(
                |info| Action::ShowParamTooltip(info).into(),
                Align2::TOP_CENTER,
            )
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .z_index(MAIN_Z_INDEX)
            .build(cx);
        let knob_0_label = Label::builder(&style.label_no_bg_style)
            .text("Normal")
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .z_index(MAIN_Z_INDEX)
            .build(cx);

        let knob_1 = Knob::builder(1, &style.knob_style_1)
            .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
            .on_open_text_entry(|info| Action::OpenTextInput(info).into())
            .on_tooltip_request(
                |info| Action::ShowParamTooltip(info).into(),
                Align2::TOP_CENTER,
            )
            .bipolar(true)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .z_index(MAIN_Z_INDEX)
            .normal_value(0.5)
            .default_normal(0.5)
            .build(cx);
        let knob_1_label = Label::builder(&style.label_no_bg_style)
            .text("Bipolar")
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .z_index(MAIN_Z_INDEX)
            .build(cx);

        let knob_2 = Knob::builder(2, &style.knob_style_2)
            .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
            .on_open_text_entry(|info| Action::OpenTextInput(info).into())
            .on_tooltip_request(
                |info| Action::ShowParamTooltip(info).into(),
                Align2::TOP_CENTER,
            )
            .num_quantized_steps(Some(5))
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .z_index(MAIN_Z_INDEX)
            .build(cx);
        let knob_2_label = Label::builder(&style.label_no_bg_style)
            .text("Stepped")
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .z_index(MAIN_Z_INDEX)
            .build(cx);

        let scroll_area = ScrollArea::builder(&style.scroll_bar_style)
            .on_scrolled(|scroll_offset| Action::ScrollOffsetChanged(scroll_offset).into())
            .z_index(0)
            .build(cx);

        let floating_text_input = FloatingTextInput::builder(&style.text_input_style)
            .on_result(|new_text| Action::FloatingTextInput(new_text).into())
            .bounding_rect(Rect::from_size(style.floating_text_input_size))
            .z_index(200)
            .build(cx);

        Self {
            knob_0,
            knob_0_label,
            knob_1,
            knob_1_label,
            knob_2,
            knob_2_label,
            scroll_area,
            floating_text_input,

            text_input_param_id: None,
        }
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
                self.show_param_tooltip(info.param_id, info.is_gesturing(), cx);

                if !info.is_gesturing() {
                    // Set the tooltip to auto-hide when gesturing is finished.
                    cx.view.auto_hide_tooltip();
                }
            }
            Action::ShowParamTooltip(info) => self.show_param_tooltip(info.param_id, false, cx),
            Action::OpenTextInput(info) => {
                self.text_input_param_id = Some(info.param_id);
                self.floating_text_input.show(
                    Some(&format!("{:.4}", info.normal_value)),
                    None,
                    info.bounds,
                    style.floating_text_input_align,
                    style.floating_text_input_padding,
                    cx.font_system,
                );
            }
            Action::FloatingTextInput(new_text) => {
                if let Some(param_id) = self.text_input_param_id.take() {
                    if let Some(new_text) = new_text {
                        if let Ok(new_val) = new_text.parse::<f64>() {
                            match param_id {
                                0 => self.knob_0.set_normal_value(new_val),
                                1 => self.knob_1.set_normal_value(new_val),
                                2 => self.knob_2.set_normal_value(new_val),
                                _ => {}
                            }
                        }
                    }
                }
            }
            Action::ScrollOffsetChanged(scroll_offset) => {
                cx.view
                    .update_scissor_rect(SCROLL_AREA_SCISSOR_RECT, None, Some(scroll_offset))
                    .unwrap();
            }
        }

        needs_layout
    }

    fn show_param_tooltip(
        &mut self,
        param_id: u32,
        is_gesturing: bool,
        cx: &WindowContext<'_, MyAction>,
    ) {
        let (normal_val, el) = match param_id {
            0 => (self.knob_0.normal_value(), &mut self.knob_0.el),
            1 => (self.knob_1.normal_value(), &mut self.knob_1.el),
            2 => (self.knob_2.normal_value(), &mut self.knob_2.el),
            _ => return,
        };

        if !is_gesturing {
            // Don't show if the element is not being gestured and
            // it is not currently hovered.
            if !cx.view.element_is_hovered(el) {
                return;
            }
        }

        el.show_tooltip(
            format!("{:.4}", normal_val),
            Align2::TOP_CENTER,
            // Don't auto-hide the tooltip when gesturing, otherwise
            // the tooltip may flicker.
            !is_gesturing,
        )
    }

    pub fn layout(
        &mut self,
        content_rect: Rect,
        style: &MyStyle,
        cx: &mut WindowContext<'_, MyAction>,
    ) {
        self.scroll_area.el.set_rect(content_rect);
        cx.view
            .update_scissor_rect(
                SCROLL_AREA_SCISSOR_RECT,
                Some(self.scroll_area.el.rect()),
                None,
            )
            .unwrap();

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
        );

        self.scroll_area.set_content_size(Size::new(
            self.knob_0.el.rect().max_x() + style.content_padding,
            self.knob_0.el.rect().max_y() + style.content_padding,
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
            scroll_area,
            floating_text_input,
            text_input_param_id: _,
        } = self;

        knob_0.el.set_hidden(hidden);
        knob_0_label.el.set_hidden(hidden);
        knob_1.el.set_hidden(hidden);
        knob_1_label.el.set_hidden(hidden);
        knob_2.el.set_hidden(hidden);
        knob_2_label.el.set_hidden(hidden);
        scroll_area.el.set_hidden(hidden);
        floating_text_input.hide();
    }
}
