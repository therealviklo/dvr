use crate::Dvr;

#[cfg(target_arch = "wasm32")]
mod wasm_state;
#[cfg(target_arch = "wasm32")]
pub use wasm_state::*;

pub trait State {
	fn logic(&mut self) -> Result<LogicStatus, String>;
	fn draw(&self, dvr: &Dvr) -> Result<(), String>;
}

pub enum LogicStatus {
	Continue,
	NewState(Box<dyn State>),
	NewStateWithClosure(Box<dyn FnOnce(Box<dyn State>) -> Box<dyn State>>),
	Stop,
}

/// Returns a LogicStatus::NewStateWithClosure with the provided function
pub fn nswc<T: FnOnce(Box<dyn State>) -> Box<dyn State> + 'static>(f: T) -> LogicStatus {
	LogicStatus::NewStateWithClosure(Box::new(f))
}