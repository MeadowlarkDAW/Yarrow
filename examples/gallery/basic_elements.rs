use crate::style::MyStyle;
use crate::{MyAction, MAIN_Z_INDEX, OVERLAY_Z_INDEX, SCROLL_AREA_Z_INDEX};
use yarrow::prelude::*;

pub const SCROLL_AREA_SCISSOR_RECT: ScissorRectID = 1;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub enum DropDownOption {
    #[display(fmt = "Option A")]
    A,
    #[display(fmt = "Option B")]
    B,
    #[display(fmt = "Option C")]
    C,
    #[display(fmt = "Option D")]
    D,
}
impl DropDownOption {
    pub const ALL: [Self; 4] = [Self::A, Self::B, Self::C, Self::D];
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub enum TextMenuOption {
    Cut,
    Copy,
    Paste,
    #[display(fmt = "Select All")]
    SelectAll,
}
impl TextMenuOption {
    pub const ALL: [Self; 4] = [Self::Cut, Self::Copy, Self::Paste, Self::SelectAll];

    pub fn right_text(&self) -> &'static str {
        match self {
            Self::Cut => "Ctrl+X",
            Self::Copy => "Ctrl+C",
            Self::Paste => "Ctrl+V",
            Self::SelectAll => "Ctrl+A",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ClickMePressed,
    ToggleValue(bool),
    OptionSelected(DropDownOption),
    OpenDropDown,
    OpenRightClickMenu(Point),
    RightClickOptionSelected(usize),
    TextChanged(String),
    OpenTextInputMenu(Point),
    TextInputMenuOptionSelected(TextMenuOption),
    ScrollOffsetChanged(Point),
}

pub struct Elements {
    label: Label,
    dual_label: DualLabel,
    click_me_btn: Button,
    switch: Switch,
    toggle_btn: ToggleButton,
    dual_toggle_btn: DualToggleButton,
    radio_group: RadioButtonGroup,
    drop_down_menu_btn: DualButton,
    drop_down_menu: DropDownMenu,
    text_input: TextInput,
    text_input_menu: DropDownMenu,
    right_click_area: ClickArea,
    right_click_menu: DropDownMenu,
    scroll_area: ScrollArea,
    separator_1: Separator,
    separator_2: Separator,
}

impl Elements {
    pub fn new(style: &MyStyle, cx: &mut WindowContext<'_, MyAction>) -> Self {
        let label = Label::builder(&style.label_style)
            .text("Label")
            .z_index(MAIN_Z_INDEX)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .build(cx);

        let dual_label = DualLabel::builder(&style.dual_label_style)
            .left_text('\u{f05a}')
            .right_text("Dual Label")
            .z_index(MAIN_Z_INDEX)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .build(cx);

        let click_me_btn = Button::builder(&style.button_style)
            .text("Click Me!")
            .on_select(Action::ClickMePressed.into())
            .tooltip_message("A cool button", Align2::TOP_CENTER)
            .z_index(MAIN_Z_INDEX)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .build(cx);

        let toggle_btn = ToggleButton::builder(&style.toggle_btn_style)
            .text("off")
            .on_toggled(|toggled| Action::ToggleValue(toggled).into())
            .z_index(MAIN_Z_INDEX)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .build(cx);

        let dual_toggle_btn = DualToggleButton::builder(&style.dual_toggle_btn_style)
            .left_text('\u{23fb}')
            .right_text("off")
            .on_toggled(|toggled| Action::ToggleValue(toggled).into())
            .z_index(MAIN_Z_INDEX)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .build(cx);

        let switch = Switch::builder(&style.switch_style)
            .on_toggled(|toggled| Action::ToggleValue(toggled).into())
            .z_index(MAIN_Z_INDEX)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .build(cx);

        let radio_group = RadioButtonGroup::new(
            DropDownOption::ALL.iter().map(|o| format!("{}", *o)),
            0,
            |id| Action::OptionSelected(DropDownOption::ALL[id]).into(),
            &style.label_no_bg_style,
            &style.radio_btn_style,
            MAIN_Z_INDEX,
            SCROLL_AREA_SCISSOR_RECT,
            cx,
        );

        let text_input = TextInput::builder(&style.text_input_style)
            .placeholder_text("write something...")
            .tooltip_message("A text input :)", Align2::TOP_LEFT)
            .on_changed(|text| Action::TextChanged(text).into())
            .on_right_click(|pos| Action::OpenTextInputMenu(pos).into())
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .password_mode(false) // There is an optional password mode if desired.
            .z_index(MAIN_Z_INDEX)
            .build(cx);
        let text_input_menu = DropDownMenu::builder(&style.menu_style)
            .entries(
                TextMenuOption::ALL
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let right_text = s.right_text();
                        MenuEntry::Option {
                            left_text: format!("{s}"),
                            right_text: right_text.into(),
                            unique_id: i,
                        }
                    })
                    .collect(),
            )
            .on_entry_selected(|id| {
                Action::TextInputMenuOptionSelected(TextMenuOption::ALL[id]).into()
            })
            .z_index(100)
            .build(cx);

