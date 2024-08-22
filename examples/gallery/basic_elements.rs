use crate::style::{MyIcon, MyStyle};
use crate::{MyAction, OVERLAY_Z_INDEX, RIGHT_CLICK_AREA_Z_INDEX, SCROLL_AREA_Z_INDEX};
use yarrow::prelude::*;

const SCROLL_AREA_SRECT: ScissorRectID = ScissorRectID(0);

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
pub enum DropDownOption {
    #[display("Option A")]
    A,
    #[display("Option B")]
    B,
    #[display("Option C")]
    C,
    #[display("Option D")]
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
    #[display("Select All")]
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

    pub fn as_text_input_option(&self) -> TextInputAction {
        match self {
            Self::Cut => TextInputAction::Cut,
            Self::Copy => TextInputAction::Copy,
            Self::Paste => TextInputAction::Paste,
            Self::SelectAll => TextInputAction::SelectAll,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputID {
    Standard,
    Search,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ClickMePressed,
    IconBtnPressed,
    ToggleValue(bool),
    OptionSelected(DropDownOption),
    OpenDropDown,
    OpenRightClickMenu(Point),
    RightClickOptionSelected(usize),
    TextChanged(String),
    SearchTextChanged(String),
    OpenTextInputMenu {
        click_pos: Point,
        text_input_id: TextInputID,
    },
    TextInputMenuOptionSelected(TextMenuOption),
}

pub struct Elements {
    label: Label,
    icon: Icon,
    icon_label: Label,
    click_me_btn: Button,
    icon_btn: Button,
    switch: Switch,
    toggle_btn: ToggleButton,
    icon_toggle_btn: ToggleButton,
    icon_label_toggle_btn: ToggleButton,
    radio_group: RadioButtonGroup,
    drop_down_menu_btn: Button,
    drop_down_menu: DropDownMenu,
    text_input: TextInput,
    text_input_menu: DropDownMenu,
    search_text_input: IconTextInput,
    right_click_area: ClickArea,
    right_click_menu: DropDownMenu,
    scroll_area: ScrollArea,
    separator_1: Separator,
    separator_2: Separator,
    active_text_input_menu: Option<TextInputID>,
}

impl Elements {
    pub fn new(cx: &mut WindowContext<'_, MyAction>) -> Self {
        let text_input_menu = DropDownMenu::builder()
            .entries(
                TextMenuOption::ALL
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let icon = match i {
                            0 => MyIcon::Cut,
                            1 => MyIcon::Copy,
                            2 => MyIcon::Paste,
                            _ => MyIcon::Select,
                        } as IconID;

                        MenuEntry::Option {
                            left_icon: Some(icon),
                            icon_scale: 1.0.into(),
                            left_text: format!("{s}"),
                            right_text: Some(s.right_text().into()),
                            unique_id: i,
                        }
                    })
                    .collect(),
            )
            .on_entry_selected(|id| {
                Action::TextInputMenuOptionSelected(TextMenuOption::ALL[id]).into()
            })
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let drop_down_menu = DropDownMenu::builder()
            .entries(
                DropDownOption::ALL
                    .iter()
                    .enumerate()
                    .map(|(i, s)| MenuEntry::option(format!("{s}"), i))
                    .collect(),
            )
            .on_entry_selected(|id| Action::OptionSelected(DropDownOption::ALL[id]).into())
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let right_click_area = ClickArea::builder()
            .button(PointerButton::Secondary)
            .on_clicked(|info| Action::OpenRightClickMenu(info.click_position).into())
            .z_index(RIGHT_CLICK_AREA_Z_INDEX)
            .build(cx);

        let right_click_menu = DropDownMenu::builder()
            .entries(
                ["I am", "a right", "click", "menu"]
                    .iter()
                    .enumerate()
                    .map(|(i, s)| MenuEntry::option(*s, i))
                    .collect(),
            )
            .on_entry_selected(|id| Action::RightClickOptionSelected(id).into())
            .z_index(OVERLAY_Z_INDEX)
            .build(cx);

        let scroll_area = ScrollArea::builder()
            .control_scissor_rect(SCROLL_AREA_SRECT)
            .z_index(SCROLL_AREA_Z_INDEX)
            .build(cx);

        cx.with_scissor_rect(SCROLL_AREA_SRECT, |cx| {
            Self {
                label: Label::builder()
                    .class(MyStyle::CLASS_FANCY_LABEL)
                    .text("Label")
                    .build(cx),

                icon: Icon::builder()
                    .class(MyStyle::CLASS_FANCY_LABEL)
                    .icon(MyIcon::Info)
                    .build(cx),

                icon_label: Label::builder()
                    .class(MyStyle::CLASS_FANCY_LABEL)
                    .icon(MyIcon::Info)
                    .text("Icon Label")
                    .build(cx),

                click_me_btn: Button::builder()
                    .text("Click Me!")
                    .on_select(Action::ClickMePressed.into())
                    .tooltip("A cool button", Align2::TOP_CENTER)
                    .build(cx),

                icon_btn: Button::builder()
                    .icon(MyIcon::Save)
                    .on_select(Action::IconBtnPressed.into())
                    .build(cx),

                toggle_btn: ToggleButton::builder()
                    .text("off")
                    .on_toggled(|toggled| Action::ToggleValue(toggled).into())
                    .build(cx),

                icon_toggle_btn: ToggleButton::builder()
                    .icon(MyIcon::PowerOn)
                    .on_toggled(|toggled| Action::ToggleValue(toggled).into())
                    .build(cx),

                icon_label_toggle_btn: ToggleButton::builder()
                    .icon(MyIcon::PowerOn)
                    .text("off")
                    .on_toggled(|toggled| Action::ToggleValue(toggled).into())
                    .build(cx),

                switch: Switch::builder()
                    .on_toggled(|toggled| Action::ToggleValue(toggled).into())
                    .build(cx),

                radio_group: RadioButtonGroup::new(
                    DropDownOption::ALL.iter().map(|o| format!("{}", *o)),
                    0,
                    |id| Action::OptionSelected(DropDownOption::ALL[id]).into(),
                    None,
                    None,
                    None,
                    None,
                    cx,
                ),

                text_input: TextInput::builder()
                    .placeholder_text("write something...")
                    .tooltip("A text input :)", Align2::TOP_LEFT)
                    .on_changed(|text| Action::TextChanged(text).into())
                    .on_right_click(|pos| {
                        Action::OpenTextInputMenu {
                            click_pos: pos,
                            text_input_id: TextInputID::Standard,
                        }
                        .into()
                    })
                    .password_mode(false) // There is an optional password mode if desired.
                    .build(cx),

                search_text_input: IconTextInput::builder()
                    .placeholder_text("search something...")
                    .icon(MyIcon::Search)
                    .on_changed(|text| Action::SearchTextChanged(text).into())
                    .on_right_click(|pos| {
                        Action::OpenTextInputMenu {
                            click_pos: pos,
                            text_input_id: TextInputID::Search,
                        }
                        .into()
                    })
                    .password_mode(false) // There is an optional password mode if desired.
                    .build(cx),

                drop_down_menu_btn: Button::builder()
                    .text_icon_layout(TextIconLayout::LeftAlignTextRightAlignIcon)
                    .text(format!("{}", DropDownOption::ALL[0]))
                    .icon(MyIcon::Dropdown)
                    .on_select(Action::OpenDropDown.into())
                    .build(cx),

                separator_1: Separator::builder().build(cx),
                separator_2: Separator::builder().build(cx),

                text_input_menu,
                drop_down_menu,
                right_click_area,
                right_click_menu,
                scroll_area,

                active_text_input_menu: None,
            }
        })
    }

    /// Returns `true` if the the contents need to be laid out.
    pub fn handle_action(&mut self, action: Action, cx: &mut WindowContext<'_, MyAction>) -> bool {
        let mut needs_layout = false;

        match action {
            Action::ClickMePressed => {}
            Action::IconBtnPressed => {}
            Action::ToggleValue(toggled) => {
                self.switch.set_toggled(toggled);
                self.toggle_btn.set_toggled(toggled);
                self.icon_toggle_btn.set_toggled(toggled);
                self.icon_label_toggle_btn.set_toggled(toggled);

                if toggled {
                    self.toggle_btn.set_text(Some("on"), cx.res);
                    self.icon_label_toggle_btn.set_text(Some("on"), cx.res);
                } else {
                    self.toggle_btn.set_text(Some("off"), cx.res);
                    self.icon_label_toggle_btn.set_text(Some("off"), cx.res);
                }

                needs_layout = true;
            }
            Action::OptionSelected(option) => {
                self.radio_group.updated_selected(option as usize);
                self.drop_down_menu_btn
                    .set_text(Some(&format!("{}", option)), cx.res);
            }
            Action::OpenDropDown => {
                // Because the drop-down menu button may be offset by the scroll area,
                // get the correct position via this method.
                let rect = self.drop_down_menu_btn.rect_in_window(cx);
                self.drop_down_menu
                    .open(Some(Point::new(rect.min_x(), rect.max_y())));
            }
            Action::OpenRightClickMenu(position) => {
                self.right_click_menu.open(Some(position));
            }
            Action::RightClickOptionSelected(_option) => {}
            Action::TextChanged(_text) => {}
            Action::SearchTextChanged(_text) => {}
            Action::OpenTextInputMenu {
                click_pos,
                text_input_id,
            } => {
                self.active_text_input_menu = Some(text_input_id);
                self.text_input_menu.open(Some(click_pos));
            }
            Action::TextInputMenuOptionSelected(option) => {
                if let Some(id) = self.active_text_input_menu.take() {
                    let action = option.as_text_input_option();

                    match id {
                        TextInputID::Standard => self.text_input.perform_action(action),
                        TextInputID::Search => self.search_text_input.perform_action(action),
                    }
                }
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
        self.right_click_area.set_rect(content_rect);

        self.scroll_area.set_rect(content_rect);

        let start_pos = Point::new(style.content_padding, style.content_padding);

        // The position of an element is relative to the scissor rect it is
        // assigned to.
        self.click_me_btn.layout(start_pos, cx.res);

        self.icon_btn.layout(
            Point::new(
                self.click_me_btn.max_x() + style.element_padding,
                start_pos.y,
            ),
            cx.res,
        );

        self.label.layout(
            Point::new(self.icon_btn.max_x() + style.element_padding, start_pos.y),
            cx.res,
        );

        self.icon.layout(
            Point::new(self.label.max_x() + style.element_padding, start_pos.y),
            cx.res,
        );

        self.icon_label.layout(
            Point::new(self.icon.max_x() + style.element_padding, start_pos.y),
            cx.res,
        );

        let mut toggle_btn_rect = Rect::new(
            Point::new(0.0, self.click_me_btn.max_y() + style.element_padding),
            self.toggle_btn.desired_size(cx.res),
        );

        self.switch.layout_aligned(
            Point::new(start_pos.x, toggle_btn_rect.center().y),
            Align2::CENTER_LEFT,
            cx.res,
        );

        toggle_btn_rect.origin.x = self.switch.max_x() + style.element_padding;
        self.toggle_btn.set_rect(toggle_btn_rect);

        self.icon_toggle_btn.layout(
            Point::new(
                toggle_btn_rect.max_x() + style.element_padding,
                self.toggle_btn.min_y(),
            ),
            cx.res,
        );

        self.icon_label_toggle_btn.layout(
            Point::new(
                self.icon_toggle_btn.max_x() + style.element_padding,
                self.toggle_btn.min_y(),
            ),
            cx.res,
        );

        self.separator_1.set_rect(Rect::new(
            Point::new(start_pos.x, toggle_btn_rect.max_y() + style.element_padding),
            Size::new(
                content_rect.width() - style.content_padding - style.content_padding,
                style.separator_width,
            ),
        ));

        self.drop_down_menu_btn.set_rect(Rect::new(
            Point::new(
                start_pos.x,
                self.separator_1.max_y() + style.element_padding,
            ),
            Size::new(
                style.drop_down_btn_width,
                self.drop_down_menu_btn.desired_size(cx.res).height,
            ),
        ));

        self.radio_group.layout(
            Point::new(
                start_pos.x,
                self.drop_down_menu_btn.max_y() + style.element_padding,
            ),
            style.radio_group_row_padding,
            style.radio_group_column_padding,
            None,
            Default::default(),
            cx.res,
        );

        self.separator_2.set_rect(Rect::new(
            Point::new(
                start_pos.x,
                self.radio_group.bounds().max_y() + style.element_padding,
            ),
            Size::new(
                content_rect.width() - style.content_padding - style.content_padding,
                style.separator_width,
            ),
        ));

        self.text_input.set_rect(Rect::new(
            Point::new(
                start_pos.x,
                self.separator_2.max_y() + style.element_padding,
            ),
            style.text_input_size,
        ));

        self.search_text_input.set_rect(Rect::new(
            Point::new(start_pos.x, self.text_input.max_y() + style.element_padding),
            style.text_input_size,
        ));

        self.scroll_area.set_content_size(Size::new(
            self.icon_label.max_x() + style.content_padding,
            self.search_text_input.max_y() + style.content_padding,
        ));
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        // Destructuring helps to make sure you didn't miss any elements.
        let Self {
            label,
            icon,
            icon_label,
            click_me_btn,
            icon_btn,
            switch,
            toggle_btn,
            icon_toggle_btn,
            icon_label_toggle_btn,
            radio_group,
            drop_down_menu_btn,
            drop_down_menu,
            text_input,
            text_input_menu,
            search_text_input,
            right_click_area,
            right_click_menu,
            scroll_area,
            separator_1,
            separator_2,
            active_text_input_menu: _,
        } = self;

        label.set_hidden(hidden);
        icon.set_hidden(hidden);
        icon_label.set_hidden(hidden);
        click_me_btn.set_hidden(hidden);
        switch.set_hidden(hidden);
        toggle_btn.set_hidden(hidden);
        icon_toggle_btn.set_hidden(hidden);
        icon_btn.set_hidden(hidden);
        icon_label_toggle_btn.set_hidden(hidden);
        radio_group.set_hidden(hidden);
        drop_down_menu_btn.set_hidden(hidden);
        drop_down_menu.set_hidden(hidden);
        text_input.set_hidden(hidden);
        text_input_menu.set_hidden(hidden);
        search_text_input.set_hidden(hidden);
        right_click_area.set_hidden(hidden);
        right_click_menu.set_hidden(hidden);
        scroll_area.set_hidden(hidden);
        separator_1.set_hidden(hidden);
        separator_2.set_hidden(hidden);
    }
}
