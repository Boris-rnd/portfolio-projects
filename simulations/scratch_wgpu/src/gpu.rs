use std::sync::Arc;
use wgpu::*;
use winit::window::Window;

pub struct GpuContext {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    #[allow(dead_code)]
    pub adapter: wgpu::Adapter,
}

impl GpuContext {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: Default::default(),
                    experimental_features: Default::default(),
                    trace: Trace::Off,
                },
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config,
            adapter,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn create_buffer<T: bytemuck::Pod>(&self, label: &str, data: &[T], usage: BufferUsages) -> Buffer {
        let buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: (data.len() * std::mem::size_of::<T>()) as u64,
            usage,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(&buffer, 0, bytemuck::cast_slice(data));
        buffer
    }

    pub fn create_compute_pipeline(&self, label: &str, shader: &ShaderModule, entry: &str) -> ComputePipeline {
        self.device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(label),
            layout: None,
            module: shader,
            entry_point: Some(entry),
            compilation_options: Default::default(),
            cache: None,
        })
    }

    pub fn create_bind_group(&self, label: &str, layout: &BindGroupLayout, entries: &[BindGroupEntry]) -> BindGroup {
        self.device.create_bind_group(&BindGroupDescriptor {
            label: Some(label),
            layout,
            entries,
        })
    }
}
