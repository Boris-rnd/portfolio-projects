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
include!("libs/common.wgsl");
include!("libs/raytrace_utils.wgsl");
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
