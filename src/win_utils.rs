use winapi::shared::ntdef::HRESULT;

pub trait WinResultFailed {
	fn failed(self, err_msg: &str) -> Result<(), String>;
}

impl WinResultFailed for HRESULT {
	fn failed(self, err_msg: &str) -> Result<(), String> {
		if self < 0 {
			Err(err_msg.into())
		} else {
			Ok(())
		}
	}
}