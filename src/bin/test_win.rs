use dvr::{*, interface::*};

fn main() -> Result<(), String> {
	let interface = Interface::new("testy", 500, 250, true)?;
	let dvr = Dvr::new(interface.get_ctx())?;
	let tex = dvr.load_texture("pluto.png")?;
	let mut angle = 0.0;
	loop {
		interface.update();
		dvr.start_draw()?;
		dvr.clear(0.5, 0.5, 0.5, 1.0)?;
		angle += 0.01;
		dvr.draw(&tex, 0.0, 0.0, None, None, angle)?;
		dvr.end_draw_sync(1)?;
	}
	Ok(())
}