use crate::gpu::GpuContext;
use wgpu::*;
use winit::event::WindowEvent;

pub trait Simulation: AsAny {
    fn name(&self) -> &str {
        "Simulation"
    }

    fn update(&mut self, ctx: &GpuContext);

    fn render(&self, ctx: &GpuContext, view: &TextureView);

    fn handle_event(&mut self, _event: &WindowEvent) {}
}

pub trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: Simulation + 'static> AsAny for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
