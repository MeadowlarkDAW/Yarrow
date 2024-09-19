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

pub mod action;
pub mod clipboard;
pub mod color;
pub mod element;
pub mod event;
pub mod layout;
pub mod math;

mod cursor_icon;
mod resource;
mod scissor_rect;
mod style_system;
mod tooltip;
mod window;

pub use cursor_icon::CursorIcon;
pub use resource::ResourceContext;
pub use scissor_rect::ScissorRectID;
pub use style_system::StyleSystem;
pub use tooltip::TooltipData;
pub use window::WindowID;

pub type ZIndex = u16;
