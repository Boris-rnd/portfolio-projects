use crate::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_resource::binding_types::{storage_buffer, storage_buffer_read_only, uniform_buffer};
use bevy::render::texture::GpuImage;
use bevy::render::{*, render_resource::*, render_asset::*, storage::*, renderer::*};
use bevy::shader::*;

use std::borrow::Cow;
#[derive(Resource)]
pub struct CameraUniform(UniformBuffer<FragCamera>);
#[derive(Resource)]
pub struct BeamCameraUniform(UniformBuffer<FragCamera>);

#[derive(Resource, ExtractResource, Clone)]
pub struct ReadbackBuffer {
    pub buffers: Vec<Handle<ShaderBuffer>>,
}

#[derive(Resource, ExtractResource, Clone)]
pub struct ComputeAtlas(Handle<Image>);

pub struct GpuReadbackPlugin;
impl Plugin for GpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            // Pipeline setup used to live in `FromWorld`/`finish()` because `RenderDevice`
            // wasn't available until after `build()`. In 0.19 this is a normal system that
            // runs once in the new `RenderStartup` schedule instead.
            .add_systems(RenderStartup, init_compute_pipeline)
            .add_systems(
                Render,
                (
                    (prepare_bind_group)
                        .in_set(RenderSystems::PrepareBindGroups)
                        // We don't need to recreate the bind group every frame
                        .run_if(not(resource_exists::<GpuShaderBufferBindGroup>)),
                    resize_cameras.after(prepare_bind_group),
                ),
            )
            // .add_systems(
            //     Render,
            //     run_compute_node.in_set(RenderSystems::Queue),
            // )
        ;
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame
        render_app
            .add_systems(
                Core3d,
                run_compute_node.in_set(Core3dSystems::MainPass),
            )
;
    }
}

pub fn resize_cameras(
    mut frag_camera: ResMut<FragCamera>,
    mut cam_uni: ResMut<CameraUniform>,
    mut beam_cam_uni: ResMut<BeamCameraUniform>,

    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    cam_uni.0.set(frag_camera.clone());
    cam_uni.0.write_buffer(&render_device, &queue);
    beam_cam_uni.0.set(frag_camera.clone());
    beam_cam_uni.0.write_buffer(&render_device, &queue);
}

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputePipeline>,
    render_device: Res<RenderDevice>,
    my_buffers: Res<ReadbackBuffer>,
    atlas: Res<ComputeAtlas>,
    image: Res<AccumulatedTexture>,
    max_depth_buffer: Res<BeamReadbackBuffer>,
    buffers: Res<RenderAssets<GpuShaderBuffer>>,
    camera: Res<FragCamera>,
    images: Res<RenderAssets<GpuImage>>,
    queue: Res<RenderQueue>,
) {
    let mut cam_buf = UniformBuffer::from(camera.clone());
    cam_buf.write_buffer(&render_device, &queue);

    let mut entries: Vec<(usize, BindingResource<'_>)> = vec![
        (
            0,
            buffers.get(&image.0).unwrap().buffer.as_entire_binding(),
        ),
        (1, cam_buf.binding().unwrap()),
        (
            2,
            buffers
                .get(&max_depth_buffer.max_depth_buffer)
                .unwrap()
                .buffer
                .as_entire_binding(),
        ),
        (3, images.get(&atlas.0).unwrap().texture_view.into_binding()),
    ];

    for (i, b) in my_buffers.buffers.iter().enumerate() {
        entries.push((4 + i, buffers.get(b).unwrap().buffer.as_entire_binding()));
    }

    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &entries
            .iter()
            .map(|(i, binding)| BindGroupEntry {
                binding: *i as u32,
                resource: binding.clone(),
            })
            .collect::<Vec<_>>(),
    );
    commands.insert_resource(CameraUniform(cam_buf));
    commands.insert_resource(GpuShaderBufferBindGroup(bind_group));
}

