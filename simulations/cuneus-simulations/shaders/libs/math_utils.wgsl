// Math and binary logic utilities
fn cmple(v1: vec3<f32>, v2: vec3<f32>) -> vec3<bool> {
    return vec3(v1.x <= v2.x, v1.y <= v2.y, v1.z <= v2.z);
}
fn cmple_to_unit(v1: vec3<f32>, v2: vec3<f32>) -> vec3<f32> {
    var v = vec3(0.);
    if v1.x <= v2.x {v.x = 1.;}
    if v1.y <= v2.y {v.y = 1.;}
    if v1.z <= v2.z {v.z = 1.;}
    return v;
}
// fn cmple(v1: vec3<i32>, v2: vec3<i32>) -> vec3<bool> {
//     return vec3(v1.x <= v2.x,v1.y <= v2.y,v1.z <= v2.z);
// }

fn fastFloor(v: vec3<f32>) -> vec3<i32> {
    return vec3<i32>(select(v - 1.0, v, fract(v) >= vec3<f32>(0.0)));
}
fn count_bits_in_range(value: u32, start: u32, end: u32) -> u32 {
    // Create mask for the range we want (e.g., bits 1-10)
    let mask = ((1u << (end - start)) - 1u) << start;
    // Apply mask and get only the bits we want
    let masked = value & mask;
    
    // Count the bits using parallel bit counting
    var x = masked;
    x = x - ((x >> 1u) & 0x55555555u);
    x = (x & 0x33333333u) + ((x >> 2u) & 0x33333333u);
    x = (x + (x >> 4u)) & 0x0F0F0F0Fu;
    x = x + (x >> 8u);
    x = x + (x >> 16u);
    return x & 0x3Fu; // Get final count
}


fn count_ones(n: u32) -> u32 {
    var count = 0u;
    var x = n;
    while x != 0u {
        count += x & 1u;
        x >>= 1u;
    }
    return count;
}



var<private> rng_state: u32;

// Init RNG: high-entropy seed from pixel + frame + camera state
fn init_rng(pixel: vec2<u32>, frame: u32, cam_seed: u32) {
    var seed = pixel.x * 374761393u ^ pixel.y * 668265263u;
    seed = seed ^ frame * 362437u ^ cam_seed * 2246822519u;
    rng_state = wang_hash(seed | 1u); // ensure non-zero
}

// Improved Wang hash function
fn wang_hash(seed: u32) -> u32 {
    var s = seed;
    s = (s ^ 61u) ^ (s >> 16u);
    s = s + (s << 3u);
    s = s ^ (s >> 4u);
    s = s * 0x27d4eb2du;
    s = s ^ (s >> 15u);
    return s;
}

// PCG random number generator (high quality, fast)
fn pcg_random() -> u32 {
    let oldstate = rng_state;
    rng_state = oldstate * 747796405u + 2891336453u;
    let word = ((oldstate >> ((oldstate >> 28u) + 4u)) ^ oldstate) * 277803737u;
    return (word >> 22u) ^ word;
}

// Xorshift32 (corrected implementation)
fn xorshift32() -> u32 {
    var x = rng_state;
    x ^= x << 13u;
    x ^= x >> 17u;
    x ^= x << 5u;
    rng_state = x;
    return x;
}

// Main random float function [0, 1)
fn random_f32() -> f32 {
    return f32(pcg_random()) * (1.0 / 4294967296.0);
}

// Alternative using bitcast for better distribution
fn random_f32_uniform() -> f32 {
    let bits = wang_hash(rng_state);
    rng_state = xorshift32();
    let float_bits = (bits >> 9u) | 0x3f800000u; // [1.0, 2.0)
    return bitcast<f32>(float_bits) - 1.0;
}

// Random float in range [min, max)
fn rand(min: f32, max: f32) -> f32 {
    return min + random_f32_uniform() * (max - min);
}

// Box-Muller transform for normal distribution (useful for blur effects)
fn random_gaussian() -> vec2<f32> {
    let u1 = max(0.00001, random_f32()); // Avoid log(0)
    let u2 = random_f32();
    let r = sqrt(-2.0 * log(u1));
    let theta = 2.0 * 3.14159265359 * u2;
    return vec2<f32>(r * cos(theta), r * sin(theta));
}
fn vec3_rand(min: f32, max: f32) -> vec3<f32> {
    return vec3(rand(min, max), rand(min, max), rand(min, max));
}


fn div_euclid_v3(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3(div_euclid(a.x, b.x), div_euclid(a.y, b.y), div_euclid(a.z, b.z));
}

fn div_euclid(a: i32, b: i32) -> i32 {
    let q = a / b;
    let r = a % b;
    return q - select(0, 1, (r < 0) && (b > 0)) + select(0, 1, (r > 0) && (b < 0));
}
fn div_euclid_f32(a: f32, b: f32) -> f32 {
    let q = floor(a / b);
    return select(q - 1.0, q, a >= 0.0);

    // let q = a / b;
    // let r = a % b;
    // return q - select(0., 1., (r < 0.) && (b > 0.)) + select(0., 1., (r > 0) && (b < 0));
}

fn div_euclid_f32_v3(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        div_euclid_f32(a.x, b.x),
        div_euclid_f32(a.y, b.y),
        div_euclid_f32(a.z, b.z),
    );
}
fn rem_euclid(a: i32, b: i32) -> i32 {
    let r = a % b;
    return select(r, r + abs(b), r < 0);
}
fn rem_euclid_v3(a: vec3<i32>, b: vec3<i32>) -> vec3<i32> {
    return vec3(rem_euclid(a.x, b.x), rem_euclid(a.y, b.y), rem_euclid(a.z, b.z));
}
fn degrees_to_radians(deg: f32) -> f32 {
    return deg / 180. * 3.14159;
}

fn near_zero(v: vec3<f32>) -> bool {
    // Return true if the vector is close to zero in all dimensions.
    let s = 1e-8;
    return (abs(v.x) < s) && (abs(v.y) < s) && (abs(v.z) < s);
}
fn reflect(v: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    return v - 2. * dot(v, n) * n;
}

// var<private> rng_seed: f32 = 1.;

fn random_unit_vector() -> vec3<f32> {
    for (var i = 0; i < 5; i++) {
        let p = vec3_rand(-1., 1.);
        let lensq = dot(p, p);
        if 1e-160 < lensq && lensq <= 1 {
            return p / sqrt(lensq);
        }
    }
    return vec3(0.);
}

fn random_on_hemisphere(normal: vec3<f32>) -> vec3<f32> {
    let on_unit_sphere = random_unit_vector();
    if dot(on_unit_sphere, normal) > 0.0 { // In the same hemisphere as the normal
        return on_unit_sphere;
    } else {
        return -on_unit_sphere;
    }
}


fn random_in_unit_disk() -> vec3<f32> {
    for (var i = 0; i < 5; i++) {
        let p = vec3(rand(-1., 1.), rand(-1., 1.), 0.);
        if dot(p, p) < 1. {
            return p;
        }
    }
    return vec3(0.);
}