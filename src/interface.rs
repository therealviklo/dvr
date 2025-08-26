#[cfg(target_os = "windows")]
mod win_interface;
#[cfg(target_os = "windows")]
pub use win_interface::*;

#[cfg(target_arch = "wasm32")]
mod wasm_interface;
#[cfg(target_arch = "wasm32")]
pub use wasm_interface::*;