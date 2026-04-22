use std::cell::{Cell, RefCell};
use std::io::Write;
use std::ops::{Deref, DerefMut};

use bytemuck::NoUninit;
use wgpu::util::DeviceExt;
use wgpu::{*,
    BindGroupLayout, Buffer, BufferDescriptor, BufferUsages, CommandEncoder, Device, Extent3d, Features, ImageCopyBuffer, ImageCopyBufferBase, ImageCopyTexture, ImageCopyTextureBase, ImageDataLayout, Origin3d, Queue, RenderPipeline, ShaderModule, SurfaceConfiguration, Texture, TextureUsages, TextureView
};
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};
use winit::dpi::PhysicalSize;
use winit::{dpi::PhysicalPosition, event::WindowEvent, window::Window};

use crate::buffer::{Vertex, INDICES, VERTICES};
use crate::gui_manager::GuiManager;
use crate::pipelines::{decor_time, PipelineManager};
use crate::texture::{load_texture, Texture2D, _load_texture};
use crate::{command, encoder, extent_3d, render_pass, App};

pub static mut STATE: Option<State> = None;
pub static mut WGPU_STATE: Option<WgpuState> = None;
pub static mut APP: Option<Box<dyn App>> = None;

#[track_caller]
pub fn get_state() -> &'static mut State {
    unsafe { STATE.as_mut().unwrap() }
}
#[track_caller]
pub fn get_wgpu_state() -> &'static mut WgpuState<'static> {
    unsafe { WGPU_STATE.as_mut().unwrap() }
}

pub struct WgpuState<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub window: Window,
    pub size: winit::dpi::PhysicalSize<u32>,
}

pub struct State {
    pub wgpu_state: &'static mut WgpuState<'static>,
    pub profiler: GpuProfiler,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    pub pipelines: PipelineManager,
    // pub render_pipeline: wgpu::RenderPipeline,
    // pub vertexs: Vec<Vertex>,
    // pub vertex_buffer: (wgpu::Buffer, u32),
    // pub indice_buffer: (wgpu::Buffer, u32),
    // pub diffuse_bind_group: wgpu::BindGroup,
    // pub diffuse_textures: Vec<Texture2D>,
    // pub instances: Vec<crate::buffer::Rect>,
    // pub instance_buffer: wgpu::Buffer,
    pub gui_manager: GuiManager,

    pub current_texture: Option<SurfaceTexture>,
}

impl State {
    pub fn get_app(&mut self) -> &mut dyn App {
        unsafe { APP.as_mut().unwrap().as_mut() }
    }
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(unsafe { &*(&window as *const _) })
            .unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: Features::TEXTURE_BINDING_ARRAY
                        | Features::TEXTURE_BINDING_ARRAY
                        | Features::MAPPABLE_PRIMARY_BUFFERS,
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let wgpu_state = WgpuState {
            window,
            surface,
            device,
            queue,
            config,
            size,
        };
        unsafe { WGPU_STATE.replace(wgpu_state) };
        let wgpu_state = get_wgpu_state();
        let profiler = GpuProfiler::new(GpuProfilerSettings::default()).unwrap();
        let pipelines =
            PipelineManager::new(&wgpu_state.device, &wgpu_state.config, &wgpu_state.queue);

