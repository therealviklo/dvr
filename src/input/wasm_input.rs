use std::{cell::RefCell, collections::HashMap, rc::Rc};
use crate::{wasm_utils::{add_event_listener, js_val_err_to_string}, Dvr};
use queues::{CircularBuffer, IsQueue, Queue};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{CompositionEvent, HtmlCanvasElement, KeyboardEvent, MouseEvent};
use super::{Event, KeyCodeEvent};

pub struct Input {
	key_states: Rc<RefCell<HashMap<String, bool>>>,
	mouse_pos: Rc<RefCell<Option<(i32, i32)>>>,
	events: Rc<RefCell<Box<dyn IsQueue<Event>>>>,
	max_events: Option<usize>,
	canvas: HtmlCanvasElement,
	keydown_closure: Closure<dyn FnMut(KeyboardEvent)>,
	keyup_closure: Closure<dyn FnMut(KeyboardEvent)>,
	compositionend_closure: Closure<dyn FnMut(CompositionEvent)>,
	mousedown_closure: Closure<dyn FnMut(MouseEvent)>,
	mouseup_closure: Closure<dyn FnMut(MouseEvent)>,
	mousemove_closure: Closure<dyn FnMut(MouseEvent)>,
	mouseleave_closure: Closure<dyn FnMut(MouseEvent)>,
}

impl Input {
	pub fn new(dvr: &Dvr, max_events: Option<usize>) -> Result<Input, String> {
		let window = web_sys::window().ok_or("Unable to get window")?;
		let canvas = dvr.canvas()?;
		let key_states = Rc::new(RefCell::new(HashMap::new()));
		let mouse_pos = Rc::new(RefCell::new(None));
		let events = Rc::new(RefCell::new(Self::new_key_events(max_events)));
		let keydown_closure;
		{
			let key_states = key_states.clone();
			let events = events.clone();
			keydown_closure = add_event_listener(
				window.as_ref(),
				"keydown",
				move |e: KeyboardEvent| {
					let key_code = e.code();
					key_states.borrow_mut().insert(key_code.clone(), true);
					let _ = events.borrow_mut().add(Event::KeyDown(js_key_event_to_dvr(&e)));
					if !e.is_composing() {
						let key = e.key();
						if key == "Enter" {
							let _ = events.borrow_mut().add(Event::Char("\n".to_string()));
						} else if key.chars().count() == 1 {
							let _ = events.borrow_mut().add(Event::Char(e.key()));
						}
					}
				}
			).map_err(js_val_err_to_string)?;
		}
		let keyup_closure;
		{
			let key_states = key_states.clone();
			let events = events.clone();
			keyup_closure = add_event_listener(
				window.as_ref(),
				"keyup",
				move |e: KeyboardEvent| {
					let key_code = e.code();
					key_states.borrow_mut().insert(key_code.clone(), false);
					let _ = events.borrow_mut().add(Event::KeyUp(js_key_event_to_dvr(&e)));
				}
			).map_err(js_val_err_to_string)?;
		}
		let compositionend_closure;
		{
			let events = events.clone();
			compositionend_closure = add_event_listener(
				window.as_ref(),
				"compositionend",
				move |e: CompositionEvent| {
					if let Some(chars) = e.data() {
						let _ = events.borrow_mut().add(Event::Char(chars));
					}
				}
			).map_err(js_val_err_to_string)?;
		}
		let mousedown_closure;
		{
			let events = events.clone();
			mousedown_closure = add_event_listener(
				canvas.as_ref(),
				"mousedown",
				move |e: MouseEvent| {
					let _ = events.borrow_mut().add(Event::MouseDown(js_mouse_event_to_dvr(&e)));
				}
			).map_err(js_val_err_to_string)?;
		}
		let mouseup_closure;
		{
			let events = events.clone();
			mouseup_closure = add_event_listener(
				canvas.as_ref(),
				"mouseup",
				move |e: MouseEvent| {
					let _ = events.borrow_mut().add(Event::MouseUp(js_mouse_event_to_dvr(&e)));
				}
			).map_err(js_val_err_to_string)?;
		}
		let mousemove_closure;
		{
			let mouse_pos = mouse_pos.clone();
			mousemove_closure = add_event_listener(
				canvas.as_ref(),
				"mousemove",
				move |e: MouseEvent| {
					*mouse_pos.borrow_mut() = Some((e.offset_x(), e.offset_y()));
				}
			).map_err(js_val_err_to_string)?;
		}
		let mouseleave_closure;
		{
			let mouse_pos = mouse_pos.clone();
			mouseleave_closure = add_event_listener(
				canvas.as_ref(),
				"mouseleave",
				move |_: MouseEvent| {
					*mouse_pos.borrow_mut() = None;
				}
			).map_err(js_val_err_to_string)?;
		}
		Ok(Input {
			key_states,
			mouse_pos,
			events,
			max_events,
			canvas,
			keydown_closure,
			keyup_closure,
			compositionend_closure,
			mousedown_closure,
			mouseup_closure,
			mousemove_closure,
			mouseleave_closure,
		})
	}

