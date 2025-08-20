use dvr::{*, window::*};

fn main() -> Result<(), String> {
	let win = Window::new("testy", 500, 250, true)?;
	let dvr = Dvr::new(win.get_ctx())?;
	let tex = dvr.load_texture("pluto.png")?;
	loop {
		win.update();
		dvr.start_draw()?;
		dvr.clear(0.5, 0.5, 0.5, 1.0)?;
		dvr.draw(&tex, 0.0, 0.0, None, None, 0.0)?;
		dvr.end_draw()?;
	}
	Ok(())
}