use cuneus::winit::{dpi::PhysicalPosition, event::ElementState};
use cuneus_simulations::*;
use glam::{Vec3, uvec2};
use log::info;
use world::{MapDataPacked, VoxelChunk};

pub const SHADER_PATH: &str = "raytrace";

cuneus::uniform_params! {
    #[derive(Default)]
    struct ShaderParams {
        camera_pos: Vec3,
        camera_dir: Vec3,
        camera_zoom: f32,
        fov: f32,
        root_max_depth: u32,
        accum_frames: u32,
        pad: [u32; 2]
    }
}
impl ShaderParams {
    fn new(root_max_depth: u32) -> Self {
        Self {
            camera_pos: Vec3::new(0., 0., 0.),
            camera_dir: Vec3::new(0., 0., 1.),
            camera_zoom: 1.0,
            fov: 90.0,
            root_max_depth,
            accum_frames: 0,
            pad: Default::default(),
        }
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
    move_speed: f32,
    accum_frame_size: glam::UVec2,
}

impl ShaderManager for Raytracer {
    fn init(core: &Core) -> Self {
        let base = RenderKit::new(core);

        let accum_frame_size = uvec2(800, 600);

        let passes = vec![
            // Update logic
            // PassDescription::new("update", &[]).with_workgroup_size([particle_count.div_ceil(64) as u32,1,1,]),
            // Render logic
            // PassDescription::new("clear_screen", &[]),
            // PassDescription::new("splat", &["update"]).with_workgroup_size([particle_count.div_ceil(64) as u32, 1, 1]),
            PassDescription::new("render", &[]),
        ];
        info!("Loading world...");
        let world = world::parser::load_world("shaders/sponza.vox").unwrap();
        // let mut world = world::GameWorld::new(4096, 8);
        log::debug!(
            "Created {} chunks, with {} block reallocations and {} pad blocks and {} chunk reallocations",
            world.voxel_chunks.len(),
            world.realloc_count,
            world.pad_count,
            world.realloc_count_chunks
        );
        info!("Done");
        let root_max_depth = world.root_max_depth();
        let params = ShaderParams::new(root_max_depth);

        let config = ComputeShader::builder()
            .with_label(&format!("Cuneus - {SHADER_PATH}"))
            .with_multi_pass(&passes)
            .with_custom_uniforms::<ShaderParams>()
            .with_mouse()
            .with_storage_buffer(StorageBufferSpec::new(
                "voxel_chunks",
                (world.voxel_chunks.len() * std::mem::size_of::<VoxelChunk>()) as u64,
            ))
            .with_storage_buffer(StorageBufferSpec::new(
                "block_data0",
                (world.block_data[0].len() * std::mem::size_of::<MapDataPacked>().max(64)) as u64,
            ))
            .with_storage_buffer(StorageBufferSpec::new(
                "block_data1",
                (world.block_data[1].len() * std::mem::size_of::<MapDataPacked>()).max(64) as u64,
            ))
            .with_storage_buffer(StorageBufferSpec::new(
                "block_data2",
                (world.block_data[2].len() * std::mem::size_of::<MapDataPacked>()).max(64) as u64,
            ))
            .with_storage_buffer(StorageBufferSpec::new(
                "block_data3",
                (world.block_data[3].len() * std::mem::size_of::<MapDataPacked>()).max(64) as u64,
            ))
            .with_storage_buffer(StorageBufferSpec::new(
                "accum_texture",
                4 * (1920 * 1080) as u64,
            )) // Make the buffer as big as possible, then we will only write to a subset
            // .with_atomic_buffer(1)
            .build();
        let compute_shader = create_compute_shader(
            core,
            config,
            params,
            &format!("compiled/{SHADER_PATH}-compiled"),
        );
        core.queue.write_buffer(
            &compute_shader.storage_buffers[0],
            0,
            bytemuck::cast_slice(&world.voxel_chunks),
        );
        for i in 0..4 {
            core.queue.write_buffer(
                &compute_shader.storage_buffers[1 + i as usize],
                0,
                bytemuck::cast_slice(&world.block_data[i as usize]),
            );
        }
        Self {
            base,
            compute_shader,
            params,
            dragging: false,
            prev_mouse_pos: PhysicalPosition::new(0., 0.),
            move_speed: 1.0,
            accum_frame_size,
        }
    }

    fn update(&mut self, _core: &Core) {
        // self.params.reset = 0; // Stop resetting after first frame
    }

