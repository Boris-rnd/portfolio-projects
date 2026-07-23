struct Params {
    camera_pos_x: f32,
    camera_pos_y: f32,
    camera_pos_z: f32,
    camera_dir_x: f32,
    camera_dir_y: f32,
    camera_dir_z: f32,
    camera_zoom: f32,
    fov: f32,
    root_max_depth: u32,
    // _pad: array<u32, 3>,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
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
// ----- START: libs/math_utils.wgsl --------
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
// ----- END: libs/math_utils.wgsl --------
// ----- START: libs/raytrace_utils.wgsl --------
// Raytrace utility functions

struct Ray {
    orig: vec3<f32>,
    dir: vec3<f32>,
}
fn at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.orig + t * ray.dir;
}
// Make sure ray.dir is normalized and != 0
// We return x but we can use other components as well
// fn ray_t_from_pos(ray: Ray, pos: vec3<f32>) -> f32 {
//     return abs(((pos - ray.orig) / ray.dir).x);
// }
fn ray_t_from_pos(ray: Ray, pos: vec3<f32>) -> f32 {
    let d = abs(ray.dir);
    if d.x >= d.y && d.x >= d.z {
        return (pos.x - ray.orig.x) / ray.dir.x;
    } else if d.y >= d.z {
        return (pos.y - ray.orig.y) / ray.dir.y;
    } else {
        return (pos.z - ray.orig.z) / ray.dir.z;
    }
}
// If t==1e30, then hit record is invalid
struct HitRecord {
    p: vec3<f32>,
    normal: vec3<f32>,
    t: f32,
    color: vec3<f32>,
}
fn valid_rec(color: vec3<f32>) -> HitRecord {
    return HitRecord(vec3(0.), vec3(0.), 0., color);
}
fn invalid_rec() -> HitRecord {
    return HitRecord(vec3(0.), vec3(0.), 1e30, vec3(0.));
}
fn to_far_away_rec() -> HitRecord {
    return HitRecord(vec3(1.), vec3(0.), 1e30, vec3(0.));
}


// fn local_pos(chunk: VoxelChunk) -> u32 {
//     // Returns the local position of the chunk in the world
//     return chunk.idx_in_parent;
// }
// fn ivec3_local_pos(chunk: VoxelChunk) -> vec3<i32> {
//     // Returns the local position of the chunk in the world as an ivec3
//     return vec3<i32>(vec3(chunk.idx_in_parent % 4, (chunk.idx_in_parent / 4) % 4, (chunk.idx_in_parent / 16) % 4));
// }

// struct Sphere {
//     pos: vec3<f32>,
//     rad: f32,
//     color: vec3<f32>,
// }
fn hit_sphere(center: vec3<f32>, radius: f32, r: Ray) -> bool {
    let oc = center - r.orig;
    let a = dot(r.dir, r.dir);
    let b = -2.0 * dot(r.dir, oc);
    let c = dot(oc, oc) - radius*radius;
    let discriminant = b*b - 4*a*c;
    return (discriminant >= 0);
}
const INVALID_BOX_HIT: f32 = 3*10e10;
fn hit_box_t(ray: Ray, bmin: vec3<f32>, bmax: vec3<f32>) -> f32 {
    let t135 = (bmax - ray.orig) / ray.dir;
    let t246 = (bmin - ray.orig) / ray.dir;

    let tmin = max(max(min(t135.x, t246.x), min(t135.y, t246.y)), min(t135.z, t246.z));
    let tmax = min(min(max(t135.x, t246.x), max(t135.y, t246.y)), max(t135.z, t246.z));

    if tmin > tmax || tmax < 0 {
        return INVALID_BOX_HIT;
    }
    return tmin;
}
fn sdBox(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}


fn set_face_normal(ray: Ray, outward_normal: vec3<f32>, r: HitRecord) -> HitRecord {
    var rec = r;
    let front_face = dot(ray.dir, outward_normal) < 0;
    rec.normal = outward_normal;
    if !front_face {
        rec.normal = -outward_normal;
    }
    return rec;
}


