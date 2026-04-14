@group(0) @binding(0)
var<storage, read_write> buffer: array<f32, 256>;

@compute @workgroup_size(256)
fn square(@builtin(global_invocation_id) global_id: vec3<u32>) {
	buffer[global_id.x] = buffer[global_id.x]*buffer[global_id.x];
}