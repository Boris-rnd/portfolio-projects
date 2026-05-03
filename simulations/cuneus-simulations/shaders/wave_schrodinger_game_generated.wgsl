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
    // fst bit = reset
    // snd bit = edge damping
    flags: u32,
    scene: u32,
    camera_pos_x: f32,
    camera_pos_y: f32,
    camera_zoom: f32,
    force: f32,
    restitution: f32,
    window_width: u32,
    window_height: u32,
    ping: u32,   // 0 or 1 — index of the READ buffer
    scroll: f32,
    control: f32,
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
    real_y: f32,
    imag_y: f32,
    mass: f32,
    _pad: u32,
};

@group(3) @binding(0) var<storage, read_write> cells_a: array<Cell>;
@group(3) @binding(1) var<storage, read_write> cells_b: array<Cell>;

// Returns a reference to the read buffer cell
fn read_cell(i: u32) -> Cell {
    if (params.ping == 0u) { return cells_a[i]; } else { return cells_b[i]; }
}
fn read_laplacian(i: u32) -> vec2<f32> {
    let dims = vec2(800u, 600u);
    let cell = read_cell(i);
    var left = read_cell(i - 1u);
    var right = read_cell(i + 1u);
    var up = read_cell(i - dims.x);
    var down = read_cell(i + dims.x);
    return vec2<f32>(left.real_y + right.real_y + up.real_y + down.real_y - 4.0 * cell.real_y, left.imag_y + right.imag_y + up.imag_y + down.imag_y - 4.0 * cell.imag_y);
}
fn write_cell(i: u32, cell: Cell) {
    if (params.ping == 0u) { cells_b[i] = cell; } else { cells_a[i] = cell; }
}

fn obstacle_potential(pos: vec2<u32>) -> f32 {
    let dims = vec2(800u, 600u);
    let x_world = f32(pos.x) + params.scroll;
    let period = 210.0;
    let thickness = 8.0;
    let x_mod = fract(x_world / period) * period;
    if (x_mod > thickness) {
        return 0.0;
    }

    let column = u32(x_world / period);
    let seed = column * 1327u + 24517u;
    let gap_center = 0.2 + 0.6 * rand(seed);
    let gap_size = 0.48;
    let gap_half = gap_size * 0.5;
    let y_norm = f32(pos.y) / f32(dims.y);
    if (y_norm < gap_center - gap_half || y_norm > gap_center + gap_half) {
        return 1.0;
    }
    return 0.0;
}

