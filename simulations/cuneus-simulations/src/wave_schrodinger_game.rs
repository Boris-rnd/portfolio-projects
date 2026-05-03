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
    // fst bit = reset
    // snd bit = edge damping
    flags: u32,
    scene: u32,
    camera_pos: [f32; 2],
    camera_zoom: f32,
    force: f32,
    restitution: f32,
    window_width: u32,
    window_height: u32,
    ping: u32,       // which buffer is the READ side (0 or 1)
    scroll: f32,
    control: f32,
    _pad: [u32; 2],
    // Total: 4+4+4+4 + 8+4+4 + 4+4+4+4 + 4+4+8 = 80 bytes (multiple of 16 ✓)
}}


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Default)]
struct Cell {
    // old_y is now read from the previous ping-pong buffer
    real_y: f32,
    imag_y: f32,
    mass: f32,
    // _pad: u32,
    _pad: [u32; 1],
}



struct WaveSchrodingerGame {
    base: RenderKit,
    compute_shader: ComputeShader,
    params: ShaderParams,
    frame_count: u32,
    // shader_text: String,
}

impl ShaderManager for WaveSchrodingerGame {
    fn init(core: &Core) -> Self {
        let base = RenderKit::new(core);

        let cell_count = 800 * 600;
        let cells = vec![
            Cell {
                ..Default::default()
            };
            cell_count
        ];

        let params = ShaderParams {
            cell_count: cell_count as u32,
            speed: 2.0,
            flags: 0b11, // Default edge damping
            scene: 0,
            camera_pos: [-0.5, -0.5],
            camera_zoom: 1.0,
            force: 0.1,
            restitution: 1.0,
            window_width: 800,
            window_height: 600,
            ping: 0,
            scroll: 0.0,
            control: 0.0,
            _pad: [0; 2],
        };

        // Stage indices (used in render with dispatch_stage_with_workgroups)
        // 0 = update, 1 = clear_screen, 2 = render
        let passes = vec![
            PassDescription::new("update", &[]).with_workgroup_size([
                cell_count.div_ceil(64) as u32,
                1,
                1,
            ]),
            PassDescription::new("clear_screen", &[]).with_workgroup_size([16, 1, 1]),
            PassDescription::new("render", &[]).with_workgroup_size([
                cell_count.div_ceil(64) as u32,
                1,
                1,
            ]),
        ];

        let cell_buf_size = (cell_count * std::mem::size_of::<Cell>()) as u64;
        let config = ComputeShader::builder()
            .with_label("Wave Schrodinger Game")
            .with_multi_pass(&passes)
            .with_custom_uniforms::<ShaderParams>()
            .with_mouse()
            // Two ping-pong buffers: [0] = cells_a, [1] = cells_b
            .with_storage_buffer(StorageBufferSpec::new("cells_a", cell_buf_size))
            .with_storage_buffer(StorageBufferSpec::new("cells_b", cell_buf_size))
            .build();

        let mut args = pico_args::Arguments::from_env();
        let freq = args.value_from_fn("--freq", |val| val.parse::<f32>()).unwrap_or(1500.0);
        let size = args.value_from_fn("--size", |val| val.parse::<f32>()).unwrap_or(0.025);
        let potential = args.value_from_str("--potential").unwrap_or("step(0.1, max(-triangle((uv-vec2(0., 0.2))*8.0), 0.))".to_string());
        dbg!(freq, &potential, size);
        // Writes a new shader, with the arguments replaced:
        let unprocessed_shader = std::fs::read_to_string("shaders/wave_schrodinger_game.wgsl").expect("Unable to find shader");
        let shader = unprocessed_shader.replace("{INPUTTED_FREQ}", &freq.to_string()).replace("{INPUTTED_SIZE}", &size.to_string()).replace("{INPUTTED_POTENTIAL}", &potential);
        std::fs::write("shaders/wave_schrodinger_game_generated.wgsl", shader).expect("Unable to write shader");

        let compute_shader = create_compute_shader(core, config, params, "wave_schrodinger_game_generated");
        // Initialise both buffers identically
        core.queue.write_buffer(
            &compute_shader.storage_buffers[0],
            0,
            bytemuck::cast_slice(&cells),
        );
        core.queue.write_buffer(
            &compute_shader.storage_buffers[1],
            0,
            bytemuck::cast_slice(&cells),
        );

        Self {
            base,
            compute_shader,
            params,
            frame_count: 0,
            // shader_text: get_shader_text()
        }
    }

    fn update(&mut self, _core: &Core) {
        // Stop resetting after first frame
    }

