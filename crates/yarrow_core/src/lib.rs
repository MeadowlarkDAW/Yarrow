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
pub mod application;
pub mod clipboard;
pub mod color;
pub mod diff;
pub mod element;
pub mod event;
pub mod layout;
pub mod math;
pub mod primitive;
pub mod renderer;
pub mod window;

mod cursor_icon;
mod resource;
mod scissor_rect;
mod style_system;
mod tooltip;

pub use self::cursor_icon::CursorIcon;
pub use self::resource::ResourceContext;
pub use self::scissor_rect::ScissorRectID;
pub use self::style_system::StyleSystem;
pub use self::tooltip::TooltipData;

pub type ZIndex = u16;
