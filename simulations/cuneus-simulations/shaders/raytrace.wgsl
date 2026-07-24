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
    accum_frames: u32,
    _pad2: u32,
    _pad3: u32,
};
include!("libs/common.wgsl");
include!("libs/math_utils.wgsl");
include!("libs/accum_frames.wgsl");
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




fn ray_color(r2: Ray) -> vec3<f32> {
    var r = r2;
    var out_c = vec3(0.);

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
            return vec3(1., 0., 1.); // Error color
        }
        return skybox(r.dir);
    } 
    // else{return res.normal;}
    out_c = res.color;
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

    // if (hit_sphere(vec3(0.,0.,1.), 0.5, r)) {
    //     return vec3(1., 0., 0.);   
    // }
    return out_c; // Idk why but using reinhard_tone_map makes everything much slower
}

@compute @workgroup_size(16, 16)
fn render(@builtin(global_invocation_id) id: vec3<u32>) {
    let udims = textureDimensions(output);
    if (id.x >= udims.x || id.y >= udims.y) {return;}
    init_rng(id.xy, time_data.frame, u32(params.camera_pos_x));
    let dims = vec2<f32>(udims);
    accum_frames_reset(id.xy);
    var uv = (vec2<f32>(id.xy)/dims-vec2(0.5))*dims*2.;
    uv.y = -uv.y;
    var col = vec3<f32>(0.0);
    
    let cam_center = vec3(params.camera_pos_x, params.camera_pos_y, params.camera_pos_z);
    let cam_dir = vec3(params.camera_dir_x, params.camera_dir_y, params.camera_dir_z);

    let lookfrom = cam_center;     
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

    let defocus_angle = 5.0;
    let defocus_radius = focal_length * tan(degrees_to_radians(defocus_angle / 2));
    let defocus_disk_u = u * defocus_radius;
    let defocus_disk_v = v * defocus_radius;

    let focus = false;
    var samples_per_pixel = 1;
    if is_accumulating_frames() {samples_per_pixel = 2;}

    for (var i = 0; i<samples_per_pixel; i++) {
        let offset = vec2((f32(i))/(f32(samples_per_pixel)/2));
        // let offset = vec2(rand(-0.5, 0.5), rand(-0.5, 0.5))*0.1;
        let pixel_center = pixel00_loc + (uv.x+offset.x) * pixel_delta_u + (uv.y+offset.y) * pixel_delta_v;
        let ray_dir = normalize(pixel_center - lookfrom);
        col += ray_color(Ray(lookfrom, ray_dir))/f32(samples_per_pixel);
    }

    if is_accumulating_frames() {
        let prev = get_previous_color(id.xy);
        col = (col+prev*f32(params.accum_frames-20))/f32(params.accum_frames-19);
    }
    textureStore(output, id.xy, vec4<f32>(col, 1.0));
    set_previous_color(id.xy, col);
}
