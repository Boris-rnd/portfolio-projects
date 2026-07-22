struct Params {
    camera_pos_x: f32,
    camera_pos_y: f32,
    camera_pos_z: f32,
    camera_dir_x: f32,
    camera_dir_y: f32,
    camera_dir_z: f32,
    camera_zoom: f32,
    _pad0: u32,
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
// ----- START: libs/raytrace_utils.wgsl --------
// utils.wgsl
// Utility functions for WGSL shaders
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

fn set_face_normal(ray: Ray, outward_normal: vec3<f32>, r: HitRecord) -> HitRecord {
    var rec = r;
    let front_face = dot(ray.dir, outward_normal) < 0;
    rec.normal = outward_normal;
    if !front_face {
        rec.normal = -outward_normal;
    }
    return rec;
}

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

// fn chunk_depth_to_size(depth: u32) -> u32 {
//     return u32(pow(f32(CHUNK_SIZE), f32(depth)));
// }

// // Small depth = big size
// // ex: depth=1 -> root_chunk_size/4
// fn depth_to_chunk_size(depth: u32) -> u32 {
//     // Convert depth to chunk size (16, 8, 4, 2, 1)
//     return root_chunk_size() / chunk_depth_to_size(depth);
// }

// fn root_chunk_size() -> u32 {
//     return chunk_depth_to_size(cam.root_max_depth);
// }


// fn is_accumulating_frames() -> bool {
//     return cam.accum_frames > 20;
// }

// struct MapDataID {
//     array_array_idx: u32,
//     array_idx: u32,
// }

// fn size_to_array_array_idx(size: u32) -> u32 {
//     if size < 8 {
//         return 0u;
//     } else if size < 24 {
//         return 1u;
//     } else if size < 40 {
//         return 2u;
//     } else {
//         return 3u;
//     }
// }
// fn array_array_idx_to_prefix_size(array_array_idx: u32) -> u32 {
//     if array_array_idx == 0u {
//         return 0u;
//     } else if array_array_idx == 1u {
//         return 8u;
//     } else if array_array_idx == 2u {
//         return 24u;
//     } else {
//         return 40u;
//     }
// }
// fn get_block_data(idx: MapDataID) -> MapData {
//     if idx.array_array_idx == 0u {
//         return block_data0[idx.array_idx];
//     } else if idx.array_array_idx == 1u {
//         return block_data1[idx.array_idx];
//     } else if idx.array_array_idx == 2u {
//         return block_data2[idx.array_idx];
//     } else {
//         return block_data3[idx.array_idx];
//     }
    
// }
// fn arrayLengthBlockData(idx: u32) -> u32 {
//     if idx == 0u {
//         return arrayLength(&block_data0);
//     } else if idx == 1u {
//         return arrayLength(&block_data1);
//     } else if idx == 2u {
//         return arrayLength(&block_data2);
//     } else {
//         return arrayLength(&block_data3);
//     }
// }


/// Returns u32::MAX if not found
// fn get_data_idx_in_chunk(chunk: VoxelChunk, _idx: u32) -> MapDataID {
//     let local_idx = _idx/32u;
//     let local_bit = _idx%32u;
//     if (chunk.inner[local_idx] & (u32(1) << local_bit)) == 0u {
//         return MapDataID(4294967295u, 4294967295u);
//     }

//     var ones = 0u;
//     var i = 0u;
//     while i < local_idx {
//         ones += countOneBits(chunk.inner[i]);
//         i += 1u;
//     }
    
//     let curr_set_bits = countOneBits(((1u << local_bit) - 1u) & chunk.inner[local_idx]);
//     let chunk_idx = curr_set_bits + ones;
//     let curr_array = size_to_array_array_idx(chunk_idx);
//     let local_array_idx = chunk_idx - array_array_idx_to_prefix_size(curr_array);
//     return MapDataID(curr_array, chunk.prefix_in_block_data_array[curr_array] + local_array_idx);
// }
// /// Returns u32::MAX if not found / invalid idx in tails chain or from start
// /// Returns block data, not idx !
// fn get_block_data_follow_tails(idx: MapDataID) -> u32 {
//     var curr_idx = idx.array_idx;
//     for (var i=0;i<100;i++) {
//         if (curr_idx >= arrayLengthBlockData(idx.array_array_idx)) {break;}
//         let curr_data = get_block_data(MapDataID(idx.array_array_idx, curr_idx)).data;
//         if (curr_data&3u) == 3u { // Tail
//             curr_idx = u32(curr_data >> 2);
//         } else {
//             return curr_data;
//         }
//     }
//     return 4294967295u;
// }


fn count_ones(n: u32) -> u32 {
    var count = 0u;
    var x = n;
    while x != 0u {
        count += x & 1u;
        x >>= 1u;
    }
    return count;
}

fn at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.orig + t * ray.dir;
}

