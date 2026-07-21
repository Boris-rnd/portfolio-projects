@group(0) @binding(0)
var<uniform> time_data: package_libs_common_TimeUniform;

@group(1) @binding(0)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    gravity: f32,
    particle_size: u32,
    particle_count: u32,
    speed: f32,
    reset: u32,
    camera_pos_x: f32,
    camera_pos_y: f32,
    camera_zoom: f32
}

@group(1) @binding(1)
var<uniform> params: Params;

@group(2) @binding(1)
var<storage, read_write> screen_atomic_buffer: array<atomic<u32>>;

@group(3) @binding(0)
var<storage, read_write> particles: array<package_libs_common_Particle>;

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
        p.pos = vec2<f32>((0.45) + (f32(i)) / (f32(params.particle_count) / 0.15), 0.0);
        p.old_pos = p.pos;
        p.enabled = 1;
        p.mass = 1.0;
    }
    else {
        if p.enabled == 0 {
            return;
        }
        let dt = time_data.delta * params.speed;
        var acc = vec2<f32>(0.0);
        acc.y += params.gravity;
        p = euler_integration(p, acc, dt);
    }
    particles[i] = p;
    count_particle(p);
}

fn pos_px_to_buffer_idx(pos_px: vec2<u32>) -> u32 {
    return pos_px.x + pos_px.y * textureDimensions(output).x;
}

fn count_particle(p: package_libs_common_Particle) {
    let pos_px = world_to_screen_pos(p.pos);
    let idx = pos_px_to_buffer_idx(pos_px);
    atomicAdd(&screen_atomic_buffer[idx], 1u);
}

fn euler_integration(p2: package_libs_common_Particle, acc: vec2<f32>, dt: f32) -> package_libs_common_Particle {
    var p = p2;
    p.old_pos += acc * dt;
    p.pos += p.old_pos * dt;
    let dist = length(p.pos);
    if dist >= 1.0 {
        let normal = p.pos / dist;
        p.pos = normal * 1.0;
        let e = 1.0;
        p.old_pos = p.old_pos - (1.0 + e) * dot(p.old_pos, normal) * normal;
    }
    return p;
}

@compute @workgroup_size(16, 16)
fn clear_screen(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let pos_px = vec2<u32>(id.xy);
    textureStore(output, pos_px, vec4<f32>(vec3(0.0), 1.0));
    let world_pos = screen_to_world_pos(pos_px);
    let idx = pos_px_to_buffer_idx(pos_px);
    let col = vec3<f32>(f32(atomicLoad(&screen_atomic_buffer[idx])) / 10.0);
    textureStore(output, pos_px, vec4<f32>(col, 1.0));
    atomicStore(&screen_atomic_buffer[idx], 0);
    if abs(dot(world_pos, world_pos) - 1.0) <= 0.01 {
        textureStore(output, pos_px, vec4<f32>(vec3(1.0), 1.0));
    }
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

fn world_to_screen_pos(world_pos: vec2<f32>) -> vec2<u32> {
    return vec2<u32>((world_pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y)) * vec2<f32>(textureDimensions(output)) * params.camera_zoom);
}

fn screen_to_world_pos(screen_pos: vec2<u32>) -> vec2<f32> {
    return vec2<f32>(((vec2<f32>(screen_pos) / params.camera_zoom) / vec2<f32>(textureDimensions(output)) + vec2<f32>(params.camera_pos_x, params.camera_pos_y)));
}

fn render_particle(p: package_libs_common_Particle) {
    let dims = textureDimensions(output);
    let pos = vec2<f32>(package_libs_fp64__2fp64_to_f32(package_libs_fp64__1fp64_(p.pos.x)), package_libs_fp64__2fp64_to_f32(package_libs_fp64__1fp64_(p.pos.y)));
    let pos_px = world_to_screen_pos(pos);
    let ps = i32(params.particle_size);
    for (var y = -ps + 1; y < ps; y++) {
        for (var x = -ps + 1; x < ps; x++) {
            let disp = vec2(x, y);
            let current = vec2<i32>(pos_px.xy) + disp;
            if (current.x >= 0i && current.y >= 0i && u32(current.x) < dims.x && u32(current.y) < dims.y) {
                let dst_sq = f32(dot(disp, disp));
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

struct package_libs_common_TimeUniform {
    time: f32,
    delta: f32,
    frame: u32,
    _padding: u32
}

struct package_libs_common_Particle {
    old_pos: vec2<f32>,
    pos: vec2<f32>,
    mass: f32,
    enabled: u32,
    _pad1: u32,
    _pad2: u32
}

const package_libs_fp64__1one_f32: f32 = 1.0;

struct package_libs_fp64_fp64 {
    high: f32,
    low: f32
}

fn package_libs_fp64_split64(a: f32) -> package_libs_fp64_fp64 {
    let c = (f32(1u << 12u) + 1.0) * a;
    let a_big = c - a;
    let a_hi = c * package_libs_fp64__1one_f32 - a_big;
    let a_lo = a * package_libs_fp64__1one_f32 - a_hi;
    return package_libs_fp64_fp64(a_hi, a_lo);
}

fn package_libs_fp64__1fp64_(a: f32) -> package_libs_fp64_fp64 {
    return package_libs_fp64_split64(a);
}

fn package_libs_fp64__2fp64_to_f32(a: package_libs_fp64_fp64) -> f32 {
    return a.high + a.low;
}
