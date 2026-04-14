struct Vertex {
    @location(0) pos: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct Fragment {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(
    vertex: Vertex,
    @location(3) rect: vec4<f32>,
) -> Fragment {
    var frag: Fragment;
    frag.color = vec3(rect.x, rect.y, rect.x+rect.y);
    frag.pos = vec4<f32>(vertex.pos.x*rect.z+rect.x, vertex.pos.y*rect.w+rect.y, 0.0, 1.);
    return frag;
}


@fragment
fn fs_main(in: Fragment) -> @location(0) vec4<f32> {
    return vec4(1.0, 1.0, 1.0, 1.0);
}
