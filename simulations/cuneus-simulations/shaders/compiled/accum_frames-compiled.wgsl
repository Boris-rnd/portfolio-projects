@group(3) @binding(5) var<storage, read_write> accum_frames: array<u32>;

fn is_accumulating_frames() -> bool {
    return params.accum_frames > 20;
}
fn accum_frames_coords_to_idx(coords: vec2<u32>) -> u32 {
    return textureDimensions(output).x*coords.y+coords.x;
}
fn accum_frames_reset(coords: vec2<u32>) {
    if (params.accum_frames==0) {
        let idx = accum_frames_coords_to_idx(coords);
        accum_frames[idx] = 0;
    }
}
fn get_previous_color(coords: vec2<u32>) -> vec3<f32> {
    let idx = accum_frames_coords_to_idx(coords);
    let r = f32(accum_frames[idx]&0xFF)/255.;
    let g = f32((accum_frames[idx]>>8)&0xFF)/255.;
    let b = f32((accum_frames[idx]>>16)&0xFF)/255.;
    return vec3(r,g,b);
}
fn set_previous_color(coords: vec2<u32>, color: vec3<f32>) {
    let idx = accum_frames_coords_to_idx(coords);
    let r = u32(color.r*255.)&0xFF;
    let g = u32(color.g*255.)&0xFF;
    let b = u32(color.b*255.)&0xFF;
    let compacted = r | (g<<8) | (b<<16);
    accum_frames[idx] = compacted;
}