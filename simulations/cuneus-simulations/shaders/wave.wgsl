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
    drag: f32,
    restitution: f32,
    window_width: u32,
    window_height: u32,
    ping: u32,   // 0 or 1 — index of the READ buffer
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
    y: f32,
    mass: f32,
    accumulated_height: f32,
    _pad: u32,
};

@group(3) @binding(0) var<storage, read_write> cells_a: array<Cell>;
@group(3) @binding(1) var<storage, read_write> cells_b: array<Cell>;


// Returns a reference to the read buffer cell
fn read_cell(i: u32) -> Cell {
    if (params.ping == 0u) { return cells_a[i]; } else { return cells_b[i]; }
}
fn write_cell(i: u32, cell: Cell) {
    if (params.ping == 0u) { cells_b[i] = cell; } else { cells_a[i] = cell; }
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
        cell.y = 0.;
        cell.mass = 1.0;
        cell.accumulated_height = 0.;

        if (params.scene == 1u) { // Prism
            let d = triangle(vec2<f32>(pos) / (vec2<f32>(dims)/5.0) - vec2<f32>(2.5, 3.0));
            if (d < 0.0) {
                let INDEX_OF_REFRACTION = 0.6; // 3.0 / 5.0
                cell.mass = 1.0 / INDEX_OF_REFRACTION;
            }
        } else if (params.scene == 2u || true) { // Double Slit
            if (pos.y > 250 && pos.y < 270) {
                let slit_width = 10;
                let slit_height = 40;
                if (((i32(pos.x) - (400-slit_height/2)) > slit_width || (i32(pos.x) - (400-slit_height/2)) < -slit_width) &&
                    ((i32(pos.x) - (400+slit_height/2)) > slit_width || (i32(pos.x) - (400+slit_height/2)) < -slit_width)) {
                    cell.mass = 1000000.0;
                }
            }
        }
    } else {
        if i < dims.x || i >= params.cell_count - dims.x || i % dims.x == 0u || (i + 1u) % dims.x == 0u {
            write_cell(i, cell);
            return;
        }
        if (cell.mass > 99999.0) {
            write_cell(i, cell);
            return;
        }

        // old_y is the current state (from the READ buffer)
        let y_curr = cell.y;
        let avg_neighbor = (read_cell(i - 1u).y + read_cell(i + 1u).y +
                            read_cell(i - dims.x).y + read_cell(i + dims.x).y) / 4.0;
        let a = (avg_neighbor - y_curr) * params.restitution / cell.mass;

        // The WRITE buffer hasn't been overwritten yet, so it holds the state from the previous iteration.
        let y_prev = select(cells_a[i].y, cells_b[i].y, params.ping == 0u);

        // Verlet velocity form
        var vel = y_curr - y_prev;
        vel += a * time_data.delta * time_data.delta * params.speed * params.speed;

        // Let waves pass through edge as if the medium was infinite using a sponge layer
        if ((params.flags & 2u) == 2u) {
            let margin = 64.0;
            let dist_x = min(f32(pos.x), f32(dims.x) - 1.0 - f32(pos.x));
            let dist_y = min(f32(pos.y), f32(dims.y) - 1.0 - f32(pos.y));
            let edge_dist = min(dist_x, dist_y);
            
            if (edge_dist < margin) {
                let normalized = edge_dist / margin;
                // Cubic falloff for smooth impedance transition (prevents damping itself from causing reflections, this took a lot of trial and error)
                let damping = pow(1.0 - normalized, 3.0) * 0.12;
                vel *= (1.0 - damping);
                cell.y *= (1.0 - damping * 0.5); // Damp displacement to prevent accumulation at the boundary
            }
        }

        let new_y = y_curr + vel;
        cell.y = new_y;
        cell.accumulated_height += abs(new_y);

        // DEBUG: Force cell.y in the center to 1.0
        // if (pos.x > 380u && pos.x < 420u && pos.y > 280u && pos.y < 320u) {
        //     cell.y = 1.0;
        // }

        if (time_data.time < 1.0) {
            let FREQUENCY = 230.0*PI;
            let RADIUS = 0.015;
            var origin = vec2<f32>(0.2, -0.5);
            if (params.scene == 1u) { // Prism
                origin = vec2<f32>(0.2, -0.4);
            }
            let wave_emit = circleWave(vec2<f32>(pos) / vec2<f32>(dims) * rot(-PI/2.0), origin, FREQUENCY, RADIUS, time_data.time);
            cell.y = wave_emit;
        }
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

    let col = u32(i) % 800u;
    let row = i / 800u;
    let world_pos = vec2<f32>(vec2(row, col));
    let pos_px = vec2<u32>((world_pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y) * vec2<f32>(dims)) * params.camera_zoom);

    if params.camera_zoom > 1.0 {
        for (var x_px = pos_px.x; x_px < pos_px.x + u32(params.camera_zoom * 10.); x_px++) {
            for (var y_px = pos_px.y; y_px < pos_px.y + u32(params.camera_zoom * 10.); y_px++) {
                if (x_px < dims.x && y_px < dims.y) {
                    textureStore(output, vec2<i32>(vec2(x_px, y_px)), vec4<f32>(cell.y, abs(cell.y), cell.mass / 100., 1.));
                }
            }
        }
    } else {
        if (pos_px.x < dims.x && pos_px.y < dims.y) {
            var b = cell.mass / 200.;
            if (cell.mass>1.5) {
                b = 0.5;
            }
            // textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(cell.y*20., abs(cell.y) / 100., b, 1.));
            textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(cell.accumulated_height/200., cell.y * 1., b, 1.));
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

fn circleWave(point: vec2<f32>, circlePosition: vec2<f32>, frequency: f32, size: f32, time: f32) -> f32 {
    let dx = point.x - circlePosition.x;
    let dy = point.y - circlePosition.y;
    let r = dx * dx + dy * dy;
    let fade = exp(-r / 2.0 / (size * size)) / size;
    return fade * cos(frequency * point.x) * abs(sin(time * 3.14159265358979323846));
}
