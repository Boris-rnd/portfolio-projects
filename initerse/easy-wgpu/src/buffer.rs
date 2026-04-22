use wgpu::{vertex_attr_array, VertexAttribute, VertexBufferLayout};
use winit::dpi::PhysicalPosition;

use crate::state::{get_state, get_wgpu_state};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    // Remember, when changing this to change ATTRIBS
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    // color: [f32; 3],
}
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
    // Attributes without macro:
    // &[
    //     wgpu::VertexAttribute {
    //         offset: 0,
    //         shader_location: 0,
    //         format: wgpu::VertexFormat::Float32x3,
    //     },
    //     wgpu::VertexAttribute {
    //         offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
    //         shader_location: 1,
    //         format: wgpu::VertexFormat::Float32x3,
    //     }
    // ]
}

// impl<T: TryInto<f32>> From<PhysicalPosition<T>> for Vec2 {
//     fn from(value: PhysicalPosition<T>) -> Self {
//         let x = value.x.try_into();
//         if let Err(x) = x {panic!("Can't make vec");}
//         let y = value.y.try_into();
//         if let Err(y) = y {panic!("Can't make vec");}
//         Self {x: unsafe { x.unwrap_unchecked() }, y: unsafe { y.unwrap_unchecked() }}
//     }
// }
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default, Debug)]
pub struct RawRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl RawRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        let (sw, sh): (f32, f32) = get_wgpu_state().screen_size().into();
        Self::_new((sw, sh), x, y, w, h)
    }
    pub fn _new(screen_size: (f32, f32), x: f32, y: f32, w: f32, h: f32) -> Self {
        let (sw, sh) = screen_size;
        let sw = sw / 2.;
        let sh = sh / 2.;
        let x = x / sw - 1.;
        let y = 1.0 - (y / sh);
        let w = w / sw;
        let h = h / sh;
        // if x < 0. {x += w*2.}
        // if y < 0. {y += h*2.}
        Self { x, y: y - h, w, h }
    }
    pub fn desc() -> VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            }],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default, Debug)]
pub struct RawRectTexture {
    pub rect: RawRect,
    pub texture_id: u32,
}
impl RawRectTexture {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![3 => Float32x4, 4 => Uint32];
    pub fn new(rect: RawRect, texture_id: u32) -> Self {
        Self { rect, texture_id }
    }
    pub fn desc() -> VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default, Debug)]
pub struct RawRectTextureAtlas {
    pub rect: RawRect,
    pub texture_rect: RawRect,
}
impl RawRectTextureAtlas {
    const ATTRIBS: &[VertexAttribute] = &vertex_attr_array![3 => Float32x4, 4 => Float32x4];
    pub fn new(rect: RawRect, texture_rect: RawRect) -> Self {
        Self { rect, texture_rect }
    }
    pub fn desc() -> VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

// pub const TRIANGLE_VERTICES: &[Vertex] = &[
//     Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
//     Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
//     Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
// ];
// Hexagon
// pub const VERTICES: &[Vertex] = &[
//     Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [1.0, 0.0, 0.0] }, // A
//     Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 1.0, 0.5] }, // B
//     Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
//     Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 1.0, 0.5] }, // D
//     Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
// ];
// Hexagon
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0., 0.],
        tex_coords: [0., 1.],
    }, // A
    Vertex {
        position: [1., 0.],
        tex_coords: [1., 1.],
    }, // B
    Vertex {
        position: [1., 1.],
        tex_coords: [1., 0.],
    }, // C
    Vertex {
        position: [0., 1.],
        tex_coords: [0., 0.],
    }, // D
];
pub const INDICES: &[u16] = &[0, 1, 3, 2, 3, 1];
