use std::sync::Arc;

use buffer::{Vertex, INDICES, VERTICES};

use super::*;

pub struct RectanglePipeline {
    pub render: RenderPipeline,
    pub vertex: (Buffer, u32),
    pub index: (Buffer, u32),
    pub instances: (Arc<Buffer>, u32),
}
impl RectanglePipeline {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let v = (Box::new([Vertex::desc(), RawRect::desc()]));
        let render =
            pipeliner::create_render_pipeline(&device, &config, "shaders/shader.wgsl", |desc| {
                desc.vertex_buffers_layouts(Box::leak(v)) // I hate it
            });
        let v_buf = vertex_buffer(&device, VERTICES);
        let index_buffer = index_buffer(&device, INDICES);
        let instances = vec![RawRect::default()];
        let instances_buffer = vertex_buffer(&device, &instances);

        Self {
            render,
            vertex: (v_buf, VERTICES.len() as u32),
            index: (index_buffer, INDICES.len() as u32),
            instances: (Arc::new(instances_buffer), instances.len() as u32),
        }
    }
    pub fn push_rect(&mut self, rect: RawRect) {
        let state = get_state();
        self.instances.1 += 1;
        if self.instances.1 * std::mem::size_of::<RawRect>() as u32 >= self.instances.0.size() as _
        {
            // dbg!(self.instances.0.size()*2);
            *unsafe { Arc::get_mut_unchecked(&mut self.instances.0) } =
                resize_buffer(self.instances.0.as_ref(), self.instances.0.size() * 2);
            // dbg!(self.instances.0.size()*2);
        }
        // dbg!(self.instances.0.size()*2, ((self.instances.1-1)*std::mem::size_of::<Rect>() as u32) as u64);

        write_to_buffer_with_staging(
            &state.device,
            &state.queue,
            &self.instances.0,
            ((self.instances.1 - 1) * std::mem::size_of::<RawRect>() as u32) as _,
            &[rect],
        );
    }

    pub fn append_rects(&mut self, rects: &[RawRect]) {
        let new_count = self.instances.1 + rects.len() as u32;
        let state = get_state();
        if new_count * std::mem::size_of::<RawRect>() as u32 >= self.instances.0.size() as u32 {
            self.instances.0 = Arc::new(resize_buffer(
                &self.instances.0,
                (self.instances.0.size() * 2)
                    .max(new_count as u64 * std::mem::size_of::<RawRect>() as u64),
            ));
            // dbg!(self.instances.0.size(), self.instances.1);
        }
        write_to_buffer_with_staging(
            &state.device,
            &state.queue,
            &self.instances.0,
            ((self.instances.1 - 1) * std::mem::size_of::<RawRect>() as u32) as _,
            rects,
        );
        self.instances.1 = new_count;
    }
}

impl Pipeline for RectanglePipeline {
    //encoder: &mut wgpu::CommandEncoder,
    fn draw(&mut self, view: &wgpu::TextureView, render_pass: &mut RenderPass, state: &mut State) {
        // let mut scope = state.profiler.scope("Main render", encoder, &state.device);
        // let mut render_pass = scope.begin_render_pass(&wgpu::RenderPassDescriptor {
        //     label: Some("Rectangle Render Pass"),
        //     color_attachments: &get_empty_color_attachment(view),
        //     depth_stencil_attachment: None,
        //     occlusion_query_set: None,
        //     timestamp_writes: None,
        // });
        render_pass.set_pipeline(&self.render);
        render_pass.set_vertex_buffer(0, self.vertex.0.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.0.slice(..));
        render_pass.set_index_buffer(self.index.0.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.index.1, 0, 0..self.instances.1 as _);

        // drop(render_pass);
        // drop(scope);
        // state.profiler.resolve_queries(encoder);
        // state.profiler.end_frame().unwrap();
        // if let Some(profiling_data) = state.profiler.process_finished_frame(state.queue.get_timestamp_period()) {
        //     wgpu_profiler::chrometrace::write_chrometrace(std::path::Path::new("mytrace.json"), &profiling_data).unwrap();
        // }
    }

    fn render_pipeline(&mut self) -> &mut RenderPipeline {
        &mut self.render
    }
}
