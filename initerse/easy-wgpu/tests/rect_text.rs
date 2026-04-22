#![allow(unused, dead_code)]
use std::{io::Write, time::SystemTime};

use buffer::{RawRect, RawRectTexture};
use easy_wgpu::*;
use pipelines::rect_texture::img_load_from_file;
use wgpu::*;

pub struct RectTextApp {
    ids: Vec<u32>,
}
impl App for RectTextApp {
    fn setup(state: &mut easy_wgpu::State) -> Self {
        let ids = vec![];
        let id = state.pipelines.rect_texture_pipeline.add_texture(img_load_from_file("happy-tree-64.png").unwrap().to_vec());
        state.pipelines.rect_texture_pipeline.push_rect(RawRectTexture::new(RawRect::new(200., 200., 200., 200.), id));
        Self {
            ids,
        }
    }

    fn render(&mut self, view: &wgpu::TextureView) {
    }

    fn handle_event(&mut self, event: &winit::event::Event<()>) {}
}
use winit_test::winit::event_loop::EventLoopWindowTarget;

fn my_test(elwt: &EventLoopWindowTarget<()>) {
    // ...
}

fn other_test(elwt: &EventLoopWindowTarget<()>) {
    // ...
}

winit_test::main!(my_test, other_test);