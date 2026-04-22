#[compute]
#version 450

// Invocations in the (x, y, z) dimension
layout(local_size_x = 2, local_size_y = 1, local_size_z = 1) in;

struct Particle {
	float x;
	float y;
	float z;
};

layout(set = 0, binding = 0, std430) restrict buffer MyDataBuffer {
	int len;
	Particle particles[];
} my_data;

void main() {
	// gl_GlobalInvocationID.x uniquely identifies this invocation across all work groups
	Particle current_particle = my_data.particles[gl_GlobalInvocationID.x];
	for (int i = 0; i < 1; i++) {
		atomicAdd(my_data.particles[i].x, my_data.len);
	}
}
