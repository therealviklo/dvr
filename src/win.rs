use windows::{core::{Interface, GUID, PCSTR}, Win32::{Foundation::{HMODULE, HWND, RECT}, Graphics::{Direct3D::{D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D11_SRV_DIMENSION_TEXTURE2D, D3D_DRIVER_TYPE_HARDWARE}, Direct3D11::{D3D11CreateDeviceAndSwapChain, ID3D11BlendState, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader, ID3D11RasterizerState, ID3D11RenderTargetView, ID3D11Resource, ID3D11SamplerState, ID3D11ShaderResourceView, ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_VERTEX_BUFFER, D3D11_BLEND_DESC, D3D11_BLEND_INV_DEST_ALPHA, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD, D3D11_BLEND_SRC_ALPHA, D3D11_BUFFER_DESC, D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_CPU_ACCESS_WRITE, D3D11_CREATE_DEVICE_FLAG, D3D11_CULL_BACK, D3D11_FILL_SOLID, D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_RASTERIZER_DESC, D3D11_RENDER_TARGET_BLEND_DESC, D3D11_SAMPLER_DESC, D3D11_SDK_VERSION, D3D11_SHADER_RESOURCE_VIEW_DESC, D3D11_SUBRESOURCE_DATA, D3D11_TEXTURE2D_DESC, D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT}, Dxgi::{Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R32G32_FLOAT, DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC}, IDXGIAdapter, IDXGIDevice, IDXGIFactory, IDXGISwapChain, DXGI_MWA_NO_ALT_ENTER, DXGI_MWA_NO_PRINT_SCREEN, DXGI_MWA_NO_WINDOW_CHANGES, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT}, Imaging::{CLSID_WICImagingFactory, GUID_WICPixelFormat32bppBGRA, IWICBitmapDecoder, IWICBitmapFrameDecode, IWICComponentInfo, IWICFormatConverter, IWICImagingFactory, IWICPixelFormatInfo, WICBitmapDitherTypeNone, WICBitmapPaletteTypeCustom}}, System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER}, UI::WindowsAndMessaging::GetClientRect}};
use std::{ffi::{c_float, CString}, ptr::{null, null_mut}};
use crate::{win_utils::*, DvrCtx};

mod shader_data;

pub struct Dvr {
	swapchain: Option<SwapChain>,
	wic_factory: IWICImagingFactory,
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

			let wic_factory: IWICImagingFactory = CoCreateInstance(
				&CLSID_WICImagingFactory,
				None,
				CLSCTX_INPROC_SERVER
			).map_err(|_| "Failed to create WIC factory")?;

			Ok(Dvr {
				swapchain: Some(swapchain),
				wic_factory,
			})
		}
	}

	// pub async fn load_texture(&self, url: &str) -> Result<Texture, String> {

	// }
}

