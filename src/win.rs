use windows::{core::{Interface, PCSTR}, Win32::{Foundation::{HMODULE, HWND, RECT}, Graphics::{Direct3D::{D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D_DRIVER_TYPE_HARDWARE}, Direct3D11::{D3D11CreateDeviceAndSwapChain, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader, ID3D11RenderTargetView, ID3D11Resource, ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_WRITE, D3D11_CREATE_DEVICE_FLAG, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_SDK_VERSION, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT}, Dxgi::{Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R32G32_FLOAT, DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC}, IDXGIAdapter, IDXGIDevice, IDXGIFactory, IDXGISwapChain, DXGI_MWA_NO_ALT_ENTER, DXGI_MWA_NO_PRINT_SCREEN, DXGI_MWA_NO_WINDOW_CHANGES, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT}}, UI::WindowsAndMessaging::GetClientRect}};
use std::{ffi::{c_float, CString}, ptr::{null, null_mut}};
use crate::{win_utils::*, DvrCtx};

mod shader_data;

struct SwapChain {

}

impl SwapChain {
	fn new(swap: &IDXGISwapChain, device: &ID3D11Device, context: &ID3D11DeviceContext, hwnd: HWND, desired_width: c_float, desired_height: c_float) -> Result<SwapChain, String> {
		unsafe {
			let backbuffer: ID3D11Resource = swap.GetBuffer(0)
				.map_err(|_| "Failed to get back buffer")?;
			
			let mut target: Option<ID3D11RenderTargetView> = None;
			device.CreateRenderTargetView(&backbuffer, None, Some(&mut target))
				.map_err(|_| "Failed to create render target view")?;

			#[repr(C)]
			struct Vertex {
				_x: c_float,
				_y: c_float,
				_u: c_float,
				_v: c_float,
			}
			let vertices = [
				Vertex { _x: -1.0, _y:  1.0, _u: 0.0, _v: 0.0 },
				Vertex { _x:  1.0, _y:  1.0, _u: 1.0, _v: 0.0 },
				Vertex { _x:  1.0, _y: -1.0, _u: 1.0, _v: 1.0 },
				Vertex { _x: -1.0, _y:  1.0, _u: 0.0, _v: 0.0 },
				Vertex { _x:  1.0, _y: -1.0, _u: 1.0, _v: 1.0 },
				Vertex { _x: -1.0, _y: -1.0, _u: 0.0, _v: 1.0 },
			];
			let bd = D3D11_BUFFER_DESC {
				ByteWidth: (size_of::<Vertex>() * vertices.len()) as u32,
				Usage: D3D11_USAGE_DEFAULT,
				BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
				StructureByteStride: size_of::<Vertex>() as u32,
				CPUAccessFlags: 0,
				MiscFlags: 0,
			};
			let srd = D3D11_SUBRESOURCE_DATA {
				pSysMem: &raw const vertices[0] as *const std::ffi::c_void,
				SysMemPitch: 0,
				SysMemSlicePitch: 0,
			};
			let mut vertex_buffer: Option<ID3D11Buffer> = None;
			device.CreateBuffer(&bd, Some(&srd), Some(&mut vertex_buffer))
				.map_err(|_| "Failed to create vertex buffer")?;
			let stride = size_of::<Vertex>() as u32;
			let offset = 0;
			context.IASetVertexBuffers(
				0, 
				1, 
				Some(&vertex_buffer),
				Some(&stride),
				Some(&offset)
			);

			let colour_shift: [c_float; 4] = [1.0, 1.0, 1.0, 1.0];
			let mbd_ps = D3D11_BUFFER_DESC {
				BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
				Usage: D3D11_USAGE_DYNAMIC,
				CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
				ByteWidth: (size_of::<c_float>() * colour_shift.len()) as u32,
				MiscFlags: 0,
				StructureByteStride: 0,
			};
			let msd_ps = D3D11_SUBRESOURCE_DATA {
				pSysMem: &raw const colour_shift[0] as *const std::ffi::c_void,
				SysMemPitch: 0,
				SysMemSlicePitch: 0,
			};
			let mut colour_shift_buffer: Option<ID3D11Buffer> = None;
			device.CreateBuffer(&mbd_ps, Some(&msd_ps), Some(&mut colour_shift_buffer))
				.map_err(|_| "Failed to create colour shift buffer")?;
			context.PSSetConstantBuffers(0, Some(&[colour_shift_buffer]));

			#[repr(C)]
			struct Mtcs {
				mtx: [[c_float; 4]; 4],
				tex_mtx: [[c_float; 4]; 4],
			}
			let mtcs = Mtcs {
				mtx: [
					[0.0, 1.0, 0.0, 0.0],
					[1.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 1.0, 0.0],
					[0.0, 0.0, 0.0, 1.0]
				],
				tex_mtx: [
					[1.0, 0.0, 0.0, 0.0],
					[0.0, 1.0, 0.0, 0.0],
					[0.0, 0.0, 1.0, 0.0],
					[0.0, 0.0, 0.0, 1.0]
				],
			};
			let mbd = D3D11_BUFFER_DESC {
				BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
				Usage: D3D11_USAGE_DYNAMIC,
				CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
				ByteWidth: size_of::<Mtcs>() as u32,
				MiscFlags: 0,
				StructureByteStride: 0
			};
			let msd = D3D11_SUBRESOURCE_DATA {
				pSysMem: &raw const mtcs as *const std::ffi::c_void,
				SysMemPitch: 0,
				SysMemSlicePitch: 0,
			};
			let mut matrix_buffer: Option<ID3D11Buffer> = None;
			device.CreateBuffer(&mbd, Some(&msd), Some(&mut matrix_buffer))
				.map_err(|_| "Failed to create matrix buffer")?;
			context.VSSetConstantBuffers(0, Some(&[matrix_buffer]));

			let mut pixel_shader: Option<ID3D11PixelShader> = None;
			device.CreatePixelShader(
				&shader_data::PIXEL_SHADER_DATA, 
				None, 
				Some(&mut pixel_shader)
			).map_err(|_| "Failed to create pixel shader")?;

			let mut vertex_shader: Option<ID3D11VertexShader> = None;
			device.CreateVertexShader(
				&shader_data::VERTEX_SHADER_DATA, 
				None, 
				Some(&mut vertex_shader)
			).map_err(|_| "Failed to create vertex shader")?;

			let position_cstr = CString::new("Position").map_err(|_| "Failed to create C string")?;
			let texcoord_cstr = CString::new("TexCoord").map_err(|_| "Failed to create C string")?;
			let ied = [
				D3D11_INPUT_ELEMENT_DESC {
					SemanticName: PCSTR(position_cstr.as_ptr() as *const u8),
					SemanticIndex: 0,
					Format: DXGI_FORMAT_R32G32_FLOAT,
					InputSlot: 0,
					AlignedByteOffset: 0,
					InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
					InstanceDataStepRate: 0,
				},
				D3D11_INPUT_ELEMENT_DESC {
					SemanticName: PCSTR(texcoord_cstr.as_ptr() as *const u8),
					SemanticIndex: 0,
					Format: DXGI_FORMAT_R32G32_FLOAT,
					InputSlot: 0,
					AlignedByteOffset: 8,
					InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
					InstanceDataStepRate: 0,
				},
			];
			let mut input_layout: Option<ID3D11InputLayout> = None;
			device.CreateInputLayout(&ied, &shader_data::VERTEX_SHADER_DATA, Some(&mut input_layout))
				.map_err(|_| "Failed to create input layout")?;
			context.IASetInputLayout(&input_layout.ok_or("Input layout was not created")?);

			context.OMSetRenderTargets(Some(&[target]), None);

			context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

			let mut wnd_size: RECT = Default::default();
			GetClientRect(hwnd, &mut wnd_size)
				.map_err(|_| "Failed to get window size")?;
			let width = wnd_size.right;
			let height = wnd_size.bottom;

			let vp = D3D11_VIEWPORT {
				Width: width as f32,
				Height: height as f32,
				MinDepth: 0.0,
				MaxDepth: 1.0,
				TopLeftX: 0.0,
				TopLeftY: 0.0,
			};
			context.RSSetViewports(Some(&[vp]));

			let size_scaling_factor = f32::max(width as f32 / desired_width, height as f32 / desired_height);
			let scissor_rect = RECT {
				left: ((width as f32 - desired_width * size_scaling_factor) * 0.5) as i32,
				top: 0,
				right: ((width as f32 + desired_width * size_scaling_factor) * 0.5) as i32,
				bottom: height,
			};
			context.RSSetScissorRects(Some(&[scissor_rect]));

			Ok(SwapChain {  })
		}
	}
}