fn get_potential(pos: vec2<u32>) -> f32 {
    if params.scene == 0u {
        let dims = vec2(800u, 600u);
        let normalized_pos = vec2<f32>(pos) / vec2<f32>(dims).yx;
        let vertical_bias = params.control * (normalized_pos.x) * 4.0;
        return vertical_bias + obstacle_potential(pos.yx);
    }
    return 0.;
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.cell_count) { return; }

    var cell = read_cell(i);
    let dims = vec2(800u, 600u);
    let pos = vec2(u32(i) % dims.x, i / dims.x);

    if (((params.flags & 1u) == 1u) || time_data.frame == 0u) {
        // Initialize or reset particle
        cell.real_y = 0.;
        cell.imag_y = 0.;
        cell.mass = 1.0;
        write_cell(i, cell);
        return;
    }
    if (i < dims.x || i >= params.cell_count - dims.x || i % dims.x == 0u || (i + 1u) % dims.x == 0u) || (cell.mass > 99999.0) {
        return;
    }

    // old_y is the current state (from the READ buffer)
    let psi_curr = vec2<f32>(cell.real_y, cell.imag_y);
    let laplacian = read_laplacian(i);
    var potential = get_potential(pos);
    let d_real = 0.5 * laplacian.y - potential * psi_curr.y;
    let d_imag = -0.5 * laplacian.x + potential * psi_curr.x;

    let dt = time_data.delta * params.speed;
    // Euler method
    // let new_psi = vec2<f32>(psi_curr.x + d_real * dt, psi_curr.y + d_imag * dt);
    
    // Leapfrog / Symplectic Euler integration to prevent explosion
    // We alternate updating real and imaginary parts using the ping-pong buffer
    var new_psi: vec2<f32>;
    if (params.ping == 0u) {
        new_psi = vec2<f32>(psi_curr.x + d_real * dt * 2.0, psi_curr.y);
    } else {
        new_psi = vec2<f32>(psi_curr.x, psi_curr.y + d_imag * dt * 2.0);
    }

    cell.real_y = new_psi.x/cell.mass;
    cell.imag_y = new_psi.y/cell.mass;
    // Let waves pass through edge as if the medium was infinite using a sponge layer
    if ((params.flags & 2u) == 2u) {
        // Quadratic falloff to take less space (supposedly more efficientt than cubic)
        let margin = 16.0;
        let edge_dist = min(min(f32(pos.x), f32(dims.x) - f32(pos.x)), min(f32(pos.y), f32(dims.y) - f32(pos.y)));

        if (edge_dist < margin) {
            let factor = 1.0 - (edge_dist / margin);
            let absorption = factor * factor * 0.5;                 
            cell.real_y *= (1.0 - absorption);
            cell.imag_y *= (1.0 - absorption);
        }
    }
    if (time_data.time < 1.0) {
        let FREQUENCY = f32(1500);
        let RADIUS = f32(0.025);
        var origin = -vec2<f32>(0.3, -0.5);
        if (params.scene == 1u) { // Prism
            origin = -vec2<f32>(0.2, -0.4);
        }
        let uv = vec2<f32>(pos) / vec2<f32>(dims);
        let wave_emit = circleWave(uv * rot(PI/2.0), origin, FREQUENCY, RADIUS, time_data.time);
        cell.real_y = wave_emit.x;
        cell.imag_y = wave_emit.y;
    }

    write_cell(i, cell);
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
    if (i >= params.cell_count) { return; }

    // After all update iterations, ping points to the last-written buffer
    let cell = read_cell(i);
    let dims = textureDimensions(output);
    let sim_dims = vec2(800u, 600u);

    let col = u32(i) % sim_dims.x;
    let row = i / sim_dims.x;
    let world_pos = vec2<f32>(vec2(row, col));
    let pos_px = vec2<u32>((world_pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y) * vec2<f32>(sim_dims)) * params.camera_zoom);

    if params.camera_zoom > 1.0 {
        // for (var x_px = pos_px.x; x_px < pos_px.x + u32(params.camera_zoom * 10.); x_px++) {
        //     for (var y_px = pos_px.y; y_px < pos_px.y + u32(params.camera_zoom * 10.); y_px++) {
        //         if (x_px < dims.x && y_px < dims.y) {
        //             textureStore(output, vec2<i32>(vec2(x_px, y_px)), vec4<f32>(cell.y, abs(cell.y), cell.mass / 100., 1.));
        //         }
        //     }
        // }
    } else {
        if (pos_px.x < dims.x && pos_px.y < dims.y) {
            let angle = atan2(cell.imag_y, cell.real_y);
            let probability_density = (cell.real_y*cell.real_y + cell.imag_y*cell.imag_y)/0.001;
            // Convert hue angle to rgb
            var r = cos(angle);
            var g = cos(angle + 2.0 * PI / 3.0);
            var b = cos(angle + 4.0 * PI / 3.0);
            var c = vec3(r,g,b)*probability_density/100.;

            // var b = cell.mass / 200.;
            if (cell.mass>1.5) {
                c += vec3(0.1);
                c *= 0.5;
            }
            // Render potential with low opacity
            var potential = abs(get_potential(vec2<u32>(col, row)));
            c = c+vec3(potential)*0.1;
            textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(c, 1.0));
            // Visualize probability density psi^2
            // textureStore(output, vec2<i32>(pos_px.xy), vec4(vec3(probability_density), 1.));
            // Added for debugging
            // if cell.y != 0. {
            //     textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(1., abs(cell.y) / 100., 1., 1.));
            // }
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

fn circleWave(point: vec2<f32>, circlePosition: vec2<f32>, frequency: f32, size: f32, time: f32) -> vec2<f32> {
    let dx = point.x - circlePosition.x;
    let dy = point.y - circlePosition.y;
    let r = dx * dx + dy * dy;
    let fade = exp(-r / 2.0 / (size * size)) / size;
    return fade * vec2<f32>(cos(frequency * point.x), sin(frequency * point.x)) * sin(time * PI);
}

// zz' = (a+bi)(c+di) = ac-bd + i(ad+bc)
fn c_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// z/z' = (a+bi)/(c+di) = (ac+bd + i(bc-ad))/(c^2+d^2)
// Multiply numerator and denominator by conjugate of denominator
fn c_div(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x + a.y * b.y, a.y * b.x - a.x * b.y) / (b.x * b.x + b.y * b.y);
}
fn c_add(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x + b.x, a.y + b.y);
}
fn c_sub(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x - b.x, a.y - b.y);
}
const j = vec2<f32>(0.0, 1.0);

// let seed = 123456789.0;
fn random(seed: f32) -> f32 {
    return fract(sin(seed) * 43758.5453123);
}
