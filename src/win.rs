use windows::{core::{Interface, PCSTR}, Win32::{Foundation::{GENERIC_READ, HMODULE, HWND, RECT}, Graphics::{Direct3D::{D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D11_SRV_DIMENSION_TEXTURE2D, D3D_DRIVER_TYPE_HARDWARE}, Direct3D11::{D3D11CreateDeviceAndSwapChain, ID3D11BlendState, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader, ID3D11RasterizerState, ID3D11RenderTargetView, ID3D11Resource, ID3D11SamplerState, ID3D11ShaderResourceView, ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_CONSTANT_BUFFER, D3D11_BIND_SHADER_RESOURCE, D3D11_BIND_VERTEX_BUFFER, D3D11_BLEND_DESC, D3D11_BLEND_INV_DEST_ALPHA, D3D11_BLEND_INV_SRC_ALPHA, D3D11_BLEND_ONE, D3D11_BLEND_OP_ADD, D3D11_BLEND_SRC_ALPHA, D3D11_BUFFER_DESC, D3D11_COLOR_WRITE_ENABLE_ALL, D3D11_CPU_ACCESS_WRITE, D3D11_CREATE_DEVICE_FLAG, D3D11_CULL_BACK, D3D11_FILL_SOLID, D3D11_FILTER_MIN_MAG_MIP_LINEAR, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_MAP_WRITE_DISCARD, D3D11_RASTERIZER_DESC, D3D11_RENDER_TARGET_BLEND_DESC, D3D11_SAMPLER_DESC, D3D11_SDK_VERSION, D3D11_SHADER_RESOURCE_VIEW_DESC, D3D11_SHADER_RESOURCE_VIEW_DESC_0, D3D11_SUBRESOURCE_DATA, D3D11_TEX2D_SRV, D3D11_TEXTURE2D_DESC, D3D11_TEXTURE_ADDRESS_CLAMP, D3D11_USAGE_DEFAULT, D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT}, Dxgi::{Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R32G32_FLOAT, DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC}, IDXGIAdapter, IDXGIDevice, IDXGIFactory, IDXGISwapChain, DXGI_MWA_NO_ALT_ENTER, DXGI_MWA_NO_PRINT_SCREEN, DXGI_MWA_NO_WINDOW_CHANGES, DXGI_PRESENT, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT}, Imaging::{CLSID_WICImagingFactory, GUID_WICPixelFormat32bppBGRA, IWICBitmapDecoder, IWICImagingFactory, IWICPixelFormatInfo, WICBitmapDitherTypeNone, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand}}, System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER}, UI::{Shell::SHCreateMemStream, WindowsAndMessaging::GetClientRect}}};
use std::{ffi::{c_float, CString}, ptr::{null, null_mut}};
use windows_strings::*;
use directx_math::*;
use crate::{win_utils::*, DvrCtx};

mod shader_data;

pub struct Dvr {
	_com_init: ComInit,
	swap: IDXGISwapChain,
	device: ID3D11Device,
	context: ID3D11DeviceContext,
	swapchain: Option<SwapChain>,
	wic_factory: IWICImagingFactory,
	_hwnd: HWND,
}

impl Dvr {
    pub fn new(ctx: DvrCtx) -> Result<Dvr, String> {
		let com_init = ComInit::init()?;
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

			let mut swap: Option<IDXGISwapChain> = None;
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
				Some(&mut swap),
				Some(&mut device),
				None,
				Some(&mut context)
			).map_err(winerr_map("Failed to initialise Direct3D 11"))?;

			let swap = swap.ok_or("Swapchain was not created")?;
			let device = device.ok_or("DirectX device was not created")?;
			let context = context.ok_or("DirectX device contect was not created")?;

