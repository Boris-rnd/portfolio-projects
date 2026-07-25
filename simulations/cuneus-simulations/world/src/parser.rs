use anyhow::Result;
use dot_vox::*;

use super::*;
use crate::*;

fn iterate_vox_tree(vox_tree: &DotVoxData, mut fun: impl FnMut(&Model, &Vec3, &Rotation)) {
    match &vox_tree.scenes[0] {
        SceneNode::Transform {
            attributes: _,
            frames: _,
            child,
            layer_id: _,
        } => {
            iterate_vox_tree_inner(
                vox_tree,
                *child,
                Vec3::new(0.0, 0.0, 0.0),
                Rotation::IDENTITY,
                &mut fun,
            );
        }
        _ => {
            panic!("The root node for a magicka voxel DAG should be a Transform node")
        }
    }
}

fn iterate_vox_tree_inner(
    vox_tree: &DotVoxData,
    current_node: u32,
    translation: Vec3,
    rotation: Rotation,
    fun: &mut impl FnMut(&Model, &Vec3, &Rotation),
) {
    match &vox_tree.scenes[current_node as usize] {
        SceneNode::Transform {
            attributes: _,
            frames,
            child,
            layer_id: _,
        } => {
            // In case of a Transform node, the potential translation and rotation is added
            // to the global transform to all of the nodes children nodes
            let translation = if let Some(t) = frames[0].attributes.get("_t") {
                let translation_delta = t
                    .split(" ")
                    .map(|x| x.parse().expect("Not an integer!"))
                    .collect::<Vec<i32>>();
                debug_assert_eq!(translation_delta.len(), 3);
                translation
                    + vec3(
                        translation_delta[0] as f32,
                        translation_delta[1] as f32,
                        translation_delta[2] as f32,
                    )
            } else {
                translation
            };
            let rotation = if let Some(r) = frames[0].attributes.get("_r") {
                rotation
                    * Rotation::from_byte(
                        r.parse()
                            .expect("Expected valid u8 byte to parse rotation matrix"),
                    )
            } else {
                Rotation::IDENTITY
            };

            iterate_vox_tree_inner(vox_tree, *child, translation, rotation, fun);
        }
        SceneNode::Group {
            attributes: _,
            children,
        } => {
            // in case the current node is a group, the index variable stores the current
            // child index
            for child_node in children {
                iterate_vox_tree_inner(vox_tree, *child_node, translation, rotation, fun);
            }
        }
        SceneNode::Shape {
            attributes: _,
            models,
        } => {
            // in case the current node is a shape: it's a leaf node and it contains
            // models(voxel arrays)
            for model in models {
                fun(
                    &vox_tree.models[model.model_id as usize],
                    &translation,
                    &rotation,
                );
            }
        }
    }
}

pub fn load_world(path: &str) -> Result<GameWorld> {
    let vox_tree = dot_vox::load(path).map_err(|e| anyhow::anyhow!(e))?;

    let mut world = GameWorld::new(4096, 8);

    let mut voxels: hashbrown::HashMap<IVec3, u32> = Default::default();
    iterate_vox_tree(&vox_tree, |model, position, orientation| {
        let rotation_mat = Mat3::from_cols_array_2d(&orientation.to_cols_array_2d());

        for voxel in &model.voxels {
            // Convert to centered coordinates
            let local_pos = vec3(
                voxel.x as f32,
                voxel.y as f32,
                voxel.z as f32, // Swap Y and Z to match MagicaVoxel's coordinate system
            );

            let world_pos = rotation_mat.mul_vec3(local_pos) + *position;

            // Convert to positive coordinates by adding an offset (I only support positive coords for now)
            let pos = ivec3(
                world_pos.x as i32 + 128,
                world_pos.z as i32 + 128,
                world_pos.y as i32 + 128,
            );

            let col = vox_tree.palette[voxel.i as usize];
            let col = col.r as u32 | ((col.g as u32) << 8u32) | ((col.b as u32) << 16u32);

            if pos.min_element() < 0
                || (pos.cmpge(IVec3::splat((world.root_size() - 1) as _))).any()
            {
                continue;
            }
            voxels.insert(pos, col);
            // world.set_block(pos, MapData::Block(col));
        }
        for (pos, voxel) in &voxels.clone() {
            if voxels.get(pos).is_none(){continue;}
            // Create recursively all necessary parent chunks
            let (_, chunk) = world.set_block(*pos, MapData::Block(*voxel)).unwrap();
            let chunk_pos = pos.div_euclid(IVec3::splat(4))*4;
            for lx in 0..CHUNK_SIZE as i32 {
                for ly in 0..CHUNK_SIZE as i32 {
                    for lz in 0..CHUNK_SIZE as i32 {
                        let global_pos = ivec3(lx, ly, lz)+chunk_pos;
                        if let Some(v) = voxels.get(&global_pos) {
                            world.set_data_in_chunk(chunk, LocalPos::new(lx as _, ly as _, lz as _), MapData::Block(*v));
                            voxels.remove(&global_pos);
                        }
                    }
                }
            }
        }

    });

    // Iterate over all models inside the vox file
    // for model in &vox_tree.models {
    // let model = vox_tree.models.get(0).unwrap();
    //     for (i, voxel) in model.voxels.iter().enumerate() {
    //         // Each voxel has x, y, z, and color_index (palette lookup)
    //         let pos = IVec3::new(voxel.x as i32 + 100, voxel.y as i32 + 100, voxel.z as i32 + 100);

    //         // Here you decide how to map palette index → block_data
    //         let block_data = voxel.i as u32;
    //         let col = vox_tree.palette[block_data as usize];
    //         let col = col.r as u32 | ((col.g as u32) << 8u32) | ((col.b as u32) << 16u32);

    //         if pos.min_element() < 0 {continue;}
    //         world.set_block(pos, MapData::Block(col));
    //     }
    // }
    dbg!(world.voxel_chunks.len());

    Ok(world)
}
