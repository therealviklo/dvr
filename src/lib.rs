#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use crate::wasm::*;
#[cfg(target_arch = "wasm32")]
mod wasm_utils;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use crate::win::*;
#[cfg(target_os = "windows")]
mod win_utils;

pub mod state;
// pub mod font;
// pub mod input;
pub mod interface;

#[cfg(target_arch = "wasm32")]
type DvrCtx = web_sys::WebGl2RenderingContext;
#[cfg(target_os = "windows")]
type DvrCtx = windows::Win32::Foundation::HWND;

// Test
#[cfg(target_arch = "wasm32")]
mod test_wasm;
#[cfg(target_arch = "wasm32")]
pub use crate::test_wasm::start;
