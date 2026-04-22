pub mod atlas;
pub mod background;
pub mod compute;
pub mod fonts;
pub mod lines;
pub mod rect_texture;
pub mod rectangle;

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
use naga::FastHashMap;
use rect_texture::RectangleTexturePipeline;
use rectangle::RectanglePipeline;
use texture::_load_texture;
use util::DeviceExt;
use wgpu::*;
use winit::dpi::PhysicalSize;

pub trait Pipeline {
    //encoder: &mut wgpu::CommandEncoder,
    fn draw(&mut self, view: &wgpu::TextureView, render_pass: &mut RenderPass, state: &mut State);
    fn render_pipeline(&mut self) -> &mut RenderPipeline;
}

pub struct PipelineManager {
    pub bg_pipeline: BgPipeline,
    pub rect_pipeline: RectanglePipeline,
    pub rect_texture_pipeline: RectangleTexturePipeline,
    pub compute_pipeline: compute::ComputePipeline,
    pub lines_pipeline: lines::LinesPipeline,
    pub anonymous_pipelines: FastHashMap<String, Box<dyn Pipeline>>,
    // pub fonts: fonts::FontRenderer,
    pub current_frame: usize,
}
impl PipelineManager {
    pub fn new(device: &Device, config: &SurfaceConfiguration, queue: &Queue) -> Self {
        let rect_pipeline = RectanglePipeline::new(device, config);
        let mut loaded_fonts = FontRenderer::load_fonts();
        let rect_texture_pipeline = RectangleTexturePipeline::new(
            device,
            config,
            queue,
            &mut loaded_fonts,
            PhysicalSize::new(64., 64.),
        );

        let raw_compute_pipeline =
            compute::create_compute_pipeline(device, "shaders/compute.wgsl", None);
        let bind_group_layout = raw_compute_pipeline.get_bind_group_layout(0);
        let bind_group = pipeliner::create_bind_group_buffers(
            device,
            &bind_group_layout,
            [rect_pipeline.instances.0.as_entire_binding()],
            0,
        );

        let compute_pipeline = compute::ComputePipeline {
            compute_pipeline: raw_compute_pipeline,
            bind_group,
            storage_buffer: Rc::into_raw(rect_pipeline.instances.0.clone()) as *mut wgpu::Buffer,
            staging_buffer: State::empty_buffer(
                device,
                size_of::<RawRect>() as _,
                BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                false,
            ),
            workgroups: (rect_pipeline.instances.1, 1, 1),
        };
        let lines_pipeline = LinesPipeline::new();
        Self {
            rect_pipeline,
            bg_pipeline: BgPipeline::default(),
            // fonts: FontRenderer::new(device, &mut rect_texture_pipeline),
            lines_pipeline,
            rect_texture_pipeline,
            compute_pipeline,
            current_frame: 0,
            anonymous_pipelines: FastHashMap::default(),
        }
    }

    pub fn draw(&mut self, view: &TextureView, render_pass: &mut RenderPass) {
        let state = get_state();
        self.bg_pipeline.update();
        self.rect_texture_pipeline
            .draw(view, render_pass, state);
        self.rect_pipeline.draw(view, render_pass, state);
        self.lines_pipeline.draw(view, render_pass, state);
        for (n,p) in self.anonymous_pipelines.iter_mut() {
            p.draw(view, render_pass, state);
        }
        self.current_frame += 1;
    }
    pub fn update(&mut self) {
        let state = get_state();
        self.compute_pipeline.compute();
    }

    pub fn insert_pipeline(&mut self, name: String, pipe: Box<dyn Pipeline>) -> Option<Box<dyn Pipeline>> {
        self.anonymous_pipelines.insert(name, pipe)
    }
}
#[macro_export]
macro_rules! create_buffer {
    ($device: ident, $size: ident, $usage: expr, MAPPED, $label: expr) => {{
        $device.create_buffer(&wgpu::BufferDescriptor {
            size: $size,
            usage: $usage,
            mapped_at_creation: true,
            label: $label,
        })
    }};
    ($size: expr, $usage: expr, MAPPED, $label: expr) => {{
        $crate::create_buffer!(get_state().device,
            $size,
            $usage,
            MAPPED,
            $label,
        )
    }};
    ($size: expr, $usage: expr, MAPPED) => {{
        $crate::create_buffer!(get_state().device,
            size,
            usage,
            MAPPED,
            None,
        )
    }};
    ($device: ident, $size: expr, $usage: expr) => {{
        device.create_buffer!(&wgpu::BufferDescriptor {
            size,
            usage,
            mapped_at_creation: false,
            label,
        })
    }};
    ($size: expr, $usage: expr) => {{
        $crate::get_wgpu_state().device.create_buffer(&wgpu::BufferDescriptor {
            size: $size,
            usage: $usage,
            mapped_at_creation: false,
            label: None,
        })
    }};
}
#[macro_export]
macro_rules! create_buffer_init {
    ($contents: expr, $usage: expr) => {
        wgpu::util::DeviceExt::create_buffer_init(&$crate::get_wgpu_state().device, &wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice($contents),
            usage: $usage,
        })
    };
}
#[macro_export]
macro_rules! extent_3d {
    ($width: expr, $height: expr) => {
        wgpu::Extent3d {
            width: $width as _,
            height: $height as _,
            depth_or_array_layers: 1,
        }
    };
    ($width: expr, $height: expr, $depth_or_array_layers: expr) => {
        wgpu::Extent3d {
            width: $width as _,
            height: $height as _,
            depth_or_array_layers: $depth_or_array_layers as _,
        }
    };
}
#[macro_export]
macro_rules! encoder {
    () => {
        $crate::state::get_wgpu_state()
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    };
    ($device: expr) => {
        $device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None })
    };
    ($device: expr, $label: expr) => {
        $device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some($label),
        })
    };
    (label=$label: expr) => {
        $crate::state::get_wgpu_state()
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some($label),
            })
    };
}

