use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use crate::{state::{LogicStatus, State}, wasm_utils::log_errors, Dvr};

pub struct StateHandler<Glob> {
	dvr: Dvr,
	state: Option<Box<dyn State<Glob>>>,
	interval_handle: Rc<RefCell<Option<i32>>>,
	interval_closure: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
	glob: Glob,
}

impl<Glob: 'static> StateHandler<Glob> {
	fn new(dvr: Dvr, initial_state: Box<dyn State<Glob>>, glob: Glob) -> StateHandler<Glob> {
		StateHandler {
			dvr,
			state: Some(initial_state),
			interval_handle: Rc::new(RefCell::new(None)),
			interval_closure: Rc::new(RefCell::new(None)),
			glob
		}
	}

	pub fn run(dvr: Dvr, initial_state: Box<dyn State<Glob>>, glob: Glob) -> Result<(), String> {
		let state_handler = Self::new(dvr, initial_state, glob);
		
		let window = web_sys::window().ok_or("Unable to get window")?;
		let interval_handle = state_handler.interval_handle.clone();
		let interval_closure = state_handler.interval_closure.clone();
		let mut state_handler_option = Some(state_handler);
		let logic = move || -> Result<(), JsValue> {
			let res =  || -> Result<(), JsValue> {
				let state_handler = state_handler_option.as_mut()
					.expect("State handler callback called without state handler existing");
				let mut state;
				loop {
					state = match state_handler.state.as_mut() {
						Some(x) => x,
						None => {
							return Err(From::from("State handler has no state to call"))
						},
					};
					match state.logic(&mut state_handler.glob) {
						Ok(LogicStatus::Continue) => {
							break
						},
						Ok(LogicStatus::NewState(new_state)) => {
							state_handler.state = Some(new_state);
						},
						Ok(LogicStatus::NewStateWithClosure(f)) => {
							let old_state = state_handler.state
								.take()
								.expect("State somehow disappeared since last check");
							state_handler.state = Some(f(old_state));
						},
						Ok(LogicStatus::Stop) => {
							state_handler_option = None;
							return Ok(());
						},
						Err(e) => {
							return Err(From::from(e));
						},
					}
				}
				state_handler.dvr.start_draw()?;
				state.draw(&state_handler.dvr, &state_handler.glob)?;
				state_handler.dvr.end_draw()?;
				Ok(())
			}();
			if res.is_err() {
				state_handler_option = None;
			}
			res
		};
		let closure = Closure::<dyn FnMut()>::new(log_errors(logic));
		*interval_handle.borrow_mut() = Some(window.set_interval_with_callback_and_timeout_and_arguments_0(
			closure.as_ref().unchecked_ref(),
			1000 / 60
		).map_err(|e| e.as_string().unwrap_or_else(|| "Unknown error".to_string()))?);
		*interval_closure.borrow_mut() = Some(closure);
		Ok(())
	}
}

impl<Glob> Drop for StateHandler<Glob> {
	fn drop(&mut self) {
		if let Some(window) = web_sys::window() {
			if let Some(handle) = *self.interval_handle.borrow() {
				let _ = window.clear_interval_with_handle(handle);
			}
		}
	}
}