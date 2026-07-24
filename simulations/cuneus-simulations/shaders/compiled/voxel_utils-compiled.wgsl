// Utilities around voxels and world data handling

// 2, 4, 8, ...
const CHUNK_SIZE: u32 = 4; // TODO: make this shared with CPU code in world
const CHUNK_U32_COUNT = CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE/32;
// const CHUNK_U32_COUNT = 1;
const CHUNK_MASK = CHUNK_SIZE - 1u;
const CHUNK_SHIFT = countOneBits(CHUNK_MASK);
struct Voxel {
    id: f32,
    pos: vec3<f32>,
}

struct MapData {
    // 2 first bits = type:
    // 00=block
    // 01=chunk
    // 10=entity
    // 11=Tail
    data: u32,
}
struct VoxelChunk {
    // idx_in_parent: u32,
    inner: array<u32, CHUNK_U32_COUNT>,
    prefix_in_block_data_array: array<u32, 4>,
}
struct DataResult {
    data: u32,
    depth: u32,
}

fn set_bit_if_in_range(bit_mask: array<u32, CHUNK_U32_COUNT>, bit_pos: vec3<u32>) -> array<u32, CHUNK_U32_COUNT> {
    if (any(bit_pos >= vec3<u32>(CHUNK_SIZE))) {
        return bit_mask;
    }
    let chunk_idx = bit_pos.x | (bit_pos.y << CHUNK_SHIFT) | (bit_pos.z << (CHUNK_SHIFT * 2u));
    let word = chunk_idx / 32u;
    let bit = 1u << (chunk_idx % 32u);
    var b = bit_mask;
    b[word] = b[word] | bit;
    return b;
}

fn gen_chunk_mask(ray: Ray, start_pos_local: vec3<f32>) -> array<u32, CHUNK_U32_COUNT> {
    var posf = start_pos_local;
    let dir = ray.dir;
    let rcp = 1.0 / dir;
    var mask = array<u32, CHUNK_U32_COUNT>(4294967295, 4294967295);

    loop {
        // integer voxel inside chunk, safe conv via floor+clamp
        let pf = floor(posf);
        if any(pf < vec3(0.)) || any(pf > vec3(f32(CHUNK_SIZE)-1.)) {break;}
        let posi = vec3<u32>(pf);

        // set current voxel and immediate forward neighbors (x+, y+, z+)
        mask = set_bit_if_in_range(mask, posi);
        mask = set_bit_if_in_range(mask, posi + vec3<u32>(1u, 0u, 0u));
        mask = set_bit_if_in_range(mask, posi + vec3<u32>(0u, 1u, 0u));
        mask = set_bit_if_in_range(mask, posi + vec3<u32>(0u, 0u, 1u));

        // compute distance to next voxel boundary along each axis (respect sign)
        let idxf = floor(posf);
        let next = select(idxf, idxf + vec3<f32>(1.0), dir > vec3<f32>(0.0));
        let tMax = (next - posf) * rcp;
        let tStep = min(tMax.x, min(tMax.y, tMax.z));

        // stop if non-finite or we will leave chunk
        if (!(tStep < 1e20)) { break; } // guard
        // step a little less than full boundary to avoid re-hitting same voxel due to float error
        let eps = 1e-4*4;
        posf = posf + dir * (tStep + eps);
    }
    return mask;
}

fn local_pos_to_ivec3(idx: u32) -> vec3<u32> {
    let x = idx & u32(CHUNK_MASK);
    let y = (idx >> CHUNK_SHIFT) & u32(CHUNK_MASK);
    let z = (idx >> (CHUNK_SHIFT * 2u)) & u32(CHUNK_MASK);
    return vec3<u32>(x, y, z);
}
fn ivec3_to_local_pos(pos: vec3<u32>) -> u32 {
    return pos.x | (pos.y << CHUNK_SHIFT) | (pos.z << (CHUNK_SHIFT * 2u));
}



/// Returns u32::MAX if not found
fn get_data_idx_in_chunk(chunk: VoxelChunk, _idx: u32) -> MapDataID {
    let local_idx = _idx/32u;
    let local_bit = _idx%32u;
    if (chunk.inner[local_idx] & (u32(1) << local_bit)) == 0u {
        return MapDataID(4294967295u, 4294967295u);
    }

    var ones = 0u;
    var i = 0u;
    while i < local_idx {
        ones += countOneBits(chunk.inner[i]);
        i += 1u;
    }
    
    let curr_set_bits = countOneBits(((1u << local_bit) - 1u) & chunk.inner[local_idx]);
    let chunk_idx = curr_set_bits + ones;
    let curr_array = size_to_array_array_idx(chunk_idx);
    let local_array_idx = chunk_idx - array_array_idx_to_prefix_size(curr_array);
    return MapDataID(curr_array, chunk.prefix_in_block_data_array[curr_array] + local_array_idx);
}
// /// Returns u32::MAX if not found / invalid idx in tails chain or from start
// /// Returns block data, not idx !
fn get_block_data_follow_tails(idx: MapDataID) -> u32 {
    var curr_idx = idx.array_idx;
    for (var i=0;i<100;i++) {
        if (curr_idx >= arrayLengthBlockData(idx.array_array_idx)) {break;}
        let curr_data = get_block_data(MapDataID(idx.array_array_idx, curr_idx)).data;
        if (curr_data&3u) == 3u { // Tail
            curr_idx = u32(curr_data >> 2);
        } else {
            return curr_data;
        }
    }
    return 4294967295u;
}

struct MapDataID {
    array_array_idx: u32,
    array_idx: u32,
}

fn size_to_array_array_idx(size: u32) -> u32 {
    if size < 8 {
        return 0u;
    } else if size < 24 {
        return 1u;
    } else if size < 40 {
        return 2u;
    } else {
        return 3u;
    }
}
fn array_array_idx_to_prefix_size(array_array_idx: u32) -> u32 {
    if array_array_idx == 0u {
        return 0u;
    } else if array_array_idx == 1u {
        return 8u;
    } else if array_array_idx == 2u {
        return 24u;
    } else {
        return 40u;
    }
}
fn get_block_data(idx: MapDataID) -> MapData {
    if idx.array_array_idx == 0u {
        return block_data0[idx.array_idx];
    } else if idx.array_array_idx == 1u {
        return block_data1[idx.array_idx];
    } else if idx.array_array_idx == 2u {
        return block_data2[idx.array_idx];
    } else {
        return block_data3[idx.array_idx];
    }
    // return MapData(0u); //TODO
}
fn arrayLengthBlockData(idx: u32) -> u32 {
    // return 0; //TODO
    if idx == 0u {
        return arrayLength(&block_data0);
    } else if idx == 1u {
        return arrayLength(&block_data1);
    } else if idx == 2u {
        return arrayLength(&block_data2);
    } else {
        return arrayLength(&block_data3);
    }
}

fn chunk_depth_to_size(depth: u32) -> u32 {
    //TODO IF CHANGE CHUNK_SIZE NEEDS UPDATE
    return 1u << ((CHUNK_SIZE/2) * depth);
    // return u32(pow(f32(CHUNK_SIZE), f32(depth)));
}

// Small depth = big size
// ex: depth=1 -> root_chunk_size/4
fn depth_to_chunk_size(depth: u32) -> u32 {
    // Convert depth to chunk size (16, 8, 4, 2, 1)
    return root_chunk_size() / chunk_depth_to_size(depth);
}

fn root_chunk_size() -> u32 {
    return chunk_depth_to_size(params.root_max_depth);
}

