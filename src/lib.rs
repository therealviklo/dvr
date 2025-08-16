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

// pub mod state;
// pub mod font;
// pub mod input;

// Test
#[cfg(target_arch = "wasm32")]
mod test_wasm;
#[cfg(target_arch = "wasm32")]
pub use crate::test_wasm::start;

#[cfg(target_os = "windows")]
mod test_win;
