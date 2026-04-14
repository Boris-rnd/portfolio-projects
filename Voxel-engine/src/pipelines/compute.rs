use std::{rc::Rc, sync::Arc};

use bytemuck::NoUninit;
use log::warn;

use super::*;

pub struct RectComputePipeline {
    pub compute_pipeline: ComputePipeline,
    pub bind_group: BindGroup,
    pub storage_buffer: *mut Buffer,
    pub staging_buffer: Buffer,
    /// Number of cells to run, the (x,y,z) size of item being processed
    /// Arguments to dispatch_work_groups
    pub workgroups: (u32, u32, u32),
}
impl RectComputePipeline {
    /// Entry point of the shader must be called main
    /// You will get the storage in binding 0
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        queue: &Queue,
        rect_pipeline: &RectanglePipeline,
    ) -> Self {
        let raw_compute_pipeline =
            compute::RectComputePipeline::create_compute_pipeline(device, "shaders/compute.wgsl", None);
        let bind_group_layout = raw_compute_pipeline.get_bind_group_layout(0);
        let bind_group = pipeliner::create_bind_group_buffers(
            device,
            &bind_group_layout,
            [rect_pipeline.instances.0.as_entire_binding()],
            0,
        );

        Self {
            compute_pipeline: raw_compute_pipeline,
            bind_group,
            storage_buffer: Arc::into_raw(rect_pipeline.instances.0.clone()) as *mut Buffer,
            staging_buffer: empty_buffer(
                device,
                size_of::<RawRect>() as _,
                BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                false,
            ),
            workgroups: (rect_pipeline.instances.1, 1, 1),
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
    pub fn create_compute_pipeline(
        device: &Device,
        shader_path: &str,
        layout: Option<&[&[BindGroupLayoutEntry]]>,
    ) -> ComputePipeline {
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
        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&format!("Compute pipeline from {shader_path}")),
            layout: layout.as_ref(),
            module: &pipeliner::create_shader(device, shader_path),
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        })
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
        assert!(device.poll(wgpu::PollType::Wait).unwrap().wait_finished());

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
