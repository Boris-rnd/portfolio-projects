pub mod atlas;
pub mod background;
pub mod compute;
pub mod fonts;
pub mod lines;
pub mod rect_texture;
pub mod rectangle;
pub mod rect_rasterize;
pub mod voxel;

pub use super::buffer::*;

use std::{
    borrow::{Borrow, BorrowMut},
    mem::size_of,
    num::{self, NonZeroU32},
    rc::Rc,
    sync::Mutex,
    time::{Duration, SystemTime},
};

use super::*;
use background::BgPipeline;
use fonts::FontRenderer;
use lines::LinesPipeline;
use rect_rasterize::RectRasterizer;
use rect_texture::{img_load_from_file, RectangleTexturePipeline};
use rectangle::RectanglePipeline;
use texture::_load_texture;
use util::DeviceExt;
use voxel::VoxelPipeline;
use winit::dpi::PhysicalSize;

pub trait Pipeline {
    //encoder: &mut CommandEncoder,
    fn draw(&mut self, view: &TextureView, render_pass: &mut RenderPass, state: &mut State);
    fn render_pipeline(&mut self) -> &mut RenderPipeline;
}

pub struct PipelineManager {
    pub bg_pipeline: BgPipeline,
    pub rect_pipeline: RectanglePipeline,
    pub rect_texture_pipeline: RectangleTexturePipeline,
    pub compute_pipeline: compute::RectComputePipeline,
    pub lines_pipeline: lines::LinesPipeline,
    pub fonts: fonts::FontRenderer,
    // pub voxels: voxel::VoxelPipeline,
    pub pipelines: Vec<Box<dyn Pipeline>>,
    // pub rect_rasterizer: RectRasterizer,
    pub current_frame: usize,
}
impl PipelineManager {
    pub fn new(device: &Device, config: &SurfaceConfiguration, queue: &Queue) -> Self {
        let rect_pipeline = RectanglePipeline::new(device, config);
        let rect_texture_pipeline = RectangleTexturePipeline::new(
            device,
            config,
            queue,
            &mut [img_load_from_file("happy-tree-64.png").unwrap().to_vec()],
            PhysicalSize::new(64., 64.),
        );

        let lines_pipeline = LinesPipeline::new();
        let pipelines = vec![

        ];
        Self {
            compute_pipeline: compute::RectComputePipeline::new(device, config, queue, &rect_pipeline),
            rect_pipeline,
            bg_pipeline: BgPipeline::default(),
            fonts: FontRenderer::new(device),
            lines_pipeline: LinesPipeline::new(),
            rect_texture_pipeline,
            current_frame: 0,
            pipelines,
            // voxels: VoxelPipeline::new(device, config),
            // rect_rasterizer: RectRasterizer::new(device, config, queue),
        }
    }

    pub fn draw(&mut self, view: &TextureView, encoder: &mut CommandEncoder) {
        let state = get_state();
        self.rect_texture_pipeline.camera.update();
        self.bg_pipeline.update();
        let mut render_pass = render_pass!(encoder, view, self.bg_pipeline.bg_color);
        if self.current_frame % 60 == 60 {
            decor_time("Rect textures", || {
                self.rect_texture_pipeline
                    .draw(view, &mut render_pass, state)
            });
            decor_time("Rect", || {
                self.rect_pipeline.draw(view, &mut render_pass, state)
            });
            decor_time("Compute", || self.compute_pipeline.compute());
        } else {
            self.rect_texture_pipeline
                .draw(view, &mut render_pass, state);
            self.rect_pipeline.draw(view, &mut render_pass, state);
            self.lines_pipeline.draw(&mut render_pass);
            // self.rect_rasterizer.compute();
            self.compute_pipeline.compute();
        }
        for p in &mut self.pipelines {
            p.draw(view, &mut render_pass, state);
        }
        self.current_frame += 1;
    }
    pub fn update(&mut self) {
        let state = get_state();
    }
}

// pub static mut EMPTY_COLOR_ATTACHMENT: Option<&[Option<RenderPassColorAttachment>]> = None;
// pub fn set_empty_color_attachment(view: &TextureView) {
//     unsafe {EMPTY_COLOR_ATTACHMENT.replace()};
// }
pub fn get_empty_color_attachment(view: &TextureView) -> [Option<RenderPassColorAttachment>; 1] {
    [Some(RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: Operations {
            load: LoadOp::Clear(Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 0.,
            }),
            store: StoreOp::Store,
        },
    })]
}

