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
    h: f32,
    rest_density: f32,
    k: f32,
    drag: f32,
    viscosity: f32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
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
    pos: vec2<f32>,
    vel: vec2<f32>,
    mass: f32,
    disabled: u32,
    density: f32,
    pressure: f32,
};

const PI: f32 = 3.14159265358979323846;

@group(3) @binding(0) var<storage, read_write> particles: array<Particle>;


@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.particle_count) {
        return;
    }

    var p = particles[i];
    if p.disabled == 1 {return;}

    if (params.reset > 0u || time_data.frame == 0u) {
        // Initialize or reset particle
        p.pos = vec2<f32>(rand(i * 2u), rand(i * 2u + 1u));
        p.vel = vec2<f32>((rand(i * 3u) - 0.5) * 0.01, (rand(i * 3u + 1u) - 0.5) * 0.01);
        p.mass = 1.0;
    } else {
        // Apply gravity
        p.vel.y += params.gravity * params.speed * time_data.delta;
        let h = params.h;
        let kernel_coef = 4.0/(PI * pow(h, 8));
        var density = 0.;
        for (var j = 0u; j < params.particle_count; j++) {
            let p2 = particles[j];
            if p2.disabled == 1 {continue;}
            let dst = p.pos - p2.pos;
            let dst_sq = dot(dst, dst);
            if (dst_sq < h*h) {
                density += p2.mass * kernel_coef * pow(h*h - dst_sq, 3);
            }
        }
        p.density = density;
        p.pressure = params.k * (density - params.rest_density);

        // Spiky kernel derivative 
        let spiky_kernel_coef = -30.0 / (PI * pow(h, 5));
        var pressure_force = vec2<f32>(0.0);
        for (var j = 0u; j < params.particle_count; j++) {
            if (i == j) { continue; }
            let p2 = particles[j];
            if p2.disabled == 1 {continue;}
            let dst = p.pos - p2.pos;
            let dst_sq = dot(dst, dst);
            if (dst_sq < h*h && dst_sq > 0.0001) {
                let slope = spiky_kernel_coef * pow(h - sqrt(dst_sq), 2);
                let shared_pressure = (p.pressure+p2.pressure)*0.5;
                pressure_force += dst * shared_pressure * p2.mass / (p2.density) * slope;
            }
        }
        if (p.density > 0.) {
            p.vel += (pressure_force / p.density) * time_data.delta * params.speed;
        }

        // Viscosity
        let viscosity_kernel_coef = 45.0 / (PI * pow(h, 6));
        var viscosity_force = vec2<f32>(0.);
        for (var j = 0u; j < params.particle_count; j++) {
            if (i == j) { continue; }
            let p2 = particles[j];
            if p2.disabled == 1 {continue;}
            let dst = p.pos - p2.pos;
            let dst_sq = dot(dst, dst);
            if (dst_sq < h*h) {
                let r = sqrt(dst_sq);
                let slope = viscosity_kernel_coef * (h - r);
                viscosity_force += (p2.vel - p.vel) * p2.mass / p2.density * slope;
            }
        }
        if (p.density != 0.) {
            p.vel += viscosity_force * time_data.delta*params.speed / p.density * params.viscosity;
        }

        // Apply gravity
        // p.vel.y += params.gravity * params.speed * time_data.delta;
        // for (var j = 0u; j < params.particle_count; j++) {
        //     let p2 = particles[j];
        //     if p2.disabled == 1 {continue;}
        //     let dst = p.pos - p2.pos;
        //     let dst_sq = dot(dst, dst);
        //     if (dst_sq > 0.00001) {
        //         let force = params.gravity * params.speed * time_data.delta / (dst_sq);
        //         p.vel -= force * dst * p2.mass / p.mass;
        //     } 
        // }

        // Apply pressure from screen bounds
        if (p.pos.x < 0.0) {
            // p.vel.x += params.gravity * params.speed * time_data.delta;
            p.vel.x *= -0.2;
            p.pos.x = 0.0;
        }
        if (p.pos.x > 1.0) {
            // p.vel.x -= params.gravity * params.speed * time_data.delta;
            p.vel.x *= -0.2;
            p.pos.x = 1.0;
        }
        if (p.pos.y < 0.0) {
            // p.vel.y += params.gravity * params.speed * time_data.delta;
            p.vel.y *= -0.2;
            p.pos.y = 0.0;
        }
        if (p.pos.y > 1.0) {
            // p.vel.y -= params.gravity * params.speed * time_data.delta;
            p.vel.y *= -0.2;
            p.pos.y = 1.0;
        }
        
        p.vel *= 1.0 - params.drag;
        p.pos += p.vel * params.speed * time_data.delta;
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


@compute @workgroup_size(100)
fn splat(@builtin(global_invocation_id) id: vec3<u32>) {
    let thread_id = id.x;
    if (thread_id >= 100u) {
        return;
    }

    let particles_per_thread = 200u; // 100 threads * 200 = 20,000 particles
    let start_idx = thread_id * particles_per_thread;
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
