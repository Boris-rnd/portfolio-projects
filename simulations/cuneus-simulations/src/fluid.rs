#![allow(unused, dead_code)]
use bytemuck::{Pod, Zeroable};
use cuneus::compute::*;
use cuneus::prelude::*;
use cuneus::compute::*;
use cuneus::winit::keyboard::Key;
use cuneus::winit::keyboard::KeyCode;
use cuneus::{Core, RenderKit, ShaderApp, ShaderManager, UniformProvider} ;
use crate::*;

cuneus::uniform_params! {
    struct ShaderParams {
    gravity: f32,
    particle_size: u32,
    particle_count: u32,
    speed: f32,
    reset: u32,
    camera_pos: [f32; 2],
    camera_zoom: f32,
    h: f32,
    rest_density: f32,
    k: f32,
    drag: f32,
    viscosity: f32,
    _pad: [u32; 3],
    
}}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Particle {
    pos: [f32; 2],
    vel: [f32; 2],
    mass: f32,
    _pad: [u32; 3],
}



struct FluidSimulation {
    base: RenderKit,
    compute_shader: ComputeShader,
    params: ShaderParams,
}

impl ShaderManager for FluidSimulation {
    fn init(core: &Core) -> Self {
        let base = RenderKit::new(core);

        let particle_count = 10_000;
        let particles = vec![
            Particle {
                pos: [0.0, 0.0],
                vel: [0.0, 0.0],
                mass: 1.0,
                _pad: [0; 3],
            };
            particle_count
        ];

        let params = ShaderParams {
            gravity: 0.0,
            particle_size: 1,
            particle_count: particle_count as u32,
            speed: 0.00,
            reset: 1,
            camera_pos: [-1., -1.],
            camera_zoom: 0.25,
            h: 0.01,
            rest_density: 1000.0,
            k: 100.0,
            drag: 0.1,
            viscosity: 0.01,
            _pad: [0; 3],
        };

        let passes = vec![
            PassDescription::new("update", &[]).with_workgroup_size([
                particle_count.div_ceil(64) as u32,
                1,
                1,
            ]),
            PassDescription::new("clear_atomics", &[]),
            PassDescription::new("splat", &[]).with_workgroup_size([1, 1, 1]),
        ];

        let config = ComputeShader::builder()
            .with_label("Fluid Simulation")
            .with_multi_pass(&passes)
            .with_custom_uniforms::<ShaderParams>()
            .with_mouse()
            .with_atomic_buffer(1)
            .with_storage_buffer(StorageBufferSpec::new(
                "particles",
                (particle_count * std::mem::size_of::<Particle>()) as u64,
            ))
            .build();
        let compute_shader = create_compute_shader(core, config, params, "fluid");
        core.queue.write_buffer(
            &compute_shader.storage_buffers[0],
            0,
            bytemuck::cast_slice(&particles),
        );

        Self {
            base,
            compute_shader,
            params,
        }
    }

    fn update(&mut self, _core: &Core) {
        self.params.reset = 0; // Stop resetting after first frame
    }

    fn render(&mut self, core: &Core) -> Result<(), SurfaceError> {
        let mut frame = self.base.begin_frame(core)?;

        // Update time and params
        let current_time = self.base.controls.get_time(&self.base.start_time);
        self.compute_shader.set_time(current_time, 1.0/60.0, &core.queue);
        self.compute_shader.set_custom_params(self.params, &core.queue);
        self.compute_shader.update_mouse_uniform(&self.base.mouse_tracker.uniform, &core.queue);

        let mut controls_request = self.base.controls.get_ui_request(&self.base.start_time, &core.size, self.base.fps_tracker.fps());
        // UI
        let full_output = self.base.render_ui(core, |ctx| {
            RenderKit::apply_default_style(ctx);
            egui::Window::new("Particle Simulation").show(ctx, |ui| {
                ui.add(egui::Slider::new(&mut self.params.gravity, 0.0..=20.0).text("Gravity").clamping(egui::SliderClamping::Never).logarithmic(true));
                ui.add(egui::Slider::new(&mut self.params.particle_size, 1..=5).text("Size (px)").clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_zoom, 0.1..=5.0).text("Zoom").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.speed, 0.0..=0.1).text("Speed").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.h, 0.001..=0.1).text("Kernel Radius").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.rest_density, 100.0..=10000.0).text("Rest Density").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.k, 10.0..=1000.0).text("Pressure").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.drag, 0.0..=1.0).text("Drag").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.viscosity, 0.0..=1.0).text("Viscosity").logarithmic(true).clamping(egui::SliderClamping::Never));
                if ui.button("Reset").clicked() {
                    self.params.reset = 1;
                }
                ui.separator();
                ShaderControls::render_controls_widget(ui, &mut controls_request);

            });
        });

        // Run compute passes
        self.compute_shader.dispatch(&mut frame.encoder, core);

        // Render to screen
        self.base.renderer.render_to_view(
            &mut frame.encoder,
            &frame.view,
            &self.compute_shader.get_output_texture().bind_group,
        );

        self.base.end_frame(core, frame, full_output);
        Ok(())
    }

    fn resize(&mut self, core: &Core) {
        self.base.default_resize(core, &mut self.compute_shader);
    }

    fn handle_input(&mut self, core: &Core, event: &winit::event::WindowEvent) -> bool {
        if self.base.default_handle_input(core, event) {return true;}
        match event {
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                // Todo zoom in and out
                dbg!(&delta);
                self.params.camera_zoom += match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                } as f32 * 0.1;
                true
            },
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(KeyCode::ArrowLeft) => {
                        self.params.camera_pos[0] -= 0.1;
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::ArrowRight) => {
                        self.params.camera_pos[0] += 0.1;
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::ArrowUp) => {
                        self.params.camera_pos[1] -= 0.1;
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::ArrowDown) => {
                        self.params.camera_pos[1] += 0.1;
                        true
                    },
                    _ => false
                }
            },
            _ => false
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (app, event_loop) = ShaderApp::new("Fluid Simulation", 800, 600);
    app.run(event_loop, FluidSimulation::init)
}
