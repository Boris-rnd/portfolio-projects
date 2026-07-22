struct Params {
    gravity: f32,
    particle_size: u32,
    particle_count: u32,
    speed: f32,
    reset: u32,
    camera_pos_x: f32,
    camera_pos_y: f32,
    camera_zoom: f32,
    // _pad0: u32,
    // _pad1: u32,
    // _pad2: u32,
};
// ----- START: libs/common.wgsl --------
@group(0) @binding(0) var<uniform> time_data: TimeUniform;
@group(1) @binding(0) var output: texture_storage_2d<rgba16float, write>;
@group(1) @binding(1) var<uniform> params: Params;
@group(2) @binding(0) var<uniform> mouse: MouseUniform;

// Group 0: Per-Frame Data (Engine-Managed)
struct TimeUniform {
    time: f32,
    delta: f32,
    frame: u32,
    _padding: u32,
};

// Group 2: Global Engine Resources
struct MouseUniform {
    position: vec2<f32>,
    click: u32,
};


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

// Returns a pseudo-rng inside [0;1]
fn hash_rand(u: u32) -> f32 {
    return f32(hash(u)) / 4294967295.0;
}
fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let cutoff = vec3<f32>(0.04045);
    let below = c / 12.92;
    let above = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return mix(above, below, cutoff);
}
// ----- END: libs/common.wgsl --------
// ----- START: libs/utils.wgsl --------
// Group 3: User Data (Particles)
struct Particle {
    old_pos: vec2<f32>, // TODO: Vec2_64 doesn't seem to add that much precision...
    pos: vec2<f32>, // TODO: Vec2_64 doesn't seem to add that much precision...
    mass: f32,
    enabled: u32,
    _pad1: u32,
    _pad2: u32,
};



fn world_to_screen_pos(world_pos: vec2<f32>) -> vec2<u32> {
    return vec2<u32>((world_pos - vec2<f32>(params.camera_pos_x, params.camera_pos_y)) * vec2<f32>(textureDimensions(output)) * params.camera_zoom);
}

fn screen_to_world_pos(screen_pos: vec2<u32>) -> vec2<f32> {
    return vec2<f32>(((vec2<f32>(screen_pos)/params.camera_zoom)/vec2<f32>(textureDimensions(output)) + vec2<f32>(params.camera_pos_x, params.camera_pos_y)));
}

fn clear_screen_at(screen_pos: vec2<u32>) {
    let dims = textureDimensions(output);
    if (screen_pos.x >= dims.x || screen_pos.y >= dims.y) {
        return;
    }
    let pos_px = vec2<u32>(screen_pos.xy);
    textureStore(output, pos_px, vec4<f32>(vec3(0.0), 1.));

    let world_pos = screen_to_world_pos(pos_px);

    let idx = pos_px_to_buffer_idx(pos_px);
    let col = vec3<f32>(f32(atomicLoad(&screen_atomic_buffer[idx]))/10.);
    textureStore(output, pos_px, vec4<f32>(col, 1.));
    atomicStore(&screen_atomic_buffer[idx], 0);

    if abs(dot(world_pos,world_pos)-1.)<=0.01 {
        textureStore(output, pos_px, vec4<f32>(vec3(1.0), 1.));
    }
}


fn verlet_integration(p2: Particle, acc: vec2<f32>, dt: f32) -> Particle {
    var p = p2;
    let temp = p.pos;
    p.pos = 2.0 * p.pos - p.old_pos + acc * dt * dt;
    p.old_pos = temp;

    if dot(p.pos,p.pos) >= 1.0 {
        let dist = length(p.pos);
        let normal = p.pos / dist;

        p.pos = normal * 1.0;

        // Bounce
        // e = 1.0 (Perfect) | e < 1.0 (Damped) | e > 1.0 (Boosted)
        let e = 1.;
        let vel = p.pos - p.old_pos;
        let v_normal = dot(vel, normal) * normal;
        let v_tangent = vel - v_normal;
        let reflected_vel = v_tangent - e * v_normal;

        // Change old pos to create virtual vel
        p.old_pos = p.pos - reflected_vel;
    }
    return p;
}
fn euler_integration(p2: Particle, acc: vec2<f32>, dt: f32) -> Particle {
    var p = p2;
    p.old_pos += acc * dt;
    p.pos += p.old_pos * dt;

    // 2. Gestion de la contrainte (Cercle de rayon 1.0)
    let dist = length(p.pos);
    if dist >= 1.0 {
        let normal = p.pos / dist;

        // Repositionne sur le bord
        p.pos = normal * 1.0;

        // Inverse la composante normale de la vitesse (e = coefficient de rebond)
        let e = 1.;
        p.old_pos = p.old_pos - (1.0 + e) * dot(p.old_pos, normal) * normal;
    }
    return p;
}

