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
const INVALID_BOX_HIT: f32 = 1e30;
const BOX_NO_HIT: f32 = 2e30;
// If t==INVALID_BOX_HIT, then hit record is invalid
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
    return HitRecord(vec3(0.), vec3(0.), INVALID_BOX_HIT, vec3(0.));
}
fn to_far_away_rec() -> HitRecord {
    return HitRecord(vec3(0.), vec3(0.), BOX_NO_HIT, vec3(0.));
}
fn depth_rec(depth: f32) -> HitRecord {
    return HitRecord(vec3(0.), vec3(0.), depth, vec3(0.));
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
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4 * a * c;
    return (discriminant >= 0);
}
fn hit_box_t(ray: Ray, bmin: vec3<f32>, bmax: vec3<f32>) -> f32 {
    let t135 = (bmax - ray.orig) / ray.dir;
    let t246 = (bmin - ray.orig) / ray.dir;

    let tmin = max(max(min(t135.x, t246.x), min(t135.y, t246.y)), min(t135.z, t246.z));
    let tmax = min(min(max(t135.x, t246.x), max(t135.y, t246.y)), max(t135.z, t246.z));

    if tmin > tmax || tmax < 0 {
        return BOX_NO_HIT;
    }
    return tmin;
}
// I used AI for this sry
fn hit_box_t_rotated(ray: Ray, bmin: vec3<f32>, bmax: vec3<f32>, rotation: vec3<f32>) -> f32 {
    // 1. Build the 3x3 rotation matrix from Euler angles (XYZ order)
    let c = cos(rotation);
    let s = sin(rotation);

    let m = mat3x3<f32>(
        vec3<f32>(c.y * c.z, c.y * s.z, -s.y),                                          // Column 0 (X axis)
        vec3<f32>(s.x * s.y * c.z - c.x * s.z, s.x * s.y * s.z + c.x * c.z, s.x * c.y), // Column 1 (Y axis)
        vec3<f32>(c.x * s.y * c.z + s.x * s.z, c.x * s.y * s.z - s.x * c.z, c.x * c.y)// Column 2 (Z axis)
    );

    // 2. Transform the ray into the box's local space
    // To rotate a ray, we rotate its direction and origin.
    // (Note: This assumes the box is centered at the origin. If it has a position,
    // subtract it from ray.orig before multiplying by the transpose).
    let local_dir = m * ray.dir;
    let local_orig = m * ray.orig;

    // 3. Create a local ray and use the fast AABB intersection
    let local_ray = Ray(local_orig, local_dir); // Adjust this to match your Ray struct constructor
    return hit_box_t(local_ray, bmin, bmax);
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
struct NormalAndUv {
    normal: vec3<f32>,
    uv: vec2<f32>
}
fn compute_normal_and_uv(point: vec3<f32>, center: vec3<f32>) -> NormalAndUv {
    // Vector from center to point (points outward)
    var local_normal = point - center;
    var normal = vec3(0.);
    var uv = vec2(0.);

    // Find the dominant axis
    if local_normal.x > abs(local_normal.y) && local_normal.x > abs(local_normal.z) { 
        uv = local_normal.zy; 
        normal = vec3(1.0, 0.0, 0.0); 
    } 
    else if local_normal.x < -abs(local_normal.y) && local_normal.x < -abs(local_normal.z) { 
        uv = local_normal.zy; 
        normal = vec3(-1.0, 0.0, 0.0); 
    } 
    else if local_normal.z > abs(local_normal.y) && local_normal.z > abs(local_normal.x) { 
        uv = local_normal.xy; 
        normal = vec3(0.0, 0.0, 1.0); 
    } 
    else if local_normal.z < -abs(local_normal.y) && local_normal.z < -abs(local_normal.x) { 
        uv = local_normal.xy; 
        normal = vec3(0.0, 0.0, -1.0); 
    } 
    else if local_normal.y > abs(local_normal.x) && local_normal.y > abs(local_normal.z) { 
        // Top face (flip Z for correct UV orientation)
        uv = vec2(local_normal.x, -local_normal.z); 
        normal = vec3(0.0, 1.0, 0.0);
    } 
    else if local_normal.y < -abs(local_normal.x) && local_normal.y < -abs(local_normal.z) { 
        // Bottom face
        uv = local_normal.xz; 
        normal = vec3(0.0, -1.0, 0.0); 
    } 
    else { 
        // Fallback (should rarely happen, but good for safety)
        normal = vec3(1.0, 1.5, 1.0); 
    } 

    return NormalAndUv(normal, uv);
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

    var data: u32 = box.texture_id;
    var light_intensity = vec3(1.);

    let normal_uv = compute_normal_and_uv(res.p, center);
    res.normal = normal_uv.normal;
    res.t = t;
    // data = data%7;
    if data < 5 {
        res.color = vec3(t/1., f32(chunk_idx)/500, normal_uv.uv.x);
    } else {
        let r = data & 0xFF;
        let g = (data >> 8) & 0xFF;
        let b = (data >> 16) & 0xFF;
        let metallic = (data >> 24) & 1;
        res.color = vec3(f32(r) / 255., f32(g) / 255., f32(b) / 255.) * light_intensity;
    }
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
fn hit_voxel(ray: Ray) -> HitRecord {return hit_voxel_depth(ray, 1e30);}
fn hit_voxel_depth(ray: Ray, max_depth: f32) -> HitRecord {
    var miss = to_far_away_rec();

    // Initialise ray inside root chunk
    var posf = ray.orig;
    let world_min = vec3<f32>(0.0);
    let world_max = vec3<f32>(f32(root_chunk_size()));
    if all(ray.orig > world_min) && all(ray.orig < world_max) {
        posf = ray.orig;
    } else {

        var t = hit_box_t(ray, world_min, world_max);
        if t == BOX_NO_HIT {
            return miss;
        }
        posf = at(ray, t + 1e-3);
        // return HitRecord(vec3(0.), vec3(0.), t, vec3(0.));
    }

    // Optimization: pre-calculate step directions
    let ray_step = vec3<i32>(select(vec3(-1), vec3(1), ray.dir >= vec3(0.)));

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
        let local_pos = (posi - parent_pos) / vec3(child_size_i); // changed from div_euclid_v3... Doesn't seem to change anything
        // let local_pos = div_euclid_v3((posi - parent_pos), vec3(child_size_i));
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
                // if iter>100 {
                //     return valid_rec(vec3(1., 1., 0.));
                // }
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
                var res = hit_box_gen(ray, Box(vec3<f32>(posi), vec3<f32>(posi) + vec3(1.0), u32(curr_data >> 2)), chunk_idx, curr_chunks[curr_chunks_len - 1]);
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
        let eps = (1e-3 * S) * (1. + f32(iter) / 10.);
        // let eps = (1e-4 * f32(child_size_i));
        posf += ray.dir * (tStep + eps);
        if dot(posf,posf)>=max_depth*max_depth {return depth_rec(ray_t_from_pos(ray, posf));}
    }
    if iter >= max_iter {
        return to_far_away_rec();
    }
    // return valid_rec(vec3(0., 0., f32(iter)/500.));
    return miss;
}

fn ray_from_screen_pos(id: vec2<u32>, dims: vec2<f32>, num_wgs: vec2<u32>) -> Ray {
    let invoke_size = num_wgs*16;
    let scale = beam_workgroup_dispatch_size_to_scale(invoke_size);
    var uv = (vec2<f32>(id.xy*2)/dims-vec2(0.5))*dims*2.;
    uv.y = -uv.y;
    
    let cam_center = vec3(params.camera_pos_x, params.camera_pos_y, params.camera_pos_z);
    let cam_dir = vec3(params.camera_dir_x, params.camera_dir_y, params.camera_dir_z);

    var lookfrom = cam_center;     
    let lookat = cam_center + cam_dir;
    let vup = vec3(0., 1., 0.);
    let fov = degrees_to_radians(50.);
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
    let pixel_center = pixel00_loc + uv.x * pixel_delta_u + uv.y * pixel_delta_v;
    let ray_dir = normalize(pixel_center - lookfrom);
    return Ray(lookfrom, ray_dir);
}

fn ray_hit(r2: Ray, beam: bool) -> HitRecord {
    var r = r2;

    // TODO: Beam splitter
    let res_box = hit_box_t_rotated(r, vec3(10., 30., 1.), vec3(-40., 80., 400.), vec3(0., 0., degrees_to_radians(-45.)));
    if (res_box != BOX_NO_HIT) {
        let inter_point = r.orig + res_box * r.dir;
        r.orig = inter_point;
        r.dir.x *= 10.1;
        // r.dir.y += wang_random_f32(u32(res_box));
        r.dir = normalize(r.dir);
        // return vec3(100., 1., 1.) - inter_point;
    }

    // First hit (main voxel at screen)
    var res = hit_voxel(r);
    if (res.t == BOX_NO_HIT) { // Not found
        if all(res.p == vec3(1.)) {
            return valid_rec(vec3(1., 0., 1.)); // Error color
        }
        // We return a Record which holds a infinite depth -> written to by beam -> read by renderer and automatically renders skybox
        return res; // valid_rec(skybox(r.dir))
    }
    if beam {
        // beam_store_depth(id, res.t);
        // return valid_rec(vec3(1., 0.5, 1.));
        // We return the record directly because we can't compute bounces from low resolution
        return res;
    }
    // else{return valid_rec(res.normal);}
    var out_c = res.color;
    r.orig = at(r, res.t)+res.normal*0.01;
    let sun_hit = hit_voxel(Ray(r.orig, normalize(vec3(0.1, 0.91141, 0.1141))));

    let dir = r.dir+random_unit_vector()*0.5;
    r.dir = normalize(dir);
    
    var bounces = 1;
    if is_accumulating_frames() {bounces = 4;}
    // else {return out_c;}
    for (var b =1;b<bounces;b++) {
        var res = hit_voxel(r);
    
        if (res.t == BOX_NO_HIT) { // Not found
            break;
        }
        out_c *= res.color;
        // out_c = (out_c*f32(b)+res.color)/f32(b+1);
        r.orig = at(r, res.t)+res.normal*0.01;
        let dir = reflect(r.dir, res.normal)+random_on_hemisphere(res.normal)*0.5;
        r.dir = dir;
    }
    if sun_hit.t != BOX_NO_HIT { // No hit => light
        out_c *= 0.5;
    }

    return valid_rec(out_c); // Idk why but using reinhard_tone_map makes everything much slower
}

fn ray_color(r2: Ray) -> vec3<f32> {
    let rec = ray_hit(r2, false);
    if rec.t>1e29 && all(rec.color==vec3(0.)) {
        return skybox(r2.dir);
    }
    return rec.color;
}

fn ray_depth(r2: Ray) -> f32 {
    return hit_voxel(r2).t;
}