        Self {
            wgpu_state,
            // render_pipeline,
            // diffuse_bind_group,
            // diffuse_textures,
            // vertexs: VERTICES.to_vec(),
            pipelines,
            profiler,
            gui_manager: GuiManager::new(),
            current_texture: None,
        }
    }
    pub fn buffer(
        device: &Device,
        buffer: &[impl bytemuck::NoUninit],
        usage: BufferUsages,
    ) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(buffer),
            usage,
        })
    }
    pub fn vertex_buffer(device: &Device, buffer: &[impl bytemuck::NoUninit]) -> Buffer {
        Self::buffer(
            device,
            buffer,
            BufferUsages::VERTEX
                | BufferUsages::COPY_DST
                | BufferUsages::COPY_SRC
                | BufferUsages::STORAGE,
        )
    }
    pub fn index_buffer(device: &Device, buffer: &[impl bytemuck::NoUninit]) -> Buffer {
        Self::buffer(device, buffer, BufferUsages::INDEX)
    }
    pub fn empty_buffer(
        device: &Device,
        size: u64,
        usage: BufferUsages,
        mapped_at_creation: bool,
    ) -> Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Bufer of {size} {usage:?}")),
            usage,
            size,
            mapped_at_creation,
        })
    }

    pub fn input(
        &mut self,
        event: &WindowEvent,
        control_flow: &winit::event_loop::EventLoopWindowTarget<()>,
    ) {
        if self.gui_manager.handle_event(event, control_flow) {
            return;
        }
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: winit::event::ElementState::Pressed,
                        physical_key:
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                        ..
                    },
                ..
            } => control_flow.exit(),
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } if event.state == winit::event::ElementState::Released => {
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(c) => match c {
                        // Shader hot reloading
                        winit::keyboard::KeyCode::KeyR => {
                            self.pipelines =
                                PipelineManager::new(&self.device, &self.config, &self.queue);
                            // Self::gen_render_pipeline(&self.device, &self.config, &Self::texture2d_bindgroup_layout(&self.device))
                        }
                        _ => {}
                    },
                    winit::keyboard::PhysicalKey::Unidentified(_) => todo!(),
                }
            }
            WindowEvent::Resized(physical_size) => {
                self.resize(*physical_size);
            }
            WindowEvent::RedrawRequested => {
                self.update();
                // let mut start = std::time::SystemTime::now();
                // let fps = 1000./start.elapsed().unwrap().as_millis() as f32;
                // print!("{fps}\r");
                // std::io::stdout().flush().unwrap();
                // start = std::time::SystemTime::now();
                match self.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => self.resize(self.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        dbg!();
                        control_flow.exit()
                    }
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.pipelines.update()
    }

    pub fn resize_buffer(
        old_buffer: &wgpu::Buffer,
        new_size: wgpu::BufferAddress,
    ) -> wgpu::Buffer {
        let state = get_wgpu_state();
        let new_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Resized Buffer"),
            size: new_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | old_buffer.usage(),
            mapped_at_creation: false,
        });

        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Buffer Resize Encoder"),
        });

        let copy_size = old_buffer.size().min(new_size);
        encoder.copy_buffer_to_buffer(old_buffer, 0, &new_buffer, 0, copy_size);

        state.queue.submit(Some(encoder.finish()));

        new_buffer
    }
    #[track_caller]
    pub fn write_to_buffer(
        buffer: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        data: &[impl NoUninit],
    ) {
        // Ensure the buffer was created with MAP_WRITE usage
        assert!(buffer.usage().contains(wgpu::BufferUsages::MAP_WRITE));

        // Map the buffer to allow CPU access
        let slice = buffer.slice(offset..offset + data.len() as wgpu::BufferAddress);
        slice.map_async(wgpu::MapMode::Write, |r| {
            r.unwrap();
        });
        get_wgpu_state().device.poll(wgpu::MaintainBase::Wait);
        // Write data into the mapped range
        let mut view = slice.get_mapped_range_mut();
        view.copy_from_slice(bytemuck::cast_slice(data));

        // Unmap the buffer to make it available for GPU use again
        buffer.unmap();
    }

    pub fn write_to_buffer_with_staging(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_buffer: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        data: &[impl NoUninit],
    ) {
        let contents: &[u8] = bytemuck::cast_slice(data);
        let staging_buffer = Self::buffer(device, contents, BufferUsages::COPY_SRC);
        let mut encoder = encoder!(device, "Buffer Copy Encoder");

        // Copy data from the staging buffer to the target buffer at the specified offset
        // dbg!(contents.len(), offset);
        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            target_buffer,
            offset,
            contents.len() as u64,
        );

        // Submit the command encoder to the queue
        queue.submit(Some(encoder.finish()));
    }





    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        self.current_texture.replace(output);
        let view = self.current_texture.as_mut().unwrap().texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = encoder!(self.device, "Render Encoder");
        let mut render_pass = render_pass!(encoder, &view, self.pipelines.bg_pipeline.bg_color);
        
        self.pipelines.draw(&view, &mut render_pass);
        drop(render_pass);
        self.queue.submit(Some(encoder.finish()));
        
        self.get_app().render(&view);

        self.current_texture.take().unwrap().present();

        Ok(())
    }




    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn load_diffuse_texture(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        path: &str,
    ) -> Texture2D {
        let diffuse_texture = _load_texture(&device, &queue, path).unwrap();
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some(&format!("diffuse_bind_group/{}", path)),
        });
        diffuse_texture
    }
    pub fn texture2d_bindgroup_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }
    
    pub(crate) fn read_texture(src: &Texture, to_read: Extent3d) -> Vec<u8> {
        assert!(src.usage() & TextureUsages::COPY_SRC != TextureUsages::empty());
        let state = get_wgpu_state();
        let extent = src.size();
        let size = (extent.width*extent.height*4*extent.depth_or_array_layers) as u64;
        let dst = state.device.create_buffer(&BufferDescriptor { label: None, size, usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ, mapped_at_creation: false });
        command!(|encoder: &mut CommandEncoder| {
            encoder.copy_texture_to_buffer(src.as_image_copy(), ImageCopyBuffer { buffer: &dst, layout: ImageDataLayout { 
                offset: 0, 
                bytes_per_row: Some(extent.width*4), 
                rows_per_image: Some(extent.height) } 
            }, 
            extent)
        });
        State::read_buffer(&dst)
    }
    pub fn read_full_texture(src: &Texture) -> Vec<u8> {
        State::read_texture(src, src.size())
    }
    
    pub fn read_buffer(src: &Buffer) -> Vec<u8> {
        assert!(src.usage() & BufferUsages::MAP_READ != BufferUsages::empty(), "Buffer must be able to be read");
        let slice = src.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
        let state = get_wgpu_state();
        state.device.poll(wgpu::MaintainBase::Wait);
        let out = slice.get_mapped_range().to_vec();
        src.unmap();
        out
    }
}
impl WgpuState<'_> {
    pub fn screen_size(&self) -> PhysicalSize<u32> {
        self.size
    }
}
impl Deref for State {
    type Target = WgpuState<'static>;

    fn deref(&self) -> &Self::Target {
        &self.wgpu_state
    }
}
impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.wgpu_state
    }
}

pub fn copy_texture_to_texture(
    encoder: &mut CommandEncoder,
    src: &Texture,
    dst: &Texture,
    size: Extent3d,
) {
    encoder.copy_texture_to_texture(src.as_image_copy(), dst.as_image_copy(), size)
}
pub fn copy_buffer_to_texture(
    encoder: &mut CommandEncoder,
    src: &Buffer,
    dst: &Texture,
    size: Extent3d,
) {
    encoder.copy_buffer_to_texture(ImageCopyBuffer { buffer: src, layout: ImageDataLayout { 
        offset: 0, bytes_per_row: Some(4*size.width), rows_per_image: Some(size.height) } }, dst.as_image_copy(), size)
}


pub fn texture_entire_binding(view: &TextureView) -> BindingResource {
    BindingResource::TextureView(view)
}