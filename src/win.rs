use windows::{core::Interface, Win32::{Foundation::{HMODULE, HWND}, Graphics::{Direct3D::D3D_DRIVER_TYPE_HARDWARE, Direct3D11::{D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION}, Dxgi::{Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC}, IDXGIAdapter, IDXGIDevice, IDXGIFactory, IDXGISwapChain, DXGI_MWA_NO_ALT_ENTER, DXGI_MWA_NO_PRINT_SCREEN, DXGI_MWA_NO_WINDOW_CHANGES, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT}}}};
use std::ptr::null_mut;
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

			let mut swapchain: Option<IDXGISwapChain> = None;
			let mut device: Option<ID3D11Device> = None;
			let mut context: Option<ID3D11DeviceContext> = None;

			D3D11CreateDeviceAndSwapChain(
				None,
				D3D_DRIVER_TYPE_HARDWARE,
				HMODULE(null_mut()),
				D3D11_CREATE_DEVICE_FLAG(0),
				None,
				D3D11_SDK_VERSION,
				Some(&sd),
				Some(&mut swapchain),
				Some(&mut device),
				None,
				Some(&mut context)
			).map_err(|_| "Failed to initialise Direct3D 11")?;

			{
				let dxgi_device: IDXGIDevice =
					device.ok_or("Device was not created")?.cast()
					.map_err(|_| "Failed to get DXGI device")?;
				
				let dxgi_adapter: IDXGIAdapter =
					dxgi_device.GetParent()
					.map_err(|_| "Failed to get DXGI adapter")?;

				let dxgi_factory: IDXGIFactory =
					dxgi_adapter.GetParent()
					.map_err(|_| "Failed to get DXGI factory")?;

				dxgi_factory.MakeWindowAssociation(hwnd, DXGI_MWA_NO_ALT_ENTER | DXGI_MWA_NO_PRINT_SCREEN | DXGI_MWA_NO_WINDOW_CHANGES)
					.map_err(|_| "Failed to make window associations")?;
			}

			Ok(Dvr {})
		}
	}
}
