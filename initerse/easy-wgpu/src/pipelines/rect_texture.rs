use std::io::Error;

use atlas::TextureAtlas;
use buffer::{RawRectTexture, Vertex, INDICES, VERTICES};
use image::RgbaImage;
use pipeliner::SimpleBindGroupLayoutDescriptor;
use state::{copy_texture_to_texture, get_wgpu_state};
use winit::dpi::PhysicalSize;

use super::*;

pub struct RectangleTexturePipeline {
    pub render: RenderPipeline,
    pub vertex: (Buffer, u32),
    pub index: (Buffer, u32),
    pub instances: (Buffer, u32),
    pub bind_group: BindGroup,
    pub textures: (Texture, Extent3d),
    pub staging_buffer: (Mutex<Buffer>, bool),
    pub atlas: TextureAtlas,
    pub atlas_render: RenderPipeline,
    pub atlas_instances: (Buffer, u32),
    pub atlas_bind_group: BindGroup,
}
impl RectangleTexturePipeline {
    pub fn new(
        device: &Device,
        config: &SurfaceConfiguration,
        queue: &Queue,
        textures: &mut [Vec<u8>],
        texture_dimensions: PhysicalSize<f32>,
    ) -> Self {
        // let images = texture_paths
        //     .into_iter()
        //     .map(|path| img_load_from_file(path).unwrap())
        //     .collect::<Vec<_>>();
        // images.iter().skip(1).for_each(|image| {
        //     assert_eq!(
        //         (image.width(), image.height()),
        //         (texture_dimensions.width as u32, texture_dimensions.height as u32)
        //     )
        // });
        let size = extent_3d!(texture_dimensions.width, texture_dimensions.height, textures.len());
        let mut raw_textures =
            Vec::with_capacity((size.width * size.height * size.depth_or_array_layers*4) as _);
        textures
            .iter_mut()
            .for_each(|mut img| raw_textures.append(&mut img));
        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    pipeliner::SimpleBindGroupLayoutDescriptor::create_texture(
                        0,
                        ShaderStages::FRAGMENT,
                        TextureViewDimension::D2Array,
                        TextureSampleType::Float { filterable: true },
                    ),
                    pipeliner::SimpleBindGroupLayoutDescriptor::create_sampler(
                        1,
                        ShaderStages::FRAGMENT,
                        SamplerBindingType::Filtering,
                    ),
                ],
                label: Some("Diffuse bind group"),
            });

        let array_texture = device.create_texture(&TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: config.format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
            label: Some("Texture array 2D"),
        });

        let staging_buffer = State::buffer(device, &raw_textures, BufferUsages::COPY_SRC);

        let mut encoder = encoder!(device, "Texture Upload Encoder");
        copy_buffer_to_texture(&mut encoder, &staging_buffer, &array_texture, size);
        queue.submit(Some(encoder.finish()));

        let array_texture_view = array_texture.create_view(&TextureViewDescriptor {
            label: Some("Array 2D texture view desc"),
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });
        let diffuse_bind_group = pipeliner::create_bind_group(
            device,
            &diffuse_bind_group_layout,
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&array_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler!(device)),
                },
            ],
        );
        let v = (Box::new([Vertex::desc(), RawRectTexture::desc()]));
        let render =
            pipeliner::create_render_pipeline(&device, &config, "shaders/textures.wgsl", |desc| {
                desc.vertex_buffers_layouts(Box::leak(v)); // I hate it
                desc.bind_group(device, &[&diffuse_bind_group_layout])
                // let mut ops = PipelineCompilationOptions::default();
                // ops.constants.insert(String::from("TEXTURE_BATCH_SIZE"), 16.);
                // desc.vertex_compilation_options(ops);
            });
        let vertex_buffer = State::vertex_buffer(&device, VERTICES);
        let index_buffer = State::index_buffer(&device, INDICES);
        let instances = vec![RawRectTexture::default()];
        // let instances = vec![RawRectTexture::new(RawRect::default(), 0)];
        let instances_buffer = State::vertex_buffer(device, &instances);
        let mut atlas = TextureAtlas::new(device, PhysicalSize::new(600, 600));
        let atlas_bind_group_layout = pipeliner::create_bind_group_layout(
            device,
            &[
                SimpleBindGroupLayoutDescriptor::create_texture(
                    0,
                    ShaderStages::FRAGMENT,
                    TextureViewDimension::D2,
                    TextureSampleType::Float { filterable: true },
                ),
                SimpleBindGroupLayoutDescriptor::create_sampler(
                    1,
                    ShaderStages::FRAGMENT,
                    SamplerBindingType::Filtering,
                ),
            ],
        );
        let atlas_bind_group = pipeliner::create_bind_group(
            device,
            &atlas_bind_group_layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&atlas.texture.create_view(
                        &TextureViewDescriptor {
                            dimension: Some(TextureViewDimension::D2),
                            ..Default::default()
                        },
                    )),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture_sampler!(device)),
                },
            ],
        );
        let img1 = atlas
            .add_image(img_load_from_file("happy-tree.png").unwrap())
            .unwrap();
        let img2 = atlas
            .add_image(img_load_from_file("happy-tree2.png").unwrap())
            .unwrap();
        
        Self {
            render,
            vertex: (vertex_buffer, VERTICES.len() as u32),
            index: (index_buffer, INDICES.len() as u32),
            instances: (instances_buffer, instances.len() as u32),
            bind_group: diffuse_bind_group,
            staging_buffer: (
                Mutex::new(device.create_buffer(&BufferDescriptor {
                    label: None,
                    size: size_of::<RawRectTexture>() as _,
                    usage: BufferUsages::COPY_SRC | BufferUsages::MAP_WRITE,
                    mapped_at_creation: false,
                })),
                false,
            ),
            textures: (array_texture, size),
            atlas_instances: (
                State::vertex_buffer(
                    device,
                    &vec![RawRectTextureAtlas::new(
                        RawRect::default(),
                        RawRect::default(),
                    )],
                ),
                1,
            ),
            atlas_render: pipeliner::create_render_pipeline(
                device,
                config,
                "shaders/atlas.wgsl",
                |desc| {
                    desc.bind_group(device, &[&atlas_bind_group_layout]);
                    desc.vertex_buffers_layouts(Box::leak(Box::new([
                        Vertex::desc(),
                        RawRectTextureAtlas::desc(),
                    ])));
                },
            ),
            atlas_bind_group,
            atlas,
        }
    }
    pub fn push_rect(&mut self, rect: RawRectTexture) {
        self.instances.1 += 1;
        let state = get_wgpu_state();
        if self.instances.1 * std::mem::size_of::<RawRectTexture>() as u32
            >= self.instances.0.size() as u32
        {
            self.instances.0 = State::resize_buffer(
                &self.instances.0,
                self.instances.0.size() * 2,
            );
            // dbg!(self.instances.0.size(), self.instances.1);
        }
        State::write_to_buffer_with_staging(
            &state.device,
            &state.queue,
            &self.instances.0,
            ((self.instances.1 - 1) * std::mem::size_of::<RawRectTexture>() as u32) as _,
            &[rect],
        );
    }
    pub fn append_rects(&mut self, rects: &[RawRectTexture]) {
        let new_count = self.instances.1 + rects.len() as u32;
        let state = get_state();
        if new_count * std::mem::size_of::<RawRectTexture>() as u32
            >= self.instances.0.size() as u32
        {
            self.instances.0 = State::resize_buffer(
                &self.instances.0,
                (self.instances.0.size() * 2)
                    .max(new_count as u64 * std::mem::size_of::<RawRectTexture>() as u64),
            );
            // dbg!(self.instances.0.size(), self.instances.1);
        }
        State::write_to_buffer_with_staging(
            &state.device,
            &state.queue,
            &self.instances.0,
            ((self.instances.1 - 1) * std::mem::size_of::<RawRectTexture>() as u32) as _,
            rects,
        );
        self.instances.1 = new_count;
    }
    /// Returns texture id
    pub fn add_texture(&mut self, raw_texture: Vec<u8>) -> u32 {
        assert_eq!(raw_texture.len(), 64*64*4);
        let state = get_state();
        let mut encoder = encoder!(state.device, "Texture Upload Encoder");
        let id = self.textures.1.depth_or_array_layers;
        self.textures.1.depth_or_array_layers += 1;
        let texture_extent = self.textures.1;
        
        let staging_buffer = State::buffer(&state.device, &raw_texture, BufferUsages::COPY_SRC);
        
        let new_texture = state.device.create_texture(&TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: state.config.format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
            label: Some("Texture array 2D"),
        });
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(std::mem::size_of::<f32>() as u32 * texture_extent.width),
                    rows_per_image: Some(texture_extent.height),
                },
            },
            ImageCopyTexture {
                texture: &new_texture,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: Origin3d { x: 0, y: 0, z: id },
            },
            extent_3d!(texture_extent.width, texture_extent.height, 1),
        );
        copy_texture_to_texture(&mut encoder, &self.textures.0, &new_texture, extent_3d!(texture_extent.width, texture_extent.height, id));
        self.textures.0 = new_texture;
        state.queue.submit(Some(encoder.finish()));

        let new_texture_view = self.textures.0.create_view(&TextureViewDescriptor {
            label: Some("Array 2D texture view desc"),
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });
        let device = &state.device;
        let diffuse_bind_group = pipeliner::create_bind_group(
            &state.device,
            &self.render.get_bind_group_layout(0),
            &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&new_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler!(device)),
                },
            ],
        );
        self.bind_group = diffuse_bind_group;
        // dbg!(State::read_full_texture(&self.textures.0).iter().filter(|x| **x!=0).collect::<Vec<_>>().len());
        id
    }
    
    pub fn add_text(&mut self, mut pos: GenericVec2<f32>, arg: &str) {
        let mut rects = Vec::with_capacity(arg.len());
        for chr in arg.chars() {
            rects.push(RawRectTexture { rect: RawRect::new(pos.x, pos.y, 64., 64.), texture_id: chr as u8 as u32-39 });
            pos.x += 64.;
        }
        dbg!(&rects);
        self.append_rects(&rects);
    }
}
impl Pipeline for RectangleTexturePipeline {
    fn draw(&mut self, view: &wgpu::TextureView, render_pass: &mut RenderPass, state: &mut State) {
        render_pass.set_pipeline(&self.render);
        render_pass.set_vertex_buffer(0, self.vertex.0.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.0.slice(..));
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_index_buffer(self.index.0.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index.1, 0, 0..self.instances.1 as _);

        render_pass.set_pipeline(&self.atlas_render);
        // render_pass.set_vertex_buffer(0, self.vertex.0.slice(..));
        render_pass.set_vertex_buffer(1, self.atlas_instances.0.slice(..));
        render_pass.set_bind_group(0, &self.atlas_bind_group, &[]);
        // render_pass.set_index_buffer(self.index.0.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index.1, 0, 0..self.atlas_instances.1 as _);
    }

    fn render_pipeline(&mut self) -> &mut RenderPipeline {
        &mut self.render
    }
}

pub fn img_load_from_file(path: &str) -> Result<RgbaImage, std::io::Error> {
    Ok(image::load_from_memory(&std::fs::read(path)?)
        .unwrap()
        .to_rgba8())
}

#[macro_export]
macro_rules! texture_sampler {
    ($device: ident, $address_mode: expr, $mag_filter: expr, $min_filter: expr) => {
        $device.create_sampler(&SamplerDescriptor {
            address_mode_u: $address_mode,
            address_mode_v: $address_mode,
            address_mode_w: $address_mode,
            mag_filter: $mag_filter,
            min_filter: $min_filter,
            mipmap_filter: $min_filter,
            label: None,
            ..Default::default()
        })
    };
    ($device: ident, $address_mode: expr) => {
        $crate::texture_sampler!(
            $device,
            $address_mode,
            wgpu::FilterMode::Linear,
            wgpu::FilterMode::Nearest
        )
    };
    ($device: ident) => {
        $crate::texture_sampler!($device, wgpu::AddressMode::ClampToEdge)
    };
}
