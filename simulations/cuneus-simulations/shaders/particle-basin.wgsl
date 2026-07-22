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
include!("libs/common.wgsl");
include!("libs/utils.wgsl");
include!("libs/fp64.wgsl");

@group(2) @binding(1) var<storage, read_write> screen_atomic_buffer: array<atomic<u32>>;
@group(3) @binding(0) var<storage, read_write> particles: array<Particle>;
// @group(3) @binding(1) var<storage, read_write> particles_grid: array<ParticleGrid>;


@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    if params.speed==0. {return;}
    let i = id.x;
    if (i >= params.particle_count) {
        return;
    }

    var p = particles[i];

    if (params.reset > 0u || time_data.frame == 0u) {
        p.pos = vec2<f32>((0.45)+(f32(i))/(f32(params.particle_count)/0.15), 0.);
        // let x = sum64(f64_(0.45), div64(f64_(f32(i)), div64(f64_(f32(params.particle_count)), f64_(0.15))));
        // p.pos = Vec2_64(x, f64_(0.));
        p.old_pos = p.pos; // vel=0
        p.enabled = 1;
        p.mass = 1.0;
    } else {
        if p.enabled == 0 {return;}
        let dt = time_data.delta * params.speed;

        var acc = vec2<f32>(0.0);
        acc.y += params.gravity;
        // p = verlet_integration(p, acc, dt);
        p = euler_integration(p, acc, dt);

    }
    particles[i] = p;

    count_particle(p);
}

fn pos_px_to_buffer_idx(pos_px: vec2<u32>) -> u32 {
    return pos_px.x+pos_px.y*textureDimensions(output).x;
}
fn count_particle(p: Particle) {
    let pos_px = world_to_screen_pos(p.pos);
    let idx = pos_px_to_buffer_idx(pos_px);
    atomicAdd(&screen_atomic_buffer[idx], 1u);
}


@compute @workgroup_size(16, 16)
fn clear_screen(@builtin(global_invocation_id) id: vec3<u32>) {
    clear_screen_at(id.xy);
}

@compute @workgroup_size(64)
fn splat(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.particle_count) {return;}

    var p = particles[i];
    if p.enabled == 0 {return;}
    render_particle(p);
}

fn render_particle(p: Particle) {
    let dims = textureDimensions(output);
    let pos = p.pos;
    // let pos = vec2<f32>(fp64_to_f32(fp64_(p.pos.x)),fp64_to_f32(fp64_(p.pos.y)));
    let pos_px = world_to_screen_pos(pos);

    let ps = i32(params.particle_size);
    // let half_size = i32(p.mass/1000.);
    // textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(p.mass/100., 0., 1., 1.));

    for (var y = -ps+1; y < ps; y++) {
        for (var x = -ps+1; x < ps; x++) {
            let disp = vec2(x, y);
            let current = vec2<i32>(pos_px.xy) + disp;
            if (current.x >= 0i && current.y >= 0i && u32(current.x) < dims.x && u32(current.y) < dims.y) {
                let dst_sq = f32(dot(disp,disp));
                let normalized_dst = dst_sq / f32(ps*ps);
                let intensity = vec3(f32(1.0)); //  - normalized_dst
                // let prev_intensity = textureLoad(output, current).xyz;
                if normalized_dst >= 0.5 {continue;}
                let color = vec3(vec2<f32>(pos_px)/vec2<f32>(dims), f32(p.mass));

                textureStore(output, current, vec4<f32>(color, 1.0));
            }
        }
    }
}