    fn render(&mut self, core: &Core) -> Result<(), SurfaceError> {
        let mut frame = self.base.begin_frame(core)?;

        // Needed because not using dispatch anymore
        self.compute_shader.check_hot_reload(&core.device);

        // Update time and params
        let current_time = self.base.controls.get_time(&self.base.start_time);
        self.compute_shader.set_time(current_time, 1.0 / 60.0, &core.queue);
        
        self.compute_shader.time_uniform.data.frame = self.frame_count;
        self.compute_shader.time_uniform.update(&core.queue);
        self.frame_count += 1;
        let size = core.window.inner_size();
        self.params.window_width = size.width as u32;
        self.params.window_height = size.height as u32;
        self.compute_shader.update_mouse_uniform(&self.base.mouse_tracker.uniform, &core.queue);

        let mut controls_request = self.base.controls.get_ui_request(&self.base.start_time, &core.size, self.base.fps_tracker.fps());
        // Scroll obstacles to the left each frame to follow the wave.
        self.params.scroll += 150.0 * (1.0 / 60.0);
        if self.params.scroll > 1e4 {
            self.params.scroll -= 1e4;
        }
        self.params.control *= 0.92;
        // UI
        let full_output = self.base.render_ui(core, |ctx| {
            RenderKit::apply_default_style(ctx);
            egui::Window::new("Wave Schrodinger Simulation").show(ctx, |ui| {
                ui.add(egui::Slider::new(&mut self.params.camera_zoom, 0.1..=5.0).text("Zoom").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.speed, 0.0..=20.).text("Speed").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.force, -1.0..=1.0).text("Force").clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.restitution, 0.0..=10.0).text("Restitution").logarithmic(true).clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_pos[0], -1.0..=1.0).text("Camera x").clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.camera_pos[1], -1.0..=1.0).text("Camera y").clamping(egui::SliderClamping::Never));
                ui.add(egui::Slider::new(&mut self.params.scene, 0..=2).text("Scene (0=Wave, 1=Prism, 2=Slit)"));
                let mut edge_damping = (self.params.flags & 2u32) == 2u32;
                if ui.checkbox(&mut edge_damping, "Edge damping").changed() {
                    if edge_damping {
                        self.params.flags |= 2u32;
                    } else {
                        self.params.flags &= !2u32;
                    }
                }
                if ui.button("Reset").clicked() {
                    self.params.flags |= 1;
                }
                ui.label("Press J to push up, K to push down.");
                ui.label("Game walls spawn from the right and move left.");
                ui.separator();
                ShaderControls::render_controls_widget(ui, &mut controls_request);
            });
        });

        // Stage indices matching the passes vec defined in init()
        const UPDATE: usize = 0;
        const CLEAR_SCREEN: usize = 1;
        const RENDER: usize = 2;

        let cell_count = self.params.cell_count;
        let update_workgroups = [cell_count.div_ceil(64), 1, 1];
        let clear_workgroups = [16, 1, 1];
        let render_workgroups = [cell_count.div_ceil(64), 1, 1];

        // ---------- update: runs N times, ping-pong flips each iteration ----------
        let iterations: u32 = 20;
        for _ in 0..iterations {
            self.compute_shader.set_custom_params(self.params, &core.queue);
            self.compute_shader.dispatch_stage_with_workgroups(
                &mut frame.encoder,
                UPDATE,
                update_workgroups,
            );
            // flush so the next iteration sees what this one wrote
            frame.encoder = core.flush_encoder(frame.encoder);
            // flip: what was the write side becomes the read side
            self.params.ping = 1 - self.params.ping;
        }

        // ---------- clear_screen: once ----------
        self.compute_shader.set_custom_params(self.params, &core.queue);
        self.compute_shader.dispatch_stage_with_workgroups(
            &mut frame.encoder,
            CLEAR_SCREEN,
            clear_workgroups,
        );
        frame.encoder = core.flush_encoder(frame.encoder);

        // ---------- render: once ----------
        self.compute_shader.set_custom_params(self.params, &core.queue);
        self.compute_shader.dispatch_stage_with_workgroups(
            &mut frame.encoder,
            RENDER,
            render_workgroups,
        );

        if self.params.flags & 1 == 1 {
            self.base.start_time = std::time::Instant::now();
        }

        // Render to screen
        self.base.renderer.render_to_view(
            &mut frame.encoder,
            &frame.view,
            &self.compute_shader.get_output_texture().bind_group,
        );

        self.base.end_frame(core, frame, full_output);
        self.params.flags &= !1;
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
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyJ) => {
                        self.params.control = 1.0;
                        true
                    },
                    winit::keyboard::PhysicalKey::Code(KeyCode::KeyK) => {
                        self.params.control = -1.0;
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
    let (app, event_loop) = ShaderApp::new("Wave Schrodinger Game", 800, 600);
    app.run(event_loop, WaveSchrodingerGame::init)
}
