use std::{cell::RefCell, collections::HashMap, future::{poll_fn, Future}, rc::Rc, task::{Poll, Waker}};
use wasm_bindgen::prelude::*;
use web_sys::{Event, HtmlCanvasElement, HtmlImageElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture, WebGlUniformLocation};
use crate::wasm_utils::log_errors;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

pub struct Dvr {
	ctx: WebGl2RenderingContext,
	program: WebGlProgram,
	vertex_position: i32,
	texture_coord: i32,
	position_matrix_location: WebGlUniformLocation,
	texture_offset_location: WebGlUniformLocation,
	texture_size_location: WebGlUniformLocation,
	sampler_location: WebGlUniformLocation,
	position_buffer: WebGlBuffer,
	texture_buffer: WebGlBuffer,
	// resize_event_closure: Closure<dyn FnMut(Event)>,
}

impl Dvr {
    pub fn new(ctx: WebGl2RenderingContext) -> Result<Dvr, String> {
		let vs_source =
		r##"
		attribute vec4 aVertexPosition;
		attribute vec2 aTextureCoord;

		uniform mat4 uPositionMatrix;
		uniform vec2 uTextureOffset;
		uniform vec2 uTextureSize;

		varying highp vec2 vTextureCoord;
		
		void main() {
			gl_Position = uPositionMatrix * aVertexPosition;
			vTextureCoord = vec2(uTextureSize.x * aTextureCoord.x, uTextureSize.y * aTextureCoord.y) + uTextureOffset;
		}
		"##;
		let fs_source =
		r##"
		varying highp vec2 vTextureCoord;

		uniform sampler2D uSampler;

		void main() {
			gl_FragColor = texture2D(uSampler, vTextureCoord);
		}
		"##;

		let program = Self::create_shader_program(&ctx, vs_source, fs_source)?;

		let vertex_position = ctx.get_attrib_location(&program, "aVertexPosition");
		let texture_coord = ctx.get_attrib_location(&program, "aTextureCoord");

		let position_matrix_location =
			ctx.get_uniform_location(&program, "uPositionMatrix")
			.ok_or("Unable to get position matrix location")?;
		let texture_offset_location =
			ctx.get_uniform_location(&program, "uTextureOffset")
			.ok_or("Unable to get texture offset location")?;
		let texture_size_location =
			ctx.get_uniform_location(&program, "uTextureSize")
			.ok_or("Unable to get texture size location")?;
		let sampler_location =
			ctx.get_uniform_location(&program, "uSampler")
			.ok_or("Unable to get sampler location")?;

		let position_buffer = Self::create_position_buffer(&ctx)?;
		let texture_buffer = Self::create_texture_buffer(&ctx)?;

		Self::resize_canvas_if_needed(&ctx)?;
		// let window = web_sys::window()
		// 	.ok_or("Unable to get window")?;
		// let resize_event_closure;
		// {
		// 	let ctx = ctx.clone();
		// 	resize_event_closure = add_event_listener(
		// 		&window,
		// 		"resize",
		// 		log_errors_arg(move |_: Event| -> Result<(), JsValue> {
		// 			// Self::resize_canvas_if_needed(&ctx)?;
		// 			Ok(())
		// 		}
		// 	)).ok().ok_or("Unable to add resize event listener")?;
		// }

		ctx.pixel_storei(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 1);
		ctx.enable(WebGl2RenderingContext::BLEND);
		ctx.blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA);