// pub struct VertexPipeline {
//     pub render: RenderPipeline,
//     pub vertex: (Buffer, u32),
//     pub index: (Buffer, u32),
// }
// impl Pipeline for VertexPipeline {
//     fn draw(&mut self, view: &TextureView, encoder: &mut CommandEncoder, state: &mut State) {
//         let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
//             label: Some("Render Pass"),
//             color_attachments: &[Some(RenderPassColorAttachment {
//                 view,
//                 resolve_target: None,
//                 ops: Operations {
//                     load: LoadOp::Clear(Color {
//                         r: state.mouse_pos.x/state.window.inner_size().width as f64,
//                         g: state.mouse_pos.y/state.window.inner_size().height as f64,
//                         b: 0.3,
//                         a: 1.0,
//                     }),
//                     store: StoreOp::Store,
//                 },
//             })],
//             depth_stencil_attachment: None,
//             occlusion_query_set: None,
//             timestamp_writes: None,
//         });
//         render_pass.set_pipeline(&self.render);
//         // render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
//         render_pass.set_vertex_buffer(0, self.vertex.0.slice(..));
//         // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
//         render_pass.set_index_buffer(self.index.0.slice(..), IndexFormat::Uint16);
//         render_pass.draw_indexed(0..self.index.1, 0, 0..1);
//         // render_pass.draw(0..self.vertex_buffer.1, 0..1);
//     }

//     fn render_pipeline(&mut self) -> &mut RenderPipeline {
//         &mut self.render
//     }
// }

pub fn decor_time<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let (res, time) = record_time(f);
    println!("{name} took {time:?} to execute");
    res
}

pub fn record_time<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = SystemTime::now();
    let res = f();
    (res, start.elapsed().unwrap())
}

pub mod pipeliner {
    use std::ops::{Deref, DerefMut};

