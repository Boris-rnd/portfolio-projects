#![allow(unused, dead_code)]
use std::{io::Write, time::SystemTime};

use easy_wgpu::{
    buffer::{RawRect, RawRectTexture},
    state::get_wgpu_state,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};

struct StressedApp {
    frame_time: SystemTime,
}
impl StressedApp {
    pub fn new() -> Self {
        Self {
            frame_time: SystemTime::now(),
        }
    }
}
impl easy_wgpu::state::App for StressedApp {
    fn setup(&mut self, state: &mut easy_wgpu::State) {
        state
            .window
            .request_inner_size(PhysicalSize::new(1000, 800));
        let mut r = Vec::with_capacity(100_000);
        for i in 0..5_000_000 {
            r.push(RawRectTexture::new(
                RawRect::new(i as f32 / 10000., i as f32 % 1000., 10., 10.),
                0,
            ));
            // r.push(RawRect::new(i as f32/100., i as f32%1000., 10., 10.));
        }
        // state.pipelines.rect_pipeline.append_rects(&r);
        state.pipelines.rect_texture_pipeline.append_rects(&r);
        self.frame_time = SystemTime::now();
    }

    fn update(&mut self) {
        // let window = &mut get_wgpu_state().window;
        // let mut pos = window.outer_position().unwrap();
        // pos.x += 1;
        // window.set_outer_position(pos);

        print!(
            "{:?} - {}      \r",
            self.frame_time.elapsed().unwrap(),
            1000 / self.frame_time.elapsed().unwrap().as_millis()
        );
        std::io::stdout().flush().unwrap();
        self.frame_time = SystemTime::now();
    }

    fn handle_event(&mut self, event: &winit::event::Event<()>) {}
}

pub fn run() {
    let mut app = StressedApp::new();
    easy_wgpu::run(app);
}
