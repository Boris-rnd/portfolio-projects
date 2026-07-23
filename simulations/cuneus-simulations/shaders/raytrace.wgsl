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
include!("libs/common.wgsl");
include!("libs/math_utils.wgsl");
include!("libs/raytrace_utils.wgsl");
include!("libs/voxel_utils.wgsl");
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
