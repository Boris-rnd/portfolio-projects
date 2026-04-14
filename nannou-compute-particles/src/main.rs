
use bytemuck::{Pod, Zeroable};
// include!("bevy.rs");
// include!("nannou.rs");
use nannou::prelude::*;
use wgpu::{BindGroup, Buffer, BufferUsages, CommandEncoderDescriptor, Maintain};

fn main() {
    nannou::app(model)
        .update(update)
        .run();
}
#[derive(Clone,Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Particle {
    pos: [f32; 2],
    vel: [f32; 2],
    color: [f32; 3],
    mass: f32,
}
impl Particle {
    pub fn rand() -> Self {
        use fastrand::*;
        Self { 
            pos: Vec2::new(f32()*800.-400.,f32()*600.-300.,).into(), 
            vel: Vec2::ZERO.into(),
            // vel: Vec2::new(f32(),f32(),).into(),
            color: [f32(),f32(),f32()],
            mass: f32()*10.,
        }
    }
    pub fn pos(pos: Vec2) -> Self {
        use fastrand::*;
        Self {
            pos: pos.into(),
            vel: Vec2::ZERO.into(),
            color: (f32(),f32(),f32()).into(),
            mass: f32()*10.,
        }
    }
    pub fn vel(&self) -> Vec2 {
        self.vel.into()
    }
    pub fn gpos(&self) -> Vec2 {
        self.pos.into()
    }
    pub const RAD: f32 = 2.5;
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Cell {
    pub inner_mass: f32,
    /// Indexes of particles in buffer
    pub parts: [u32; PARTS_PER_CELL],
    pub parts_len: u32,
}

struct Model {
    particles: Vec<Particle>,
    buffers: (Buffer, Buffer, Buffer),
    buffers_grid: (Buffer, Buffer),
    buffer_size: wgpu::BufferAddress,
    bind_groups: [BindGroup; 1],
    current_bind_group: usize,
    pipeline: wgpu::ComputePipeline,
}

fn model(app: &App) -> Model {
    let particles: [_; 300] = std::array::from_fn(|_| Particle::rand());
    let particles = particles.to_vec();
    let contents: &[u8] = bytemuck::cast_slice(&particles);
    let w_id = app.new_window().size(800, 600).view(view).build().unwrap();
    let window = app.window(w_id).unwrap();
    let device = window.device();
    let cs_desc = wgpu::include_wgsl!("../shaders/cs.wgsl");
    let cs_mod = device.create_shader_module(cs_desc);

    let buffer_size = contents.len() as wgpu::BufferAddress;
        
    let buffer = device.create_buffer_init(&wgpu::BufferInitDescriptor {
        label: Some("in"),
        usage: wgpu::BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        contents: &contents,
    });
    let buffer2 = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("out"),
        size: buffer_size,
        usage: wgpu::BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let buffer3 = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: buffer_size,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let grid = get_grid(&particles);
    let buffer_grid = device.create_buffer_init(&wgpu::BufferInitDescriptor {
        label: Some("grid"),
        usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
        contents: bytemuck::cast_slice(grid.as_ref()),
    });
    let buffer_grid_staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("buffer_grid_staging"),
        usage: BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC,
        size: buffer_grid.size(),
        mapped_at_creation: false,
    });
    let buffers = (buffer, buffer2, buffer3);
    let buffers_grid = (buffer_grid, buffer_grid_staging);
    let bind_group_layout = wgpu::BindGroupLayoutBuilder::new()
    .storage_buffer(
        wgpu::ShaderStages::COMPUTE,
        false,
        true,
    )
    .storage_buffer(
        wgpu::ShaderStages::COMPUTE,
        false,
        false,
    )
    .storage_buffer(
        wgpu::ShaderStages::COMPUTE,
        false,
        true,
    )
    .build(device);
    let bind_group = 
    wgpu::BindGroupBuilder::new()
        .buffer_bytes(&buffers.0, 0, None)
        .buffer_bytes(&buffers.1, 0, None)
        .buffer_bytes(&buffers_grid.0, 0, None)
        .build(device, &bind_group_layout);
    let bind_groups = [bind_group];
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("nannou"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });
    let desc = wgpu::ComputePipelineDescriptor {
        label: Some("nannou"),
        layout: Some(&pipeline_layout),
        module: &cs_mod,
        entry_point: "main",
    };
    let pipeline = device.create_compute_pipeline(&desc);

    Model {
        // particles: particles.to_vec()
        particles,
        buffers,
        buffer_size,
        bind_groups,
        pipeline,
        current_bind_group: 0,
        buffers_grid,
    }
}