// struct DDAResult {
//     sideDist: vec3<f32>,
//     pos: vec3<i32>,
//     mask: vec3<f32>,
// }
// fn branchless_dda(sideDist: vec3<f32>, pos: vec3<i32>, deltaDist: vec3<f32>, rayStep: vec3<i32>) -> DDAResult {
//     var res = DDAResult(sideDist, pos, vec3(0.));
//     if sideDist.x < sideDist.y {
//         if sideDist.x < sideDist.z {
//             res.sideDist.x = sideDist.x + deltaDist.x;
//             res.pos.x = pos.x + rayStep.x;
//             res.mask = vec3(1., 0., 0.);
//         } else {
//             res.sideDist.z = sideDist.z + deltaDist.z;
//             res.pos.z = pos.z + rayStep.z;
//             res.mask = vec3(0., 0., 1.);
//         }
//     } else {
//         if sideDist.y < sideDist.z {
//             res.sideDist.y = sideDist.y + deltaDist.y;
//             res.pos.y = pos.y + rayStep.y;
//             res.mask = vec3(0., 1., 0.);
//         } else {
//             res.sideDist.z = sideDist.z + deltaDist.z;
//             res.pos.z = pos.z + rayStep.z;
//             res.mask = vec3(0., 0., 1.);
//         }
//     }
//     return res;
// }



// fn is_accumulating_frames() -> bool {
//     return cam.accum_frames > 20;
// }


// Better tone mapping function
fn reinhard_tone_map(color: vec3<f32>) -> vec3<f32> {
    // Extended Reinhard tone mapping
    let white_point = 2.0;
    let numerator = color * (1.0 + color / (white_point * white_point));
    let denominator = 1.0 + color;
    
    // Apply gamma correction
    return pow(numerator / denominator, vec3(1.0 / 2.2));
}
fn skybox(ray_dir: vec3<f32>) -> vec3<f32> {
    let a = 0.5 * (ray_dir.y + 1.0);
    var c = (1.0 - a) * vec3(1.0, 1.0, 1.0) + a * vec3(0.5, 0.7, 1.0);
    return c;
}


struct Box {
    min: vec3<f32>,
    max: vec3<f32>,
    texture_id: u32,
}
fn hit_box_gen(ray: Ray, box: Box, chunk_idx: u32, chunk: VoxelChunk) -> HitRecord {
    var res = invalid_rec();

    var t = hit_box_t(ray, box.min, box.max);
    if t == INVALID_BOX_HIT {
        
        return valid_rec(vec3(1., 0., 0.)); // No hit
    }
    res.t = t;
    res.p = at(ray, t);
    let center = (box.min + box.max) / 2.;
    
    var uv: vec2<f32>;
    var data: u32 = box.texture_id;
    var light_intensity = vec3(1.);
    var circle_normal = center - res.p;
    var n = normalize(circle_normal);
    var abs_n = abs(n);

    // if abs_n.x >= abs_n.y && abs_n.x >= abs_n.z {
    //     circle_normal = vec3(sign(n.x), 0.0, 0.0);
    //     uv = res.p.zy;
    // } else if abs_n.y >= abs_n.x && abs_n.y >= abs_n.z {
    //     circle_normal = vec3(0.0, sign(n.y), 0.0);
    //     uv = res.p.xz;
    // } else {
    //     circle_normal = vec3(0.0, 0.0, sign(n.z));
    //     uv = res.p.xy;
    // }
    if circle_normal.x > abs(circle_normal.y) && circle_normal.x > abs(circle_normal.z) { uv = (circle_normal).zy; circle_normal = vec3(1., 0., 0.); } else if circle_normal.x < -abs(circle_normal.y) && circle_normal.x < -abs(circle_normal.z) { uv = (circle_normal).zy; circle_normal = vec3(-1., 0., 0.); } else if circle_normal.z > abs(circle_normal.y) && circle_normal.z > abs(circle_normal.x) { uv = (circle_normal).xy; circle_normal = vec3(0., 0., 1.); } else if circle_normal.z < -abs(circle_normal.y) && circle_normal.z < -abs(circle_normal.x) { uv = (circle_normal).xy; circle_normal = vec3(0., 0., -1.); } else if (circle_normal.y) > abs(circle_normal.x) && (circle_normal.y) > abs(circle_normal.z) { // Bottom face 
        uv = (circle_normal).xz; circle_normal = vec3(0., 1., 0.); } else if circle_normal.y < -abs(circle_normal.x) && circle_normal.y < -abs(circle_normal.z) { uv = (circle_normal).xz; circle_normal = vec3(0., -1., 0.); } else { circle_normal = vec3(1., 1.5, 1.); } res.normal = circle_normal;

    res.normal = circle_normal;
    res.t = t;
    // data = data%7;
    let r = data & 0xFF;
    let g = (data >> 8) & 0xFF;
    let b = (data >> 16) & 0xFF;
    let metallic = (data >> 24) & 1;
    res.color = vec3(f32(r) / 255., f32(g) / 255., f32(b) / 255.)*light_intensity;
    // if data > 5 {
    //     res.color = vec3(f32(data) / 255., f32(data) / 255., f32(data) / 255.);
    // } else {
    //     let texcoord = vec2<u32>((uv + vec2(0.5)) * 32.0);
    //     // let srgb = textureLoad(atlas, texcoord, data).xyz;
    //     let srgb = (textureLoad(atlas, texcoord, data).xyz - vec3(0.5)) * 1.2 + vec3(0.5);
    //     res.color = srgb_to_linear(srgb);
    //     if data==2 {
    //         res.color *= 4.;
    //     }
    // }
    return res;
}