			{
				let dxgi_device: IDXGIDevice =
					device.cast()
					.map_err(winerr_map("Failed to get DXGI device"))?;
				
				let dxgi_adapter: IDXGIAdapter =
					dxgi_device.GetParent()
					.map_err(winerr_map("Failed to get DXGI adapter"))?;

				let dxgi_factory: IDXGIFactory =
					dxgi_adapter.GetParent()
					.map_err(winerr_map("Failed to get DXGI factory"))?;

				dxgi_factory.MakeWindowAssociation(ctx, DXGI_MWA_NO_ALT_ENTER | DXGI_MWA_NO_PRINT_SCREEN | DXGI_MWA_NO_WINDOW_CHANGES)
					.map_err(winerr_map("Failed to make window associations"))?;
			}

			let swapchain = SwapChain::new(
				&swap,
				&device,
				&context,
				ctx,
				500.0,
				250.0
			)?;

			let wic_factory: IWICImagingFactory = CoCreateInstance(
				&CLSID_WICImagingFactory,
				None,
				CLSCTX_INPROC_SERVER
			).map_err(winerr_map("Failed to create WIC factory"))?;

			Ok(Dvr {
				_com_init: com_init,
				swap: swap,
				device: device,
				context: context,
				swapchain: Some(swapchain),
				wic_factory,
				_hwnd: ctx,
			})
		}
	}

	pub fn start_draw(&self) -> Result<(), String> {
		Ok(())
	}

	pub fn end_draw(&self) -> Result<(), String> {
		unsafe {
			self.swap.Present(0, DXGI_PRESENT(0))
				.ok().map_err(winerr_map("Failed to present"))?;
		}
		Ok(())
	}

	pub fn end_draw_sync(&self, sync_interval: u32) -> Result<(), String> {
		unsafe {
			self.swap.Present(sync_interval, DXGI_PRESENT(0))
				.ok().map_err(winerr_map("Failed to present"))?;
		}
		Ok(())
	}

	pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) -> Result<(), String> {
		let clr_arr = [r, g, b, a];
		unsafe {
			self.context.ClearRenderTargetView(&self.get_swapchain()?.target, &clr_arr);
		}
		Ok(())
	}

	pub fn draw(&self, texture: &Texture, x: f32, y: f32, size: Option<(f32, f32)>, tex_pos_size: Option<((f32, f32), (f32, f32))>, angle: f32) -> Result<(), String> {
		let (width, height): (f32, f32) = match size {
			Some(size) => size,
			None => (texture.get_width() as f32, texture.get_height() as f32),
		};
		let ((src_x, src_y), (src_width, src_height)): ((f32, f32), (f32, f32)) = match tex_pos_size {
			Some(pos_size) => pos_size,
			None => ((0.0, 0.0), (texture.get_width() as f32, texture.get_height() as f32)),
		};
		unsafe {
			self.context.PSSetShaderResources(0, Some(&texture.tex_view_arr));

			let swapchain = self.get_swapchain()?;

			let mut msr_ps = Default::default();
			self.context.Map(
				&swapchain.colour_shift_buffer,
				0,
				D3D11_MAP_WRITE_DISCARD,
				0,
				Some(&mut msr_ps)
			).map_err(winerr_map("Failed to map colour shift buffer"))?;

			// TODO: Replace with actual colour shift
			let clr_shift: [c_float; 4] = [1.0, 1.0, 1.0, 1.0];
			std::ptr::copy_nonoverlapping(
				clr_shift.as_ptr(),
				msr_ps.pData as *mut c_float,
				clr_shift.len()
			);
			self.context.Unmap(&swapchain.colour_shift_buffer, 0);

			// TODO: actual desired size
			let desired_width = 500.0;
			let desired_height = 250.0;

			let scaling_factor = f32::min(desired_width / swapchain.width, desired_height / swapchain.height);

			#[repr(C)]
			struct Mtcs {
				mtx: XMMATRIX,
				tex_mtx: XMMATRIX,
			}
			let mtcs = Mtcs {
				mtx: XMMatrixTranspose(
					*(XMMatrix(XMMatrixScaling(width, height, 1.0)) *
					XMMatrix(XMMatrixRotationZ(angle)) *
					XMMatrix(XMMatrixTranslation(x * 2.0, y * 2.0, 0.0)) *
					XMMatrix(XMMatrixScaling(
						1.0 / scaling_factor / swapchain.width,
						1.0 / scaling_factor / swapchain.height,
						1.0
					)))
				),
				tex_mtx: XMMatrixTranspose(
					*(XMMatrix(XMMatrixScaling(
						src_width / texture.get_width() as f32,
						src_height / texture.get_height() as f32,
						1.0
					)) *
					XMMatrix(XMMatrixTranslation(
						src_x / texture.get_width() as f32,
						src_y / texture.get_height() as f32,
						0.0
					)))
				)
			};

			let mut msr = Default::default();
			self.context.Map(
				&swapchain.matrix_buffer,
				0,
				D3D11_MAP_WRITE_DISCARD,
				0,
				Some(&mut msr)
			).map_err(winerr_map("Failed to map matrix buffer"))?;

			std::ptr::copy_nonoverlapping(
				&mtcs as *const Mtcs as *const u8,
				msr.pData as *mut u8,
				std::mem::size_of::<Mtcs>()
			);
			self.context.Unmap(&swapchain.matrix_buffer, 0);

			self.context.Draw(6, 0);
		}
		Ok(())
	}

	pub /* async */ fn load_texture(&self, filename: &str) -> Result<Texture, String> {
		unsafe {
			let decoder = self.wic_factory.CreateDecoderFromFilename(
				&HSTRING::from(filename),
				None,
				GENERIC_READ,
				WICDecodeMetadataCacheOnDemand
			).map_err(winerr_map("Failed to create decoder for image file"))?;
			Texture::new(decoder, &self.device, &self.wic_factory)
		}
	}

	pub /* async */ fn load_texture_raw(&self, data: &[u8]) -> Result<Texture, String> {
		unsafe {
			let stream = SHCreateMemStream(Some(data))
				.ok_or("Failed to create IStream")?;
			let decoder = self.wic_factory.CreateDecoderFromStream(
				&stream,
				null(),
				WICDecodeMetadataCacheOnDemand
			).map_err(winerr_map("Failed to create decoder for image"))?;
			Texture::new(decoder, &self.device, &self.wic_factory)
		}
	}

	fn get_swapchain(&self) -> Result<&SwapChain, String> {
		self.swapchain.as_ref().ok_or(String::from("Swapchain is not available"))
	}
}

