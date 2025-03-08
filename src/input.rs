#[cfg(target_arch = "wasm32")]
mod wasm_input;
#[cfg(target_arch = "wasm32")]
pub use wasm_input::*;

#[derive(Clone)]
pub enum Event {
	KeyDown(KeyCodeEvent),
	KeyUp(KeyCodeEvent),
	Char(String),
	MouseDown(MouseEvent),
	MouseUp(MouseEvent),
}

#[derive(Clone)]
pub struct KeyCodeEvent {
	pub key_code: String,
	pub ctrl_down: bool,
	pub shift_down: bool,
	pub alt_down: bool,
}

#[derive(Clone)]
pub struct MouseEvent {
	pub x: i32,
	pub y: i32,
	pub button: MouseButton,
	pub ctrl_down: bool,
	pub shift_down: bool,
}

#[derive(Clone, Copy)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
	Back,
	Forward,
}