// struct Camera {
//     center: vec3<f32>,
//     direction: vec3<f32>,
//     fov: f32,
//     // root_max_depth: u32,
//     // accum_frames: u32,
//     // img_size: vec2<u32>,
// }
// struct Sphere {
//     pos: vec3<f32>,
//     rad: f32,
//     color: vec3<f32>,
// }
struct Ray {
    orig: vec3<f32>,
    dir: vec3<f32>,
}
// struct VoxelChunk {
//     // idx_in_parent: u32,
//     inner: array<u32, CHUNK_U32_COUNT>,
//     prefix_in_block_data_array: array<u32, 4>,
// }
// struct Voxel {
//     pos: vec3<f32>,
//     texture_id: u32,
// }
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

struct MapData {
    // 2 first bits = type:
    // 00=block
    // 01=chunk
    // 10=entity
    // 11=Tail
    data: u32,
}


struct Box {
    min: vec3<f32>,
    max: vec3<f32>,
    texture_id: u32,
}


// fn local_pos(chunk: VoxelChunk) -> u32 {
//     // Returns the local position of the chunk in the world
//     return chunk.idx_in_parent;
// }
// fn ivec3_local_pos(chunk: VoxelChunk) -> vec3<i32> {
//     // Returns the local position of the chunk in the world as an ivec3
//     return vec3<i32>(vec3(chunk.idx_in_parent % 4, (chunk.idx_in_parent / 4) % 4, (chunk.idx_in_parent / 16) % 4));
// }

// No tuples
// ----- END: libs/raytrace_utils.wgsl --------
@group(2) @binding(1) var<storage, read_write> atomic_buffer: array<atomic<u32>>;
struct Voxel {
    id: f32,
    pos_x: f32,
    pos_y:f32,
    pos_z: f32
}
@group(3) @binding(0) var<storage, read_write> voxels: array<Voxel>;

fn hit_sphere(center: vec3<f32>, radius: f32, r: Ray) -> bool {
    let oc = center - r.orig;
    let a = dot(r.dir, r.dir);
    let b = -2.0 * dot(r.dir, oc);
    let c = dot(oc, oc) - radius*radius;
    let discriminant = b*b - 4*a*c;
    return (discriminant >= 0);
}


fn ray_color(r: Ray) -> vec3<f32> {
    if (hit_sphere(vec3(0.,0.,-1.), 0.5, r)) {
        return vec3(1., 0., 0.);   
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
    let uv = (vec2<f32>(id.xy)/dims-vec2(0.5))*dims*2.;
    let camera_center = vec3<f32>(params.camera_pos_x, params.camera_pos_y, params.camera_pos_z);
    let camera_dir = vec3<f32>(params.camera_dir_x, params.camera_dir_y, params.camera_dir_z);
    var col = vec3<f32>(uv, 0.0);

    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (dims.x/dims.y);
    let viewport_u = vec3(viewport_width, 0., 0.);
    let viewport_v = vec3(0., -viewport_height, 0.);

    let pixel_delta_u = viewport_u/dims.x;
    let pixel_delta_v = viewport_v/dims.y;

    let viewport_upper_left = camera_center - vec3(0., 0., focal_length) - viewport_u/2. - viewport_v/2.;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);
    let pixel_loc = pixel00_loc + (uv.x * pixel_delta_u + uv.y * pixel_delta_v);
    let ray_dir = pixel_loc - camera_center;
    col = ray_color(Ray(camera_center, ray_dir+camera_dir));
    
    textureStore(output, id.xy, vec4<f32>(col, 1.0));
}