		Ok(Dvr {
			ctx,
			program,
			vertex_position,
			texture_coord,
			position_matrix_location,
			texture_offset_location,
			texture_size_location,
			sampler_location,
			position_buffer,
			texture_buffer,
			// resize_event_closure
		})
	}

	pub fn get_screen_size(&self) -> (i32, i32) {
		(self.ctx.drawing_buffer_width(), self.ctx.drawing_buffer_height())
	}

	pub fn get_screen_width(&self) -> i32 {
		self.ctx.drawing_buffer_width()
	}

	pub fn get_screen_height(&self) -> i32 {
		self.ctx.drawing_buffer_height()
	}

	pub fn native_mouse_x_to_dvr(&self, x: i32) -> f32 {
		x as f32 - self.get_screen_width() as f32 * 0.5
	}

	pub fn native_mouse_y_to_dvr(&self, y: i32) -> f32 {
		self.get_screen_height() as f32 * 0.5 - y as f32
	}

	// TODO: snyggare lösning?
	pub fn native_mouse_coords_to_dvr(&self, (x, y): (i32, i32)) -> (f32, f32) {
		(self.native_mouse_x_to_dvr(x), self.native_mouse_y_to_dvr(y))
	}

	pub fn start_draw(&self) -> Result<(), String> {
		Self::resize_canvas_if_needed(&self.ctx)?;
		Ok(())
	}

	pub fn end_draw(&self) -> Result<(), String> {
		Ok(())
	}

	pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
		self.ctx.clear_color(r, g, b, a);
		self.ctx.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
	}

	pub fn draw(&self, texture: &Texture, x: f32, y: f32, size: Option<(f32, f32)>, tex_pos_size: Option<((f32, f32), (f32, f32))>, angle: f32) -> Result<(), String> {
		let screen_width = self.get_screen_width();
		let screen_height = self.get_screen_height();

		self.ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.position_buffer));
		self.ctx.vertex_attrib_pointer_with_i32(
			self.vertex_position as u32,
			2,
			WebGl2RenderingContext::FLOAT,
			false,
			0,
			0
		);
		self.ctx.enable_vertex_attrib_array(self.vertex_position as u32);

		self.set_texture_attribute();

		self.ctx.use_program(Some(&self.program));

		let ws = 1.0 / screen_width as f32;
		let hs = 1.0 / screen_height as f32;
		let (w, h) = match size {
			Some((w, h)) => (w, h),
			None => (1.0, 1.0)
		};
		let mtx: [f32; 4 * 4] = [
			ws * w * angle.cos(), ws * w * -angle.sin(), 0.0, ws * x * 2.0,
			hs * h * angle.sin(), hs * h *  angle.cos(), 0.0, hs * y * 2.0,
			                 0.0,                   0.0, 1.0,          0.0,
			                 0.0,                   0.0, 0.0,          1.0,
		];
		self.ctx.uniform_matrix4fv_with_f32_array(
			Some(&self.position_matrix_location),
			true,
			&mtx
		);

		let (pos, size) = match tex_pos_size {
			Some(((x, y), (w, h))) => {
				let (tw, th) = texture.get_size();
				([x / tw as f32, 1.0 - (h + y) / th as f32], [w / tw as f32, h / th as f32])
			},
			None => ([0.0, 0.0], [1.0, 1.0]),
		};
		self.ctx.uniform2fv_with_f32_array(
			Some(&self.texture_offset_location),
			&pos
		);
		self.ctx.uniform2fv_with_f32_array(
			Some(&self.texture_size_location),
			&size
		);

		self.ctx.active_texture(WebGl2RenderingContext::TEXTURE0);
		self.ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture.texture));
		self.ctx.uniform1i(Some(&self.sampler_location), 0);

		self.ctx.draw_arrays(WebGl2RenderingContext::TRIANGLE_STRIP, 0, 4);

		Ok(())
	}

	pub fn load_texture(&self, url: &str) -> Result<impl Future<Output = Result<Texture, String>>, String> {
		enum TextureLoadStatus {
			Loading,
			Loaded,
			Error,
		}

		let texture = self.ctx.create_texture()
			.ok_or("Unable to create texture")?;
		let status: Rc<RefCell<TextureLoadStatus>> = Rc::new(RefCell::new(TextureLoadStatus::Loading));
		let waker: Rc<RefCell<Option<Waker>>> = Rc::new(RefCell::new(None));

		self.ctx.bind_texture(
			WebGl2RenderingContext::TEXTURE_2D,
			Some(&texture)
		);

		self.ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
			WebGl2RenderingContext::TEXTURE_2D,
			0,
			WebGl2RenderingContext::RGBA as i32,
			1,
			1,
			0,
			WebGl2RenderingContext::RGBA,
			WebGl2RenderingContext::UNSIGNED_BYTE,
			&[255, 255, 255, 255],
			0
		).ok().ok_or("Failed to create default texture")?;

		let load_error_closures: Rc<RefCell<Option<(Closure<dyn FnMut()>, Closure<dyn FnMut(Event)>)>>>
			= Rc::new(RefCell::new(None));
		let image = HtmlImageElement::new()
			.ok().ok_or("Failed to create image element")?;
		let load_closure;
		{
			let ctx = self.ctx.clone();
			let texture = texture.clone();
			let img = image.clone();
			let load_error_closures = load_error_closures.clone();
			let status = status.clone();
			let waker = waker.clone();
			load_closure = Closure::<dyn FnMut()>::new(log_errors(move || -> Result<(), JsValue> {
				*status.borrow_mut() = TextureLoadStatus::Error;

				ctx.bind_texture(
					WebGl2RenderingContext::TEXTURE_2D,
					Some(&texture)
				);
				ctx.tex_image_2d_with_u32_and_u32_and_html_image_element(
					WebGl2RenderingContext::TEXTURE_2D,
					0,
					WebGl2RenderingContext::RGBA as i32,
					WebGl2RenderingContext::RGBA,
					WebGl2RenderingContext::UNSIGNED_BYTE,
					&img
				)?;

				let is_power_of_2 = |x: u32| (x & (x - 1)) == 0;
				if is_power_of_2(img.width()) && is_power_of_2(img.height()) {
					ctx.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
				} else {
					ctx.tex_parameteri(
						WebGl2RenderingContext::TEXTURE_2D,
						WebGl2RenderingContext::TEXTURE_WRAP_S,
						WebGl2RenderingContext::CLAMP_TO_EDGE as i32
					);
					ctx.tex_parameteri(
						WebGl2RenderingContext::TEXTURE_2D,
						WebGl2RenderingContext::TEXTURE_WRAP_T,
						WebGl2RenderingContext::CLAMP_TO_EDGE as i32
					);
					ctx.tex_parameteri(
						WebGl2RenderingContext::TEXTURE_2D,
						WebGl2RenderingContext::TEXTURE_MIN_FILTER,
						WebGl2RenderingContext::LINEAR as i32
					);
				}

				*status.borrow_mut() = TextureLoadStatus::Loaded;
				if let Some(waker) = waker.borrow_mut().take() {
					waker.wake();
				}
				// TODO: else panic?
				// Ta ut closurarna så att den droppas när funktionen tar slut
				let _ = (*load_error_closures.borrow_mut()).take();

				Ok(())
			}));
			image.set_onload(Some(load_closure.as_ref().unchecked_ref()));
		}
		let error_closure;
		{
			let load_error_closures = load_error_closures.clone();
			let status = status.clone();
			let waker = waker.clone();
			error_closure = Closure::<dyn FnMut(_)>::new(move |_: Event|{
				web_sys::console::error_1(&JsValue::from_str("Unable to load texture"));
				*status.borrow_mut() = TextureLoadStatus::Error;
				if let Some(waker) = waker.borrow_mut().take() {
					waker.wake();
				}
				// TODO: else panic?
				// Ta ut closurarna så att den droppas när funktionen tar slut
				let _ = (*load_error_closures.borrow_mut()).take();
			});
			image.set_onerror(Some(error_closure.as_ref().unchecked_ref()));
		}
		*load_error_closures.borrow_mut() = Some((load_closure, error_closure));
		image.set_src(url);

		Ok(poll_fn(move |cx| -> Poll<Result<Texture, String>> {
			match *status.borrow_mut() {
				TextureLoadStatus::Loading => {
					if waker.borrow().is_none() {
						// TODO: clone_from?
						*waker.borrow_mut() = Some(cx.waker().clone());
					}
					Poll::Pending
				},
				TextureLoadStatus::Loaded => Poll::Ready(Ok(Texture {
					texture: texture.clone(), // Kan inte movea av någon anledning
					size: (image.width(), image.height()),
				})),
				// TODO: bättre felmeddelanden?
				TextureLoadStatus::Error => Poll::Ready(Err("Error when loading texture".to_string())),
			}
		}))
	}

	fn resize_canvas_if_needed(ctx: &WebGl2RenderingContext) -> Result<(), String> {
		let canvas = Self::get_canvas(&ctx)?;
		let cw = canvas.client_width();
		let ch = canvas.client_height();
		let w = canvas.width();
		let h = canvas.height();
		if w != cw as u32 || h != ch as u32 {
			canvas.set_width(cw as u32);
			canvas.set_height(ch as u32);
			ctx.viewport(0, 0, cw, ch);
		}
		Ok(())
	}

	// TODO: bör den här finnas?
	pub fn resize(&self) -> Result<(), String> {
		Self::resize_canvas_if_needed(&self.ctx)
	}

	fn create_shader_program(ctx: &WebGl2RenderingContext, vs_source: &str, fs_source: &str) -> Result<WebGlProgram, String> {
		let vs_shader = Self::load_shader(&ctx, WebGl2RenderingContext::VERTEX_SHADER, vs_source)?;
		let fs_shader = Self::load_shader(&ctx, WebGl2RenderingContext::FRAGMENT_SHADER, fs_source)?;

		let shader_program = ctx.create_program()
			.ok_or("Unable to create shader program")?;
		ctx.attach_shader(&shader_program, &vs_shader);
		ctx.attach_shader(&shader_program, &fs_shader);
		ctx.link_program(&shader_program);

		let link_status = ctx.get_program_parameter(&shader_program, WebGl2RenderingContext::LINK_STATUS);
		if !link_status.as_bool().ok_or("Link status is not boolean")? {
			return Err(ctx.get_program_info_log(&shader_program)
				.filter(|s| s != "")
				.unwrap_or_else(|| String::from("Unable to link shader program")));
		}

		Ok(shader_program)
	}

	fn load_shader(ctx: &WebGl2RenderingContext, kind: u32, source: &str) -> Result<WebGlShader, String> {
		let shader = ctx.create_shader(kind)
			.ok_or("Unable to create shader")?;
		ctx.shader_source(&shader, source);
		ctx.compile_shader(&shader);
		let compile_status = ctx.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS);
		if !compile_status.as_bool().ok_or("Compile status is not boolean")? {
			let err_text = ctx.get_shader_info_log(&shader)
				.filter(|s| s != "")
				.unwrap_or_else(|| String::from("Unable to load shader"));
			ctx.delete_shader(Some(&shader));
			return Err(err_text);
		}
		Ok(shader)
	}

	fn create_position_buffer(ctx: &WebGl2RenderingContext) -> Result<WebGlBuffer, String> {
		let buffer = ctx.create_buffer().ok_or("Unable to create buffer")?;
		ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
		let vertices: [f32; 8] = [1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, -1.0];
		unsafe {
			let verices_view = js_sys::Float32Array::view(&vertices);
			ctx.buffer_data_with_array_buffer_view(
				WebGl2RenderingContext::ARRAY_BUFFER, 
				&verices_view,
				WebGl2RenderingContext::STATIC_DRAW
			);
		}
		Ok(buffer)
	}

	fn create_texture_buffer(ctx: &WebGl2RenderingContext) -> Result<WebGlBuffer, String> {
		let buffer = ctx.create_buffer().ok_or("Unable to create buffer")?;
		ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
		let tex_coords: [f32; 8] = [1.0, 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0];
		unsafe {
			let tex_coords_view = js_sys::Float32Array::view(&tex_coords);
			ctx.buffer_data_with_array_buffer_view(
				WebGl2RenderingContext::ARRAY_BUFFER,
				&tex_coords_view,
				WebGl2RenderingContext::STATIC_DRAW
			);
		}
		Ok(buffer)
	}

	fn set_texture_attribute(&self) {
		self.ctx.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.texture_buffer));
		self.ctx.vertex_attrib_pointer_with_i32(
			self.texture_coord as u32,
			2,
			WebGl2RenderingContext::FLOAT,
			false,
			0,
			0
		);
		self.ctx.enable_vertex_attrib_array(self.texture_coord as u32);
	}

	fn get_canvas(ctx: &WebGl2RenderingContext) -> Result<HtmlCanvasElement, String> {
		Ok(ctx
			.canvas()
			.ok_or("Unable to get canvas")?
			.dyn_into::<web_sys::HtmlCanvasElement>()
			.ok().ok_or("Canvas is not an HtmlCanvasElement")?)
	}

	pub fn canvas(&self) -> Result<HtmlCanvasElement, String> {
		Self::get_canvas(&self.ctx)
	}

	// fn get_error(&self) -> Result<(), u32> {
	// 	match self.ctx.get_error() {
	// 		0 => Ok(()),
	// 		x => Err(x),
	// 	}
	// }
}