// ----- END: libs/raytrace_utils.wgsl --------
// ----- START: libs/voxel_utils.wgsl --------
// Utilities around voxels and world data handling

// 2, 4, 8, ...
const CHUNK_SIZE: u32 = 4; // TODO: make this shared with CPU code in world
const CHUNK_U32_COUNT = CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE/32;
// const CHUNK_U32_COUNT = 1;
const CHUNK_MASK = CHUNK_SIZE - 1u;
const CHUNK_SHIFT = countOneBits(CHUNK_MASK);
struct Voxel {
    id: f32,
    pos: vec3<f32>,
}

struct MapData {
    // 2 first bits = type:
    // 00=block
    // 01=chunk
    // 10=entity
    // 11=Tail
    data: u32,
}
struct VoxelChunk {
    // idx_in_parent: u32,
    inner: array<u32, CHUNK_U32_COUNT>,
    prefix_in_block_data_array: array<u32, 4>,
}
struct DataResult {
    data: u32,
    depth: u32,
}

fn set_bit_if_in_range(bit_mask: array<u32, CHUNK_U32_COUNT>, bit_pos: vec3<u32>) -> array<u32, CHUNK_U32_COUNT> {
    if (any(bit_pos >= vec3<u32>(CHUNK_SIZE))) {
        return bit_mask;
    }
    let chunk_idx = bit_pos.x | (bit_pos.y << CHUNK_SHIFT) | (bit_pos.z << (CHUNK_SHIFT * 2u));
    let word = chunk_idx / 32u;
    let bit = 1u << (chunk_idx % 32u);
    var b = bit_mask;
    b[word] = b[word] | bit;
    return b;
}

fn gen_chunk_mask(ray: Ray, start_pos_local: vec3<f32>) -> array<u32, CHUNK_U32_COUNT> {
    var posf = start_pos_local;
    let dir = ray.dir;
    let rcp = 1.0 / dir;
    var mask = array<u32, CHUNK_U32_COUNT>(4294967295, 4294967295);

    loop {
        // integer voxel inside chunk, safe conv via floor+clamp
        let pf = floor(posf);
        if any(pf < vec3(0.)) || any(pf > vec3(f32(CHUNK_SIZE)-1.)) {break;}
        let posi = vec3<u32>(pf);

        // set current voxel and immediate forward neighbors (x+, y+, z+)
        mask = set_bit_if_in_range(mask, posi);
        mask = set_bit_if_in_range(mask, posi + vec3<u32>(1u, 0u, 0u));
        mask = set_bit_if_in_range(mask, posi + vec3<u32>(0u, 1u, 0u));
        mask = set_bit_if_in_range(mask, posi + vec3<u32>(0u, 0u, 1u));

        // compute distance to next voxel boundary along each axis (respect sign)
        let idxf = floor(posf);
        let next = select(idxf, idxf + vec3<f32>(1.0), dir > vec3<f32>(0.0));
        let tMax = (next - posf) * rcp;
        let tStep = min(tMax.x, min(tMax.y, tMax.z));

        // stop if non-finite or we will leave chunk
        if (!(tStep < 1e20)) { break; } // guard
        // step a little less than full boundary to avoid re-hitting same voxel due to float error
        let eps = 1e-4*4;
        posf = posf + dir * (tStep + eps);
    }
    return mask;
}

