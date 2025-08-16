#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use crate::wasm::*;
#[cfg(target_arch = "wasm32")]
mod wasm_utils;

pub mod state;
pub mod font;
pub mod input;

// Test
#[cfg(target_arch = "wasm32")]
mod test_wasm;
#[cfg(target_arch = "wasm32")]
pub use crate::test_wasm::start;
