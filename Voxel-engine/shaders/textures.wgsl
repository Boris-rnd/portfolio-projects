struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

struct Vertex {
    @location(0) pos: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}
struct Fragment {
    @builtin(position)  pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_id: u32,
}

@vertex
fn vs_main(
    in: Vertex,
    @location(3) rect: vec4<f32>,
    @location(4) id: u32,
) -> Fragment {
    var out: Fragment;
    out.tex_coords = in.tex_coords;
    out.pos = camera.view_proj * vec4<f32>(in.pos.x*rect.z+rect.x,in.pos.y*rect.w+rect.y, 0.,1.);
    out.tex_id = id;
    return out;
}

// const TEXTURE_BATCH_SIZE
@group(0) @binding(0)
var textures: texture_2d_array<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: Fragment) -> @location(0) vec4<f32> {
    let coords = in.tex_coords;

    // let color: vec4<f32> = textureLoad(textures, coords, idx, 0);
    let color: vec4<f32> = textureSample(textures, t_sampler, coords, in.tex_id);
    // Config.format is bgra, but is showed are rgba, so convert this here
    return vec4(color.b, color.g, color.r, color.a);
}