fn local_pos_to_ivec3(idx: u32) -> vec3<u32> {
    let x = idx & u32(CHUNK_MASK);
    let y = (idx >> CHUNK_SHIFT) & u32(CHUNK_MASK);
    let z = (idx >> (CHUNK_SHIFT * 2u)) & u32(CHUNK_MASK);
    return vec3<u32>(x, y, z);
}
fn ivec3_to_local_pos(pos: vec3<u32>) -> u32 {
    return pos.x | (pos.y << CHUNK_SHIFT) | (pos.z << (CHUNK_SHIFT * 2u));
}



/// Returns u32::MAX if not found
fn get_data_idx_in_chunk(chunk: VoxelChunk, _idx: u32) -> MapDataID {
    let local_idx = _idx/32u;
    let local_bit = _idx%32u;
    if (chunk.inner[local_idx] & (u32(1) << local_bit)) == 0u {
        return MapDataID(4294967295u, 4294967295u);
    }

    var ones = 0u;
    var i = 0u;
    while i < local_idx {
        ones += countOneBits(chunk.inner[i]);
        i += 1u;
    }
    
    let curr_set_bits = countOneBits(((1u << local_bit) - 1u) & chunk.inner[local_idx]);
    let chunk_idx = curr_set_bits + ones;
    let curr_array = size_to_array_array_idx(chunk_idx);
    let local_array_idx = chunk_idx - array_array_idx_to_prefix_size(curr_array);
    return MapDataID(curr_array, chunk.prefix_in_block_data_array[curr_array] + local_array_idx);
}
// /// Returns u32::MAX if not found / invalid idx in tails chain or from start
// /// Returns block data, not idx !
fn get_block_data_follow_tails(idx: MapDataID) -> u32 {
    var curr_idx = idx.array_idx;
    for (var i=0;i<100;i++) {
        if (curr_idx >= arrayLengthBlockData(idx.array_array_idx)) {break;}
        let curr_data = get_block_data(MapDataID(idx.array_array_idx, curr_idx)).data;
        if (curr_data&3u) == 3u { // Tail
            curr_idx = u32(curr_data >> 2);
        } else {
            return curr_data;
        }
    }
    return 4294967295u;
}

struct MapDataID {
    array_array_idx: u32,
    array_idx: u32,
}

fn size_to_array_array_idx(size: u32) -> u32 {
    if size < 8 {
        return 0u;
    } else if size < 24 {
        return 1u;
    } else if size < 40 {
        return 2u;
    } else {
        return 3u;
    }
}
fn array_array_idx_to_prefix_size(array_array_idx: u32) -> u32 {
    if array_array_idx == 0u {
        return 0u;
    } else if array_array_idx == 1u {
        return 8u;
    } else if array_array_idx == 2u {
        return 24u;
    } else {
        return 40u;
    }
}
fn get_block_data(idx: MapDataID) -> MapData {
    // if idx.array_array_idx == 0u {
    //     return block_data0[idx.array_idx];
    // } else if idx.array_array_idx == 1u {
    //     return block_data1[idx.array_idx];
    // } else if idx.array_array_idx == 2u {
    //     return block_data2[idx.array_idx];
    // } else {
    //     return block_data3[idx.array_idx];
    // }
    return MapData(0u); //TODO
}
fn arrayLengthBlockData(idx: u32) -> u32 {
    return 0; //TODO
    // if idx == 0u {
    //     return arrayLength(&block_data0);
    // } else if idx == 1u {
    //     return arrayLength(&block_data1);
    // } else if idx == 2u {
    //     return arrayLength(&block_data2);
    // } else {
    //     return arrayLength(&block_data3);
    // }
}

fn chunk_depth_to_size(depth: u32) -> u32 {
    return u32(pow(f32(CHUNK_SIZE), f32(depth)));
}

// Small depth = big size
// ex: depth=1 -> root_chunk_size/4
fn depth_to_chunk_size(depth: u32) -> u32 {
    // Convert depth to chunk size (16, 8, 4, 2, 1)
    return root_chunk_size() / chunk_depth_to_size(depth);
}

fn root_chunk_size() -> u32 {
    return chunk_depth_to_size(params.root_max_depth);
}