fn beam_prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<BeamComputePipeline>,
    render_device: Res<RenderDevice>,
    world_buffers: Res<ReadbackBuffer>,
    my_buffers: Res<BeamReadbackBuffer>,
    buffers: Res<RenderAssets<GpuShaderBuffer>>,
    camera: Res<FragCamera>,
    queue: Res<RenderQueue>,
) {
    let mut cam_buf = UniformBuffer::from(camera.clone());
    cam_buf.write_buffer(&render_device, &queue);

    let mut entries: Vec<(usize, BindingResource<'_>)> = vec![
        (
            0,
            buffers
                .get(&my_buffers.max_depth_buffer)
                .unwrap()
                .buffer
                .as_entire_binding(),
        ),
        (1, cam_buf.binding().unwrap()),
    ];

    for (i, b) in world_buffers.buffers.iter().enumerate() {
        entries.push((2 + i, buffers.get(b).unwrap().buffer.as_entire_binding()));
    }

    let bind_group = render_device.create_bind_group(
        None,
        &pipeline.layout,
        &entries
            .iter()
            .map(|(i, binding)| BindGroupEntry {
                binding: *i as u32,
                resource: binding.clone(),
            })
            .collect::<Vec<_>>(),
    );
    commands.insert_resource(BeamCameraUniform(cam_buf));
    commands.insert_resource(BeamGpuBufferBindGroup(bind_group));
}

#[derive(Resource)]
pub struct GpuShaderBufferBindGroup(pub BindGroup);
pub fn setup_compute(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut storage_buffers: ResMut<Assets<ShaderBuffer>>,
    window_query: Single<&Window, With<bevy::window::PrimaryWindow>>,
    game_world: Res<GameWorld>,
    camera: Res<FragCamera>,
) {
    let _win_size = window_query.resolution.size();

    let (data, size) = get_raw_atlas().unwrap();
    let mut image = Image::new(
        Extent3d {
            width: size.x,
            height: size.y * size.z,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC | TextureUsages::STORAGE_BINDING;
    image.reinterpret_stacked_2d_as_array(size.z);

    commands.insert_resource(ComputeAtlas(images.add(image)));

    commands.insert_resource(ReadbackBuffer {
        buffers: vec![], // my_buffers
    });
    commands.insert_resource(AccumulatedTexture(
        storage_buffers.add(ShaderBuffer::from(vec![0u32; (1920 * 1080) as usize])),
        storage_buffers.add(ShaderBuffer::from(vec![0u32; (1920 * 1080) as usize])),
    ));

    commands.insert_resource(BeamReadbackBuffer {
        max_depth_buffer: storage_buffers.add(ShaderBuffer::from(vec![0.0f32; (1920 * 1080) / 4 as usize])),
    });
}
#[derive(Resource)]
pub struct ComputePipeline {
    pub layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
}

// This used to be `impl FromWorld for ComputePipeline`, called from `Plugin::finish()`.
// In 0.19, pipeline resources are set up via a plain system run once in `RenderStartup`.
fn init_compute_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "Bind group layout compute",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<Vec<u32>>(false),
                uniform_buffer::<FragCamera>(false),
                storage_buffer_read_only::<Vec<f32>>(false),
                binding_types::texture_storage_2d_array(
                    TextureFormat::Rgba8Unorm,
                    StorageTextureAccess::ReadOnly,
                ),
                storage_buffer_read_only::<Vec<VoxelChunk>>(false),
                storage_buffer_read_only::<Vec<MapDataPacked>>(false),
                storage_buffer_read_only::<Vec<MapDataPacked>>(false),
                storage_buffer_read_only::<Vec<MapDataPacked>>(false),
                storage_buffer_read_only::<Vec<MapDataPacked>>(false),
            ),
        ),
    );
    let shader = asset_server.load("shaders/raytrace-compiled.wgsl");
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("GPU readback compute shader".into()),
        layout: vec![layout.clone()],
        shader_defs: vec![ShaderDefVal::UInt("_CHUNK_SIZE".into(), CHUNK_SIZE as u32)],
        entry_point: Some("main".into()),
        zero_initialize_workgroup_memory: false,
        immediate_size: todo!(),
        shader,
    });
    commands.insert_resource(ComputePipeline { layout, pipeline });
}

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone)] // RenderLabel
pub struct ComputeNodeLabel;

/// The node that will execute the compute shader
fn run_compute_node(
    render_context: &mut RenderContext,
    world: &World,
) {
    if world.get_resource::<FragCamera>().is_none() {
        info!("Couldn't get frag camera, skipping compute pass.");
        // return Ok(());
    }
    let pipeline_cache = world.resource::<PipelineCache>();
    let pipeline = world.resource::<ComputePipeline>();
    let bind_group = world.resource::<GpuShaderBufferBindGroup>();
    let camera = world.resource::<FragCamera>();
    if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
        let mut pass =
            render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("GPU readback compute pass"),
                    ..default()
                });

        pass.set_bind_group(0, &bind_group.0, &[]);
        pass.set_pipeline(init_pipeline);
        pass.dispatch_workgroups(
            camera.img_dims.x.div_ceil(8),
            camera.img_dims.y.div_ceil(8),
            1,
        );
    }
    // Ok(())
}

