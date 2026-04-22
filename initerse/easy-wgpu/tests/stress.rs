#![allow(unused, dead_code)]
use std::{io::Write, time::SystemTime};

use easy_wgpu::{
    buffer::{RawRect, RawRectTexture},
    state::get_wgpu_state,
    App,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};

pub struct StressedApp {
    frame_time: SystemTime,
}
impl App for StressedApp {
    fn setup(state: &mut easy_wgpu::State) -> Self {
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
        Self {
            frame_time: SystemTime::now()
        }
    }

    fn render(&mut self, view: &wgpu::TextureView) {
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