// ----- END: libs/voxel_utils.wgsl --------
// @group(2) @binding(1) var<storage, read_write> atomic_buffer: array<atomic<u32>>;
// Read_write but we should'nt write to it from the shader... For now
// @group(3) @binding(0) var<storage, read_write> voxels: array<Voxel>;
@group(3) @binding(0) var<storage, read_write> voxel_chunks: array<VoxelChunk>;
@group(3) @binding(1) var<storage, read_write> block_data0: array<MapData>;
@group(3) @binding(2) var<storage, read_write> block_data1: array<MapData>;
@group(3) @binding(3) var<storage, read_write> block_data2: array<MapData>;
@group(3) @binding(4) var<storage, read_write> block_data3: array<MapData>;


fn prev_hit(ray: Ray) -> HitRecord {
    var miss = invalid_rec();

    // Initialise ray inside root chunk
    var posf = ray.orig;
    let world_min = vec3<f32>(0.0);
    let world_max = vec3<f32>(f32(root_chunk_size()));
    if all(ray.orig > world_min) && all(ray.orig < world_max) {
        posf = ray.orig;
    } else {

        var t = hit_box_t(ray, world_min, world_max);
        if t == INVALID_BOX_HIT {
            return miss;
        }
        posf = at(ray, t + 1e-3);
    }
    if (true) {
        return valid_rec(posf, ray.dir, 0u);
    }

    // Setup stacks for the descent of sparse tree
    var curr_chunks = array<VoxelChunk, 6>();
    var parent_pos_stack: array<vec3<i32>, 7>;

    parent_pos_stack[0] = vec3<i32>(0);
    var curr_depth = 1u;
    var curr_chunks_len = 1u;
    curr_chunks[0] = voxel_chunks[0];
    var chunk_size = root_chunk_size();
    
    // Main traversal
    var stepf = sign(ray.dir);
    let rcp = 1. / ray.dir;




    // Hard cap to avoid infinite loops
    var max_iter = 500;
    // if is_accumulating_frames() == true {
    //     max_iter = 1000;
    // }
    var iter = 0;
    for (; iter < max_iter; iter = iter + 1) {
        let posi = vec3<i32>(posf);
        let parent_pos = parent_pos_stack[curr_depth - 1u];
        let child_size_i = i32(depth_to_chunk_size(curr_depth));
        let local_pos = div_euclid_v3(posi - parent_pos, vec3(child_size_i));
        // Check if outside of current chunk
        if any((posi - parent_pos) < vec3(0)) || any(local_pos >= vec3(i32(CHUNK_SIZE))) {
            // Outside of previous chunk, if curr_depth==1, then outside of root chunk so won't hit anything else
            if curr_depth == 1u { 
                break;
            }
            // Ascent
            curr_depth -= 1u;
            curr_chunks_len -= 1u;
            continue;
        }

        var chunk_idx = u32(local_pos.x) | (u32(local_pos.y) << CHUNK_SHIFT) | (u32(local_pos.z) << (CHUNK_SHIFT * 2));
        // Checks if bit is set, if so computes the idx, else returns U32::MAX (which will be bigger than arrayLength)
        let map_data_idx = get_data_idx_in_chunk(curr_chunks[curr_chunks_len - 1u], chunk_idx);
        if map_data_idx.array_idx < arrayLengthBlockData(map_data_idx.array_array_idx) {
            let curr_data = get_block_data_follow_tails(map_data_idx);
            if curr_data == 4294967295u { // Never happens but maybe one day i'll introduce a breaking bug
                return valid_rec(vec3(1., 0., 1.));
            }
            // let curr_data = get_block_data(MapDataID(map_data_idx.array_array_idx, map_data_idx.array_idx)).data;

            let ty = curr_data & 3u;
            if ty == 1u { // Chunk, so we descend into it
                parent_pos_stack[curr_depth] = parent_pos + vec3<i32>(
                    local_pos.x * child_size_i,
                    local_pos.y * child_size_i,
                    local_pos.z * child_size_i
                );
                curr_chunks[curr_chunks_len] = voxel_chunks[curr_data >> 2];
                curr_chunks_len += 1u;
                curr_depth += 1u;
                continue; // IMPORTANT: re-evaluate at new depth
            } else if ty == 2u { // Block
                var res = hit_box_gen(ray, Box(vec3<f32>(posi), vec3<f32>(posi) + vec3(1.0), u32(curr_data >> 2)), chunk_idx, curr_chunks[curr_chunks_len-1]);
                // res = edge_chunk_shadows(res, chunk_idx, curr_chunks[curr_chunks_len-1], curr_chunks[curr_chunks_len-2]);
                return res; // making posi = 0 and rb 10000 is fun
            }
        }
        // Should be useless check but I like to keep it
        // Check if we have found something
        if map_data_idx.array_array_idx != 4294967295u {
            return valid_rec(vec3(0., 1., 1.));
        }
        let S = f32(child_size_i);
        let world_pos_in_parent = posf - vec3<f32>(parent_pos);

        // handle zeros
        let inf = 1e30;
        let idxf = floor(world_pos_in_parent / S);
        let next = select(idxf * S, (idxf + vec3(1.)) * S, stepf > vec3(0.));
        var tMax = (next - world_pos_in_parent) * rcp;
        let tStep = min(tMax.x, min(tMax.y, tMax.z));
        if !(tStep < inf) {
            return valid_rec(vec3(1., 0., 1.));
        }

        // nudge with scale-aware epsilon
        let eps = (1e-3 * S)*(1. + f32(iter)/100.);
        posf += ray.dir * (tStep + eps);
    }
    if iter >= max_iter {
        return to_far_away_rec();
    }
    // return valid_rec(vec3(0., 0., f32(iter)/500.));
    return miss;
}


