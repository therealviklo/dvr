use dvr::{interface::*, state::{LogicStatus, State, StateHandler}, *};

struct TestState {
	angle: f32,
	tex: Texture,
}

impl TestState {
	fn new(dvr: &Dvr) -> Result<TestState, String> {
		let tex = dvr.load_texture("pluto.png")?;
		Ok(TestState {
			angle: 0.0,
			tex,
		})
	}
}

impl State for TestState {
	fn logic(&mut self) -> Result<LogicStatus, String> {
		self.angle += 0.01;
		Ok(LogicStatus::Continue)
	}

	fn draw(&self, dvr: &Dvr) -> Result<(), String> {
		dvr.draw(&self.tex, 0.0, 0.0, None, None, self.angle)?;
		Ok(())
	}
}

fn main() -> Result<(), String> {
	let interface = Interface::new("testy", 500, 250, true)?;
	let dvr = Dvr::new(interface.get_ctx())?;
	let test_state = TestState::new(&dvr)?;
	StateHandler::run(dvr, Box::new(test_state), interface.get_ctx())?;
	Ok(())
}