    use super::*;
    use {
        Device, PipelineCompilationOptions, PipelineLayout, RenderPipeline,
        RenderPipelineDescriptor, TextureFormat, VertexBufferLayout,
    };
    pub fn create_shader(device: &Device, shader_path: &str) -> ShaderModule {
        device.create_shader_module(ShaderModuleDescriptor {
            label: Some(shader_path),
            source: ShaderSource::Wgsl(std::fs::read_to_string(shader_path).unwrap().into()),
        })
    }
    pub fn create_pipeline_layout(
        device: &Device,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> PipelineLayout {
        device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        })
    }

    pub fn create_render_pipeline(
        device: &Device,
        config: &SurfaceConfiguration,
        shader_path: &str,
        pipeline_descriptor_mutator: impl FnOnce(&mut SimpleRenderPipelineDescriptor),
    ) -> RenderPipeline {
        let shader = create_shader(&device, shader_path);
        let render_pipeline_layout = create_pipeline_layout(device, &[]);
        let mut desc = SimpleRenderPipelineDescriptor::new(
            &shader,
            Some(&render_pipeline_layout),
            config.format,
        );
        pipeline_descriptor_mutator(&mut desc);
        device.create_render_pipeline(&desc.desc)
    }

    pub struct SimpleRenderPipelineDescriptor<'a> {
        // pub targets: [Option<ColorTargetState>; 1],
        pub desc: RenderPipelineDescriptor<'a>,
    }
    impl<'a> SimpleRenderPipelineDescriptor<'a> {
        pub fn new(
            shader: &'a ShaderModule,
            layout: Option<&'a PipelineLayout>,
            format: TextureFormat,
        ) -> Self {
            let targets = Box::new([Some(ColorTargetState {
                format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })]);
            Self {
                desc: RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout,
                    vertex: VertexState {
                        module: shader,
                        entry_point: Some("vs_main"),
                        buffers: &[],
                        compilation_options: PipelineCompilationOptions::default(),
                    },
                    fragment: Some(FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        // Sorry, but like so I'm able to return this descriptor easily
                        targets: Box::leak(targets),
                        compilation_options: PipelineCompilationOptions::default(),
                    }),
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: FrontFace::Ccw,
                        cull_mode: Some(Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                },
                // targets
            }
        }
        pub fn label(&mut self, label: &'a str) {
            self.label = Some(label);
        }
        pub fn vertex_buffers_layouts(&mut self, buffers: &'a [VertexBufferLayout<'a>]) {
            self.vertex.buffers = buffers
        }
        pub fn vertex_entry(&mut self, vertex_entry: &'a str) {
            self.vertex.entry_point = Some(vertex_entry)
        }
        pub fn fragment_entry(&mut self, fragment_entry: &'a str) {
            self.fragment
                .as_mut()
                .map(|f| f.entry_point = Some(fragment_entry));
        }
        pub fn vertex_compilation_options(
            &mut self,
            vertex_compilation_options: PipelineCompilationOptions<'a>,
        ) {
            self.vertex.compilation_options = vertex_compilation_options
        }
        pub fn fragment_compilation_options(
            &mut self,
            fragment_compilation_options: PipelineCompilationOptions<'a>,
        ) {
            self.fragment
                .as_mut()
                .map(|f| f.compilation_options = fragment_compilation_options);
        }
        pub fn bind_group(&mut self, device: &Device, bind_group_layouts: &[&BindGroupLayout]) {
            self.layout.as_mut().map(|layout| {
                *layout = Box::leak(Box::new(create_pipeline_layout(device, bind_group_layouts)))
                // todo!
            });
        }
    }
    impl<'a> Deref for SimpleRenderPipelineDescriptor<'a> {
        type Target = RenderPipelineDescriptor<'a>;

        fn deref(&self) -> &Self::Target {
            &self.desc
        }
    }
    impl<'a> DerefMut for SimpleRenderPipelineDescriptor<'a> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.desc
        }
    }
    pub struct SimpleBindGroupLayoutDescriptor {
        pub entries: [BindGroupLayoutEntry; 2],
        pub desc: BindGroupLayoutDescriptor<'static>,
    }
    impl SimpleBindGroupLayoutDescriptor {
        pub fn create_texture(
            binding: u32,
            visibility: ShaderStages,
            view_dimension: TextureViewDimension,
            sample_type: TextureSampleType,
        ) -> BindGroupLayoutEntry {
            BindGroupLayoutEntry {
                binding,
                visibility,
                ty: BindingType::Texture {
                    sample_type,
                    view_dimension,
                    multisampled: false,
                },
                count: None,
            }
        }
        pub fn create_sampler(
            binding: u32,
            visibility: ShaderStages,
            sampler: SamplerBindingType,
        ) -> BindGroupLayoutEntry {
            BindGroupLayoutEntry {
                binding,
                visibility,
                ty: BindingType::Sampler(sampler),
                count: None,
            }
        }
    }
    pub fn create_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        entries: &[BindGroupEntry],
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("Bind group {:?}", entries)),
            layout,
            entries,
        })
    }
    pub fn create_bind_group_buffers<'a>(
        device: &Device,
        layout: &BindGroupLayout,
        resources: impl IntoIterator<Item = BindingResource<'a>>,
        offset: usize,
    ) -> BindGroup {
        create_bind_group(
            device,
            layout,
            &resources
                .into_iter()
                .enumerate()
                .map(|(i, resource)| BindGroupEntry {
                    binding: (i + offset).try_into().unwrap(),
                    resource,
                })
                .collect::<Vec<_>>(),
        )
    }

    // pub fn create_bind_group_layout(entries: &[Bind]) -> SimpleBindGroupLayoutDescriptor {
    //     let entries = [
    //         SimpleBindGroupLayoutDescriptor::create_texture(
    //             0, ShaderStages::FRAGMENT,
    //             TextureViewDimension::D2Array,
    //             TextureSampleType::Float { filterable: true }
    //         ),
    //         SimpleBindGroupLayoutDescriptor::create_sampler(
    //             1, ShaderStages::FRAGMENT,
    //             SamplerBindingType::Filtering
    //         ),
    //     ];

    //     SimpleBindGroupLayoutDescriptor {
    //         entries,
    //         desc: BindGroupLayoutDescriptor {
    //             entries,
    //             label: Some("texture_bind_group_layout"),
    //         }
    //     }
    // }
    pub fn create_bind_group_layout(
        device: &Device,
        entries: &[BindGroupLayoutEntry],
    ) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries,
        })
    }
}
