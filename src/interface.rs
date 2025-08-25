#[cfg(target_os = "windows")]
mod win_interface;
#[cfg(target_os = "windows")]
pub use win_interface::*;