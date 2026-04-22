#![allow(
    unused_parens,
    unused_braces,
    dead_code,
    unused_import_braces,
    unused_imports,
    unused_variables
)]

use buffer::{RawRect, Vertex};
use easy_wgpu::*;
use encase::DynamicStorageBuffer;
use pipelines::{compute::create_compute_pipeline, pipeliner::create_render_pipeline};
use state::texture_entire_binding;
use wgpu::*;
use winit::event::Event;




// Sadly I need this because winit doesn't want to be launched from any thread :c
#[cfg(feature = "testing")]
mod test {
    include!("../tests/mod.rs");
}
#[cfg(feature = "testing")]
fn main() {
    test::run_tests()
}

#[cfg(not(feature = "testing"))]
fn main() {
    easy_wgpu::run::<VoxelApp>();
}


pub struct VoxelApp {
    voxels_pipeline: VoxelPipeline,
}
impl App for VoxelApp {
    fn setup(state: &mut State) -> Self where Self: Sized {
        let voxels_pipeline = VoxelPipeline {
            voxels: DynamicStorageBuffer::new(buffer),
        }
        Self {
            voxels_pipeline
        }
    }

    fn render(&mut self, screen: &wgpu::TextureView) {
        todo!()
    }

    fn handle_event(&mut self, event: &Event<()>) {
        
    }
}

pub struct Voxel {
    pos: [f32;3],
}

pub struct VoxelPipeline {
    voxels: encase::DynamicStorageBuffer<Voxel>
}