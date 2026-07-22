use cuneus::winit::{dpi::PhysicalPosition, event::ElementState};
use cuneus_simulations::*;

pub const SHADER_PATH: &str = "raytrace";

cuneus::uniform_params! {
    struct ShaderParams {
    camera_pos: [f32; 3],
    camera_dir: [f32; 3],
    camera_zoom: f32,
    padding: [u32;1]
}}
impl Default for ShaderParams {
    fn default() -> Self {
        Self { camera_pos: [0.; 3], camera_dir: [0.,0., 1.], camera_zoom: 1.0, padding: [0] }
    }
}

cuneus::uniform_params! {
    struct Voxel {
        id: f32,
        pos_x: f32,
        pos_y:f32,
        pos_z: f32
    }
}


struct Raytracer {
    base: RenderKit,
    compute_shader: ComputeShader,
    params: ShaderParams,
    dragging: bool,
    prev_mouse_pos: PhysicalPosition<f64>,
    move_speed: f32
}

impl ShaderManager for Raytracer {
    fn init(core: &Core) -> Self {
        let base = RenderKit::new(core);

        let params = ShaderParams::default();
        let passes = vec![
            // Update logic
            // PassDescription::new("update", &[]).with_workgroup_size([particle_count.div_ceil(64) as u32,1,1,]),
            // Render logic
            // PassDescription::new("clear_screen", &[]),
            // PassDescription::new("splat", &["update"]).with_workgroup_size([particle_count.div_ceil(64) as u32, 1, 1]),
            PassDescription::new("render", &[]),
        ];

        let voxels = vec![
            Voxel {
                id: 0.0,
                pos_x: 0.0,
                pos_y: 0.0,
                pos_z: 0.0,
            };
            100
        ];

        let config = ComputeShader::builder()
            .with_label(&format!("Cuneus - {SHADER_PATH}"))
            .with_multi_pass(&passes)
            .with_custom_uniforms::<ShaderParams>()
            .with_mouse()
            .with_storage_buffer(StorageBufferSpec::new(
                "voxels",
                (voxels.len() * std::mem::size_of::<Voxel>()) as u64,
            ))
            // .with_atomic_buffer(1)
            .build();
        let compute_shader = create_compute_shader(core, config, params, &format!("compiled/{SHADER_PATH}-compiled"));
        core.queue.write_buffer(
            &compute_shader.storage_buffers[0],
            0,
            bytemuck::cast_slice(&voxels),
        );
        Self {
            base,
            compute_shader,
            params,
            dragging: false,
            prev_mouse_pos: PhysicalPosition::new(0., 0.),
            move_speed: 1.0,
        }
    }

    fn update(&mut self, _core: &Core) {
        // self.params.reset = 0; // Stop resetting after first frame
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
            egui::Window::new("Params").show(ctx, |ui| {
                // ui.add(egui::Slider::new(&mut self.params.gravity, -1.0..=10.).text("Gravity").logarithmic(true).clamping(SliderClamping::Never));
                // ui.add(egui::Slider::new(&mut self.params.particle_size, 1..=5).text("Size (px)").clamping(SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_zoom, 0.1..=5.0).text("Zoom").logarithmic(true).clamping(SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.move_speed, 0.1..=5.0).text("Move Speed").logarithmic(true).clamping(SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_pos[0], 0.1..=5.0).text("Camera Pos X").clamping(SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_pos[1], 0.1..=5.0).text("Camera Pos Y").clamping(SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_pos[2], 0.1..=5.0).text("Camera Pos Z").clamping(SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_dir[0], 0.1..=5.0).text("Camera Dir X").clamping(SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_dir[1], 0.1..=5.0).text("Camera Dir Y").clamping(SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_dir[2], 0.1..=5.0).text("Camera Dir Z").clamping(SliderClamping::Never));
                // ui.add(egui::Slider::new(&mut self.params.speed, 0.0..=10.).text("Speed").logarithmic(true).clamping(SliderClamping::Never));
                // if ui.button("Reset").clicked() {
                //     self.params.reset = 1;
                // }
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
        let cam_dir = glam::Vec3::new(self.params.camera_dir[0], self.params.camera_dir[1], self.params.camera_dir[2]); // We consider camera_dir to be always normalized
        let cam_pos = glam::Vec3::new(self.params.camera_pos[0], self.params.camera_pos[1], self.params.camera_pos[2]);
        match event {
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                // Todo zoom in and out
                self.params.camera_zoom += match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                } * 0.005;
                true
            },
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyW) => {
                        // Forward
                        self.params.camera_pos = [cam_pos.x + cam_dir.x * self.move_speed, cam_pos.y + cam_dir.y * self.move_speed, cam_pos.z + cam_dir.z * self.move_speed];
                        
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyD) => {
                        // Right
                        let side = glam::Vec3::new(cam_dir.z, 0.0, -cam_dir.x);
                        self.params.camera_pos = [cam_pos.x + side.x * self.move_speed, cam_pos.y + side.y * self.move_speed, cam_pos.z + side.z * self.move_speed];
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyS) => {
                        // Backward
                        self.params.camera_pos = [cam_pos.x - cam_dir.x * self.move_speed, cam_pos.y - cam_dir.y * self.move_speed, cam_pos.z - cam_dir.z * self.move_speed];
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyA) => {
                        // Left
                        let side = glam::Vec3::new(-cam_dir.z, 0.0, cam_dir.x);
                        self.params.camera_pos = [cam_pos.x + side.x * self.move_speed, cam_pos.y + side.y * self.move_speed, cam_pos.z + side.z * self.move_speed];
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::Space) => {
                        // Up
                        self.params.camera_pos = [cam_pos.x, cam_pos.y + self.move_speed, cam_pos.z];
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::ShiftLeft) => {
                        // Down
                        self.params.camera_pos = [cam_pos.x, cam_pos.y - self.move_speed, cam_pos.z];
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::ArrowDown) => {
                        self.move_speed *= 0.5;
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::ArrowUp) => {
                        self.move_speed *= 2.0;
                        true
                    },
                    _ => false
                }
            },
            winit::event::WindowEvent::CursorMoved { device_id: _, position } => {
                use glam::*;
                let mouse_delta = vec2(position.x as f32, position.y as f32) - vec2(self.prev_mouse_pos.x as f32, self.prev_mouse_pos.y as f32);
                self.prev_mouse_pos = *position;
                if self.dragging {
                    let cam_dir = vec3(self.params.camera_dir[0], self.params.camera_dir[1], self.params.camera_dir[2]);
                    let sensitivity = vec2(1., -1.) * 0.003;

                    let yaw = Quat::from_axis_angle(Vec3::Y, -mouse_delta.x * sensitivity.x);
                    let right = Vec3::Y.cross(cam_dir).normalize();
                    let pitch = Quat::from_axis_angle(right, -mouse_delta.y * sensitivity.y);

                    let direction = (yaw * pitch * cam_dir).normalize();
                    self.params.camera_dir = [direction.x, direction.y, direction.z];
                    true
                } else {
                    false
                }
            }
            winit::event::WindowEvent::MouseInput { device_id: _, state, button } => {
                if button == &winit::event::MouseButton::Left {match state {
                    ElementState::Pressed => self.dragging = true,
                    ElementState::Released => self.dragging = false,
                }}
                
                true
            }
            
            _ => false
        }
    }
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (app, event_loop) = ShaderApp::new(&format!("Cuneus - {SHADER_PATH}"), 800, 600);
    app.run(event_loop, Raytracer::init)
}
