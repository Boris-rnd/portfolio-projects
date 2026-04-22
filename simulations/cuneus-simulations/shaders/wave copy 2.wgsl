// Group 0: Per-Frame Data (Engine-Managed)
struct TimeUniform {
    time: f32,
    delta: f32,
    frame: u32,
    _padding: u32,
};
@group(0) @binding(0) var<uniform> time_data: TimeUniform;

// Group 1: Primary Pass I/O & Custom Parameters
@group(1) @binding(0) var output: texture_storage_2d<rgba16float, write>;

struct Params {
    speed: f32,
    reset: u32,
    scene: u32,
    camera_pos_x: f32,
    camera_pos_y: f32,
    camera_zoom: f32,
    drag: f32,
    restitution: f32,
    window_width: u32,
    window_height: u32,
    _pad0: vec2<u32>,
};
@group(1) @binding(1) var<uniform> params: Params;

// Group 2: Global Engine Resources
struct MouseUniform {
    position: vec2<f32>,
    click: u32,
};
@group(2) @binding(0) var<uniform> mouse: MouseUniform;
@group(2) @binding(1) var<storage, read_write> atomic_buffer: array<atomic<u32>>;

// Group 3: Multi-pass feedback
@group(3) @binding(0) var input_texture0: texture_2d<f32>;
@group(3) @binding(1) var input_sampler0: sampler;

// Cell mapping: cell.x = y, cell.y = old_y, cell.z = vel_y, cell.w = mass

@compute @workgroup_size(16, 16, 1)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    
    let pos_i = vec2<i32>(id.xy);
    var cell = textureLoad(input_texture0, pos_i, 0);

    let pos = vec2<f32>(id.xy);
    
    if (params.reset > 0u || time_data.frame == 0u) {
        // Initialize or reset particle
        cell.x = 0.0;
        cell.y = 0.0;
        cell.z = 0.0;
        cell.w = 1.0; // Default mass
        
        if (params.scene == 1u) { // Prism
            let d = triangle(pos * 300.0 / vec2<f32>(dims) + vec2<f32>(100.0, 200.0));
            if (d < 0.0) {
                let INDEX_OF_REFRACTION = 0.6; // 3.0 / 5.0
                cell.w = 1.0 / INDEX_OF_REFRACTION;
            }
        } else if (params.scene == 2u || true) { // Double Slit
            if (pos.y > 400.0 && pos.y < 420.0) {
                let slit_width = 10.0;
                let slit_gap = 100.0;
                if (((pos.x - 350.0) > slit_width || (pos.x - 350.0) < -slit_width) && ((pos.x - 450.0) > slit_width || (pos.x - 450.0) < -slit_width)) {
                    cell.w = 1000000.0;
                }
            }
        }
    } else {
        if (pos_i.x == 0 || pos_i.y == 0 || pos_i.x >= i32(dims.x) - 1 || pos_i.y >= i32(dims.y) - 1) {
            textureStore(output, pos_i, cell);
            return;
        }

        if (cell.w > 99999.0) {
            textureStore(output, pos_i, cell);
            return;
        }

        let nN = textureLoad(input_texture0, pos_i + vec2(0, -1), 0).y;
        let nS = textureLoad(input_texture0, pos_i + vec2(0, 1), 0).y;
        let nE = textureLoad(input_texture0, pos_i + vec2(1, 0), 0).y;
        let nW = textureLoad(input_texture0, pos_i + vec2(-1, 0), 0).y;
        
        let avg_neighbor_height = (nN + nS + nE + nW) / 4.0;
        let a = ((avg_neighbor_height - cell.y) * params.restitution) / cell.w;

        // Verlet integration setup
        cell.z += a * time_data.delta * params.speed;
        
        // Edge damping
        let edge_dist = min(min(pos.x, pos.y), min(vec2<f32>(dims).x - pos.x, vec2<f32>(dims).y - pos.y));
        if (edge_dist < 50.0) {
            cell.z *= 0.9;
        }
        
        let temp = cell.x;
        cell.x = cell.y + cell.z * time_data.delta * params.speed - cell.x * params.drag * time_data.delta * params.speed; 
        cell.y = temp;
        
        if (time_data.time < 1.0) { 
            let FREQUENCY = 700.0;
            let RADIUS = 0.01; 
            let wave_emit = circleWave(pos / vec2<f32>(dims) * rot(-0.625), vec2<f32>(0.5, 0.15), FREQUENCY, RADIUS, time_data.time);
            cell.x = wave_emit;
        }
    }

    textureStore(output, pos_i, cell);
}

@compute @workgroup_size(16, 16, 1)
fn main_image(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output); // Window dimensions
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }
    
    let pos_px = vec2<f32>(id.xy);
    let sim_dims = vec2<f32>(800.0, 600.0);
    
    // Reverse the camera transform to find the uv coordinate in the simulation texture
    // pos_px = (world_pos - camera_pos * sim_dims) * camera_zoom
    // -> world_pos = pos_px / camera_zoom + camera_pos * sim_dims
    let world_pos = pos_px / params.camera_zoom + vec2<f32>(params.camera_pos_x, params.camera_pos_y) * sim_dims;
    let uv = world_pos / sim_dims;
    
    if (uv.x < 0.0 || uv.y < 0.0 || uv.x > 1.0 || uv.y > 1.0) {
        textureStore(output, vec2<i32>(id.xy), vec4<f32>(0.0, 0.0, 0.0, 1.0));
        // return;
    }
    
    let cell = textureSampleLevel(input_texture0, input_sampler0, uv, 0.0);
    // cell.x = y, cell.y = old_y, cell.z = vel_y, cell.w = mass
    let vel = abs(cell.y - cell.x); // magnitude of height change
    
    // In original code, visualization was cell.y (mapped to my cell.x), vel/100, mass/100
    // I mapped mass correctly but color was hard to see perhaps?
    let col = vec4<f32>(abs(cell.x) * 2.0, vel * 10.0, 1., 1.0);
    textureStore(output, vec2<i32>(id.xy), col);
}

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

fn rand(u: u32) -> f32 {
    return f32(hash(u)) / 4294967295.0;
}

const PI: f32 = 3.14159265358979323846;

fn triangle(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    let k = sqrt(3.0);
    p.x = abs(p.x) - 1.0;
    p.y = p.y + 1.0 / k;
    if (p.x + k * p.y > 0.0) {
        p = vec2<f32>(p.x - k * p.y, -k * p.x - p.y) / 2.0;
    }
    p.x -= clamp(p.x, -2.0, 0.0);
    return -length(p) * sign(p.y);
}

fn rot(a: f32) -> mat2x2<f32> {
    let s = sin(a);
    let c = cos(a);
    return mat2x2<f32>(c, -s, s, c);
}

fn circleWave(point: vec2<f32>, circlePosition: vec2<f32>, frequency: f32, size: f32, time: f32) -> f32 {
    let dx = point.x - circlePosition.x;
    let dy = point.y - circlePosition.y;
    let r = dx * dx + dy * dy;
    let fade = exp(-r / 2.0 / (size * size)) / size;
    return fade * cos(frequency * point.x) * abs(sin(time * 3.14159265358979323846));
}
