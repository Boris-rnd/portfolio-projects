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
    drag: f32,
    restitution: f32,
    window_width: u32,
    window_height: u32,
    ping: u32
}

@group(1) @binding(1)
var<uniform> params: Params;

struct Cell {
    y: f32,
    mass: f32,
    accumulated_height: f32,
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

fn write_cell(i: u32, cell: Cell) {
    if (params.ping == 0u) {
        cells_b[i] = cell;
    }
    else {
        cells_a[i] = cell;
    }
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
        cell.y = 0.0;
        cell.mass = 1.0;
        cell.accumulated_height = 0.0;
        if (params.scene == 1u) {
            let d = triangle(vec2<f32>(pos) / (vec2<f32>(dims) / 5.0) - vec2<f32>(2.5, 3.0));
            if (d < 0.0) {
                let INDEX_OF_REFRACTION = 0.6;
                cell.mass = 1.0 / INDEX_OF_REFRACTION;
            }
        }
        else if (params.scene == 2u) {
            if (pos.y > 250 && pos.y < 270) {
                let slit_width = 10;
                let slit_height = 40;
                if (((i32(pos.x) - (400 - slit_height / 2)) > slit_width || (i32(pos.x) - (400 - slit_height / 2)) < -slit_width) && ((i32(pos.x) - (400 + slit_height / 2)) > slit_width || (i32(pos.x) - (400 + slit_height / 2)) < -slit_width)) {
                    cell.mass = 1000000.0;
                }
            }
        }
        else if (params.scene == 3u) {
            let N_IMP: u32 = 1280u;
            let px = f32(pos.x);
            let py = f32(pos.y);
            for (var ic = 0u; ic < N_IMP; ic = ic + 1u) {
                let seed = ic * 15485863u + 32452843u * time_data.frame;
                let cx = (rand(seed + 1u)) * f32(dims.x);
                let cy = (rand(seed + 2u)) * f32(dims.y);
                let r = 3.0 + rand(seed + 3u) * 0.5;
                let refr = 1.6 + rand(seed + 4u) * 1.4;
                let dx = px - cx;
                let dy = py - cy;
                if (dx * dx + dy * dy < r * r) {
                    cell.mass = 1.6;
                    break;
                }
            }
        }
    }
    else {
        if i < dims.x || i >= params.cell_count - dims.x || i % dims.x == 0u || (i + 1u) % dims.x == 0u {
            write_cell(i, cell);
            return;
        }
        if (cell.mass > 99999.0) {
            write_cell(i, cell);
            return;
        }
        let y_curr = cell.y;
        let avg_neighbor = (read_cell(i - 1u).y + read_cell(i + 1u).y + read_cell(i - dims.x).y + read_cell(i + dims.x).y) / 4.0;
        let a = (avg_neighbor - y_curr) * params.restitution / cell.mass;
        let y_prev = select(cells_a[i].y, cells_b[i].y, params.ping == 0u);
        var vel = y_curr - y_prev;
        vel += a * time_data.delta * time_data.delta * params.speed * params.speed;
        if ((params.flags & 2u) == 2u) {
            let margin = 64.0;
            let dist_x = min(f32(pos.x), f32(dims.x) - 1.0 - f32(pos.x));
            let dist_y = min(f32(pos.y), f32(dims.y) - 1.0 - f32(pos.y));
            let edge_dist = min(dist_x, dist_y);
            if (edge_dist < margin) {
                let normalized = edge_dist / margin;
                let damping = pow(1.0 - normalized, 3.0) * 0.12;
                vel *= (1.0 - damping);
                cell.y *= (1.0 - damping * 0.5);
            }
        }
        let new_y = y_curr + vel;
        cell.y = new_y;
        cell.accumulated_height += abs(new_y);
        if (time_data.time < 1.0) {
            let FREQUENCY = 230.0 * PI;
            let RADIUS = 0.015;
            var origin = vec2<f32>(0.2, -0.5);
            if (params.scene == 1u) {
                origin = vec2<f32>(0.2, -0.4);
            }
            let wave_emit = circleWave(vec2<f32>(pos) / vec2<f32>(dims) * rot(-PI / 2.0), origin, FREQUENCY, RADIUS, time_data.time);
            cell.y = wave_emit;
        }
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
    let col = u32(i) % 800u;
    let row = i / 800u;
    let world_pos = vec2<f32>(vec2(row, col));
    let pos_px = vec2<u32>((world_pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y) * vec2<f32>(dims)) * params.camera_zoom);
    if params.camera_zoom > 1.0 {
        for (var x_px = pos_px.x; x_px < pos_px.x + u32(params.camera_zoom * 10.0); x_px++) {
            for (var y_px = pos_px.y; y_px < pos_px.y + u32(params.camera_zoom * 10.0); y_px++) {
                if (x_px < dims.x && y_px < dims.y) {
                    textureStore(output, vec2<i32>(vec2(x_px, y_px)), vec4<f32>(cell.y, abs(cell.y), cell.mass / 100.0, 1.0));
                }
            }
        }
    }
    else {
        if (pos_px.x < dims.x && pos_px.y < dims.y) {
            var b = cell.mass / 200.0;
            if (cell.mass > 1.5) {
                b = 0.5;
            }
            if (params.flags & u32(4)) == 4 {
                textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(cell.accumulated_height / 200.0, cell.y * 1.0, b, 1.0));
            }
            else {
                textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(0.0, cell.y * 1.0, (cell.mass - 1.0), 1.0));
            }
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
    return fade * cos(frequency * point.x) * abs(sin(time * 3.141592653589793));
}
