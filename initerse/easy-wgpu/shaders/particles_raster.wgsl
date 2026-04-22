struct Particle {
    pos: vec3<f32>,
}

@group(0) @binding(0)
var<storage, read> particles_in: array<Particle>;
@group(0) @binding(1)
var texture_out: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var c = vec4(1.);
    var full_byte = u32(0xFF);
    for(var i = global_id.x; i < 40000; i++) {
        textureStore(texture_out, vec2<u32>(i%800, i/800%600), vec4<f32>(1.0));
    // u32(c.r*255.) & full_byte |
    // u32(c.g*255.) & full_byte<<8 |
    // u32(c.b*255.) & full_byte<<16 |
    // u32(c.a*255.) & full_byte<<24
    // ;
    }
}