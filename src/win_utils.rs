use windows::{core::Error, Win32::{Foundation::{GetLastError, HWND}, System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE}, UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, WM_QUIT}}};

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

/// Updates the window using PeekMessageW(), TranslateMessage() and DispatchMessageW().
/// Returns the exit code from WM_QUIT if it has been received, otherwise returns None.
pub fn update_window(hwnd: HWND) -> Option<usize> {
	unsafe {
		let mut msg: MSG = Default::default();
		while PeekMessageW(&mut msg, Some(hwnd), 0, 0, PM_REMOVE).0 != 0 {
			if msg.message == WM_QUIT {
				return Some(msg.wParam.0)
			}
			let _ = TranslateMessage(&mut msg);
			DispatchMessageW(&mut msg);
		}
		None
	}
}

/// Updates the window using GetMessageW(), TranslateMessage() and DispatchMessageW.
/// Returns the exit code from WM_QUIT if it has been received, otherwise returns None.
pub fn update_window_blocking(hwnd: HWND) -> Result<Option<usize>, String> {
	unsafe {
		let mut msg: MSG = Default::default();
		match GetMessageW(&mut msg, Some(hwnd), 0, 0).0 {
			0 => {
				Ok(Some(msg.wParam.0))
			},
			-1 => {
				Err(format!("Failed to get window message: {}", GetLastError().to_hresult().message()))
			},
			_ => {
				let _ = TranslateMessage(&mut msg);
				DispatchMessageW(&mut msg);
				Ok(None)
			},
		}
	}
}