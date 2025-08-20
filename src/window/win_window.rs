use uuid::Uuid;
use windows::Win32::{Foundation::{HWND, RECT}, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::{AdjustWindowRect, CreateWindowExW, LoadCursorW, RegisterClassExW, ShowWindow, UnregisterClassW, CS_OWNDC, CW_USEDEFAULT, IDC_ARROW, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSEXW, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME}};
use windows_strings::{HSTRING, PCWSTR};

pub struct Window {
	_wnd_class: WndClass,
	hwnd: HWND,
}

impl Window {
	pub fn new(title: &str, width: i32, height: i32, resizeable: bool) -> Result<Window, String> {
		unsafe {
			let wnd_class = WndClass::new()?;
			let mut r: RECT = Default::default();
			let style = WS_SYSMENU | WS_CAPTION | WS_MINIMIZEBOX | if resizeable { WS_THICKFRAME | WS_MAXIMIZEBOX } else { WINDOW_STYLE(0) };
			AdjustWindowRect(
				&mut r,
				style,
				false.into()
			).map_err(|_| "Failed to adjust window rectangle")?;
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
				Some(GetModuleHandleW(None)
					.map_err(|_| "Failed to get module")?.into()),
				None // TODO: how to deal with this?
			).map_err(|_| "Failed to create window")?;
			let window = Window {
				_wnd_class: wnd_class,
				hwnd,
			};
			if ShowWindow(hwnd, SW_SHOW).0 != 0 {
				return Err("Failed to show window".to_string())
			}
			Ok(window)
		}
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
				// lpfnWndProc: ,
				hInstance: GetModuleHandleW(None)
					.map_err(|_| "Failed to get module")?.into(),
				hCursor: LoadCursorW(None, IDC_ARROW)
					.map_err(|_| "Failed to load cursor")?,
				lpszClassName: PCWSTR(wnd_class_name.as_ptr()),
				..Default::default()
			};
			if RegisterClassExW(&wnd_class) != 0 {
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
