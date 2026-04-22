mod gpu;
mod simulation;

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use gpu::GpuContext;
use simulation::Simulation;

struct DoubleSimulation {
    compute_pipeline: wgpu::ComputePipeline,
    data_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl DoubleSimulation {
    fn new(ctx: &GpuContext) -> Self {
        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));
        let input_data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        
        let data_buffer = ctx.create_buffer(
            "Data Buffer", 
            &input_data, 
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST
        );

        let compute_pipeline = ctx.create_compute_pipeline("Compute Pipeline", &shader, "main");

        let bind_group = ctx.create_bind_group(
            "Bind Group",
            &compute_pipeline.get_bind_group_layout(0),
            &[wgpu::BindGroupEntry {
                binding: 0,
                resource: data_buffer.as_entire_binding(),
            }],
        );

        Self {
            compute_pipeline,
            data_buffer,
            bind_group,
        }
    }

    async fn run_and_print_result(&self, ctx: &GpuContext) {
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.dispatch_workgroups(1, 1, 1);
        }

        ctx.queue.submit(Some(encoder.finish()));

        // Read back data
        let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: self.data_buffer.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(&self.data_buffer, 0, &staging_buffer, 0, self.data_buffer.size());
        ctx.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |res| sender.send(res).unwrap());

        let _ = ctx.device.poll(wgpu::PollType::wait_indefinitely());

        if let Ok(Ok(_)) = receiver.recv() {
            let data = buffer_slice.get_mapped_range();
            let result: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
            drop(data);
            staging_buffer.unmap();
            println!("Compute Result: {:?}", result);
        }
    }
}

impl Simulation for DoubleSimulation {
    fn name(&self) -> &str {
        "Double Simulation"
    }

    fn update(&mut self, ctx: &GpuContext) {
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.dispatch_workgroups(1, 1, 1);
        }

        ctx.queue.submit(Some(encoder.finish()));
    }

    fn render(&self, ctx: &GpuContext, view: &wgpu::TextureView) {
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        ctx.queue.submit(Some(encoder.finish()));
    }

    fn handle_event(&mut self, _event: &WindowEvent) {
        // We could handle events here if needed
        // But run_and_print_result is async and needs ctx
    }
}

struct AppState {
    window: Arc<Window>,
    ctx: GpuContext,
    simulation: Box<dyn Simulation>,
}

struct App {
    state: Option<AppState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("wgpu Simulations"))
                .unwrap(),
        );

        let ctx = pollster::block_on(GpuContext::new(window.clone()));
        let simulation = Box::new(DoubleSimulation::new(&ctx));
        
        self.state = Some(AppState {
            window,
            ctx,
            simulation,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return,
        };

        state.simulation.handle_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                state.ctx.resize(physical_size);
            }
            WindowEvent::RedrawRequested => {
                let output = match state.ctx.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(texture) => texture,
                    wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
                    wgpu::CurrentSurfaceTexture::Outdated => {
                        state.ctx.resize(state.window.inner_size());
                        return;
                    }
                    _ => return,
                };
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                state.simulation.update(&state.ctx);
                state.simulation.render(&state.ctx, &view);

                output.present();
                state.window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) {
                    event_loop.exit();
                }
                if event.state.is_pressed() && event.logical_key == winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter) {
                    // Manual trigger for the simulation print
                    // This is slightly tricky with the trait, but possible for this specific demo
                    if let Some(demo) = state.simulation.as_any().downcast_ref::<DoubleSimulation>() {
                         pollster::block_on(demo.run_and_print_result(&state.ctx));
                    }
                }
            }
            _ => (),
        }
    }
}

// AsAny is now provided by simulation.rs
// Update Simulation trait to include AsAny if needed, or just add it as a supertrait
// Actually I'll just add it to Simulation trait definition in simulation.rs

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App { state: None };
    event_loop.run_app(&mut app).unwrap();
}