    fn render(&mut self, core: &Core) -> Result<(), SurfaceError> {
        let mut frame = self.base.begin_frame(core)?;
        self.params.accum_frames += 1;

        // Update time and params
        let current_time = self.base.controls.get_time(&self.base.start_time);
        self.compute_shader
            .set_time(current_time, 1.0 / 60.0, &core.queue);
        self.compute_shader
            .set_custom_params(self.params, &core.queue);
        self.compute_shader
            .update_mouse_uniform(&self.base.mouse_tracker.uniform, &core.queue);

        let mut controls_request = self.base.controls.get_ui_request(
            &self.base.start_time,
            &core.size,
            self.base.fps_tracker.fps(),
        );
        // UI
        let full_output = self.base.render_ui(core, |ctx| {
            RenderKit::apply_default_style(ctx);
            egui::Window::new("Params").show(ctx, |ui| {
                // ui.add(egui::Slider::new(&mut self.params.gravity, -1.0..=10.).text("Gravity").logarithmic(true).clamping(SliderClamping::Never));
                // ui.add(egui::Slider::new(&mut self.params.particle_size, 1..=5).text("Size (px)").clamping(SliderClamping::Never));
                ui.add(
                    egui::Slider::new(&mut self.params.camera_zoom, 0.1..=5.0)
                        .text("Zoom")
                        .logarithmic(true)
                        .clamping(SliderClamping::Never),
                );
                ui.add(
                    egui::Slider::new(&mut self.move_speed, 0.1..=5.0)
                        .text("Move Speed")
                        .logarithmic(true)
                        .clamping(SliderClamping::Never),
                );
                ui.add(
                    egui::Slider::new(&mut self.params.camera_pos[0], 0.1..=5.0)
                        .text("Camera Pos X")
                        .clamping(SliderClamping::Never),
                );
                ui.add(
                    egui::Slider::new(&mut self.params.camera_pos[1], 0.1..=5.0)
                        .text("Camera Pos Y")
                        .clamping(SliderClamping::Never),
                );
                ui.add(
                    egui::Slider::new(&mut self.params.camera_pos[2], 0.1..=5.0)
                        .text("Camera Pos Z")
                        .clamping(SliderClamping::Never),
                );
                ui.add(
                    egui::Slider::new(&mut self.params.camera_dir[0], 0.1..=5.0)
                        .text("Camera Dir X")
                        .clamping(SliderClamping::Never),
                );
                ui.add(
                    egui::Slider::new(&mut self.params.camera_dir[1], 0.1..=5.0)
                        .text("Camera Dir Y")
                        .clamping(SliderClamping::Never),
                );
                ui.add(
                    egui::Slider::new(&mut self.params.camera_dir[2], 0.1..=5.0)
                        .text("Camera Dir Z")
                        .clamping(SliderClamping::Never),
                );
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
        self.accum_frame_size = uvec2(core.size.width, core.size.height);
        self.params.accum_frames = 0;
        // self.base.compute_shader.unwrap().dispatch_stage(encoder, core, 1);
        // self.base.compute_shader.unwrap().storage_buffers[4].
    }

    fn handle_input(&mut self, core: &Core, event: &winit::event::WindowEvent) -> bool {
        if self.base.default_handle_input(core, event) {
            return true;
        }
        let handled = match event {
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                // Todo zoom in and out
                self.params.camera_zoom += match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                } * 0.005;
                true
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyW) => {
                        // Forward
                        self.params.camera_pos += self.params.camera_dir * self.move_speed;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyA) => {
                        // Right
                        let side = glam::Vec3::new(
                            self.params.camera_dir.z,
                            0.0,
                            -self.params.camera_dir.x,
                        );
                        self.params.camera_pos += side * self.move_speed;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyS) => {
                        // Backward
                        self.params.camera_pos -= self.params.camera_dir * self.move_speed;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyD) => {
                        // Left
                        let side = glam::Vec3::new(
                            -self.params.camera_dir.z,
                            0.0,
                            self.params.camera_dir.x,
                        );
                        self.params.camera_pos += side * self.move_speed;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::Space) => {
                        // Up
                        self.params.camera_pos.y += self.move_speed;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::ShiftLeft) => {
                        // Down
                        self.params.camera_pos.y -= self.move_speed;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::ArrowDown) => {
                        self.move_speed *= 0.5;
                        true
                    }
                    winit::keyboard::PhysicalKey::Code(KeyCode::ArrowUp) => {
                        self.move_speed *= 2.0;
                        true
                    }
                    _ => false,
                }
            }
            winit::event::WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                use glam::*;
                let mouse_delta = vec2(position.x as f32, position.y as f32)
                    - vec2(self.prev_mouse_pos.x as f32, self.prev_mouse_pos.y as f32);
                self.prev_mouse_pos = *position;
                if self.dragging {
                    let sensitivity = vec2(1., -1.) * 0.003;

                    let yaw = Quat::from_axis_angle(Vec3::Y, -mouse_delta.x * sensitivity.x);
                    let right = Vec3::Y.cross(self.params.camera_dir).normalize();
                    let pitch = Quat::from_axis_angle(right, -mouse_delta.y * sensitivity.y);

                    let direction = (yaw * pitch * self.params.camera_dir).normalize();
                    self.params.camera_dir = direction;
                    true
                } else {
                    false
                }
            }
            winit::event::WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                if button == &winit::event::MouseButton::Left {
                    match state {
                        ElementState::Pressed => self.dragging = true,
                        ElementState::Released => self.dragging = false,
                    }
                }

                true
            }

            _ => false,
        };
        if handled {
            self.params.accum_frames = 0;
        }
        handled
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (app, event_loop) = ShaderApp::new(&format!("Cuneus - {SHADER_PATH}"), 800, 600);
    app.run(event_loop, Raytracer::init)
}
