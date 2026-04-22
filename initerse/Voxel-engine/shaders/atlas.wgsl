struct Fragment {
    @builtin(position)  pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    @location(0) pos: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(3) rect: vec4<f32>,
    @location(4) atlas_rect: vec4<f32>,
) -> Fragment {
    var out: Fragment;
    out.pos = vec4<f32>(pos.x*rect.z+rect.x,pos.y*rect.w+rect.y, 0.,1.);
    out.tex_coords = vec2(tex_coords.x*atlas_rect.z+atlas_rect.x, tex_coords.y*atlas_rect.w+atlas_rect.y);
    return out;
}

// const TEXTURE_BATCH_SIZE
@group(0) @binding(0)
var texture_atlas: texture_2d<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: Fragment) -> @location(0) vec4<f32> {
    let color: vec4<f32> = textureSample(texture_atlas, t_sampler, in.tex_coords);
    // Config.format is bgra, but is showed are rgba, so convert this here (It's ~zero cost)
    return vec4(color.b, color.g, color.r, color.a);
}

