#![allow(
    unused_parens,
    unused_braces,
    dead_code,
    unused_import_braces,
    unused_imports,
    unused_variables
)]

use std::mem::transmute;

use easy_wgpu::get_state;
use wgpu::PollType;

#[cfg(not(feature = "testing"))]
fn main() {
    easy_wgpu::run(BaseApp {});
}


pub struct BaseApp {}
impl easy_wgpu::state::App for BaseApp {
    fn setup(&mut self, state: &mut easy_wgpu::State) {
    }

    fn update(&mut self) {
        // let state = get_state();
        // state.pipelines.rect_pipeline.instances.0.unmap();
        // state.pipelines.rect_pipeline.instances.0.unmap();
        // state.pipelines.rect_pipeline.instances.0.map_async(wgpu::MapMode::Write, .., |buffer| {
        //     buffer.unwrap();
        //     let buffer = state.pipelines.rect_pipeline.instances.0.get_mapped_range_mut(..);
        //     for b in buffer.chunks(size_of::<easy_wgpu::buffer::RawRect>()) {
        //         let r: easy_wgpu::buffer::RawRect = unsafe { transmute(b) };
        //         dbg!(r);
        //     }
        // });
        // get_state().device.poll(PollType::Wait).unwrap();
        // state.pipelines.rect_pipeline.instances.0.unmap();

    }

    fn handle_event(&mut self, event: &winit::event::Event<()>) {}
}

// Sadly I need this because winit doesn't want to be launched from any thread :c
#[cfg(feature = "testing")]
mod test {
    include!("../tests/mod.rs");
}
#[cfg(feature = "testing")]
fn main() {
    test::run_tests()
}