// fn euler_integration64(p2: Particle, acc_f32: vec2<f32>, dt_f32: f32) -> Particle {
//     var p = p2;
//     let dt = f64_(dt_f32);
//     let acc = Vec2_64(f64_(acc_f32.x), f64_(acc_f32.y));

//     let acc_dt = vec2_64_mul_s(acc, dt);
//     p.old_pos = vec2_64_add(p.old_pos, acc_dt);

//     let vel_dt = vec2_64_mul_s(p.old_pos, dt);
//     p.pos = vec2_64_add(p.pos, vel_dt);

//     let dist = vec2_64_len(p.pos);
//     if ge_64(dist, F64_ONE) {
//         let normal = vec2_64_div_s(p.pos, dist);
//         p.pos = normal; // normal * 1.0

//         let e = F64_ONE;
//         let one_plus_e = sum64(F64_ONE, e); // 2.0
//         let d = vec2_64_dot(p.old_pos, normal);
//         let factor = mul64(one_plus_e, d);
//         let corr = vec2_64_mul_s(normal, factor);
//         p.old_pos = vec2_64_sub(p.old_pos, corr);
//     }
//     return p;
// }
// ----- END: libs/utils.wgsl --------
// ----- START: libs/fp64.wgsl --------

// Whitepaper: https://andrewthall.org/papers/dfp64_qf128.pdf
// WGSL port of https://github.com/visgl/luma.gl/blob/291a2fdfb1cfdb15405032b3dcbfbe55133ead61/modules/shadertools/src/modules/math/fp64/fp64-arithmetic.glsl.ts

const one_f32: f32 = 1.0;

struct fp64 {
	high: f32,
	low: f32,
}

// Divide float number to high and low floats to extend fraction bits
fn split64(a: f32) -> fp64 {
	let c = (f32(1u << 12u) + 1.0) * a;
	let a_big = c - a;
	let a_hi = c * one_f32 - a_big;
	let a_lo = a * one_f32 - a_hi;
	return fp64(a_hi, a_lo);
}
fn fp64_(a:f32) -> fp64{return split64(a);}

// Special sum operation when a > b
fn quickTwoSum(a: f32, b: f32) -> fp64 {
	let x = (a + b) * one_f32;
	let b_virt = (x - a) * one_f32;
	let y = b - b_virt;
	return fp64(x, y);
}

fn twoSum(a: f32, b: f32) -> fp64 {
	let x = (a + b);
	let b_virt = (x - a) * one_f32;
	let a_virt = (x - b_virt) * one_f32;
	let b_err = b - b_virt;
	let a_err = a - a_virt;
	let y = a_err + b_err;
	return fp64(x, y);
}

fn twoSub(a: f32, b: f32) -> fp64 {
	let s = (a - b);
	let v = (s * one_f32 - a) * one_f32;
	let err = (a - (s - v) * one_f32) * one_f32 - (b + v);
	return fp64(s, err);
}

fn twoProd(a: f32, b: f32) -> fp64 {
	let x = a * b;
	let a2 = split64(a);
	let b2 = split64(b);
	let err1 = x - (a2.high * b2.high * one_f32) * one_f32;
	let err2 = err1 - (a2.low * b2.high * one_f32) * one_f32;
	let err3 = err2 - (a2.high * b2.low * one_f32) * one_f32;
	let y = a2.low * b2.low - err3;
	return fp64(x, y);
}

fn sum64(a: fp64, b: fp64) -> fp64 {
	var s = twoSum(a.high, b.high);
	var t = twoSum(a.low, b.low);
	s.low += t.high;
	s = quickTwoSum(s.high, s.low);
	s.low += t.low;
	s = quickTwoSum(s.high, s.low);
	return s;
}

fn sub64(a: fp64, b: fp64) -> fp64 {
	var s = twoSub(a.high, b.high);
	var t = twoSub(a.low, b.low);
	s.low += t.high;
	s = quickTwoSum(s.high, s.low);
	s.low += t.low;
	s = quickTwoSum(s.high, s.low);
	return fp64(s.high, s.low);
}