pub struct Dvr {
	swapchain : Option<SwapChain>
}

impl Dvr {
    pub fn new(ctx: DvrCtx) -> Result<Dvr, String> {
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
				OutputWindow: ctx,
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
					device.as_mut().ok_or("Device was not created")?.cast()
					.map_err(|_| "Failed to get DXGI device")?;
				
				let dxgi_adapter: IDXGIAdapter =
					dxgi_device.GetParent()
					.map_err(|_| "Failed to get DXGI adapter")?;

				let dxgi_factory: IDXGIFactory =
					dxgi_adapter.GetParent()
					.map_err(|_| "Failed to get DXGI factory")?;

				dxgi_factory.MakeWindowAssociation(ctx, DXGI_MWA_NO_ALT_ENTER | DXGI_MWA_NO_PRINT_SCREEN | DXGI_MWA_NO_WINDOW_CHANGES)
					.map_err(|_| "Failed to make window associations")?;
			}

			let swapchain = SwapChain::new(
				&swapchain.ok_or("Swapchain was not created")?,
				&device.ok_or("DirectX device was not created")?,
				&context.ok_or("DirectX device contect was not created")?,
				ctx,
				500.0,
				250.0
			)?;

			Ok(Dvr {
				swapchain: Some(swapchain)
			})
		}
	}
}
