use super::*;

pub struct RectRasterizer {
    pub rects: (Buffer, u32),
    // pub texture: Texture,
    pub bind_group: Option<BindGroup>,
    pub pipeline: ComputePipeline,
}
impl RectRasterizer {
    /// Entry point of the shader must be called main
    /// You will get the storage in binding 0
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        queue: &Queue,
    ) -> Self {
        let workgroups = (1, 1, 1);
        let size = 10*size_of::<RawRect>();
        
        let rect_buffer = create_buffer!(size, BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC);
        // let texture = create_texture!(800, 600, TextureUsages::STORAGE_BINDING);
        
        let pipeline = Self::create_compute_pipeline(device, "shaders/rect_rasterize.wgsl", Some(&[&[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Texture { sample_type: TextureSampleType::Float { filterable: false }, view_dimension: TextureViewDimension::D2, multisampled: false },
                count: None,
            },            
            ]]));
        Self {
            rects: (rect_buffer, 0),
            // texture,
            bind_group: None,
            pipeline,
        }
    }
    pub fn get_bind_group(&mut self) -> &mut BindGroup {
        if self.bind_group.is_none() {
            let bind_group_layout = self.pipeline.get_bind_group_layout(0);
            let bind_group = pipeliner::create_bind_group_buffers(
                &get_wgpu_state().device,
                &bind_group_layout,
                [self.rects.0.as_entire_binding(), BindingResource::TextureView(&create_view!(get_wgpu_state().current_texture.as_mut().unwrap().texture))],
                0,
            );
            self.bind_group.replace(bind_group);
        } 
        self.bind_group.as_mut().unwrap()
    }
    pub fn compute(&mut self) {
        let state = get_state();
        command!(|encoder| {
            let mut cpass = compute_pass!(encoder);
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, Some(&*self.get_bind_group()), &[]);
            cpass.dispatch_workgroups(self.rects.1, 1, 1);
            // copy_texture_to_texture(encoder, &self.texture, &state.current_texture.as_mut().unwrap().texture, self.texture.size());
        });
            
            // pollster::block_on(self.read_back(&state.device, encoder, &state.queue))
            // cpass.insert_debug_marker(&format!(
            //     "Compute (x: {}, y: {}, z: {})",
            //     self.workgroups.0, self.workgroups.1, self.workgroups.2
            // ));
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
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(&format!("Compute pipeline from {shader_path}")),
                layout: layout.as_ref(),
                module: &pipeliner::create_shader(device, shader_path),
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            })
        }    
}