#[macro_export]
macro_rules! command {
    ($closure: expr) => {
        let state = $crate::state::get_wgpu_state();
        let mut encoder: wgpu::CommandEncoder = $crate::encoder!(state.device);
        let mut clos: Box<dyn FnMut(&mut wgpu::CommandEncoder)> = Box::new($closure);
        clos(&mut encoder);
        state.queue.submit(Some(encoder.finish()))
    };
}

#[macro_export]
macro_rules! compute_pass {
    () => {
        todo!();
        $crate::compute_pass!(encoder!())
    };
    ($encoder: expr) => {
        $encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        })
    };
}
#[macro_export]
macro_rules! render_pass {
    ($encoder: ident) => {
        $encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Empty Render Pass"),
            color_attachments: &[],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    };
    ($encoder: ident, $view: expr, $clear_color: expr) => {
        $encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: $view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear($clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    };
    ($encoder: ident, $view: ident) => {
        $encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: $view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear($clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    };
}

pub fn copy_buffer_to_texture(
    encoder: &mut CommandEncoder,
    buffer: &Buffer,
    texture: &Texture,
    texture_extent: Extent3d,
) {
    encoder.copy_buffer_to_texture(
        wgpu::ImageCopyBuffer {
            buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(std::mem::size_of::<f32>() as u32 * texture_extent.width),
                rows_per_image: Some(texture_extent.height),
            },
        },
        ImageCopyTexture {
            texture,
            mip_level: 0,
            aspect: wgpu::TextureAspect::All,
            origin: Origin3d { x: 0, y: 0, z: 0 },
        },
        texture_extent,
    )
}

// pub static mut EMPTY_COLOR_ATTACHMENT: Option<&[Option<RenderPassColorAttachment>]> = None;
// pub fn set_empty_color_attachment(view: &TextureView) {
//     unsafe {EMPTY_COLOR_ATTACHMENT.replace()};
// }
pub fn get_empty_color_attachment(view: &TextureView) -> [Option<RenderPassColorAttachment>; 1] {
    [Some(wgpu::RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 0.,
            }),
            store: wgpu::StoreOp::Store,
        },
    })]
}

// pub struct VertexPipeline {
//     pub render: RenderPipeline,
//     pub vertex: (Buffer, u32),
//     pub index: (Buffer, u32),
// }
// impl Pipeline for VertexPipeline {
//     fn draw(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, state: &mut State) {
//         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//             label: Some("Render Pass"),
//             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                 view,
//                 resolve_target: None,
//                 ops: wgpu::Operations {
//                     load: wgpu::LoadOp::Clear(wgpu::Color {
//                         r: state.mouse_pos.x/state.window.inner_size().width as f64,
//                         g: state.mouse_pos.y/state.window.inner_size().height as f64,
//                         b: 0.3,
//                         a: 1.0,
//                     }),
//                     store: wgpu::StoreOp::Store,
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
//         render_pass.set_index_buffer(self.index.0.slice(..), wgpu::IndexFormat::Uint16);
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
    use wgpu::{
        Device, PipelineCompilationOptions, PipelineLayout, RenderPipeline,
        RenderPipelineDescriptor, TextureFormat, VertexBufferLayout,
    };
    pub fn create_shader(device: &Device, shader_path: &str) -> ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader_path),
            source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string(shader_path).unwrap().into()),
        })
    }
    pub fn create_pipeline_layout(
        device: &Device,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
        // pub targets: [Option<wgpu::ColorTargetState>; 1],
        pub desc: RenderPipelineDescriptor<'a>,
    }
    impl<'a> SimpleRenderPipelineDescriptor<'a> {
        pub fn new(
            shader: &'a ShaderModule,
            layout: Option<&'a PipelineLayout>,
            format: TextureFormat,
        ) -> Self {
            let targets = Box::new([Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })]);
            Self {
                desc: wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout,
                    vertex: wgpu::VertexState {
                        module: shader,
                        entry_point: "vs_main",
                        buffers: &[],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        // Sorry, but like so I'm able to return this descriptor easily
                        targets: Box::leak(targets),
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
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
            self.vertex.entry_point = vertex_entry
        }
        pub fn fragment_entry(&mut self, fragment_entry: &'a str) {
            self.fragment
                .as_mut()
                .map(|f| f.entry_point = fragment_entry);
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
        device.create_bind_group(&wgpu::BindGroupDescriptor {
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
