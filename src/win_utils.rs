use windows::{core::Error, Win32::{Foundation::HWND, System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE}, UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE}}};

pub struct ComInit {}

impl ComInit {
	pub fn init() -> Result<ComInit, String> {
		unsafe {
			CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
				.ok().map_err(|_| "Failed to initialise COM")?;
			Ok(ComInit {})
		}
	}
}

impl Drop for ComInit {
	fn drop(&mut self) {
		unsafe {
			CoUninitialize();
		}
	}
}

pub fn winerr_map(msg: &str) -> impl Fn(Error) -> String {
	let msg = msg.to_string();
	move |err: Error| -> String {
		format!("{msg} ({}: {})", err.code(), err.message())
	}
}

pub fn update_window(hwnd: HWND) {
	unsafe {
		let mut msg: MSG = Default::default();
		while PeekMessageW(&mut msg, Some(hwnd), 0, 0, PM_REMOVE).0 != 0 {
			let _ = TranslateMessage(&mut msg);
			DispatchMessageW(&mut msg);
		}
	}
}

pub fn update_window_blocking(hwnd: HWND) {
	unsafe {
		let mut msg: MSG = Default::default();
		if GetMessageW(&mut msg, Some(hwnd), 0, 0).0 != 0 {
			let _ = TranslateMessage(&mut msg);
			DispatchMessageW(&mut msg);
		}
	}
}