struct SwapChain {
	// width: c_float,
	// height c_float,
	target: ID3D11RenderTargetView,
	vertex_buffer: ID3D11Buffer,
	colour_shift_buffer: ID3D11Buffer,
	matrix_buffer: ID3D11Buffer,
	pixel_shader: ID3D11PixelShader,
	vertex_shader: ID3D11VertexShader,
	input_layout: ID3D11InputLayout,
	blend_state: ID3D11BlendState,
	rasterizer_state: ID3D11RasterizerState,
	sampler_state: ID3D11SamplerState,
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
				..Default::default()
			};
			let srd = D3D11_SUBRESOURCE_DATA {
				pSysMem: &raw const vertices[0] as *const std::ffi::c_void,
				..Default::default()
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
				..Default::default()
			};
			let msd_ps = D3D11_SUBRESOURCE_DATA {
				pSysMem: &raw const colour_shift[0] as *const std::ffi::c_void,
				..Default::default()
			};
			let mut colour_shift_buffer: Option<ID3D11Buffer> = None;
			device.CreateBuffer(&mbd_ps, Some(&msd_ps), Some(&mut colour_shift_buffer))
				.map_err(|_| "Failed to create colour shift buffer")?;
			let mut cs_arr = [colour_shift_buffer];
			context.PSSetConstantBuffers(0, Some(&cs_arr));
			let colour_shift_buffer = cs_arr[0].take();

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
				..Default::default()
			};
			let msd = D3D11_SUBRESOURCE_DATA {
				pSysMem: &raw const mtcs as *const std::ffi::c_void,
				..Default::default()
			};
			let mut matrix_buffer: Option<ID3D11Buffer> = None;
			device.CreateBuffer(&mbd, Some(&msd), Some(&mut matrix_buffer))
				.map_err(|_| "Failed to create matrix buffer")?;
			let mut mb_arr = [matrix_buffer];
			context.VSSetConstantBuffers(0, Some(&mb_arr));
			let matrix_buffer = mb_arr[0].take();

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
			context.IASetInputLayout(input_layout.as_ref().ok_or("Input layout was not created")?);

			let mut rt_arr = [target];
			context.OMSetRenderTargets(Some(&rt_arr), None);
			let target = rt_arr[0].take();

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

			let rasterizer_desc = D3D11_RASTERIZER_DESC {
				FillMode: D3D11_FILL_SOLID,
				CullMode: D3D11_CULL_BACK,
				FrontCounterClockwise: false.into(),
				DepthBias: 0,
				SlopeScaledDepthBias: 0.0,
				DepthBiasClamp: 0.0,
				DepthClipEnable: true.into(),
				ScissorEnable: true.into(),
				MultisampleEnable: false.into(),
				AntialiasedLineEnable: false.into()
			};
			let mut rasterizer_state: Option<ID3D11RasterizerState> = None;
			device.CreateRasterizerState(&rasterizer_desc, Some(&mut rasterizer_state))
				.map_err(|_| "Failed to create rasterizer state")?;
			context.RSSetState(rasterizer_state.as_ref().ok_or("Rasterizer state was not created")?);

			let sampler_desc = D3D11_SAMPLER_DESC {
				Filter: D3D11_FILTER_MIN_MAG_MIP_LINEAR,
				AddressU: D3D11_TEXTURE_ADDRESS_CLAMP,
				AddressV: D3D11_TEXTURE_ADDRESS_CLAMP,
				AddressW: D3D11_TEXTURE_ADDRESS_CLAMP,
				..Default::default()
			};
			let mut sampler_state: Option<ID3D11SamplerState> = None;
			device.CreateSamplerState(&sampler_desc, Some(&mut sampler_state))
				.map_err(|_| "Failed to create sampler state")?;
			let mut ss_arr = [sampler_state];
			context.PSSetSamplers(0, Some(&ss_arr));
			let sampler_state = ss_arr[0].take();

			let bsd = D3D11_BLEND_DESC {
				RenderTarget: [
					D3D11_RENDER_TARGET_BLEND_DESC {
						BlendEnable: true.into(),
						RenderTargetWriteMask: D3D11_COLOR_WRITE_ENABLE_ALL.0 as u8,
						SrcBlend: D3D11_BLEND_SRC_ALPHA,
						DestBlend: D3D11_BLEND_INV_SRC_ALPHA,
						SrcBlendAlpha: D3D11_BLEND_INV_DEST_ALPHA,
						DestBlendAlpha: D3D11_BLEND_ONE,
						BlendOp: D3D11_BLEND_OP_ADD,
						BlendOpAlpha: D3D11_BLEND_OP_ADD
					},
					Default::default(),
					Default::default(),
					Default::default(),
					Default::default(),
					Default::default(),
					Default::default(),
					Default::default(),
				],
				..Default::default()
			};
			let mut blend_state: Option<ID3D11BlendState> = None;
			device.CreateBlendState(&bsd, Some(&mut blend_state))
				.map_err(|_| "Failed to create blend state")?;

			let blend_factor: [c_float; 4] = [0.0, 0.0, 0.0, 0.0];
			context.OMSetBlendState(
				blend_state.as_ref().ok_or("Blend state was not created")?,
				Some(&blend_factor),
				0xffffffff
			);

			Ok(SwapChain {
				target: target.ok_or("Target was not created")?,
				vertex_buffer: vertex_buffer.ok_or("Vertex buffer was not created")?,
				colour_shift_buffer: colour_shift_buffer.ok_or("Colour shift buffer was not created")?,
				matrix_buffer: matrix_buffer.ok_or("Matrix buffer was not created")?,
				pixel_shader: pixel_shader.ok_or("Pixel shader was not created")?,
				vertex_shader: vertex_shader.ok_or("Vertex shader was not created")?,
				input_layout: input_layout.ok_or("Input layout was not created")?,
				blend_state: blend_state.ok_or("Blend state was not created")?,
				rasterizer_state: rasterizer_state.ok_or("Rasterizer state was not created")?,
				sampler_state: sampler_state.ok_or("Sampler state was not created")?,
			})
		}
	}
}

pub struct Texture {
	tex: ID3D11Texture2D,
	tex_view: ID3D11ShaderResourceView,
	size: (u32, u32),
}

