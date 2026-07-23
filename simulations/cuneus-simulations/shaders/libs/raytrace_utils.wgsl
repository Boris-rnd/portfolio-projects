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

