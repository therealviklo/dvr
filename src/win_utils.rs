use windows::{core::HRESULT, Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE}};

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