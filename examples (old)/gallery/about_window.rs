use yarrow::prelude::*;

use crate::{style::MyStyle, MyAction};

pub const ABOUT_WINDOW_ID: WindowID = 1;

const ABOUT_TEXT: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed\
do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim\
veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo\
consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum\
dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, \
sunt in culpa qui officia deserunt mollit anim id est laborum. 🦀🚀🎛️";

pub fn window_config() -> WindowConfig {
    WindowConfig {
        title: "About".into(),
        size: Size::new(400.0, 400.0),
        resizable: false,
        ..Default::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    CloseAboutWindow,
}

pub struct Elements {
    _paragraph: Paragraph,
    _separator: Separator,
    _close_btn: Button,
}

impl Elements {
    pub fn new(style: &MyStyle, window_cx: &mut WindowContext<MyAction>) -> Self {
        window_cx.set_clear_color(style.clear_color);

        let window_size = window_cx.logical_size();

        let mut close_btn = Button::builder()
            .text("Close")
            .on_select(Action::CloseAboutWindow.into())
            .build(window_cx);

        close_btn.layout_aligned(
            point(
                window_size.width * 0.5,
                window_size.height - style.content_padding,
            ),
            Align2::BOTTOM_CENTER,
            window_cx.res,
        );

        let separator = Separator::builder()
            .rect(rect(
                style.content_padding,
                close_btn.min_y() - style.element_padding,
                window_size.width - style.content_padding - style.content_padding,
                style.separator_width,
            ))
            .build(window_cx);

        let paragraph = Paragraph::builder()
            .text(ABOUT_TEXT)
            .rect(rect(
                style.content_padding,
                style.content_padding,
                window_size.width - style.content_padding - style.content_padding,
                separator.min_y() - style.element_padding,
            ))
            .build(window_cx);

        Self {
            _paragraph: paragraph,
            _separator: separator,
            _close_btn: close_btn,
        }
    }

    /// Returns `true` if the about window should be closed.
    pub fn handle_action(
        &mut self,
        action: Action,
        _window_cx: &mut WindowContext<MyAction>,
    ) -> bool {
        let close_window = match action {
            Action::CloseAboutWindow => true,
        };

        close_window
    }
}
