use super::*;

#[derive(Default)]
pub struct BgPipeline {
    pub bg_color: Color,
}
impl BgPipeline {
    // Functionnality to make a dynamic background color
    pub fn update(&mut self) {
        let state = get_state();
        self.bg_color = wgpu::Color {
            r: (state.gui_manager.mouse_pos().x / state.window.inner_size().width as f32) as _,
            g: (state.gui_manager.mouse_pos().y / state.window.inner_size().height as f32) as _,
            b: 0.3,
            a: 1.0,
        };
    }
}
// impl Pipeline for BgPipeline {
//     fn draw(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, state: &mut State) {
//         let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//             label: Some("Background Render Pass"),
//             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
//                 view,
//                 resolve_target: None,
//                 ops: wgpu::Operations {
//                     load: wgpu::LoadOp::Clear(self.bg_color),
//                     store: wgpu::StoreOp::Store,
//                 },
//             })],
//             depth_stencil_attachment: None,
//             occlusion_query_set: None,
//             timestamp_writes: None,
//         });
//     }

//     fn render_pipeline(&mut self) -> &mut RenderPipeline {
//         todo!()
//     }
// }