fn ray_color(r: Ray) -> vec3<f32> {
    var res = prev_hit(r);

    if (res.t == INVALID_BOX_HIT) {
        if all(res.p == vec3(1., 1., 1.)) {
            return vec3(1., 0., 0.); // Error color
        }
        // Sky contribution
        return vec3(1., 1., 0.);
    }

    if (hit_sphere(vec3(0.,0.,1.), 0.5, r)) {
        return vec3(1., 0., 0.);   
    }
    let res_box = hit_box_t(r, vec3(-1., -1., -1.), vec3(1., 1., 1.));
    if (res_box != INVALID_BOX_HIT) {
        let inter_point = r.orig + res_box * r.dir;
        return vec3(100., 1., 1.) - inter_point;
    }
    let unit_direction = normalize(r.dir);
    let a = 0.5*(unit_direction.y + 1.0);
    return (1.0-a)*vec3(1.0) + a*vec3(0.5, 0.7, 1.0);
}

@compute @workgroup_size(16, 16)
fn render(@builtin(global_invocation_id) id: vec3<u32>) {
    let udims = textureDimensions(output);
    if (id.x >= udims.x || id.y >= udims.y) {return;}
    let dims = vec2<f32>(udims);
    var uv = (vec2<f32>(id.xy)/dims-vec2(0.5))*dims*2.;
    uv.y = -uv.y;
    var col = vec3<f32>(uv, 0.0);
    
    let cam_center = vec3(params.camera_pos_x, params.camera_pos_y, params.camera_pos_z);
    let cam_dir = vec3(params.camera_dir_x, params.camera_dir_y, params.camera_dir_z);

    let lookfrom = cam_center;     
    let lookat = cam_center + cam_dir;
    let vup = vec3(0., 1., 0.);
    let fov = degrees_to_radians(90.);
    let h = tan(fov / 2);
    let focal_length = 2.0;
    let viewport_height = 2. * h * focal_length;
    let viewport_width = viewport_height * (dims.x/dims.y);

    let w = normalize(lookfrom - lookat);
    let u = normalize(cross(vup, w));
    let v = cross(w, u);


    let viewport_u = viewport_width * u; // Vector across viewport horizontal edge
    let viewport_v = viewport_height * (v); // Vector down viewport vertical edge

    let pixel_delta_u = viewport_u/dims.x;
    let pixel_delta_v = viewport_v/dims.y;

    let viewport_upper_left = lookfrom - focal_length * w - viewport_u / 2 - viewport_v / 2;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    let defocus_angle = 5.0;
    let defocus_radius = focal_length * tan(degrees_to_radians(defocus_angle / 2));
    let defocus_disk_u = u * defocus_radius;
    let defocus_disk_v = v * defocus_radius;

    let focus = false;
    var samples_per_pixel = 1;

    let pixel_center = pixel00_loc + ((uv.x) * pixel_delta_u) + ((uv.y) * pixel_delta_v);
    let ray_dir = normalize(pixel_center - lookfrom);
    col = ray_color(Ray(lookfrom, ray_dir));
    
    textureStore(output, id.xy, vec4<f32>(col, 1.0));
}