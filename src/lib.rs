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

use std::error::Error;

pub use yarrow_core::*;

use yarrow_core::application::{AppConfig, Application};

#[cfg(feature = "skia")]
pub use yarrow_skia as skia;

#[cfg(feature = "skia")]
use yarrow_skia::SkiaRenderer as Renderer;

#[cfg(feature = "winit")]
pub fn run_blocking<A: Application>(config: AppConfig) -> Result<(), Box<dyn Error>>
where
    A::Action: Send,
{
    yarrow_winit::run_blocking::<A, Renderer>(config)
}
