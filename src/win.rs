use winapi::{shared::{dxgi::{IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD}, dxgiformat::DXGI_FORMAT_B8G8R8A8_UNORM, dxgitype::{DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT}, windef::HWND}, um::{d3d11::{D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext, D3D11_SDK_VERSION}, d3dcommon::D3D_DRIVER_TYPE_HARDWARE}};
use std::ptr::{null, null_mut};
use crate::win_utils::*;

pub struct Dvr {
}

impl Dvr {
    pub fn new(hwnd: HWND) -> Result<Dvr, String> {
		unsafe {
			let sd = DXGI_SWAP_CHAIN_DESC {
				BufferDesc: DXGI_MODE_DESC {
					Width: 0,
					Height: 0,
					Format: DXGI_FORMAT_B8G8R8A8_UNORM,
					RefreshRate: DXGI_RATIONAL {
						Numerator: 0,
						Denominator: 0,
					},
					Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
					ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
				},
				SampleDesc: DXGI_SAMPLE_DESC {
					Count: 1,
					Quality: 0,
				},
				BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
				BufferCount: 1,
				OutputWindow: hwnd,
				Windowed: true.into(),
				SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
				Flags: 0,
			};

			// TODO: should be something like ComPtr
			let mut swapchain: *mut IDXGISwapChain = null_mut();
			let mut device: *mut ID3D11Device = null_mut();
			let mut context: *mut ID3D11DeviceContext = null_mut();

			D3D11CreateDeviceAndSwapChain(
				null_mut(),
				D3D_DRIVER_TYPE_HARDWARE,
				null_mut(),
				0,
				null(),
				0,
				D3D11_SDK_VERSION,
				&sd,
				&mut swapchain,
				&mut device,
				null_mut(),
				&mut context
			).failed("Failed to initialise Direct3D 11")?;

			Ok(Dvr {})
		}
	}
}
