// Group 3: User Data (Particles)
struct Particle {
    old_pos: vec2<f32>, // TODO: Vec2_64 doesn't seem to add that much precision...
    pos: vec2<f32>, // TODO: Vec2_64 doesn't seem to add that much precision...
    mass: f32,
    enabled: u32,
    _pad1: u32,
    _pad2: u32,
};



fn world_to_screen_pos(world_pos: vec2<f32>) -> vec2<u32> {
    return vec2<u32>((world_pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y)) * vec2<f32>(textureDimensions(output)) * params.camera_zoom);
}

fn screen_to_world_pos(screen_pos: vec2<u32>) -> vec2<f32> {
    return vec2<f32>(((vec2<f32>(screen_pos)/params.camera_zoom)/vec2<f32>(textureDimensions(output)) + vec2<f32>(params.camera_pos_x, params.camera_pos_y)));
}

fn clear_screen_at(screen_pos: vec2<u32>) {
    let dims = textureDimensions(output);
    if (screen_pos.x >= dims.x || screen_pos.y >= dims.y) {
        return;
    }
    let pos_px = vec2<u32>(screen_pos.xy);
    textureStore(output, pos_px, vec4<f32>(vec3(0.0), 1.));

    let world_pos = screen_to_world_pos(pos_px);

    let idx = pos_px_to_buffer_idx(pos_px);
    let col = vec3<f32>(f32(atomicLoad(&screen_atomic_buffer[idx]))/10.);
    textureStore(output, pos_px, vec4<f32>(col, 1.));
    atomicStore(&screen_atomic_buffer[idx], 0);

    if abs(dot(world_pos,world_pos)-1.)<=0.01 {
        textureStore(output, pos_px, vec4<f32>(vec3(1.0), 1.));
    }
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
