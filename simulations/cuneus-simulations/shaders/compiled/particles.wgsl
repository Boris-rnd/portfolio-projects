struct TimeUniform {
    time: f32,
    delta: f32,
    frame: u32,
    _padding: u32
}

@group(0) @binding(0)
var<uniform> time_data: TimeUniform;

@group(1) @binding(0)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    gravity: f32,
    particle_size: u32,
    particle_count: u32,
    grid_count_x: u32,
    grid_count_y: u32,
    speed: f32,
    reset: u32,
    camera_pos_x: f32,
    camera_pos_y: f32,
    camera_zoom: f32,
    _pad0: u32,
    _pad1: u32
}

@group(1) @binding(1)
var<uniform> params: Params;

@group(2) @binding(1)
var<storage, read_write> particles_atomic_buffer: array<atomic<u32>>;

struct Particle {
    old_pos: vec2<f32>,
    pos: vec2<f32>,
    mass: f32,
    enabled: u32,
    _pad1: u32,
    _pad2: u32
}

@group(3) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(3) @binding(1)
var<storage, read_write> particles_grid: array<ParticleGrid>;

fn hash(u: u32) -> u32 {
    var v = u;
    v = v ^ (v >> 16u);
    v = v * 73244475u;
    v = v ^ (v >> 16u);
    v = v * 73244475u;
    v = v ^ (v >> 16u);
    return v;
}

fn rand(u: u32) -> f32 {
    return f32(hash(u)) / 4294967295.0;
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    if params.speed == 0.0 {
        return;
    }
    let i = id.x;
    if (i >= params.particle_count) {
        return;
    }
    var p = particles[i];
    if (params.reset > 0u || time_data.frame == 0u) {
        p.pos = vec2<f32>(rand(i * 2u), rand(i * 2u + 1u));
        p.old_pos = p.pos;
        p.enabled = 1;
        p.mass = 1.0;
    }
    else {
        if p.enabled == 0 {
            return;
        }
        var total_acc = vec2<f32>(0.0);
        for (var j = 0u; j < params.particle_count; j++) {
            if (i == j) {
                continue;
            }
            let p2 = particles[j];
            if (p2.enabled == 0u) {
                continue;
            }
            let diff = p2.pos - p.pos;
            let dst_sq = max(dot(diff, diff), 0.001);
            let force_mag = (params.gravity * p2.mass) / dst_sq;
            total_acc += normalize(diff) * force_mag;
        }
        let max_accel = 1000.0;
        if (length(total_acc) > max_accel) {
            total_acc = normalize(total_acc) * max_accel;
        }
        let dt = time_data.delta * params.speed;
        let temp = p.pos;
        p.pos = 2.0 * p.pos - p.old_pos + total_acc * dt * dt;
        p.old_pos = temp;
    }
    particles[i] = p;
}

struct ParticleGrid {
    inner_mass: f32,
    particle_count: u32,
    holding_particles: array<vec4<u32>, 25>,
    pad: array<u32, 2>
}

fn grid_pos_to_idx(pos: vec2<u32>) -> u32 {
    return u32(pos.x + pos.y * (params.grid_count_x));
}

fn grid_idx_to_grid_pos(idx: u32) -> vec2<u32> {
    return vec2(idx % params.grid_count_x, idx / params.grid_count_x);
}

fn lock_grid(grid_id: u32) -> bool {
    if atomicLoad(&particles_atomic_buffer[grid_id]) == 1u {
        return false;
    }
    atomicStore(&particles_atomic_buffer[grid_id], 1u);
    return true;
}

const MAX_RETRIES_BLOCKING: u32 = 500;

fn lock_grid_blocking(grid_id: u32) -> bool {
    var i = MAX_RETRIES_BLOCKING;
    while (i > 0) {
        if lock_grid(grid_id) == true {
            return true;
        }
        var delay = 100u;
        while (delay > 0u) {
            delay--;
        }
        i--;
    }
    return false;
}

fn unlock_grid(grid_id: u32) -> bool {
    if atomicLoad(&particles_atomic_buffer[grid_id]) == 0u {
        return false;
    }
    atomicStore(&particles_atomic_buffer[grid_id], 0u);
    return true;
}

@compute @workgroup_size(64)
fn update_grids(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output);
    let grid_size = vec2(f32(dims.x) / f32(params.grid_count_x), f32(dims.y) / f32(params.grid_count_y));
    let i = id.x;
    if i >= params.particle_count {
        return;
    }
    let p = particles[i];
    if p.enabled == 0 {
        return;
    }
    let grid_idx = grid_pos_to_idx(vec2<u32>(p.pos / (grid_size / 800.0)));
    lock_grid_blocking(grid_idx);
    var cell = particles_grid[grid_idx];
    if cell.particle_count >= 100 {
        return;
    }
    particles_grid[grid_idx].particle_count += 1;
    unlock_grid(grid_idx);
}

@compute @workgroup_size(16, 16)
fn debug_grids(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let pos_px = vec2<u32>(id.xy);
    let grid_size = vec2(f32(dims.x) / f32(params.grid_count_x), f32(dims.y) / f32(params.grid_count_y));
    let grid_pos = vec2<u32>(vec2<f32>(pos_px) / grid_size);
    let grid_idx = grid_pos_to_idx(grid_pos);
    let grid_ip = grid_idx_to_grid_pos(grid_idx);
    textureStore(output, pos_px, vec4<f32>(f32(grid_ip.x) / 128.0, f32(grid_ip.y) / 128.0, min(f32(particles_grid[grid_idx].particle_count) / 100.0, 0.5), 1.0));
}

@compute @workgroup_size(64)
fn reset_grids(@builtin(global_invocation_id) id: vec3<u32>) {
    let grid_id = id.x;
    if grid_id >= params.grid_count_x * params.grid_count_y {
        return;
    }
    particles_grid[grid_id].particle_count = 0;
    particles_grid[grid_id].inner_mass = 0.0;
    unlock_grid(grid_id);
}

@compute @workgroup_size(16, 16)
fn clear_atomics(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let pos_px = vec2<u32>(id.xy);
    textureStore(output, pos_px, vec4<f32>(vec3(0.0), 1.0));
}

@compute @workgroup_size(64)
fn splat(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.particle_count) {
        return;
    }
    var p = particles[i];
    if p.enabled == 0 {
        return;
    }
    render_particle(p);
}

fn render_particle(p: Particle) {
    let dims = textureDimensions(output);
    let pos_px = vec2<u32>((p.pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y)) * vec2<f32>(dims) * params.camera_zoom);
    let ps = i32(params.particle_size);
    for (var y = -ps + 1; y < ps; y++) {
        for (var x = -ps + 1; x < ps; x++) {
            let disp = vec2(x, y);
            let current = vec2<i32>(pos_px.xy) + disp;
            if (current.x >= 0i && current.y >= 0i && u32(current.x) < dims.x && u32(current.y) < dims.y) {
                let dst_sq = f32(disp.x * disp.x + disp.y * disp.y);
                let normalized_dst = dst_sq / f32(ps * ps);
                let intensity = vec3(f32(1.0));
                if normalized_dst >= 0.5 {
                    continue;
                }
                let color = vec3(vec2<f32>(pos_px) / vec2<f32>(dims), f32(p.mass));
                textureStore(output, current, vec4<f32>(color, 1.0));
            }
        }
    }
}
