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
    _close_btn: Button,
}

impl Elements {
    pub fn new(style: &MyStyle, cx: &mut WindowContext<'_, MyAction>) -> Self {
        cx.view.clear_color = style.clear_color.into();

        let window_size = cx.view.size();

        let mut close_btn = Button::builder(&style.button_style)
            .text("Close")
            .on_select(Action::CloseAboutWindow.into())
            .build(cx);
        close_btn.layout_aligned(
            Point::new(
                window_size.width * 0.5,
                window_size.height - style.content_padding,
            ),
            Align2::BOTTOM_CENTER,
        );

        let paragraph = Paragraph::builder(&style.paragraph_style)
            .text(ABOUT_TEXT)
            .bounding_rect(Rect::new(
                Point::new(style.content_padding, style.content_padding),
                Size::new(
                    window_size.width - style.content_padding - style.content_padding,
                    close_btn.el.rect().min_y() - style.content_padding,
                ),
            ))
            .build(cx);

        Self {
            _paragraph: paragraph,
            _close_btn: close_btn,
        }
    }

    /// Returns `true` if the about window should be closed.
    pub fn handle_action(&mut self, action: Action, _cx: &mut WindowContext<'_, MyAction>) -> bool {
        let close_window = match action {
            Action::CloseAboutWindow => true,
        };

        close_window
    }
}