#[derive(Resource, ExtractResource, Clone)]
pub struct BeamReadbackBuffer {
    pub max_depth_buffer: Handle<ShaderBuffer>,
}

pub struct BeamGpuReadbackPlugin;
impl Plugin for BeamGpuReadbackPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(RenderStartup, init_beam_compute_pipeline)
            .add_systems(
                Render,
                (
                    (beam_prepare_bind_group)
                        .in_set(RenderSystems::PrepareBindGroups)
                        // We don't need to recreate the bind group every frame
                        .run_if(not(resource_exists::<BeamGpuBufferBindGroup>)),
                    resize_cameras.after(prepare_bind_group),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        // Add the compute node as a top level node to the render graph
        // This means it will only execute once per frame
        todo!()
        // render_app
        //     .world_mut()
        //     .resource_mut::<RenderGraph>()
        //     .add_node(BeamComputeNodeLabel, BeamComputeNode::default());
    }
}

#[derive(Resource)]
pub struct BeamGpuBufferBindGroup(pub BindGroup);

#[derive(Resource)]
pub struct BeamComputePipeline {
    layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

// Same conversion as `ComputePipeline`: `FromWorld` -> `RenderStartup` system.
fn init_beam_compute_pipeline(
    mut commands: Commands,
    render_device: RenderResource<RenderDevice>,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "Beam Bind group layout compute",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                storage_buffer::<Vec<u32>>(false),
                uniform_buffer::<FragCamera>(false),
                storage_buffer_read_only::<Vec<VoxelChunk>>(false),
                storage_buffer_read_only::<Vec<MapDataPacked>>(false),
                storage_buffer_read_only::<Vec<MapDataPacked>>(false),
                storage_buffer_read_only::<Vec<MapDataPacked>>(false),
                storage_buffer_read_only::<Vec<MapDataPacked>>(false),
            ),
        ),
    );
    let shader = asset_server.load("shaders/beam-compiled.wgsl");
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("Beam optimizer".into()),
        layout: vec![],
        // push_constant_ranges: vec![PushConstantRange {
        //     stages: ShaderStages::COMPUTE,
        //     range: 0..std::mem::size_of::<u32>() as u32,
        // }],
        shader,
        shader_defs: vec![ShaderDefVal::UInt("_CHUNK_SIZE".into(), CHUNK_SIZE as u32)],
        entry_point: Some("main".into()),
        zero_initialize_workgroup_memory: false,
        immediate_size: 0,
    });
    let layout = render_device.create_bind_group_layout();
    commands.insert_resource(BeamComputePipeline { layout, pipeline });
}

/// Label to identify the node in the render graph
#[derive(Debug, Hash, PartialEq, Eq, Clone)] // RenderLabel
pub struct BeamComputeNodeLabel;

/// The node that will execute the compute shader
fn run_beam_compute(
    _graph: &mut RenderContext,
    render_context: &mut RenderContext,
    world: &World,
) {
    if world.get_resource::<FragCamera>().is_none() {
        info!("Couldn't get frag camera, skipping compute pass.");
        // return Ok(());
    }
    let pipeline_cache = world.resource::<PipelineCache>();
    let pipeline = world.resource::<BeamComputePipeline>();
    let bind_group = world.resource::<BeamGpuBufferBindGroup>();
    let camera = world.resource::<FragCamera>();
    if let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) {
        // Two passes: i=1 (1/4 resolution), i=0 (1/2 resolution)
        for i in (0..=1u32).rev() {
            let mut pass =
                render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor {
                        label: Some("Beam optimizer"),
                        ..default()
                    });

            pass.set_bind_group(0, &bind_group.0, &[]);
            pass.set_pipeline(init_pipeline);
            // pass.set_push_constants(0, &i.to_le_bytes());
            let scale = 2u32 << i;
            let wg_x = (camera.img_dims.x / scale + 7) / 8;
            let wg_y = (camera.img_dims.y / scale + 7) / 8;
            pass.dispatch_workgroups(wg_x, wg_y, 1);
        }
    }
    // Ok(())
}
