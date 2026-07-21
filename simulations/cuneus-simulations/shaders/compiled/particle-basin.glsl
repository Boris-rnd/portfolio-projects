#version 450
// NOTE: GLSL does not support multiple compute entry points in one file like WGSL.
// These sections (update, clear_screen, splat) must be split into separate shader files
// or selected via preprocessor directives. Each has its own local_size layout.

// --- Group 0: Per-Frame Data (Engine-Managed) ---
layout(std140, binding = 0) uniform TimeUniform {
    float time;
    float delta;
    uint frame;
    uint _padding;
} time_data;

// --- Group 1: Primary Pass I/O & Custom Parameters ---
layout(rgba16f, binding = 1) uniform image2D output_image; // WGSL 'output' is reserved keyword

layout(std140, binding = 2) uniform Params {
    float gravity;
    uint particle_size;
    uint particle_count;
    float speed;
    uint reset;
    float camera_pos_x;
    float camera_pos_y;
    float camera_zoom;
    // _pad0: u32,
    // _pad1: u32,
    // _pad2: u32,
} params;

// --- Group 2: Global Engine Resources ---
layout(std140, binding = 3) uniform MouseUniform {
    vec2 position;
    uint click;
    uint _pad; // WGSL struct alignment padding
} mouse;
// @group(2) @binding(1) var<storage, read_write> particles_atomic_buffer: array<atomic<u32>>;

// --- Group 3: User Data (Particles) ---
struct Particle {
    vec2 old_pos;
    vec2 pos;
    float mass;
    uint enabled;
    uint _pad1;
    uint _pad2;
};

layout(std430, binding = 4) buffer ParticleBuffer {
    Particle particles[];
};
// @group(3) @binding(1) var<storage, read_write> particles_grid: array<ParticleGrid>;


// Utility function for random values
uint hash(uint u) {
    uint v = u;
    v = v ^ (v >> 16u);
    v = v * 0x45d9f3bu;
    v = v ^ (v >> 16u);
    v = v * 0x45d9f3bu;
    v = v ^ (v >> 16u);
    return v;
}

// Returns a pseudo-rng inside [0;1]
float rand(uint u) {
    return float(hash(u)) / 4294967295.0;
}

// --- SHADER: update ---
// layout (local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
void main_update() {
    if (params.speed == 0.0) { return; }
    uint i = gl_GlobalInvocationID.x;
    if (i >= params.particle_count) {
        return;
    }

    Particle p = particles[i];
    
    if (params.reset > 0u || time_data.frame == 0u) {
        p.pos = vec2(0.45 + float(i) / (float(params.particle_count) / 0.15), 0.0);
        p.old_pos = p.pos; // vel=0
        p.enabled = 1u;
        p.mass = 1.0;
    } else {
        if (p.enabled == 0u) { return; }
        float dt = time_data.delta * params.speed;
        
        vec2 acc = vec2(0.0);
        acc.y += params.gravity;
        // p = verlet_integration(p, acc, dt);
        p = euler_integration(p, acc, dt);
        
    }
    particles[i] = p;
}

Particle verlet_integration(Particle p2, vec2 acc, float dt) {
    Particle p = p2;
    vec2 temp = p.pos;
    p.pos = 2.0 * p.pos - p.old_pos + acc * dt * dt;
    p.old_pos = temp;

    if (dot(p.pos, p.pos) >= 1.0) {
        float dist = length(p.pos);
        vec2 normal = p.pos / dist;
        
        p.pos = normal * 1.0;
        
        // Bounce
        // e = 1.0 (Perfect) | e < 1.0 (Damped) | e > 1.0 (Boosted)
        float e = 1.0;
        vec2 vel = p.pos - p.old_pos;
        vec2 v_normal = dot(vel, normal) * normal;
        vec2 v_tangent = vel - v_normal;
        vec2 reflected_vel = v_tangent - e * v_normal;
        
        // Change old pos to create virtual vel
        p.old_pos = p.pos - reflected_vel;
    }
    return p;
} 

Particle euler_integration(Particle p2, vec2 acc, float dt) {
    Particle p = p2;
    p.old_pos += acc * dt;
    p.pos += p.old_pos * dt;
    
    // 2. Gestion de la contrainte (Cercle de rayon 1.0)
    float dist = length(p.pos);
    if (dist >= 1.0) {
        vec2 normal = p.pos / dist;
        
        // Repositionne sur le bord
        p.pos = normal * 1.0;
        
        // Inverse la composante normale de la vitesse (e = coefficient de rebond)
        float e = 1.0;
        p.old_pos = p.old_pos - (1.0 + e) * dot(p.old_pos, normal) * normal;
    }
    return p;
}


// --- SHADER: clear_screen ---
// layout (local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
void main_clear_screen() {
    ivec2 dims = imageSize(output_image);
    if (gl_GlobalInvocationID.x >= uint(dims.x) || gl_GlobalInvocationID.y >= uint(dims.y)) {
        return;
    }
    uvec2 pos_px = gl_GlobalInvocationID.xy;
    imageStore(output_image, ivec2(pos_px), vec4(0.0, 0.0, 0.0, 1.0));

    vec2 world_pos = screen_to_world_pos(pos_px);
    if (abs(dot(world_pos, world_pos) - 1.0) <= 0.01) {
        imageStore(output_image, ivec2(pos_px), vec4(1.0, 1.0, 1.0, 1.0));
    }
}

// --- SHADER: splat ---
// layout (local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
void main_splat() {
    uint i = gl_GlobalInvocationID.x;
    if (i >= params.particle_count) { return; }

    Particle p = particles[i];
    if (p.enabled == 0u) { return; }
    render_particle(p);
}

uvec2 world_to_screen_pos(vec2 world_pos) {
    return uvec2((world_pos - vec2(params.camera_pos_x, params.camera_pos_y)) * vec2(imageSize(output_image)) * params.camera_zoom);
}

vec2 screen_to_world_pos(uvec2 screen_pos) {
    return vec2((vec2(screen_pos) / params.camera_zoom) / vec2(imageSize(output_image)) + vec2(params.camera_pos_x, params.camera_pos_y));
}

void render_particle(Particle p) {
    ivec2 dims = imageSize(output_image);
    uvec2 pos_px = world_to_screen_pos(p.pos);

    int ps = int(params.particle_size);
    // let half_size = i32(p.mass/1000.);
    // textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(p.mass/100., 0., 1., 1.));

    for (int y = -ps + 1; y < ps; y++) {
        for (int x = -ps + 1; x < ps; x++) {
            vec2 disp = vec2(x, y);
            ivec2 current = ivec2(pos_px.xy) + ivec2(x, y);
            if (current.x >= 0 && current.y >= 0 && uint(current.x) < uint(dims.x) && uint(current.y) < uint(dims.y)) {
                float dst_sq = dot(disp, disp);
                float normalized_dst = dst_sq / float(ps * ps);
                vec3 intensity = vec3(1.0); //  - normalized_dst
                // let prev_intensity = textureLoad(output, current).xyz;
                if (normalized_dst >= 0.5) { continue; }
                vec3 color = vec3(vec2(pos_px) / vec2(dims), p.mass);

                imageStore(output_image, current, vec4(color, 1.0));
            }
        }
    }
}