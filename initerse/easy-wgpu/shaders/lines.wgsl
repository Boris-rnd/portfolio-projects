struct Fragment {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(
    @location(0) point: vec2<f32>,
) -> Fragment {
    var frag: Fragment;
    frag.color = vec3(0.);
    frag.pos = vec4(point.x, point.y, 0.0, 1.);
    return frag;
}


@fragment
fn fs_main(in: Fragment) -> @location(0) vec4<f32> {
    return vec4(1.0, 1.0, 1.0, 1.0);
}
