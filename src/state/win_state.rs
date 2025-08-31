use crate::{interface::Interface, state::State, Dvr};
use super::LogicStatus::{Continue, NewState, NewStateWithClosure, Stop};

pub struct StateHandler<Glob> {
	dvr: Dvr,
	state: Option<Box<dyn State<Glob>>>,
	glob: Glob,
}

impl<Glob> StateHandler<Glob> {
	fn new(dvr: Dvr, initial_state: Box<dyn State<Glob>>, glob: Glob) -> StateHandler<Glob> {
		StateHandler {
			dvr,
			state: Some(initial_state),
			glob: glob,
		}
	}

	pub fn run(dvr: Dvr, initial_state: Box<dyn State<Glob>>, glob: Glob, interface: &Interface) -> Result<(), String> {
		Self::run_with_closures(
			dvr,
			initial_state,
			glob,
			|| { interface.update(); },
			|| interface.exists()
		)
	}

	// TODO: update_window return
	pub fn run_with_closures(dvr: Dvr, initial_state: Box<dyn State<Glob>>, glob: Glob, mut update_window: impl FnMut(), mut window_exists: impl FnMut() -> bool) -> Result<(), String> {
		let mut state_handler = Self::new(dvr, initial_state, glob);
		while window_exists() {
			update_window();
			let mut state;
			loop {
				state = match state_handler.state.as_mut() {
					Some(x) => x,
					None => {
						return Err(From::from("State handler has no state to call"))
					},
				};
				match state.logic(&mut state_handler.glob)? {
					Continue => break,
					NewState(new_state) => {
						state_handler.state = Some(new_state)
					},
					NewStateWithClosure(f) => {
						let old_state = state_handler.state
							.take()
							.expect("State somehow disappeared since last check");
						state_handler.state = Some(f(old_state));
					},
					Stop => return Ok(()),
				}
			}
			state_handler.dvr.start_draw()?;
			state.draw(&state_handler.dvr, &state_handler.glob)?;
			state_handler.dvr.end_draw_sync(1)?;
		}
		Ok(())
	}
}