fn mul64(a: fp64, b: fp64) -> fp64 {
	var p = twoProd(a.high, b.high);
	p.low += a.high * b.low;
	p.low += a.low * b.high;
	p = quickTwoSum(p.high, p.low);
	return p;
}

fn vec4_sub64(a: array<fp64, 4>, b: array<fp64, 4>) -> array<fp64, 4> {
	return array<fp64, 4>(
		sub64(a[0], b[0]),
		sub64(a[1], b[1]),
		sub64(a[2], b[2]),
		sub64(a[3], b[3]),
	);
}

fn vec4_dot64(a: array<fp64, 4>, b: array<fp64, 4>) -> fp64 {
	var v = array<fp64, 4>(mul64(a[0], b[0]),mul64(a[1], b[1]), mul64(a[2], b[2]), mul64(a[3], b[3]));

	return sum64(sum64(v[0], v[1]), sum64(v[2], v[3]));
}

// fn mat4_vec4_mul64(b: array<fp64, 16>, a: array<fp64, 4>) -> array<fp64, 4> {
// 	var res = array<fp64, 4>();
// 	var tmp = array<fp64, 4>();

// 	for (var i = 0u; i < 4u; i++) {
// 		for (var j = 0u; j < 4u; j++) {
// 			tmp[j] = b[j * 4u + i];
// 		}
// 		res[i] = vec4_dot64(a, tmp);
// 	}

// 	return res;
// }

fn toVec4(v: array<fp64, 4>) -> vec4f {
	return vec4f(
		v[0].high + v[0].low,
		v[1].high + v[1].low,
		v[2].high + v[2].low,
		v[3].high + v[3].low,
	);
}

fn mat64(high: mat4x4f, low: mat4x4f) -> array<fp64, 16> {
	return array<fp64, 16>(
		fp64(high[0][0], low[0][0]),
		fp64(high[0][1], low[0][1]),
		fp64(high[0][2], low[0][2]),
		fp64(high[0][3], low[0][3]),

		fp64(high[1][0], low[1][0]),
		fp64(high[1][1], low[1][1]),
		fp64(high[1][2], low[1][2]),
		fp64(high[1][3], low[1][3]),

		fp64(high[2][0], low[2][0]),
		fp64(high[2][1], low[2][1]),
		fp64(high[2][2], low[2][2]),
		fp64(high[2][3], low[2][3]),

		fp64(high[3][0], low[3][0]),
		fp64(high[3][1], low[3][1]),
		fp64(high[3][2], low[3][2]),
		fp64(high[3][3], low[3][3]),
	);
}

fn vec4_64(high: vec4f, low: vec4f) -> array<fp64, 4> {
	return array<fp64, 4>(
		fp64(high[0], low[0]),
		fp64(high[1], low[1]),
		fp64(high[2], low[2]),
		fp64(high[3], low[3]),
	);
}
struct Vec2_64 { x: fp64, y: fp64, }
const F64_ZERO = fp64(0.0,0.0);
const F64_ONE = fp64(1.0,0.0);
const F64_TWO = fp64(2.0,0.0);
const F64_HALF = fp64(0.5,0.0);

fn f64_(v: f32) -> fp64 { return fp64(v,0.0); }
fn neg64(a: fp64) -> fp64 { return fp64(-a.high,-a.low); }
fn ge_64(a: fp64, b: fp64) -> bool {
    if a.high > b.high { return true; }
    if a.high < b.high { return false; }
    return a.low >= b.low;
}
fn div64(a: fp64, b: fp64) -> fp64 {
    let q1 = a.high / b.high;
    var r = sub64(a, mul64(b, f64_(q1)));
    let q2 = r.high / b.high;
    r = sub64(r, mul64(b, f64_(q2)));
    let q3 = r.high / b.high;
    var q = quickTwoSum(q1, q2);
    q.low = q.low + q3;
    q = quickTwoSum(q.high, q.low);
    return q;
}
fn sqrt64(a: fp64) -> fp64 {
    let s0 = sqrt(a.high);
    var r = f64_(s0);
    r = mul64(sum64(r, div64(a, r)), F64_HALF);
    r = mul64(sum64(r, div64(a, r)), F64_HALF);
    return r;
}

