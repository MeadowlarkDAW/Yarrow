// ---------------------------------------------------------------------------------
//
//    '%%' '%% '%%'
//    %'%\% | %/%'%     Yarrow GUI Library
//        \ | /
//         \|/          https://github.com/MeadowlarkDAW/Yarrow
//          |
//
//
// MIT License Copyright (c) 2024 Billy Messenger
// https://github.com/MeadowlarkDAW/Yarrow/blob/main/LICENSE
//
// ---------------------------------------------------------------------------------

pub mod action_queue;
mod application;
pub mod clipboard;
pub(crate) mod cursor_icon;
pub mod elements;
pub mod event;
pub mod layout;
pub mod prelude;
pub(crate) mod stmpsc_queue;
pub mod style;
pub(crate) mod view;
pub mod window;

pub use application::{AppConfig, AppContext, Application};
pub use cursor_icon::CursorIcon;
pub use event::AppWindowEvent;
pub use view::{ScissorRectID, TooltipInfo, View, MAIN_SCISSOR_RECT};
pub use window::{WindowContext, WindowID, MAIN_WINDOW};

pub use rootvg as vg;
pub use rootvg::math;

#[cfg(feature = "winit")]
pub use window::run_blocking;
