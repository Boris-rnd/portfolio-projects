include!(libs/common.wgsl);
@group(0) @binding(0) var<uniform> time_data: TimeUniform;
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
@group(2) @binding(0) var<uniform> mouse: MouseUniform;
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

fn verlet_integration(p2: Particle, acc: vec2<f32>, dt: f32) -> Particle {
    var p = p2;
    let temp = p.pos;
    p.pos = 2.0 * p.pos - p.old_pos + acc * dt * dt;
    p.old_pos = temp;

    if dot(p.pos,p.pos) >= 1.0 {
        let dist = length(p.pos);
        let normal = p.pos / dist;

        p.pos = normal * 1.0;

        // Bounce
        // e = 1.0 (Perfect) | e < 1.0 (Damped) | e > 1.0 (Boosted)
        let e = 1.;
        let vel = p.pos - p.old_pos;
        let v_normal = dot(vel, normal) * normal;
        let v_tangent = vel - v_normal;
        let reflected_vel = v_tangent - e * v_normal;

        // Change old pos to create virtual vel
        p.old_pos = p.pos - reflected_vel;
    }
    return p;
}
fn euler_integration(p2: Particle, acc: vec2<f32>, dt: f32) -> Particle {
    var p = p2;
    p.old_pos += acc * dt;
    p.pos += p.old_pos * dt;

    // 2. Gestion de la contrainte (Cercle de rayon 1.0)
    let dist = length(p.pos);
    if dist >= 1.0 {
        let normal = p.pos / dist;

        // Repositionne sur le bord
        p.pos = normal * 1.0;

        // Inverse la composante normale de la vitesse (e = coefficient de rebond)
        let e = 1.;
        p.old_pos = p.old_pos - (1.0 + e) * dot(p.old_pos, normal) * normal;
    }
    return p;
}

// fn euler_integration64(p2: Particle, acc_f32: vec2<f32>, dt_f32: f32) -> Particle {
//     var p = p2;
//     let dt = f64_(dt_f32);
//     let acc = Vec2_64(f64_(acc_f32.x), f64_(acc_f32.y));

//     let acc_dt = vec2_64_mul_s(acc, dt);
//     p.old_pos = vec2_64_add(p.old_pos, acc_dt);

//     let vel_dt = vec2_64_mul_s(p.old_pos, dt);
//     p.pos = vec2_64_add(p.pos, vel_dt);

//     let dist = vec2_64_len(p.pos);
//     if ge_64(dist, F64_ONE) {
//         let normal = vec2_64_div_s(p.pos, dist);
//         p.pos = normal; // normal * 1.0

//         let e = F64_ONE;
//         let one_plus_e = sum64(F64_ONE, e); // 2.0
//         let d = vec2_64_dot(p.old_pos, normal);
//         let factor = mul64(one_plus_e, d);
//         let corr = vec2_64_mul_s(normal, factor);
//         p.old_pos = vec2_64_sub(p.old_pos, corr);
//     }
//     return p;
// }


@compute @workgroup_size(16, 16)
fn clear_screen(@builtin(global_invocation_id) id: vec3<u32>) {
    return clear_screen_at(id.xy);
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
    // let pos = p.pos;
    let pos = vec2<f32>(fp64_to_f32(fp64_(p.pos.x)),fp64_to_f32(fp64_(p.pos.y)));
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

// @compute @workgroup_size(16, 16)
// fn render(@builtin(global_invocation_id) id: vec3<u32>) {
//     let dims = textureDimensions(output);
//     if (id.x >= dims.x || id.y >= dims.y) {
//         return;
//     }

//     let idx = id.y * dims.x + id.x;
//     let count = atomicLoad(&atomic_buffer[idx]);

//     // Background color
//     var col = vec4<f32>(0.00, 0.00, 0.0, 1.0);

//     let ps = i32(params.particle_size);
//     let half_size = ps/2;
//     if (count > 0u) {
//         let intensity = min(f32(count) * 0.4, 1.0);
//         col += vec4<f32>(1.0, 1.0, 1.0, 1.0) * intensity;
//         for (var y = -half_size; y < half_size; y++) {
//             for (var x = -half_size; x < half_size; x++) {
//                 let current = vec2<i32>(id.xy) + vec2(x, y);
//                 textureStore(output, current, col);
//                 // let dst = id.xy-current;
//                 // if dst.x < u32(dims.x) && dst.y < u32(dims.y) {
//                 // }
//             }
//         }
//     }

//     // textureStore(output, id.xy, col);
// }
