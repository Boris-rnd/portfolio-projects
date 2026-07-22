@group(0) @binding(0) var<uniform> time_data: TimeUniform;
@group(1) @binding(0) var output: texture_storage_2d<rgba16float, write>;
@group(1) @binding(1) var<uniform> params: Params;
@group(2) @binding(0) var<uniform> mouse: MouseUniform;

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
fn hash_rand(u: u32) -> f32 {
    return f32(hash(u)) / 4294967295.0;
}
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let cutoff = vec3<f32>(0.04045);
    let below = c / 12.92;
    let above = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return mix(above, below, cutoff);
}