struct SwapChain {
	width: c_float,
	height: c_float,
	target: ID3D11RenderTargetView,
	_vertex_buffer: ID3D11Buffer,
	colour_shift_buffer: ID3D11Buffer,
	matrix_buffer: ID3D11Buffer,
	_pixel_shader: ID3D11PixelShader,
	_vertex_shader: ID3D11VertexShader,
	_input_layout: ID3D11InputLayout,
	_blend_state: ID3D11BlendState,
	_rasterizer_state: ID3D11RasterizerState,
	_sampler_state: ID3D11SamplerState,
}

impl SwapChain {
	fn new(swap: &IDXGISwapChain, device: &ID3D11Device, context: &ID3D11DeviceContext, hwnd: HWND, desired_width: c_float, desired_height: c_float) -> Result<SwapChain, String> {
		unsafe {
			let backbuffer: ID3D11Resource = swap.GetBuffer(0)
				.map_err(winerr_map("Failed to get back buffer"))?;
			
			let mut target: Option<ID3D11RenderTargetView> = None;
			device.CreateRenderTargetView(&backbuffer, None, Some(&mut target))
				.map_err(winerr_map("Failed to create render target view"))?;

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
				.map_err(winerr_map("Failed to create vertex buffer"))?;
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
				.map_err(winerr_map("Failed to create colour shift buffer"))?;
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
					[1.0, 0.0, 0.0, 0.0],
					[0.0, 1.0, 0.0, 0.0],
					[0.0, 0.0, 1.0, 0.0],
					[0.0, 0.0, 0.0, 1.0],
				],
				tex_mtx: [
					[1.0, 0.0, 0.0, 0.0],
					[0.0, 1.0, 0.0, 0.0],
					[0.0, 0.0, 1.0, 0.0],
					[0.0, 0.0, 0.0, 1.0],
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
				.map_err(winerr_map("Failed to create matrix buffer"))?;
			let mut mb_arr = [matrix_buffer];
			context.VSSetConstantBuffers(0, Some(&mb_arr));
			let matrix_buffer = mb_arr[0].take();

			let mut pixel_shader: Option<ID3D11PixelShader> = None;
			device.CreatePixelShader(
				&shader_data::PIXEL_SHADER_DATA, 
				None, 
				Some(&mut pixel_shader)
			).map_err(winerr_map("Failed to create pixel shader"))?;
			context.PSSetShader(
				pixel_shader.as_ref().ok_or("Pixel shader was not created")?,
				None
			);

			let mut vertex_shader: Option<ID3D11VertexShader> = None;
			device.CreateVertexShader(
				&shader_data::VERTEX_SHADER_DATA, 
				None, 
				Some(&mut vertex_shader)
			).map_err(winerr_map("Failed to create vertex shader"))?;
			context.VSSetShader(
				vertex_shader.as_ref().ok_or("Vertex shader was not created")?,
				None
			);

			let position_cstr = CString::new("Position")
				.map_err(|_| "Failed to create C string")?;
			let texcoord_cstr = CString::new("TexCoord")
				.map_err(|_| "Failed to create C string")?;
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
				.map_err(winerr_map("Failed to create input layout"))?;
			context.IASetInputLayout(input_layout.as_ref().ok_or("Input layout was not created")?);

			let mut rt_arr = [target];
			context.OMSetRenderTargets(Some(&rt_arr), None);
			let target = rt_arr[0].take();

			context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

			let mut wnd_size: RECT = Default::default();
			GetClientRect(hwnd, &mut wnd_size)
				.map_err(winerr_map("Failed to get window size"))?;
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
				.map_err(winerr_map("Failed to create rasterizer state"))?;
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
				.map_err(winerr_map("Failed to create sampler state"))?;
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
				.map_err(winerr_map("Failed to create blend state"))?;

			let blend_factor: [c_float; 4] = [0.0, 0.0, 0.0, 0.0];
			context.OMSetBlendState(
				blend_state.as_ref().ok_or("Blend state was not created")?,
				Some(&blend_factor),
				0xffffffff
			);

			Ok(SwapChain {
				width: width as f32,
				height: height as f32,
				target: target.ok_or("Target was not created")?,
				_vertex_buffer: vertex_buffer.ok_or("Vertex buffer was not created")?,
				colour_shift_buffer: colour_shift_buffer.ok_or("Colour shift buffer was not created")?,
				matrix_buffer: matrix_buffer.ok_or("Matrix buffer was not created")?,
				_pixel_shader: pixel_shader.ok_or("Pixel shader was not created")?,
				_vertex_shader: vertex_shader.ok_or("Vertex shader was not created")?,
				_input_layout: input_layout.ok_or("Input layout was not created")?,
				_blend_state: blend_state.ok_or("Blend state was not created")?,
				_rasterizer_state: rasterizer_state.ok_or("Rasterizer state was not created")?,
				_sampler_state: sampler_state.ok_or("Sampler state was not created")?,
			})
		}
	}
}

