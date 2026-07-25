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

// Beam (lower resolution => approximate depth)
const BEAM_SCALE: u32=2;
@group(3) @binding(6) var<storage, read_write> max_depth: array<f32>;
fn beam_workgroup_dispatch_size_to_scale(w_size: vec2<u32>) -> u32 {
    let full_scale = textureDimensions(output);
    return u32(round(f32(full_scale.x)/f32(w_size.x))); // TODO: use floats to make sure no rounding errors + use x and y to verify
}

fn beam_local_coords_to_idx(coords: vec2<u32>, stride: u32) -> u32 {
    return stride*coords.y+coords.x;
}
// Call this with full size coords (because next pass will have 2x resolution)
fn beam_store_depth(scr_coords: vec2<u32>, depth: f32) {
    let full_scale = textureDimensions(output);
    let scale = BEAM_SCALE; 
    // scr_coords are already low-res here, so use the low-res dimensions directly
    max_depth[beam_local_coords_to_idx(scr_coords, full_scale.x / scale)] = depth;
}
fn beam_load_prev_depth(scr_coords: vec2<u32>) -> f32 {
    let full_scale = textureDimensions(output);
    let scale = BEAM_SCALE;
    let low_res_coords = scr_coords / scale;
    return max_depth[beam_local_coords_to_idx(low_res_coords, full_scale.x / scale)];
}
// fn beam_get_previous_depth(scr_coords: vec2<u32>, w_size: vec2<u32>) -> f32 {
//     let scale = beam_workgroup_dispatch_size_to_scale(w_size);
//     return max_depth[beam_local_coords_to_idx(scr_coords / scale, w_size)];
// }
// fn beam_set_previous_depth(scr_coords: vec2<u32>, w_size: vec2<u32>, depth: f32) {
//     let scale = beam_workgroup_dispatch_size_to_scale(w_size);
//     max_depth[beam_local_coords_to_idx(scr_coords / scale, w_size)] = depth;
// }