// impl Drop for Dvr {
// 	fn drop(&mut self) {
// 		if let Some(window) = web_sys::window() {
// 			let _ = window.remove_event_listener_with_callback(
// 				"resize",
// 				self.resize_event_closure.as_ref().unchecked_ref()
// 			);
// 		}
// 	}
// }

pub struct Texture {
	texture: WebGlTexture,
	size: (u32, u32),
}

impl Texture {
	pub fn get_size(&self) -> (u32, u32) {
		self.size
	}

	pub fn get_width(&self) -> u32 {
		self.size.0
	}

	pub fn get_height(&self) -> u32 {
		self.size.1
	}
}

pub struct TextureHandler {
	textures: HashMap<String, Texture>,
}

impl TextureHandler {
	pub async fn new(dvr: &Dvr, names: &[String]) -> Result<TextureHandler, String> {
		let mut texture_futures: HashMap<String, Box<dyn Future<Output = Result<Texture, String>> + Unpin>> = HashMap::new();
		for name in names {
			texture_futures.insert(name.to_string(), Box::new(dvr.load_texture(&name)?));
		}
		let mut textures: HashMap<String, Texture> = HashMap::new();
		for (name, future) in texture_futures {
			textures.insert(name, future.await?);
		}
		Ok(TextureHandler { textures })
		// let completed = Rc::new(RefCell::new(0));
		// let name_count = names.len();
		// for name in names {
		// 	let completed_load = completed.clone();
		// 	let completed_error = completed_load.clone();
		// 	let waker_load = waker.clone();
		// 	let waker_error = waker.clone();
		// 	textures.insert(name.to_string(), dvr.load_texture(&name,
		// 		Some(Box::new(move || {
		// 			*completed_load.borrow_mut() += 1;
		// 			if *completed_load.borrow() == name_count {
		// 				if let Some(waker) = waker_load.borrow_mut().take() {
		// 					waker.wake();
		// 				}
		// 				// TODO: else panic?
		// 			}
		// 		})),
		// 		Some(Box::new(move || {
		// 			*completed_error.borrow_mut() += 1;
		// 			if *completed_error.borrow() == name_count {
		// 				if let Some(waker) = waker_error.borrow_mut().take() {
		// 					waker.wake();
		// 				}
		// 				// TODO: else panic?
		// 			}
		// 		})))?
		// 	);
		// }
		// let texture_handler = Rc::new(TextureHandler {
		// 	textures,
		// 	completed,
		// });
		// Ok(poll_fn(move |cx| -> Poll<Rc<TextureHandler>> {
		// 	// TODO: panic om borrow_mut är aktiv
		// 	if *texture_handler.completed.borrow() == name_count {
		// 		Poll::Ready(texture_handler.clone())
		// 	} else {
		// 		if waker.borrow().is_none() {
		// 			// TODO: clone_from?
		// 			*waker.borrow_mut() = Some(cx.waker().clone());
		// 		}
		// 		Poll::Pending
		// 	}
		// }))
	}

	pub fn get(&self, name: String) -> Option<&Texture> {
		self.textures.get(&name)
	}
}
