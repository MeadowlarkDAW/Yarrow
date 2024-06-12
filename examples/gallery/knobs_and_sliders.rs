use crate::style::MyStyle;
use crate::{MyAction, MAIN_Z_INDEX};
use yarrow::prelude::*;

pub const SCROLL_AREA_SCISSOR_RECT: ScissorRectID = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ParamUpdate(ParamUpdate),
    ScrollOffsetChanged(Point),
}

pub struct Elements {
    knob_1: Knob,
    scroll_area: ScrollArea,
}

impl Elements {
    pub fn new(style: &MyStyle, cx: &mut WindowContext<'_, MyAction>) -> Self {
        let knob_1 = Knob::builder(0, &style.knob_style_1)
            .on_gesture(|param_update| Action::ParamUpdate(param_update).into())
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .z_index(MAIN_Z_INDEX)
            .normal_value(0.5)
            .default_normal(0.5)
            .bipolar(true)
            .build(cx);

        let scroll_area = ScrollArea::builder(&style.scroll_bar_style)
            .on_scrolled(|scroll_offset| Action::ScrollOffsetChanged(scroll_offset).into())
            .z_index(0)
            .build(cx);

        Self {
            knob_1,
            scroll_area,
        }
    }

    /// Returns `true` if the the contents need to be laid out.
    pub fn handle_action(&mut self, action: Action, cx: &mut WindowContext<'_, MyAction>) -> bool {
        let needs_layout = false;

        match action {
            Action::ParamUpdate(_param_update) => {}
            Action::ScrollOffsetChanged(scroll_offset) => {
                cx.view
                    .update_scissor_rect(SCROLL_AREA_SCISSOR_RECT, None, Some(scroll_offset))
                    .unwrap();
            }
        }

        needs_layout
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

        self.knob_1
            .el
            .set_rect(Rect::new(start_pos, Size::new(35.0, 35.0)));

        self.scroll_area.set_content_size(Size::new(
            200.0 + style.content_padding,
            200.0 + style.content_padding,
        ));
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        // Destructuring helps to make sure you didn't miss any elements.
        let Self {
            knob_1,
            scroll_area,
        } = self;

        knob_1.el.set_hidden(hidden);
        scroll_area.el.set_hidden(hidden);
    }
}
