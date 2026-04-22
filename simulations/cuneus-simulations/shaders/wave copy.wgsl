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
    cell_count: u32,
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
    _pad0: u32,
};
@group(1) @binding(1) var<uniform> params: Params;

// Group 2: Global Engine Resources
struct MouseUniform {
    position: vec2<f32>,
    click: u32,
};
@group(2) @binding(0) var<uniform> mouse: MouseUniform;
@group(2) @binding(1) var<storage, read_write> atomic_buffer: array<atomic<u32>>;

// Group 3: User Data
struct Cell {
    old_y: f32,
    y: f32,
    vel_y: f32,
    mass: f32,
    accumulated_height: f32,
    _pad: u32,
    // _pad: [u32; 1],
};

@group(3) @binding(0) var<storage, read_write> cells: array<Cell>;


@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.cell_count) {
        return;
    }

    var cell = cells[i];
    let dims = vec2(800u, 600u);
    let pos = vec2(u32(i) % dims.x, i/dims.x);
    
    if (params.reset > 0u || time_data.frame == 0u) {
        // Initialize or reset particle
        cell.y = 0.;
        cell.old_y = 0.;
        cell.vel_y = 0.;
        cell.mass = 1.0;
        
        if (params.scene == 1u) { // Prism
            let d = triangle(vec2<f32>(pos) * 300.0 + vec2<f32>(100.0, 200.0));
            if (d < 0.0) {
                let INDEX_OF_REFRACTION = 0.6; // 3.0 / 5.0
                cell.mass = 1.0 / INDEX_OF_REFRACTION;
            }
        } else if (params.scene == 2u || true) { // Double Slit
            if (pos.y > 400 && pos.y < 420) {
                let slit_width = 10;
                let slit_gap = 100.0;
                if (((i32(pos.x) - 350) > slit_width || (i32(pos.x) - 350) < -slit_width) && ((i32(pos.x) - 450) > slit_width || (i32(pos.x) - 450) < -slit_width)) {
                    cell.mass = 1000000.0;
                    // cell.y = 1000.0;
                }
            }
        } else { // Default, single wave initially for fun?
            // if i == 400*600/2+200 { cell.y = 1.0; }
        }
    } else {
        if i < dims.x || i >= params.cell_count - dims.x || i % dims.x == 0u || (i+1) % dims.x == 0u {
            return;
        }

        if (cell.mass > 99999.0) {
            return;
        }
        let avg_neighbor_height = (cells[i-1].old_y + cells[i+1].old_y + cells[i-dims.x].old_y + cells[i+dims.x].old_y) / 4.0;
        let a = ((avg_neighbor_height - cell.old_y) * params.restitution)/cell.mass;

        // Verlet integration
        cell.vel_y += a*time_data.delta*params.speed;
        // Implement a smooth damping at the edges
        let edge_dist = min(min(pos.x, pos.y), min(dims.x-pos.x, dims.y-pos.y));
        if (edge_dist < 50) {
            cell.vel_y *= 0.9;
            // cell.y *= 0.9;
        }
        let temp = cell.y;
        cell.y = cell.old_y + cell.vel_y * time_data.delta*params.speed; 
        // cell.y = 2.0*cell.y - cell.old_y + a*time_data.delta*time_data.delta*params.speed*params.speed;
        cell.old_y = temp;
        cell.accumulated_height += cell.y;
        
        if (time_data.time < 1.0) { //  
            let FREQUENCY = 700.0;
            let RADIUS = 0.01; //  
            let wave_emit = circleWave(vec2<f32>(pos)/vec2<f32>(dims)*rot(-0.625), vec2<f32>(0.5, 0.15), FREQUENCY, RADIUS, time_data.time);
            cell.y = wave_emit;
        }
        // storageBarrier();
    }

    cells[i] = cell;
}

@compute @workgroup_size(64)
fn clear_screen(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output);
    let pixel_per_invoke = params.window_width * params.window_height / (64*16);
    let i = id.x * pixel_per_invoke;
    for (var j = 0u; j < pixel_per_invoke; j++) {
        let pixel_idx = i + j;
        if (pixel_idx >= params.window_width * params.window_height) {
            return;
        }
        let x = pixel_idx % params.window_width;
        let y = pixel_idx / params.window_width;
        textureStore(output, vec2<i32>(vec2(x, y)), vec4<f32>(0.0, 0.0, 0.0, 1.0));
        
        // let pixel_pos = vec2(x,y);
        // let world_pos = (vec2<f32>(pixel_pos)+vec2<f32>(params.camera_pos_x, params.camera_pos_y)*vec2<f32>(dims))/params.camera_zoom;
        // let idx = u32(world_pos.x) + u32(world_pos.y) * 800u;
        // var cell = cells[idx];
        // let col = u32(idx) % 800u;
        // let row = idx/800u;
        // let pos_px = vec2<u32>((world_pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y)*vec2<f32>(dims)) * params.camera_zoom);

        // if (pos_px.x < dims.x && pos_px.y < dims.y) {
        //     let vel = abs(cell.old_y-cell.y);
        //     textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(cell.y, vel, cell.mass/100., 1.));
        // }

    }
}

@compute @workgroup_size(64)
fn render(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.cell_count) {
        return;
    }

    var cell = cells[i];
    let dims = textureDimensions(output);

    let col = u32(i) % 800u;
    let row = i/800u;
    let world_pos = vec2<f32>(vec2(row, col));
    let pos_px = vec2<u32>((world_pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y)*vec2<f32>(dims)) * params.camera_zoom);
    let vel = abs(cell.old_y-cell.y);

    if params.camera_zoom > 1.0 {
        for (var x_px = pos_px.x; x_px < pos_px.x + u32(params.camera_zoom*10.); x_px++) {
            for (var y_px = pos_px.y; y_px < pos_px.y + u32(params.camera_zoom*10.); y_px++) {
                if (x_px < dims.x && y_px < dims.y) {
                    // let current_color = textureLoad(output, vec2<i32>(vec2(x_px, y_px)));
                    textureStore(output, vec2<i32>(vec2(x_px, y_px)), (vec4<f32>(cell.y, vel, cell.mass/100., 1.)));
                }
            }
        }
    } else {
        if (pos_px.x < dims.x && pos_px.y < dims.y) {
            textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(cell.y, vel/100., cell.mass/100., 1.));
        }
    }
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