        let drop_down_menu_btn = DualButton::builder(&style.drop_down_btn_style)
            .left_text(format!("{}", DropDownOption::ALL[0]))
            .right_text('\u{2304}')
            .on_select(Action::OpenDropDown.into())
            .z_index(MAIN_Z_INDEX)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .build(cx);
        let drop_down_menu = DropDownMenu::builder(&style.menu_style)
            .entries(
                DropDownOption::ALL
                    .iter()
                    .enumerate()
                    .map(|(i, s)| MenuEntry::Option {
                        left_text: format!("{s}"),
                        right_text: String::new(),
                        unique_id: i,
                    })
                    .collect(),
            )
            .on_entry_selected(|id| Action::OptionSelected(DropDownOption::ALL[id]).into())
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let right_click_area = ClickArea::builder()
            .button(PointerButton::Secondary)
            .on_clicked(|info| Action::OpenRightClickMenu(info.click_position).into())
            .z_index(0)
            .build(cx);
        let right_click_menu = DropDownMenu::builder(&style.menu_style)
            .entries(
                ["I am", "a right", "click", "menu"]
                    .iter()
                    .enumerate()
                    .map(|(i, s)| MenuEntry::Option {
                        left_text: String::from(*s),
                        right_text: String::new(),
                        unique_id: i,
                    })
                    .collect(),
            )
            .on_entry_selected(|id| Action::RightClickOptionSelected(id).into())
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let scroll_area = ScrollArea::builder(&style.scroll_bar_style)
            .on_scrolled(|scroll_offset| Action::ScrollOffsetChanged(scroll_offset).into())
            .z_index(SCROLL_AREA_Z_INDEX)
            .build(cx);

        let separator_1 = Separator::builder(&style.separator_style)
            .z_index(MAIN_Z_INDEX)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .build(cx);

        let separator_2 = Separator::builder(&style.separator_style)
            .z_index(MAIN_Z_INDEX)
            .scissor_rect(SCROLL_AREA_SCISSOR_RECT)
            .build(cx);

        Self {
            label,
            dual_label,
            click_me_btn,
            switch,
            toggle_btn,
            dual_toggle_btn,
            radio_group,
            drop_down_menu_btn,
            drop_down_menu,
            text_input,
            text_input_menu,
            right_click_area,
            right_click_menu,
            scroll_area,
            separator_1,
            separator_2,
        }
    }

