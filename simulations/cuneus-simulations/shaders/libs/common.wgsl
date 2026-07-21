// Group 0: Per-Frame Data (Engine-Managed)
struct TimeUniform {
    time: f32,
    delta: f32,
    frame: u32,
    _padding: u32,
};

// Group 2: Global Engine Resources
struct MouseUniform {
    position: vec2<f32>,
    click: u32,
};


// Group 3: User Data (Particles)
struct Particle {
    old_pos: vec2<f32>, // TODO: Vec2_64 doesn't seem to add that much precision...
    pos: vec2<f32>, // TODO: Vec2_64 doesn't seem to add that much precision...
    mass: f32,
    enabled: u32,
    _pad1: u32,
    _pad2: u32,
};


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

// Returns a pseudo-rng inside [0;1]
fn rand(u: u32) -> f32 {
    return f32(hash(u)) / 4294967295.0;
}



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