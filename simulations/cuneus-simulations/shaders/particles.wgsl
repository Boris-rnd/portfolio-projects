// Group 0: Per-Frame Data (Engine-Managed)
struct TimeUniform {
    time: f32,
    delta: f32,
    frame: u32,
    _padding: u32,
};
@group(0) @binding(0) var<uniform> time_data: TimeUniform;

// Group 1: Primary Pass I/O & Custom Parameters
@group(1) @binding(0) var output: texture_storage_2d<rgba16float, write>;

struct Params {
    gravity: f32,
    particle_size: u32,
    particle_count: u32,
    speed: f32,
    reset: u32,
    camera_pos_x: f32,
    camera_pos_y: f32,
    camera_zoom: f32,
    // _pad0: u32,
    // _pad1: u32,
    // _pad2: u32,
};
@group(1) @binding(1) var<uniform> params: Params;

// Group 2: Global Engine Resources
struct MouseUniform {
    position: vec2<f32>,
    click: u32,
};
@group(2) @binding(0) var<uniform> mouse: MouseUniform;
@group(2) @binding(1) var<storage, read_write> atomic_buffer: array<atomic<u32>>;

// Group 3: User Data (Particles)
struct Particle {
    old_pos: vec2<f32>,
    pos: vec2<f32>,
    mass: f32,
    disabled: u32,
    _pad1: u32,
    _pad2: u32,
};

struct ParticleGrid {
    inner_mass: f32,
}

@group(3) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(3) @binding(1) var<storage, read_write> particles_grid: array<ParticleGrid>;


// Utility function for random values
fn hash(u: u32) -> u32 {
    var v = u;
    v = v ^ (v >> 16u);
    v = v * 0x45d9f3bu;
    v = v ^ (v >> 16u);
    v = v * 0x45d9f3bu;
    v = v ^ (v >> 16u);
    return v;
}

fn rand(u: u32) -> f32 {
    return f32(hash(u)) / 4294967295.0;
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.particle_count) {
        return;
    }

    var p = particles[i];
    if p.disabled == 1 {return;}

    if (params.reset > 0u || time_data.frame == 0u) {
        p.pos = vec2<f32>(rand(i * 2u), rand(i * 2u + 1u));
        p.old_pos = p.pos; // vel=0
        p.mass = 1.0;
    } else {
        var total_acc = vec2<f32>(0.0);

        for (var j = 0u; j < params.particle_count; j++) {
            if (i == j) { continue; }
            
            let p2 = particles[j];
            if (p2.disabled == 1u) { continue; }

            let diff = p2.pos - p.pos;
            let dst_sq = max(dot(diff, diff), 0.001);
            // TODO params.gravity can go in p.pos update
            let force_mag = (params.gravity * p2.mass) / dst_sq;
            total_acc += normalize(diff) * force_mag;
        }

        // Clamp acceleration to prevent massive jumps in a single frame
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

@compute @workgroup_size(16, 16)
fn clear_atomics(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    let idx = id.y * dims.x + id.x;
    let pos_px = vec2<u32>(id.xy);
    if (pos_px.x < u32(dims.x) && pos_px.y < u32(dims.y)) {
        textureStore(output, pos_px, vec4<f32>(0.0));
    }
}

@compute @workgroup_size(64)
fn splat(@builtin(global_invocation_id) id: vec3<u32>) {
    let particles_per_thread = params.particle_count/(64u*64u) + 1u;
    let start_idx = id.x * particles_per_thread;
    let dims = textureDimensions(output);

    for (var j = 0u; j < particles_per_thread; j++) {
        let i = start_idx + j;
        if (i >= params.particle_count) {
            break;
        }

        let p = particles[i];
        if p.disabled == 1 {continue;}
        let pos_px = vec2<u32>((p.pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y)) * vec2<f32>(dims) * params.camera_zoom);

        if (pos_px.x < dims.x && pos_px.y < dims.y) {
                let ps = i32(params.particle_size);
                let half_size = i32(p.mass/1000.);
                textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(p.mass/100., 0., 1., 1.));

                for (var y = -half_size; y < half_size; y++) {
                    for (var x = -half_size; x < half_size; x++) {
                        let disp = vec2(x, y);
                        let current = vec2<i32>(pos_px.xy) + disp;
                        if (current.x >= 0i && current.y >= 0i && u32(current.x) < dims.x && u32(current.y) < dims.y) {
                            let intensity = f32(1.0) - f32(disp.x*disp.x + disp.y*disp.y) / f32(half_size*half_size)*1.0;
                            textureStore(output, current, vec4<f32>(intensity, intensity, intensity, 1.0));
                        }
                    }
                }
        }
    }
}

@compute @workgroup_size(16, 16)
fn render(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }

    let idx = id.y * dims.x + id.x;
    let count = atomicLoad(&atomic_buffer[idx]);

    // Background color
    var col = vec4<f32>(0.00, 0.00, 0.0, 1.0);

    let ps = i32(params.particle_size);
    let half_size = ps/2;
    if (count > 0u) {
        let intensity = min(f32(count) * 0.4, 1.0);
        col += vec4<f32>(1.0, 1.0, 1.0, 1.0) * intensity;
        for (var y = -half_size; y < half_size; y++) {
            for (var x = -half_size; x < half_size; x++) {
                let current = vec2<i32>(id.xy) + vec2(x, y);
                textureStore(output, current, col);
                // let dst = id.xy-current;
                // if dst.x < u32(dims.x) && dst.y < u32(dims.y) {
                // }
            }
        }
    }

    // textureStore(output, id.xy, col);
}