	pub fn key_down(&self, key: &str) -> bool {
		denormalise(key)
			.iter()
			.any(|x| self.key_down_raw(&x))
	}

	pub fn key_down_raw(&self, key: &str) -> bool {
		*self.key_states.borrow().get(key).unwrap_or(&false)
	}

	pub fn get_mouse_pos(&self) -> Option<(i32, i32)> {
		*self.mouse_pos.borrow()
	}

	pub fn get_mouse_x(&self) -> Option<i32> {
		self.get_mouse_pos().map(|(x, _y)| x)
	}

	pub fn get_mouse_y(&self) -> Option<i32> {
		self.get_mouse_pos().map(|(_x, y)| y)
	}

	pub fn clear_events(&mut self) {
		*self.events.borrow_mut() = Self::new_key_events(self.max_events);
	}

	fn new_key_events(max_events: Option<usize>) -> Box<dyn IsQueue<Event>> {
		match max_events {
			Some(max_events) => Box::new(CircularBuffer::new(max_events)),
			None => Box::new(Queue::new()),
		}
	}
}

impl Iterator for &Input {
	type Item = Event;

	fn next(&mut self) -> Option<Self::Item> {
		self.events.borrow_mut().remove().ok()
	}
}

impl Drop for Input {
	fn drop(&mut self) {
		let _ = self.canvas.remove_event_listener_with_callback(
			"mouseleave",
			self.mouseleave_closure.as_ref().unchecked_ref()
		);
		let _ = self.canvas.remove_event_listener_with_callback(
			"mousemove",
			self.mousemove_closure.as_ref().unchecked_ref()
		);
		let _ = self.canvas.remove_event_listener_with_callback(
			"mouseup",
			self.mouseup_closure.as_ref().unchecked_ref()
		);
		let _ = self.canvas.remove_event_listener_with_callback(
			"mousedown",
			self.mousedown_closure.as_ref().unchecked_ref()
		);
		if let Some(window) = web_sys::window() {
			let _ = window.remove_event_listener_with_callback(
				"compositionend",
				self.compositionend_closure.as_ref().unchecked_ref()
			);
			let _ = window.remove_event_listener_with_callback(
				"keyup",
				self.keyup_closure.as_ref().unchecked_ref()
			);
			let _ = window.remove_event_listener_with_callback(
				"keydown",
				self.keydown_closure.as_ref().unchecked_ref()
			);
		}
	}
}

pub fn denormalise(key_code: &str) -> Vec<String> {
	match key_code {
		"Shift" => vec!["ShiftLeft".to_string(), "ShiftRight".to_string()],
		"Control" => vec!["ControlLeft".to_string(), "ControlRight".to_string()],
		"Alt" => vec!["AltLeft".to_string(), "AltRight".to_string()],
		_ => if key_code.len() == 1 {
			if key_code.as_bytes()[0].is_ascii_digit() {
				vec!["Digit".to_string() + key_code]
			} else {
				vec!["Key".to_string() + key_code]
			}
		} else {
			vec![key_code.to_string()]
		},
	}
}

fn mouse_button(code: i16) -> super::MouseButton {
	match code {
		0 => super::MouseButton::Left,
		1 => super::MouseButton::Middle,
		2 => super::MouseButton::Right,
		3 => super::MouseButton::Back,
		4 => super::MouseButton::Forward,
		_ => super::MouseButton::Left,
	}
}

fn js_key_event_to_dvr(e: &KeyboardEvent) -> KeyCodeEvent {
	KeyCodeEvent {
		key_code: e.code(),
		ctrl_down: e.ctrl_key(),
		shift_down: e.shift_key(),
		alt_down: e.alt_key(),
	}
}

fn js_mouse_event_to_dvr(e: &MouseEvent) -> super::MouseEvent {
	super::MouseEvent {
		x: e.offset_x(),
		y: e.offset_y(),
		button: mouse_button(e.button()),
		ctrl_down: e.ctrl_key(),
		shift_down: e.shift_key(),
	}
}