pub const GRID_ROW: usize = 80;
pub const GRID_COL: usize = 60;
pub const PARTS_PER_CELL: usize = 256;

// Returns a box because stack size too small
fn get_grid(particles: &[Particle]) -> Box<[Cell; GRID_ROW*GRID_COL]> {
    let mut grid = Box::new([Cell {inner_mass: 0., parts: [0; PARTS_PER_CELL], parts_len: 0 }; GRID_ROW*GRID_COL]);
    for (i, p) in particles.iter().enumerate() {
        let (x,y) = (p.pos[0],p.pos[1]);
        let gx = (x/10.) as usize;
        let gy = (y/10.) as usize;
        let ix = GRID_ROW*gy+gx;
        grid[ix].inner_mass += p.mass;
        grid[ix].parts[grid[ix].parts_len as usize] = i.try_into().unwrap();
        grid[ix].parts_len += 1;
    }
    grid
}

fn update(app: &App, model: &mut Model, time: Update) {
    let window = app.main_window();
    let device = window.device();

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
    let pass_desc = wgpu::ComputePassDescriptor {
        label: Some("nannou-wgpu_compute_shader-compute_pass"),
    };
    {
        let mut cpass = encoder.begin_compute_pass(&pass_desc);
        cpass.set_pipeline(&model.pipeline);
        cpass.set_bind_group(0, &model.bind_groups[model.current_bind_group], &[]);
        cpass.dispatch_workgroups(model.particles.len() as u32+1, 1, 1);
    }
    // Copy out buffer to staging buffer, to read and update particle positions
    encoder.copy_buffer_to_buffer(&model.buffers.1, 0, &model.buffers.2, 0, model.buffers.1.size());
    // Copy out staging buffer to in buffer, so that compute shader has the updated info
    encoder.copy_buffer_to_buffer(&model.buffers.1, 0, &model.buffers.0, 0, model.buffers.1.size());
    // Copy grid buffer to staging grid
    window.queue().submit(Some(encoder.finish()));

    let (snd,rec) = flume::bounded(1);
    model.buffers.2.slice(..).map_async(wgpu::MapMode::Read, move |res| {
        res.unwrap();
        snd.send(true).unwrap();
    });
    device.poll(Maintain::Wait);
    rec.recv().unwrap();
    
    {let range = model.buffers.2.slice(..).get_mapped_range();
    const STEP: usize = std::mem::size_of::<Particle>();
    let mut i = 0;
    while i*STEP < range.len() {
        let mut raw_part: [u8; STEP] = [0; STEP];
        raw_part.copy_from_slice(&range[i*STEP..(i+1)*STEP]);
        let particle: Particle = bytemuck::cast(raw_part);
        model.particles[i] = particle;
        i += 1;
    }}
    model.buffers.2.unmap();
    
    let grid = get_grid(&model.particles);
    let (snd,rec) = flume::bounded(1);
    model.buffers_grid.1.slice(..).map_async(wgpu::MapMode::Write, move |res| {
        res.unwrap();
        snd.send(true).unwrap();
    });
    device.poll(Maintain::Wait);
    rec.recv().unwrap();
    {
        let mut range = model.buffers_grid.1.slice(..).get_mapped_range_mut();
        range.copy_from_slice(bytemuck::cast_slice(grid.as_ref()));
    }

    model.buffers_grid.1.unmap();
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
    // Copy the new grid to the storage grid buffer
    encoder.copy_buffer_to_buffer(&model.buffers_grid.1, 0, &model.buffers_grid.0, 0, model.buffers_grid.1.size());
    device.poll(Maintain::Wait);

}


fn view(app: &App, model: &Model, frame: Frame){
    let draw = app.draw();
    draw.background().color(PLUM);
    for p in &model.particles {
        draw.ellipse().radius(Particle::RAD).xy(p.gpos()).color(rgb(p.color[0], p.color[1], p.color[2]));
    }
    draw.to_frame(app, &frame).unwrap();

}