fn vec2_64_add(a: Vec2_64, b: Vec2_64) -> Vec2_64 { return Vec2_64(sum64(a.x,b.x), sum64(a.y,b.y)); }
fn vec2_64_sub(a: Vec2_64, b: Vec2_64) -> Vec2_64 { return Vec2_64(sub64(a.x,b.x), sub64(a.y,b.y)); }
fn vec2_64_mul_s(v: Vec2_64, s: fp64) -> Vec2_64 { return Vec2_64(mul64(v.x,s), mul64(v.y,s)); }
fn vec2_64_div_s(v: Vec2_64, s: fp64) -> Vec2_64 { return Vec2_64(div64(v.x,s), div64(v.y,s)); }
fn vec2_64_dot(a: Vec2_64, b: Vec2_64) -> fp64 { return sum64(mul64(a.x,b.x), mul64(a.y,b.y)); }
fn vec2_64_len(v: Vec2_64) -> fp64 { return sqrt64(vec2_64_dot(v,v)); }

fn fp64_to_f32(a: fp64) -> f32 { return a.high + a.low; }
// ----- END: libs/fp64.wgsl --------

@group(2) @binding(1) var<storage, read_write> screen_atomic_buffer: array<atomic<u32>>;
@group(3) @binding(0) var<storage, read_write> particles: array<Particle>;
// @group(3) @binding(1) var<storage, read_write> particles_grid: array<ParticleGrid>;


@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    if params.speed==0. {return;}
    let i = id.x;
    if (i >= params.particle_count) {
        return;
    }

    var p = particles[i];

    if (params.reset > 0u || time_data.frame == 0u) {
        p.pos = vec2<f32>((0.45)+(f32(i))/(f32(params.particle_count)/0.15), 0.);
        // let x = sum64(f64_(0.45), div64(f64_(f32(i)), div64(f64_(f32(params.particle_count)), f64_(0.15))));
        // p.pos = Vec2_64(x, f64_(0.));
        p.old_pos = p.pos; // vel=0
        p.enabled = 1;
        p.mass = 1.0;
    } else {
        if p.enabled == 0 {return;}
        let dt = time_data.delta * params.speed;

        var acc = vec2<f32>(0.0);
        acc.y += params.gravity;
        // p = verlet_integration(p, acc, dt);
        p = euler_integration(p, acc, dt);

    }
    particles[i] = p;

    count_particle(p);
}

fn pos_px_to_buffer_idx(pos_px: vec2<u32>) -> u32 {
    return pos_px.x+pos_px.y*textureDimensions(output).x;
}
fn count_particle(p: Particle) {
    let pos_px = world_to_screen_pos(p.pos);
    let idx = pos_px_to_buffer_idx(pos_px);
    atomicAdd(&screen_atomic_buffer[idx], 1u);
}


@compute @workgroup_size(16, 16)
fn clear_screen(@builtin(global_invocation_id) id: vec3<u32>) {
    clear_screen_at(id.xy);
}

@compute @workgroup_size(64)
fn splat(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.particle_count) {return;}

    var p = particles[i];
    if p.enabled == 0 {return;}
    render_particle(p);
}

fn render_particle(p: Particle) {
    let dims = textureDimensions(output);
    let pos = p.pos;
    // let pos = vec2<f32>(fp64_to_f32(fp64_(p.pos.x)),fp64_to_f32(fp64_(p.pos.y)));
    let pos_px = world_to_screen_pos(pos);

    let ps = i32(params.particle_size);
    // let half_size = i32(p.mass/1000.);
    // textureStore(output, vec2<i32>(pos_px.xy), vec4<f32>(p.mass/100., 0., 1., 1.));

    for (var y = -ps+1; y < ps; y++) {
        for (var x = -ps+1; x < ps; x++) {
            let disp = vec2(x, y);
            let current = vec2<i32>(pos_px.xy) + disp;
            if (current.x >= 0i && current.y >= 0i && u32(current.x) < dims.x && u32(current.y) < dims.y) {
                let dst_sq = f32(dot(disp,disp));
                let normalized_dst = dst_sq / f32(ps*ps);
                let intensity = vec3(f32(1.0)); //  - normalized_dst
                // let prev_intensity = textureLoad(output, current).xyz;
                if normalized_dst >= 0.5 {continue;}
                let color = vec3(vec2<f32>(pos_px)/vec2<f32>(dims), f32(p.mass));

                textureStore(output, current, vec4<f32>(color, 1.0));
            }
        }
    }
}
