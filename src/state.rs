use crate::Dvr;

#[cfg(target_arch = "wasm32")]
mod wasm_state;
#[cfg(target_arch = "wasm32")]
pub use wasm_state::*;

#[cfg(target_os = "windows")]
mod win_state;
#[cfg(target_os = "windows")]
pub use win_state::*;

pub trait State<Glob> {
	fn logic(&mut self, glob: &mut Glob) -> Result<LogicStatus<Glob>, String>;
	fn draw(&self, dvr: &Dvr, glob: &Glob) -> Result<(), String>;
}

pub enum LogicStatus<Glob> {
	Continue,
	NewState(Box<dyn State<Glob>>),
	NewStateWithClosure(Box<dyn FnOnce(Box<dyn State<Glob>>) -> Box<dyn State<Glob>>>),
	Stop,
}

impl<Glob> LogicStatus<Glob> {
	/// Returns a LogicStatus::NewStateWithClosure with the provided function
	pub fn nswc<T: FnOnce(Box<dyn State<Glob>>) -> Box<dyn State<Glob>> + 'static>(f: T) -> LogicStatus<Glob> {
		LogicStatus::NewStateWithClosure(Box::new(f))
	}
}