pub struct Texture {
	_tex: ID3D11Texture2D,
	// This is in an array due to the call to PSSetShaderResources() in draw()
	tex_view_arr: [Option<ID3D11ShaderResourceView>; 1],
	size: (u32, u32),
}

impl Texture {
	fn new(decoder: IWICBitmapDecoder, device: &ID3D11Device, wic_factory: &IWICImagingFactory) -> Result<Texture, String> {
		unsafe {
			let frame = decoder.GetFrame(0)
				.map_err(winerr_map("Failed to get image frame"))?;

			let pixel_format = frame.GetPixelFormat()
				.map_err(winerr_map("Failed to get pixel format"))?;

			let mut width = 0u32;
			let mut height = 0u32;
			let row_pitch;
			let buf_size;
			let mut buf: Vec<u8>;

			// Check if conversion is needed
			if pixel_format != GUID_WICPixelFormat32bppBGRA {
				// Conversion is needed

				let format_converter = wic_factory.CreateFormatConverter()
					.map_err(winerr_map("Failed to create WIC format converter"))?;

				format_converter.Initialize(
					&frame,
					&GUID_WICPixelFormat32bppBGRA,
					WICBitmapDitherTypeNone,
					None,
					0.0,
					WICBitmapPaletteTypeCustom,
				).map_err(winerr_map("Failed to initialise WIC format converter"))?;

				format_converter.GetSize(&mut width, &mut height)
					.map_err(winerr_map("Failed to get image size"))?;

				let new_pixel_format = format_converter.GetPixelFormat()
					.map_err(winerr_map("Failed to get pixel format"))?;
				if new_pixel_format != GUID_WICPixelFormat32bppBGRA {
					return Err("Failed to convert image format".to_string());
				}

				let compinfo = wic_factory.CreateComponentInfo(&new_pixel_format)
					.map_err(winerr_map("Failed to get pixel format info"))?;

				let pfi: IWICPixelFormatInfo = compinfo.cast()
					.map_err(winerr_map("Failed to get pixel format info"))?;
				let bpp = pfi.GetBitsPerPixel()
					.map_err(winerr_map("Failed to get bits per pixel"))?;

				row_pitch = ((width as usize * bpp as usize) + 7) / 8;
				buf_size = row_pitch * height as usize;
				buf = vec![0u8; buf_size];

				format_converter.CopyPixels(
					null(),
					row_pitch as u32,
					buf.as_mut_slice(),
				).map_err(winerr_map("Failed to copy pixels"))?;
			} else {
				// Conversion is not needed

				frame.GetSize(&mut width, &mut height)
					.map_err(winerr_map("Failed to get image size"))?;

				let compinfo = wic_factory.CreateComponentInfo(&pixel_format)
					.map_err(winerr_map("Failed to get pixel format info"))?;

				let pfi: IWICPixelFormatInfo = compinfo.cast()
					.map_err(winerr_map("Failed to get pixel format info"))?;
				let bpp = pfi.GetBitsPerPixel()
					.map_err(winerr_map("Failed to get bits per pixel"))?;

				row_pitch = ((width as usize * bpp as usize) + 7) / 8;
				buf_size = row_pitch * height as usize;
				buf = vec![0u8; buf_size];

				frame.CopyPixels(
					null(),
					row_pitch as u32,
					buf.as_mut_slice()
				).map_err(winerr_map("Failed to copy pixels"))?;
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
				.map_err(winerr_map("Failed to create texture"))?;

			let srv_desc = D3D11_SHADER_RESOURCE_VIEW_DESC {
				Format: DXGI_FORMAT_B8G8R8A8_UNORM,
				ViewDimension: D3D11_SRV_DIMENSION_TEXTURE2D,
				Anonymous: D3D11_SHADER_RESOURCE_VIEW_DESC_0 {
					Texture2D: D3D11_TEX2D_SRV {
						MostDetailedMip: 0,
						MipLevels: 1,
					}
				}
			};
			let mut tex_view = None;
			device.CreateShaderResourceView(
				tex.as_ref().ok_or("Texture was not created")?,
				Some(&srv_desc),
				Some(&mut tex_view)
			).map_err(|e| format!("Failed to create texture view {} {}", e.code(), e.message()))?; // TODO: clean up

			Ok(Texture {
				_tex: tex.ok_or("Texture was not created")?,
				tex_view_arr: [Some(tex_view.ok_or("Texture view was not created")?)],
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
