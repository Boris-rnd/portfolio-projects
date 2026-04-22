use super::*;
use state::get_wgpu_state;
use wgpu::*;

pub struct LinesPipeline {
    pub render: RenderPipeline,
    pub instances: (Buffer, u32),
}
impl LinesPipeline {
    pub fn new() -> Self {
        let state = get_wgpu_state();
        let render = pipeliner::create_render_pipeline(
            &state.device,
            &state.config,
            "shaders/lines.wgsl",
            |desc| {
                desc.vertex_buffers_layouts(Box::leak(Box::new([Line::desc()])));
                desc.primitive.topology = PrimitiveTopology::LineList;
            },
        );
        let instances = vec![Line::new(vec2f(0., 0.), vec2f(0., 0.))];
        let inst_buf = State::buffer(
            &state.device,
            &instances,
            BufferUsages::MAP_WRITE | BufferUsages::VERTEX | BufferUsages::COPY_SRC,
        );
        let instances = (inst_buf, instances.len() as u32 * 2);
        Self { render, instances }
    }
    pub fn add_line(&mut self, line: Line) -> u32 {
        let (buf, n) = &mut self.instances;
        let idx = *n;
        *n += 2;
        if (*n) as usize * size_of::<Line>() >= buf.size() as usize {
            *buf = State::resize_buffer(&buf, (*n * 2 * 8) as _)
        }
        let state = get_wgpu_state();
        State::write_to_buffer_with_staging(
            &state.device,
            &state.queue,
            buf,
            (idx * 8 as u32) as _,
            &[line],
        );
        *n
    }
    pub fn append_lines(&mut self, lines: &[Line]) -> u32 {
        let (buf, n) = &mut self.instances;
        let idx = *n;
        *n += 2 * lines.len() as u32;
        if (*n) as usize * size_of::<Line>() >= buf.size() as usize {
            *buf = State::resize_buffer(&buf, (*n * 2 * 8) as _)
        }
        let state = get_wgpu_state();
        State::write_to_buffer_with_staging(
            &state.device,
            &state.queue,
            buf,
            (idx * 8 as u32) as _,
            lines,
        );
        *n
    }
    pub fn draw(&mut self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.render);
        render_pass.set_vertex_buffer(0, self.instances.0.slice(..));
        render_pass.draw(0..self.instances.1, 0..self.instances.1);
    }
}

// impl Pipeline for LinesPipeline {

//     fn render_pipeline(&mut self) -> &mut RenderPipeline {
//         &mut self.render
//     }
// }

pub fn add_line(line: Line) -> u32 {
    let state = get_state();
    state.pipelines.lines_pipeline.add_line(line)
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default, Debug)]
pub struct Line {
    pub start: Vec2f,
    pub end: Vec2f,
}
impl Line {
    pub fn new(mut start: Vec2f, mut end: Vec2f) -> Self {
        let (sw, sh): (f32, f32) = get_wgpu_state().screen_size().into();
        start.x /= sw;
        end.x /= sw;
        start.y /= sh;
        end.y /= sh;
        Self { start, end }
    }
    /// Lines are seen by shader with 2 vertices, here we say that the vertice is only two floats, and he will make lines with current vertice and next vertice (so start and end)
    pub const ATTRIBS: &[VertexAttribute] = &vertex_attr_array![0 => Float32x2];
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: 8,
            step_mode: VertexStepMode::Vertex,
            attributes: Self::ATTRIBS,
        }
    }
}
