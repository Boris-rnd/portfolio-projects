struct Particle {
    pos: vec3<f32>,
}

@group(0) @binding(0)
var<storage, read> particles_in: array<Particle>;
@group(0) @binding(1)
var<storage, read_write> particles_out: array<Particle>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    particles_out[global_id.x] = particles_in[global_id.x];
}