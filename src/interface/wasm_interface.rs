use web_sys::WebGl2RenderingContext;
use crate::DvrCtx;

pub struct Interface {
	context: WebGl2RenderingContext
}

impl Interface {
	pub fn new() -> Result<Interface, String> {
		let window = web_sys::window().ok_or("Unable to get window")?;
		let document = window.document().ok_or("Unable to get document")?;

		let canvas: web_sys::HtmlCanvasElement = document
			.get_element_by_id("canvas")
			.unwrap()
			.dyn_into::<web_sys::HtmlCanvasElement>()?;

		let context = canvas
			.get_context("webgl2")?
			.unwrap()
			.dyn_into::<WebGl2RenderingContext>()?;

		Ok(Interface { context })
	}

	pub fn get_ctx(&self) -> DvrCtx {
		self.context
	}
}