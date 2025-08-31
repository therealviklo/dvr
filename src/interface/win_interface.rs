use std::{cell::RefCell, rc::Rc};
use uuid::Uuid;
use windows::Win32::{Foundation::{GetLastError, SetLastError, HWND, LPARAM, LRESULT, RECT, WIN32_ERROR, WPARAM}, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::{AdjustWindowRect, CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW, LoadCursorW, RegisterClassExW, SetWindowLongPtrW, ShowWindow, UnregisterClassW, CREATESTRUCTW, CS_OWNDC, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME}};
use windows_strings::{HSTRING, PCWSTR};
use crate::{win_utils::{update_window, update_window_blocking}, DvrCtx};

pub struct Interface {
	_wnd_class: WndClass,
	shared: Rc<RefCell<Shared>>,
}

impl Interface {
	pub fn new(title: &str, width: i32, height: i32, resizeable: bool) -> Result<Interface, String> {
		unsafe {
			let wnd_class = WndClass::new()?;
			let mut r = RECT {
				left: 0,
				top: 0,
				right: width,
				bottom: height,
			};
			let style = WS_SYSMENU | WS_CAPTION | WS_MINIMIZEBOX | if resizeable { WS_THICKFRAME | WS_MAXIMIZEBOX } else { WINDOW_STYLE(0) };
			AdjustWindowRect(
				&mut r,
				style,
				false.into()
			).map_err(|_| "Failed to adjust window rectangle")?;
			let hinstance = GetModuleHandleW(None)
					.map_err(|_| "Failed to get module")?;
			let shared = Rc::new(RefCell::new(Shared {
				hwnd: None,
			}));
			// Pointer must be dealt with to avoid a memory leak.
			// If there is an error when creating the window, make sure
			// it doesn't return without dealing with this pointer.
			// If the window is successfully created, the WM_DESTROY message
			// will drop the pointer instead.
			let shared_ptr = Rc::into_raw(shared.clone());
			let hwnd = CreateWindowExW(
				WINDOW_EX_STYLE(0), 
				&wnd_class.wnd_class_name, 
				&HSTRING::from(title),
				style,
				CW_USEDEFAULT,
				CW_USEDEFAULT,
				r.right - r.left,
				r.bottom - r.top, 
				None,
				None,
				Some(hinstance.into()),
				Some(shared_ptr as *mut std::ffi::c_void)
			).map_err(|_| "Failed to create window");
			let hwnd = match hwnd {
				Ok(hwnd) => hwnd,
				Err(e) => {
					// Drop the raw shared rc pointer to prevent leak
					drop(Rc::from_raw(shared_ptr));
					return Err(e.into());
				},
			};
			shared.borrow_mut().hwnd = Some(hwnd);
			let window = Interface {
				_wnd_class: wnd_class,
				shared,
			};
			let _ = ShowWindow(hwnd, SW_SHOW);
			Ok(window)
		}
	}

	pub fn update(&self) {
		update_window(self.get_hwnd());
	}

	pub fn update_blocking(&self) {
		update_window_blocking(self.get_hwnd());
	}

	pub fn get_ctx(&self) -> DvrCtx {
		self.get_hwnd()
	}

	pub fn get_hwnd(&self) -> HWND {
		match self.shared.borrow().hwnd {
			Some(hwnd) => hwnd,
			None => panic!("Interface has no associated window"),
		}
	}

	pub fn exists(&self) -> bool {
		self.shared.borrow().hwnd.is_some()
	}
}

impl Drop for Interface {
	fn drop(&mut self) {
		unsafe {
			if let Ok(shared) = self.shared.try_borrow() {
				if let Some(hwnd) = shared.hwnd {
					let _ = DestroyWindow(hwnd);
				}
			}
		}
	}
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
	if msg == WM_CREATE {
		let shared_ptr = (*(lparam.0 as *const CREATESTRUCTW)).lpCreateParams;
		if shared_ptr.is_null() {
			return LRESULT(-1);
		}
		SetLastError(WIN32_ERROR(0));
		let res = SetWindowLongPtrW(
			hwnd,
			GWLP_USERDATA,
			shared_ptr as isize
		);
		if res == 0 && GetLastError().0 != 0 {
			return LRESULT(-1);
		}
		return LRESULT(0);
	}

	let shared_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<Shared>;
	if shared_ptr.is_null() {
		return DefWindowProcW(hwnd, msg, wparam, lparam);
	}
	if msg == WM_DESTROY {
		// Try to clear the hwnd in shared
		if let Ok(mut shared) = (*shared_ptr).try_borrow_mut() {
			shared.hwnd = None;
		}
		// Drop the wndproc's reference to the rc
		drop(Rc::from_raw(shared_ptr));
		// Set the pointer to null, since it has been dropped
		SetLastError(WIN32_ERROR(0));
		SetWindowLongPtrW(
			hwnd,
			GWLP_USERDATA,
			0
		);
		
		return LRESULT(0);
	}
	
	let shared = &(*shared_ptr);

	match msg {
		_ => DefWindowProcW(hwnd, msg, wparam, lparam)
	}
}

struct WndClass {
	wnd_class_name: HSTRING,
}

impl WndClass {
	fn new() -> Result<WndClass, String> {
		unsafe {
			let wnd_class_name = HSTRING::from("DVR wndclass ".to_owned() + &Uuid::new_v4().to_string());
			let wnd_class = WNDCLASSEXW {
				cbSize: size_of::<WNDCLASSEXW>() as u32,
				style: CS_OWNDC,
				lpfnWndProc: Some(wndproc),
				hInstance: GetModuleHandleW(None)
					.map_err(|_| "Failed to get module")?.into(),
				hCursor: LoadCursorW(None, IDC_ARROW)
					.map_err(|_| "Failed to load cursor")?,
				lpszClassName: PCWSTR(wnd_class_name.as_ptr()),
				..Default::default()
			};
			if RegisterClassExW(&wnd_class) == 0 {
				return Err("Failed to register window class".to_string())
			}
			Ok(WndClass {
				wnd_class_name
			})
		}
	}
}

impl Drop for WndClass {
	fn drop(&mut self) {
		unsafe {
			if let Ok(module_handle) = GetModuleHandleW(None) {
				let _ = UnregisterClassW(
					&self.wnd_class_name,
					Some(module_handle.into())
				);
			}
		}
	}
}

struct Shared {
	hwnd: Option<HWND>,
}
