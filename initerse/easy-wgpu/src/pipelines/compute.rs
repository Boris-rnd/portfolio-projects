use std::rc::Rc;

use bytemuck::NoUninit;
use log::warn;

use super::*;

pub struct ComputePipeline {
    pub compute_pipeline: wgpu::ComputePipeline,
    pub bind_group: BindGroup,
    pub storage_buffer: *mut Buffer,
    pub staging_buffer: Buffer,
    /// Number of cells to run, the (x,y,z) size of item being processed
    /// Arguments to dispatch_work_groups
    pub workgroups: (u32, u32, u32),
}
impl ComputePipeline {
    /// Entry point of the shader must be called main
    /// You will get the storage in binding 0
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        queue: &Queue,
        data: &[impl NoUninit],
        shader_path: &str,
    ) -> Self {
        let workgroups = (data.len() as u32, 1, 1);
        let contents = bytemuck::cast_slice(data);
        // Instantiates buffer without data.
        // `usage` of buffer specifies how it can be used:
        //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
        //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: contents.len() as _,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Instantiates buffer with data (`numbers`).
        // Usage allowing the buffer to be:
        //   A storage buffer (can be bound within a bind group and thus available to a shader).
        //   The destination of a copy.
        //   The source of a copy.
        let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });
        // A bind group defines how buffers are accessed by shaders.
        // It is to WebGPU what a descriptor set is to Vulkan.
        // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).
        // A pipeline specifies the operation of a shader
        let compute_pipeline = create_compute_pipeline(device, shader_path, None);

        // Instantiates the bind group, once again specifying the binding of buffers.
        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = pipeliner::create_bind_group_buffers(
            device,
            &bind_group_layout,
            [storage_buffer.as_entire_binding()],
            0,
        );
        Self {
            compute_pipeline: compute_pipeline,
            bind_group,
            storage_buffer: todo!(), //storage_buffer,
            staging_buffer,
            workgroups,
        }
    }
    pub fn compute(&self) {
        let state = get_state();
        let mut encoder = encoder!(state.device);
        let mut cpass = compute_pass!(encoder);
        cpass.set_pipeline(&self.compute_pipeline);
        cpass.set_bind_group(0, &self.bind_group, &[]);
        cpass.insert_debug_marker(&format!(
            "Compute (x: {}, y: {}, z: {})",
            self.workgroups.0, self.workgroups.1, self.workgroups.2
        ));
        cpass.dispatch_workgroups(self.workgroups.0, self.workgroups.1, self.workgroups.2);
        drop(cpass);
        // dbg!();
        // let read = pollster::block_on(self.read_back(&state.device, encoder, &state.queue));
        // dbg!(read);
    }
    pub async fn read_back(
        &self,
        device: &Device,
        mut encoder: CommandEncoder,
        queue: &Queue,
    ) -> Option<Vec<Vec2f>> {
        let st_buffer = unsafe { &mut *self.storage_buffer };
        dbg!(st_buffer.size());
        encoder.copy_buffer_to_buffer(&st_buffer, 0, &self.staging_buffer, 0, st_buffer.size());

        // Submits command encoder for processing
        queue.submit(Some(encoder.finish()));
        // Note that we're not calling `.await` here.
        let buffer_slice = self.staging_buffer.slice(..);
        // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        device.poll(wgpu::Maintain::wait()).panic_on_timeout();

        // Awaits until `buffer_future` can be read from
        if let Ok(Ok(())) = receiver.recv_async().await {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to u32
            let result = bytemuck::cast_slice(&data).to_vec();

            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            self.staging_buffer.unmap(); // Unmaps buffer from memory
                                         // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                         //   delete myPointer;
                                         //   myPointer = NULL;
                                         // It effectively frees the memory

            // Returns data from buffer
            Some(result)
        } else {
            warn!("failed to run compute on gpu!");
            None
        }
    }
}


pub fn create_compute_pipeline(
    device: &Device,
    shader_path: &str,
    layout: Option<&[&[BindGroupLayoutEntry]]>,
) -> wgpu::ComputePipeline {
    let layout = if let Some(layout) = layout {
        let mut bind_group_layouts = Vec::with_capacity(layout.len());
        layout.into_iter().for_each(|set| {
            bind_group_layouts.push(device.create_bind_group_layout(
                &BindGroupLayoutDescriptor {
                    label: None,
                    entries: set,
                },
            ))
        });
        let b = bind_group_layouts.iter().collect::<Vec<_>>();
        Some(device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("")),
            bind_group_layouts: &b,
            push_constant_ranges: &[],
        }))
    } else {
        None
    };
    // Instantiates the pipeline.
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(&format!("Compute pipeline from {shader_path}")),
        layout: layout.as_ref(),
        module: &pipeliner::create_shader(device, shader_path),
        entry_point: "main",
        compilation_options: Default::default(),
        cache: None,
    })
}