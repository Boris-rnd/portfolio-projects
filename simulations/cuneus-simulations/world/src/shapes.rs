use bytemuck::{Pod, Zeroable};
use glam::vec3;

use super::*;
use crate::*;
#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy, Debug)]
pub struct Sphere {
    pos: Vec3,
    rad: f32,
    color: Vec3,
    pad: f32
}

pub fn sphere(pos: Vec3, rad: f32, color: Vec3) -> Sphere {
    Sphere {
        pos,
        rad,
        color,
        ..Default::default()
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy, Debug)]
pub struct Box {
    min: Vec3,
    max: Vec3,
    color: Vec3,
    pad: [f32; 3]
}

pub fn new_box(min: Vec3, max: Vec3, color: Vec3) -> Box {
    Box {
        min,
        max,
        color,
        ..Default::default()
    }
}
pub fn new_voxel(pos: Vec3) -> Box {
    new_box(
        pos - vec3(0.5, 0.5, 0.5),
        pos + vec3(0.5, 0.5, 0.5),
        vec3(1., 1., 1.),
    )
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy, Debug)]
pub struct Voxel {
    pos: Vec3,
    texture_id: u32,
}

