struct TimeUniform {
    time: f32,
    delta: f32,
    frame: u32,
    _padding: u32
}

@group(0) @binding(0)
var<uniform> time_data: TimeUniform;

@group(1) @binding(0)
var output: texture_storage_2d<rgba16float, write>;

struct Params {
    cell_count: u32,
    speed: f32,
    flags: u32,
    scene: u32,
    camera_pos_x: f32,
    camera_pos_y: f32,
    camera_zoom: f32,
    force: f32,
    restitution: f32,
    window_width: u32,
    window_height: u32,
    ping: u32,
    scroll: f32,
    control: f32
}

@group(1) @binding(1)
var<uniform> params: Params;

struct Cell {
    real_y: f32,
    imag_y: f32,
    mass: f32,
    _pad: u32
}

@group(3) @binding(0)
var<storage, read_write> cells_a: array<Cell>;

@group(3) @binding(1)
var<storage, read_write> cells_b: array<Cell>;

fn read_cell(i: u32) -> Cell {
    if (params.ping == 0u) {
        return cells_a[i];
    }
    else {
        return cells_b[i];
    }
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
    if (params.ping == 0u) {
        cells_b[i] = cell;
    }
    else {
        cells_a[i] = cell;
    }
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
    return 0.0;
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.cell_count) {
        return;
    }
    var cell = read_cell(i);
    let dims = vec2(800u, 600u);
    let pos = vec2(u32(i) % dims.x, i / dims.x);
    if (((params.flags & 1u) == 1u) || time_data.frame == 0u) {
        cell.real_y = 0.0;
        cell.imag_y = 0.0;
        cell.mass = 1.0;
        write_cell(i, cell);
        return;
    }
    if (i < dims.x || i >= params.cell_count - dims.x || i % dims.x == 0u || (i + 1u) % dims.x == 0u) || (cell.mass > 99999.0) {
        return;
    }
    let psi_curr = vec2<f32>(cell.real_y, cell.imag_y);
    let laplacian = read_laplacian(i);
    var potential = get_potential(pos);
    let d_real = 0.5 * laplacian.y - potential * psi_curr.y;
    let d_imag = -0.5 * laplacian.x + potential * psi_curr.x;
    let dt = time_data.delta * params.speed;
    var new_psi: vec2<f32>;
    if (params.ping == 0u) {
        new_psi = vec2<f32>(psi_curr.x + d_real * dt * 2.0, psi_curr.y);
    }
    else {
        new_psi = vec2<f32>(psi_curr.x, psi_curr.y + d_imag * dt * 2.0);
    }
    cell.real_y = new_psi.x / cell.mass;
    cell.imag_y = new_psi.y / cell.mass;
    if ((params.flags & 2u) == 2u) {
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
        let FREQUENCY = f32();
        let RADIUS = f32();
        var origin = -vec2<f32>(0.3, -0.5);
        if (params.scene == 1u) {
            origin = -vec2<f32>(0.2, -0.4);
        }
        let uv = vec2<f32>(pos) / vec2<f32>(dims);
        let wave_emit = circleWave(uv * rot(PI / 2.0), origin, FREQUENCY, RADIUS, time_data.time);
        cell.real_y = wave_emit.x;
        cell.imag_y = wave_emit.y;
    }
    write_cell(i, cell);
}

@compute @workgroup_size(64)
fn clear_screen(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(output);
    let pixel_per_invoke = params.window_width * params.window_height / (64 * 16);
    let i = id.x * pixel_per_invoke;
    for (var j = 0u; j < pixel_per_invoke; j++) {
        let pixel_idx = i + j;
        if (pixel_idx >= params.window_width * params.window_height) {
            return;
        }
        let x = pixel_idx % params.window_width;
        let y = pixel_idx / params.window_width;
        textureStore(output, vec2<i32>(vec2(x, y)), vec4<f32>(0.0, 0.0, 0.0, 1.0));
    }
}

@compute @workgroup_size(64)
fn render(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.cell_count) {
        return;
    }
    let cell = read_cell(i);
    let dims = textureDimensions(output);
    let sim_dims = vec2(800u, 600u);
    let col = u32(i) % sim_dims.x;
    let row = i / sim_dims.x;
    let world_pos = vec2<f32>(vec2(row, col));
    let pos_px = vec2<u32>((world_pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y) * vec2<f32>(sim_dims)) * params.camera_zoom);
    if params.camera_zoom > 1.0 {
    
    }
    else {
        if (pos_px.x < dims.x && pos_px.y < dims.y) {
            let angle = atan2(cell.imag_y, cell.real_y);
            let probability_density = (cell.real_y * cell.real_y + cell.imag_y * cell.imag_y) / 0.001;
            var r = cos(angle);
            var g = cos(angle + 2.0 * PI / 3.0);
            var b = cos(angle + 4.0 * PI / 3.0);
            var c = vec3(r, g, b) * probability_density / 100.0;
            if (cell.mass > 1.5) {
                c += vec3(0.1);
                c *= 0.5;
            }
            var potential = abs(get_potential(vec2<u32>(col, row)));
            c = c + vec3(potential) * 0.1;
            textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(c, 1.0));
        }
    }
}

fn hash(u: u32) -> u32 {
    var v = u;
    v = v ^ (v >> 16u);
    v = v * 73244475u;
    v = v ^ (v >> 16u);
    v = v * 73244475u;
    v = v ^ (v >> 16u);
    return v;
}

fn rand(u: u32) -> f32 {
    return f32(hash(u)) / 4294967295.0;
}

const PI: f32 = 3.141592653589793;

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