    /// Returns `true` if the the contents need to be laid out.
    pub fn handle_action(&mut self, action: Action, cx: &mut WindowContext<'_, MyAction>) -> bool {
        let mut needs_layout = false;

        match action {
            Action::ClickMePressed => {}
            Action::ToggleValue(toggled) => {
                self.switch.set_toggled(toggled);
                self.toggle_btn.set_toggled(toggled);
                self.dual_toggle_btn.set_toggled(toggled);

                let toggle_text = if toggled { "on" } else { "off" };
                self.toggle_btn.set_text(toggle_text, &mut cx.font_system);
                self.dual_toggle_btn
                    .set_right_text(toggle_text, &mut cx.font_system);

                needs_layout = true;
            }
            Action::OptionSelected(option) => {
                self.radio_group.updated_selected(option as usize);
                self.drop_down_menu_btn
                    .set_left_text(&format!("{}", option), &mut cx.font_system);
            }
            Action::OpenDropDown => {
                // Because the drop-down menu button may be offset by the scroll area,
                // get the correct position via this method.
                let rect = cx.view.element_rect(&self.drop_down_menu_btn.el).unwrap();
                self.drop_down_menu
                    .open(Some(Point::new(rect.min_x(), rect.max_y())));
            }
            Action::OpenRightClickMenu(position) => {
                self.right_click_menu.open(Some(position));
            }
            Action::RightClickOptionSelected(_option) => {}
            Action::TextChanged(_text) => {}
            Action::OpenTextInputMenu(position) => {
                self.text_input_menu.open(Some(position));
            }
            Action::TextInputMenuOptionSelected(option) => match option {
                TextMenuOption::Cut => self.text_input.perform_cut_action(),
                TextMenuOption::Copy => self.text_input.perform_copy_action(),
                TextMenuOption::Paste => self.text_input.perform_paste_action(),
                TextMenuOption::SelectAll => self.text_input.perform_select_all_action(),
            },
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
        self.right_click_area.el.set_rect(content_rect);

        self.scroll_area.el.set_rect(content_rect);
        cx.view
            .update_scissor_rect(
                SCROLL_AREA_SCISSOR_RECT,
                Some(self.scroll_area.el.rect()),
                None,
            )
            .unwrap();

        let start_pos = Point::new(style.content_padding, style.content_padding);

        // The position of an element is relative to the scissor rect it is
        // assigned to.
        self.click_me_btn.layout(start_pos);

        self.label.layout(Point::new(
            self.click_me_btn.el.rect().max_x() + style.element_padding,
            start_pos.y,
        ));

        self.dual_label.layout(Point::new(
            self.label.el.rect().max_x() + style.element_padding,
            start_pos.y,
        ));

        let mut toggle_btn_rect = Rect::new(
            Point::new(
                0.0,
                self.click_me_btn.el.rect().max_y() + style.element_padding,
            ),
            self.toggle_btn.desired_padded_size(),
        );

        self.switch.layout_aligned(
            Point::new(start_pos.x, toggle_btn_rect.center().y),
            Align2::CENTER_LEFT,
        );

        toggle_btn_rect.origin.x = self.switch.el.rect().max_x() + style.element_padding;
        self.toggle_btn.el.set_rect(toggle_btn_rect);

        self.dual_toggle_btn.layout(Point::new(
            toggle_btn_rect.max_x() + style.element_padding,
            self.toggle_btn.el.rect().min_y(),
        ));

        self.separator_1.el.set_rect(Rect::new(
            Point::new(start_pos.x, toggle_btn_rect.max_y() + style.element_padding),
            Size::new(
                content_rect.width() - style.content_padding - style.content_padding,
                style.separator_width,
            ),
        ));

        self.drop_down_menu_btn.el.set_rect(Rect::new(
            Point::new(
                start_pos.x,
                self.separator_1.el.rect().max_y() + style.element_padding,
            ),
            Size::new(
                style.drop_down_btn_width,
                self.drop_down_menu_btn.desired_padded_size().height,
            ),
        ));

        self.radio_group.layout(
            Point::new(
                start_pos.x,
                self.drop_down_menu_btn.el.rect().max_y() + style.element_padding,
            ),
            style.radio_group_row_padding,
            style.radio_group_column_padding,
            None,
            Point::default(),
        );

        self.separator_2.el.set_rect(Rect::new(
            Point::new(
                start_pos.x,
                self.radio_group.bounds().max_y() + style.element_padding,
            ),
            Size::new(
                content_rect.width() - style.content_padding - style.content_padding,
                style.separator_width,
            ),
        ));

        self.text_input.el.set_rect(Rect::new(
            Point::new(
                start_pos.x,
                self.separator_2.el.rect().max_y() + style.element_padding,
            ),
            style.text_input_size,
        ));

        self.scroll_area.set_content_size(Size::new(
            self.dual_label.el.rect().max_x() + style.content_padding,
            self.text_input.el.rect().max_y() + style.content_padding,
        ));
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        // Destructuring helps to make sure you didn't miss any elements.
        let Self {
            label,
            dual_label,
            click_me_btn,
            switch,
            toggle_btn,
            dual_toggle_btn,
            radio_group,
            drop_down_menu_btn,
            drop_down_menu,
            text_input,
            text_input_menu,
            right_click_area,
            right_click_menu,
            scroll_area,
            separator_1,
            separator_2,
        } = self;

        label.el.set_hidden(hidden);
        dual_label.el.set_hidden(hidden);
        click_me_btn.el.set_hidden(hidden);
        switch.el.set_hidden(hidden);
        toggle_btn.el.set_hidden(hidden);
        dual_toggle_btn.el.set_hidden(hidden);
        radio_group.set_hidden(hidden);
        drop_down_menu_btn.el.set_hidden(hidden);
        drop_down_menu.el.set_hidden(hidden);
        text_input.el.set_hidden(hidden);
        text_input_menu.el.set_hidden(hidden);
        right_click_area.el.set_hidden(hidden);
        right_click_menu.el.set_hidden(hidden);
        scroll_area.el.set_hidden(hidden);
        separator_1.el.set_hidden(hidden);
        separator_2.el.set_hidden(hidden);
    }
}
