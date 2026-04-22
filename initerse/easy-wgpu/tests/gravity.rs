
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pos: [f32; 3],
    vel: [f32; 3],
    color: [u8; 3],
    mass: u8,
}
impl Particle {
    pub fn new(pos: [f32; 3],vel: [f32; 3],color: [u8; 3], mass: u8) -> Self {
        Self {
            pos,
            vel,
            color,
            mass,
        }
    }
}


pub struct GravityApp {
    particle_buffers: (Buffer, Buffer, Texture),
    particle_count: u32,
    particle_bind_group: BindGroup,
    particle_render_bind_group: BindGroup,
    particle_rasterizer_pipeline: ComputePipeline,
    pipeline: ComputePipeline,
}
impl App for GravityApp {
    fn setup(state: &mut State) -> Self {
        let pos = [
            Particle::new([0., 0., 0.], [1., 0., 0.], [255, 0, 0], 10),
            Particle::new([10., 0., 0.], [0., 0., 0.], [0, 0, 255], 10),
        ];
        let pos_bytes: &[u8] = bytemuck::cast_slice(&pos);
        let particle_count = pos.len() as u32;
        let particle_buffer_in = create_buffer_init!(&pos_bytes, BufferUsages::STORAGE);
        let particle_buffer_out = create_buffer!(pos_bytes.len() as _, BufferUsages::STORAGE);
        let pipeline = easy_wgpu::pipelines::compute::create_compute_pipeline(&state.device, "shaders/gravity.wgsl", None);
        let layout = pipeline.get_bind_group_layout(0);
        let particle_bind_group = pipelines::pipeliner::create_bind_group(&state.device, &layout, &[
            BindGroupEntry {
                binding: 0,
                resource: particle_buffer_in.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: particle_buffer_out.as_entire_binding(),
            },
        ]);
        let particle_rasterizer_pipeline = create_compute_pipeline(&state.device, "shaders/particles_raster.wgsl", Some(&[&[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            // BindGroupLayoutEntry {
            //     binding: 1,
            //     visibility: ShaderStages::COMPUTE,
            //     ty: BindingType::Buffer { ty: BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None },
            //     count: None,
            // },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture { access: StorageTextureAccess::WriteOnly, format: TextureFormat::Rgba8Unorm, view_dimension: TextureViewDimension::D2 },
                count: None,
            },
        ]]));
        let particle_rastered_out = state.device.create_texture(&TextureDescriptor { label: None, 
            size: extent_3d!(800, 600), 
            mip_level_count: 1, 
            sample_count: 1, 
            dimension: TextureDimension::D2, 
            format: TextureFormat::Rgba8Unorm, 
            usage: TextureUsages::STORAGE_BINDING, 
            view_formats: &[]
        });

        let particle_render_bind_group = pipelines::pipeliner::create_bind_group(&state.device, &particle_rasterizer_pipeline.get_bind_group_layout(0), &[
            BindGroupEntry {
                binding: 0,
                resource: particle_buffer_out.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: texture_entire_binding(&particle_rastered_out.create_view(&TextureViewDescriptor::default())),
            },
        ]);
        Self {
            particle_buffers: (particle_buffer_in, particle_buffer_out, particle_rastered_out),
            particle_count,
            particle_bind_group,
            pipeline,
            particle_render_bind_group,
            particle_rasterizer_pipeline,
        }
    }

    fn render(&mut self, screen: &wgpu::TextureView) {
        let state = get_state();
        command!(|encoder| {
            let mut cpass = compute_pass!(encoder);
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &self.particle_bind_group, &[]);
            cpass.dispatch_workgroups(self.particle_count, 1, 1);
            cpass.set_pipeline(&self.particle_rasterizer_pipeline);
            cpass.set_bind_group(0, &self.particle_render_bind_group, &[]);
            cpass.dispatch_workgroups(self.particle_count, 1, 1);
            drop(cpass);
            // encoder.copy_buffer_to_texture(ImageCopyBuffer { buffer: &self.particle_buffers.2, layout: ImageDataLayout { 
            //     offset: 0, 
            //     bytes_per_row: Some(4 * 1024),
            //     rows_per_image: None
            // } }, 
            // ImageCopyTexture { texture: &get_state().current_texture.as_mut().unwrap().texture, mip_level: 0, origin: Origin3d::ZERO, aspect: TextureAspect::All }, 
        // extent_3d!(800, 600));
            // let mut _pass = encoder.begin_render_pass(&RenderPassDescriptor { label: None, color_attachments: &[
            //     Some(RenderPassColorAttachment { view: screen, resolve_target: None, ops: Operations { load: LoadOp::Load, store: StoreOp::Store } })
            // ], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None });
            // _pass.draw(0..1, 0..1);
        });
        get_state().pipelines.rect_pipeline.instances
        // state.pipelines.rect_texture_pipeline.add_texture();
    // let back = State::read_buffer(&self.particle_buffers.2);
    // dbg!(back.into_iter().filter(|x|*x != 0).collect::<Vec<_>>().len());
    }

    fn handle_event(&mut self, event: &Event<()>) {}
}