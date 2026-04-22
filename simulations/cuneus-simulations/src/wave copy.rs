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
    cell_count: u32,
    speed: f32,
    reset: u32,
    scene: u32,
    camera_pos: [f32; 2],
    camera_zoom: f32,
    drag: f32,
    restitution: f32,
    window_width: u32,
    window_height: u32,
    _pad: u32,
}}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Cell {
    // Verlet integration
    old_y: f32,
    y: f32,
    vel_y: f32,
    mass: f32,
    accumulated_height: f32,
    // _pad: u32,
    _pad: [u32; 1],
}



struct WaveSimulation {
    base: RenderKit,
    compute_shader: ComputeShader,
    params: ShaderParams,
}

impl ShaderManager for WaveSimulation {
    fn init(core: &Core) -> Self {
        let base = RenderKit::new(core);

        let cell_count = 800*600;
        let cells = vec![
            Cell {
                y: 0.0,
                old_y: 0.0,
                vel_y: 0.0,
                mass: 1.0,
                accumulated_height: 0.0,
                _pad: [0; _],
            };
            cell_count
        ];

        let params = ShaderParams {
            cell_count: cell_count as u32,
            speed: 1.0,
            reset: 1,
            scene: 0,
            camera_pos: [-0.5, -0.5],
            camera_zoom: 1.0,
            drag: 0.1,
            restitution: 1.0,
            window_width: 800 as u32,
            window_height: 600 as u32,
            _pad: 0,
        };

        let mut passes = vec![
            PassDescription::new("update", &[]).with_workgroup_size([
                cell_count.div_ceil(64) as u32,
                1,
                1,
            ]); 35
        ];
        
        passes.append(&mut vec![
            PassDescription::new("clear_screen", &[]).with_workgroup_size([16, 1, 1]),
            PassDescription::new("render", &["update"]).with_workgroup_size([cell_count.div_ceil(64) as u32, 1, 1]),
        ]);

        let config = ComputeShader::builder()
            .with_label("Wave Simulation")
            .with_multi_pass(&passes)
            .with_custom_uniforms::<ShaderParams>()
            .with_mouse()
            .with_storage_buffer(StorageBufferSpec::new(
                "Cells",
                (cell_count * std::mem::size_of::<Cell>()) as u64,
            ))
            .build();
        let compute_shader = create_compute_shader(core, config, params, "wave");
        core.queue.write_buffer(
            &compute_shader.storage_buffers[0],
            0,
            bytemuck::cast_slice(&cells),
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
        let size = core.window.inner_size();
        self.params.window_width = size.width as u32;
        self.params.window_height = size.height as u32;
        self.compute_shader.set_custom_params(self.params, &core.queue);
        self.compute_shader.update_mouse_uniform(&self.base.mouse_tracker.uniform, &core.queue);
        

        let mut controls_request = self.base.controls.get_ui_request(&self.base.start_time, &core.size, self.base.fps_tracker.fps());
        // UI
        let full_output = self.base.render_ui(core, |ctx| {
            RenderKit::apply_default_style(ctx);    
            egui::Window::new("Cell Simulation").show(ctx, |ui| {
                ui.add(egui::Slider::new(&mut self.params.camera_zoom, 0.1..=5.0).text("Zoom").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.speed, 0.0..=20.).text("Speed").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.drag, 0.0..=1.0).text("Drag").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.restitution, 0.0..=10.0).text("Restitution").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_pos[0], -1.0..=1.0).text("Camera x").clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_pos[1], -1.0..=1.0).text("Camera y").clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.scene, 0..=2).text("Scene (0=Wave, 1=Prism, 2=Slit)"));
                if ui.button("Reset").clicked() {
                    self.params.reset = 1;
                }
                ui.separator();
                ShaderControls::render_controls_widget(ui, &mut controls_request);
            });
        });
        
        // Sub-stepping: each dispatch gets a smaller dt → better stability.
        // Total physics time per real frame stays the same (speed / 60).
        let iterations: u32 = 1;
        // let sub_dt = (1.0 / 60.0) / iterations as f32;
        // self.compute_shader.set_time(current_time, sub_dt, &core.queue);

        for _ in 0..iterations {
            self.compute_shader.dispatch(&mut frame.encoder, core);
        }
        if self.params.reset == 1 {
            self.base.start_time = std::time::Instant::now();
        }
        
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
    let (app, event_loop) = ShaderApp::new("Wave Simulation", 800, 600);
    app.run(event_loop, WaveSimulation::init)
}