impl Texture {
	fn create_texture_with_decoder(decoder: IWICBitmapDecoder, device: &ID3D11Device, wic_factory: &IWICImagingFactory) -> Result<Texture, String> {
		unsafe {
			let frame = decoder.GetFrame(0)
				.map_err(|_| "Failed to get image frame")?;

			let pixel_format = frame.GetPixelFormat()
				.map_err(|_| "Failed to get pixel format")?;

			let mut width = 0u32;
			let mut height = 0u32;
			let row_pitch;
			let buf_size;
			let mut buf: Vec<u8>;

			// Check if conversion is needed
			if pixel_format != GUID_WICPixelFormat32bppBGRA {
				// Conversion is needed

				let format_converter = wic_factory.CreateFormatConverter()
					.map_err(|_| "Failed to create WIC format converter")?;

				format_converter.Initialize(
					&frame,
					&GUID_WICPixelFormat32bppBGRA,
					WICBitmapDitherTypeNone,
					None,
					0.0,
					WICBitmapPaletteTypeCustom,
				).map_err(|_| "Failed to initialise WIC format converter")?;

				format_converter.GetSize(&mut width, &mut height)
					.map_err(|_| "Failed to get image size")?;

				let new_pixel_format = format_converter.GetPixelFormat()
					.map_err(|_| "Failed to get pixel format")?;
				if new_pixel_format != GUID_WICPixelFormat32bppBGRA {
					return Err("Failed to convert image format".to_string());
				}

				let compinfo = wic_factory.CreateComponentInfo(&new_pixel_format)
					.map_err(|_| "Failed to get pixel format info")?;

				let pfi: IWICPixelFormatInfo = compinfo.cast()
					.map_err(|_| "Failed to get pixel format info")?;
				let bpp = pfi.GetBitsPerPixel()
					.map_err(|_| "Failed to get bits per pixel")?;

				row_pitch = ((width as usize * bpp as usize) + 7) / 8;
				buf_size = row_pitch * height as usize;
				buf = vec![0u8; buf_size];

				format_converter.CopyPixels(
					null(),
					row_pitch as u32,
					buf.as_mut_slice(),
				).map_err(|_| "Failed to copy pixels")?;
			} else {
				// Conversion is not needed

				frame.GetSize(&mut width, &mut height).map_err(|_| "Failed to get image size")?;

				let compinfo = wic_factory.CreateComponentInfo(&pixel_format)
					.map_err(|_| "Failed to get pixel format info")?;

				let pfi: IWICPixelFormatInfo = compinfo.cast()
					.map_err(|_| "Failed to get pixel format info")?;
				let bpp = pfi.GetBitsPerPixel()
					.map_err(|_| "Failed to get bits per pixel")?;

				row_pitch = ((width as usize * bpp as usize) + 7) / 8;
				buf_size = row_pitch * height as usize;
				buf = vec![0u8; buf_size];

				frame.CopyPixels(
					null(),
					row_pitch as u32,
					buf.as_mut_slice()
				).map_err(|_| "Failed to copy pixels")?;
			}

			let tex_desc = D3D11_TEXTURE2D_DESC {
				Width: width,
				Height: height,
				ArraySize: 1,
				Format: DXGI_FORMAT_B8G8R8A8_UNORM,
				SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
				Usage: D3D11_USAGE_DEFAULT,
				BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
				MipLevels: 1,
				..Default::default()
			};
			let tex_sd = D3D11_SUBRESOURCE_DATA {
				pSysMem: buf.as_ptr() as *const std::ffi::c_void,
				SysMemPitch: row_pitch as u32,
				SysMemSlicePitch: buf_size as u32,
			};

			let mut tex = None;
			device.CreateTexture2D(&tex_desc, Some(&tex_sd), Some(&mut tex))
				.map_err(|_| "Failed to create texture")?;

			let srv_desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
				Format: DXGI_FORMAT_B8G8R8A8_UNORM,
				ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
				Anonymous: Default::default(),
			};
			let mut tex_view = None;
			device.CreateShaderResourceView(tex.as_ref().unwrap(), Some(&srv_desc), Some(&mut tex_view))
				.map_err(|_| "Failed to create texture view")?;

			Ok(Texture {
				tex: tex.ok_or("Texture was not created")?,
				tex_view: tex_view.ok_or("Texture view was not created")?,
				size: (width, height),
			})
		}
	}

	pub fn get_size(&self) -> (u32, u32) {
		self.size
	}

	pub fn get_width(&self) -> u32 {
		self.size.0
	}

	pub fn get_height(&self) -> u32 {
		self.size.1
	}
}
