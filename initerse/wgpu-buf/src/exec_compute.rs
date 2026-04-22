use bytemuck::{NoUninit, Pod};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BufferDescriptor, BufferUsages, CommandEncoder, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions, Queue, ShaderModule, ShaderModuleDescriptor};

use crate::ComputePipelineBundle;

pub struct ComputeProgram<T: NoUninit + Pod> {
    pub shader: ShaderModule,
    pub buf: wgpu::Buffer,
    pub stage_buf: wgpu::Buffer,
    pub pipe: ComputePipelineBundle,
    _buf: std::marker::PhantomData<T>
}
impl<T: NoUninit + Pod> ComputeProgram<T> {
    pub fn new(device: &Device, shader: ShaderModule, data: &[T]) -> Self {
        let buf = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(data),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        });
        let stage_buf = device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            size: buf.size(),
            mapped_at_creation: false,
        });
        let pipe = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Pipeline compute executor"),
            layout: None,
            module: &shader,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });
        
        Self {
            shader,
            pipe: ComputePipelineBundle::new(device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &pipe.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: buf.as_entire_binding(),
                    }
                ],
            }), pipe, (data.len() as _, 1, 1)),
            buf,
            _buf: std::marker::PhantomData,
            stage_buf,
        }
    }
    pub fn execute(&self, encoder: &mut CommandEncoder) {
        self.pipe.draw(encoder);
    }
    pub fn read_back(&self, dev: &Device, queue: &Queue) -> Option<Vec<T>> {
        let mut encoder = dev.create_command_encoder(&CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&self.buf, 0, &self.stage_buf, 0, self.buf.size());
        queue.submit(Some(encoder.finish()));
        let (snd, rec) = flume::bounded(1);
        self.stage_buf.slice(..).map_async(wgpu::MapMode::Read, move |res| {
            res.unwrap();
            snd.send(true).unwrap();
        });
        dev.poll(wgpu::MaintainBase::Wait);
        rec.recv().unwrap();
        let range = self.stage_buf.slice(..).get_mapped_range();
        let mut out = Vec::with_capacity(self.pipe.workgroups.0 as usize);
        unsafe {out.set_len(out.capacity());}
        out.copy_from_slice(&bytemuck::cast_slice(&range));
        drop(range);
        self.stage_buf.unmap();
        Some(out)
    }
}

pub async fn compute_default_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;
    Some(adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await
        .ok()?)
}

pub fn _compute<T: Pod>(dev: &Device, queue: &Queue, shader_path: &str, data: &[T]) -> Option<Vec<T>> {
    let prog = ComputeProgram::new(&dev, dev.create_shader_module(ShaderModuleDescriptor {
        label: Some(shader_path),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&std::fs::read_to_string(shader_path).unwrap())),
    }), data);
    let mut enc = dev.create_command_encoder(&CommandEncoderDescriptor { label: None });
    prog
        .execute(&mut enc);
    queue.submit(Some(enc.finish()));
    prog.read_back(&dev, &queue)
}

pub fn compute<T: Pod>(shader_path: &str, data: &[T]) -> Option<Vec<T>> {
    let (dev, queue) = pollster::block_on(compute_default_device()).unwrap();
    _compute(&dev, &queue, shader_path, data)
}
