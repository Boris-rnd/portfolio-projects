use crate::create_compute_shader;
use cuneus::{
    bytemuck,
    compute::{
        ComputeShader, PassDescription,
        COMPUTE_TEXTURE_FORMAT_RGBA16,
    },
    egui, Core, RenderKit, ShaderControls, ShaderManager, SurfaceError,
};

cuneus::uniform_params! {
    struct ShaderParams {
    speed: f32,
    reset: u32,
    scene: u32,
    camera_pos: [f32; 2],
    camera_zoom: f32,
    drag: f32,
    restitution: f32,
    window_width: u32,
    window_height: u32,
    _pad: [u32; 2], // keep 16-byte alignment
}}


pub struct WaveSimulation {
    base: RenderKit,
    compute_shader: ComputeShader,
    params: ShaderParams,
}

impl ShaderManager for WaveSimulation {
    fn init(core: &Core) -> Self {
        let base = RenderKit::new(core);

        // Simulation runs at a fixed 800x600 resolution (managed by the update buffer)
        let sim_width = 800;
        let sim_height = 600;

        let params = ShaderParams {
            speed: 1.0,
            reset: 1,
            scene: 0,
            camera_pos: [0.0, 0.0],
            camera_zoom: 1.0,
            drag: 0.1,
            restitution: 1.0,
            window_width: sim_width as u32,
            window_height: sim_height as u32,
            _pad: [0, 0],
        };

        let passes = vec![
            PassDescription::new("update", &["update"])
                .with_resolution(sim_width, sim_height)
                .with_workgroup_size([16, 16, 1]), // Fits the fixed 800x600 size
            PassDescription::new("main_image", &["update"])
                .with_workgroup_size([16, 16, 1]), // Maps to screen resolution
        ];

        let config = ComputeShader::builder()
            .with_label("Wave Simulation")
            .with_texture_format(cuneus::wgpu::TextureFormat::Rgba16Float)
            .with_multi_pass(&passes)
            .with_custom_uniforms::<ShaderParams>()
            .with_mouse()
            .build();
            
        let compute_shader = create_compute_shader(core, config, params, "wave");

        Self {
            base,
            compute_shader,
            params,
        }
    }

    fn update(&mut self, _core: &Core) {
        
    }

    fn render(&mut self, core: &Core) -> Result<(), SurfaceError> {
        let mut frame = self.base.begin_frame(core)?;

        // Handle UI Reset First
        if self.params.reset == 1 {
            self.base.start_time = std::time::Instant::now();
        }

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
        
        // This transparently runs 'update' then 'main_image' correctly using textures
        for _ in 0..iterations {
            self.compute_shader.dispatch(&mut frame.encoder, core);
        }
        
        // Clear flag AFTER GPU dispatch sees it
        if self.params.reset == 1 {
            self.params.reset = 0;
            self.compute_shader.set_custom_params(self.params, &core.queue); // prepare 0 for next frame
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

    fn handle_input(&mut self, core: &Core, event: &cuneus::winit::event::WindowEvent) -> bool {
        if self.base.default_handle_input(core, event) {return true;}
        match event {
            cuneus::winit::event::WindowEvent::MouseWheel { delta, .. } => {
                self.params.camera_zoom += match delta {
                    cuneus::winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    cuneus::winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                } as f32 * 0.1;
                true
            },
            cuneus::winit::event::WindowEvent::KeyboardInput { event, .. } => {
                match event.physical_key {
                    cuneus::winit::keyboard::PhysicalKey::Code(cuneus::winit::keyboard::KeyCode::ArrowLeft) => {
                        self.params.camera_pos[0] -= 0.1;
                        true
                    },
                    cuneus::winit::keyboard::PhysicalKey::Code(cuneus::winit::keyboard::KeyCode::ArrowRight) => {
                        self.params.camera_pos[0] += 0.1;
                        true
                    },
                    cuneus::winit::keyboard::PhysicalKey::Code(cuneus::winit::keyboard::KeyCode::ArrowUp) => {
                        self.params.camera_pos[1] -= 0.1;
                        true
                    },
                    cuneus::winit::keyboard::PhysicalKey::Code(cuneus::winit::keyboard::KeyCode::ArrowDown) => {
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
    let (app, event_loop) = cuneus::ShaderApp::new("Wave Simulation", 800, 600);
    app.run(event_loop, WaveSimulation::init)
}
