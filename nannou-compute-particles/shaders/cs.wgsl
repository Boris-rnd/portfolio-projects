struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
    color: vec3<f32>,
    mass: f32,
}
const PART_RAD: f32 = 0.5;
const GRAV_FRC: f32 = 0.01;

struct Buffer {
    particles: array<Particle>,
}
struct Cell {
    inner_mass: f32,
    parts: array<u32, 256>,
    parts_len: u32
}

@group(0) @binding(0)
var<storage, read> in: Buffer;
@group(0) @binding(1)
var<storage, read_write> out: Buffer;
@group(0) @binding(2)
var<storage, read> cells: array<Cell>;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var p1 = in.particles[id.x];
    let ipos = p1.pos/10.;
    let cell_i = u32(ipos.x*80.+ipos.y);
    var cell = cells[cell_i];

    // Compute with near particles in cell
    for (var i: u32=u32(0); i < u32(cell.parts_len); i+=u32(1)) {
        if i == id.x {continue;}
        let p2_idx = cell.parts[i];
        let p2 = in.particles[p2_idx];
        let dir = p2.pos - p1.pos;
        let dst_sq = dot(dir, dir);
        if dst_sq < 7. {continue;}
        let f = GRAV_FRC*p2.mass/dst_sq;
        p1.vel += f*dir;
    }
    
    // Compute with all cell's approximated masses
    for (var i: u32=u32(0); i < u32(arrayLength(&cells)); i+=u32(1)) {
        let cell = cells[i];
        if i == id.x {continue;}
        let cell_pos = vec2(f32(i)%80., f32(i)/80.);
        let dir = cell_pos - p1.pos;
        let dst_sq = dot(dir, dir);
        if dst_sq < 25. {continue;}
        let f = GRAV_FRC*cell.inner_mass/dst_sq;
        p1.vel += f*dir;
    }
    if p1.pos.x+p1.vel.x+PART_RAD > 800./2. || p1.pos.x+p1.vel.x-PART_RAD < -800./2. {
        p1.vel.x *= -1.;
    }
    if p1.pos.y+p1.vel.y+PART_RAD > 600./2. || p1.pos.y+p1.vel.y-PART_RAD < -600./2. {
        p1.vel.y *= -1.;
    }
    p1.pos += p1.vel;
    out.particles[id.x] = p1;
    return;
}
    // for (var i: u32=u32(0); i < u32(arrayLength(&in.particles)); i+=u32(1)) {
    //     if i == id.x {continue;}
    //     let p2 = in.particles[i];
    //     let dir = p2.pos - p1.pos;
    //     let dst_sq = dot(dir, dir);
    //     if dst_sq < 25. {continue;}
    //     let f = 0.1/dst_sq;
    //     p1.vel += f*dir;
    // }