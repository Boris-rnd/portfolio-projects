use crate::*;
use std::cell::{Cell, RefCell};
use std::io::Write;
use std::ops::{Deref, DerefMut};

use winit::event::{Event, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::*;

use crate::buffer::{Vertex, INDICES, VERTICES};
use crate::pipelines::{decor_time, PipelineManager};
use crate::texture::{load_texture, Texture2D, _load_texture};
use crate::{command, create_view, encoder, extent_3d};

pub static mut STATE: Option<StateWrapper> = None;
pub static mut WGPU_STATE: Option<WgpuState> = None;

#[track_caller]
pub fn get_state() -> &'static mut State {
    get_state_wrapper().state.as_mut().unwrap()
}
pub fn get_state_wrapper() -> &'static mut StateWrapper {
    #[allow(static_mut_refs)]
    unsafe { STATE.as_mut().unwrap() }
}

#[track_caller]
pub fn set_state(s: StateWrapper) {
    #[allow(static_mut_refs)]
    unsafe { STATE.replace(s) };
}

#[track_caller]
pub fn get_wgpu_state() -> &'static mut WgpuState<'static> {
    #[allow(static_mut_refs)]
    unsafe { WGPU_STATE.as_mut().unwrap() }
}

pub struct WgpuState<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub window: Window,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub current_texture: Option<SurfaceTexture>,
}

pub struct State {
    pub wgpu_state: &'static mut WgpuState<'static>,
    // pub profiler: GpuProfiler,
    pub pipelines: PipelineManager,
    pub gui_manager: GuiManager,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
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
                    trace: wgpu::Trace::Off,
                }
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
            current_texture: None,
            surface,
            device,
            queue,
            config,
            size,
        };
        #[allow(static_mut_refs)]
        unsafe { WGPU_STATE.replace(wgpu_state) }; // From now on it's safe to call get_wgpu_state()
        let wgpu_state = get_wgpu_state();
        // let profiler = GpuProfiler::new(GpuProfilerSettings::default()).unwrap();
        let pipelines =
            PipelineManager::new(&wgpu_state.device, &wgpu_state.config, &wgpu_state.queue);

        Self {
            wgpu_state,
            pipelines,
            // profiler,
            gui_manager: GuiManager::new(),
        }
    }
    pub fn input(
        &mut self,
        event: &WindowEvent,
        control_flow: &winit::event_loop::ActiveEventLoop,
    ) {
        if self.gui_manager.handle_event(event, control_flow) {
            return;
        }
        self.pipelines.rect_texture_pipeline.camera.controller.process_events(event); // TODO! Something cleaner
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
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.pipelines.update()
    }


    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.wgpu_state.current_texture.replace(self.surface.get_current_texture()?);
        let view = create_view!(self.current_texture.as_mut().unwrap().texture);

        let mut encoder = encoder!(self.device, "Render Encoder");

        self.pipelines.draw(&view, &mut encoder);

        self.queue.submit(Some(encoder.finish()));

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
        let size = (extent.width * extent.height * 4 * extent.depth_or_array_layers) as u64;
        let dst = state.device.create_buffer(&BufferDescriptor {
            label: None,
            size,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        command!(|encoder| {
            encoder.copy_texture_to_buffer(
                src.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &dst,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(extent.width * 4),
                        rows_per_image: Some(extent.height),
                    },
                },
                extent,
            )
        });
        State::read_buffer(&dst)
    }
    pub fn read_full_texture(src: &Texture) -> Vec<u8> {
        State::read_texture(src, src.size())
    }

    fn read_buffer(src: &Buffer) -> Vec<u8> {
        assert!(
            src.usage() & BufferUsages::MAP_READ != BufferUsages::empty(),
            "Buffer must be able to be read"
        );
        let slice = src.slice(..);
        slice.map_async(wgpu::MapMode::Read, |r| r.unwrap());
        let state = get_wgpu_state();
        state.device.poll(wgpu::MaintainBase::Wait).unwrap();
        let out = slice.get_mapped_range().to_vec();
        src.unmap();
        out
    }
}
impl WgpuState<'_> {
    pub fn screen_size(&self) -> dpi::PhysicalSize<u32> {
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



pub struct BaseApp {}
impl App for BaseApp {
    fn setup(&mut self, state: &mut State) {}
    fn update(&mut self) {}
    fn handle_event(&mut self, event: &Event<()>) {}
}

pub trait App {
    /// Called once at beginning of loop
    fn setup(&mut self, state: &mut State);
    /// Called every frame (after main draw and screen clear)
    fn update(&mut self);
    /// Called for ALL events
    fn handle_event(&mut self, event: &Event<()>);
}

pub struct StateWrapper {
    pub state: Option<State>,
    pub app: Box<dyn App>,
}
impl winit::application::ApplicationHandler for StateWrapper {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        match cause {
            winit::event::StartCause::Init => {
                let window_attributes = winit::window::WindowAttributes::default();
                let window = match event_loop.create_window(window_attributes) {
                    Ok(window) => Some(window),
                    Err(err) => {
                        eprintln!("error creating window: {err}");
                        return;
                    },
                }.unwrap();
                self.state.replace(pollster::block_on(State::new(window)));
                self.app.setup(get_state());
            },
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let s = self.state.as_mut().unwrap();
        if event == WindowEvent::RedrawRequested {
            self.app.update();
        }
        s.input(&event, event_loop);
    }
}

