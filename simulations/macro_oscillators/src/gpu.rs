use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scene {
    Prism,
    DoubleSlit,
}

pub async fn main() {
    let mut pause = true;
    let mut frame_idx = 0;

    // Simulation dimensions
    let width = screen_width() as u32;
    let height = screen_height() as u32;

    let mut current_rt = render_target(width, height);
    let mut next_rt = render_target(width, height);
    current_rt.texture.set_filter(FilterMode::Nearest);
    next_rt.texture.set_filter(FilterMode::Nearest);

    let mut wall_rt = render_target(width, height);
    wall_rt.texture.set_filter(FilterMode::Nearest);

    let sim_material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: SIMULATION_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("resolution", UniformType::Float2),
                UniformDesc::new("time", UniformType::Float1),
                UniformDesc::new("sub_dt", UniformType::Float1),
            ],
            textures: vec!["TextureWall".to_string()],
            pipeline_params: PipelineParams {
                depth_write: false,
                depth_test: Comparison::Always,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();

    let vis_material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: VIS_FRAGMENT_SHADER,
        },
        MaterialParams {
            textures: vec!["TextureWall".to_string()],
            pipeline_params: PipelineParams {
                depth_write: false,
                depth_test: Comparison::Always,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .unwrap();

    let mut scene = Scene::DoubleSlit;

    let draw_wall = |scene: Scene, wall_rt: &mut RenderTarget, state_rt: &mut RenderTarget| {
        let cam = Camera2D {
            render_target: Some(wall_rt.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, width as _, height as _))
        };
        set_camera(&cam);
        clear_background(Color::new(1.0, 0.0, 0.0, 1.0)); // Speed 1.0 (vacuum)

        match scene {
            Scene::Prism => {
                let cx = width as f32 * 0.5;
                let cy = height as f32 * 0.5;
                let size = height as f32 * 0.4;
                // speed = 1.0 / 1.66 = 0.602
                draw_triangle(
                    vec2(cx - size * 0.5, cy - size * 0.5),
                    vec2(cx - size * 0.5, cy + size * 0.5),
                    vec2(cx + size * 0.5, cy),
                    Color::new(0.602, 0.0, 0.0, 1.0),
                );
            }
            Scene::DoubleSlit => {
                let cx = width as f32 * 0.8;
                let cy = height as f32 * 0.7;
                let slit_dist = height as f32 * 0.15;
                let slit_radius = height as f32 * 0.03;

                // Block walls with speed 0.0
                draw_rectangle(
                    cx - 10.0,
                    0.0,
                    20.0,
                    height as f32,
                    Color::new(0.0, 0.0, 0.0, 1.0),
                );
                // Cut slits with speed 1.0
                draw_rectangle(
                    cx - 10.0,
                    cy - slit_dist - slit_radius,
                    20.0,
                    slit_radius * 2.0,
                    Color::new(1.0, 0.0, 0.0, 1.0),
                );
                draw_rectangle(
                    cx - 10.0,
                    cy + slit_dist - slit_radius,
                    20.0,
                    slit_radius * 2.0,
                    Color::new(1.0, 0.0, 0.0, 1.0),
                );
            }
        }
        set_default_camera();

        // Clear State Map
        let cam = Camera2D {
            render_target: Some(state_rt.clone()),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, width as _, height as _))
        };
        set_camera(&cam);
        // Pack h=0 into RG, v=0 into BA (High=0.5, Low=0.0)
        clear_background(Color::new(0.5, 0.0, 0.5, 0.0));
        set_default_camera();
    };

    draw_wall(scene, &mut wall_rt, &mut current_rt);

    loop {
        let time = get_time() as f64;
        let dt = get_frame_time() as f64;

        if is_key_pressed(KeyCode::Space) {
            pause = !pause;
        }

        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let cam = Camera2D {
                render_target: Some(current_rt.clone()),
                ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, width as _, height as _))
            };
            set_camera(&cam);
            draw_circle(mx, my, 15.0, Color::new(1.0, 0.5, 1.0, 1.0));
            set_default_camera();
        }
        if is_key_pressed(KeyCode::R) {
            scene = Scene::Prism;
            draw_wall(scene, &mut wall_rt, &mut current_rt);
            frame_idx = 0;
        }

        if is_mouse_button_down(MouseButton::Right) {
            let (mx, my) = mouse_position();
            let cam = Camera2D {
                render_target: Some(current_rt.clone()),
                ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, width as _, height as _))
            };
            set_camera(&cam);
            draw_circle(mx, my, 20.0, Color::new(0.5, 0.5, 0.0, 1.0)); // Draw wall
            set_default_camera();
        }
        if is_key_pressed(KeyCode::S) {
            scene = Scene::DoubleSlit;
            draw_wall(scene, &mut wall_rt, &mut current_rt);
            frame_idx = 0;
        }

        if !pause {
            let mut speed_mult = 1.0;
            if is_key_down(KeyCode::Up) {
                speed_mult *= 10.0;
            }
            if is_key_down(KeyCode::Left) {
                speed_mult *= 20.0;
            }
            if is_key_down(KeyCode::Down) {
                speed_mult /= 10.0;
            }

            let sub_steps = 256;
            let physics_scale = 1.0;
            let sub_dt = (dt * speed_mult / sub_steps as f64) * physics_scale;
            sim_material.set_uniform("resolution", (width as f32, height as f32));
            sim_material.set_uniform("time", time as f32);
            sim_material.set_uniform("sub_dt", sub_dt as f32);

            sim_material.set_texture("TextureWall", wall_rt.texture.clone());

            for _ in 0..sub_steps {
                let cam = Camera2D {
                    render_target: Some(next_rt.clone()),
                    ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, width as _, height as _))
                };
                set_camera(&cam);

                gl_use_material(&sim_material);

                draw_texture_ex(
                    &current_rt.texture,
                    0.0,
                    0.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(width as _, height as _)),
                        flip_y: true,
                        ..Default::default()
                    },
                );

                gl_use_default_material();
                set_default_camera();

                std::mem::swap(&mut current_rt, &mut next_rt);
            }

            frame_idx += 1;
        }

        clear_background(BLACK);

        vis_material.set_texture("TextureWall", wall_rt.texture.clone());

        gl_use_material(&vis_material);
        draw_texture_ex(
            &current_rt.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();

        draw_text(
            "Space: Pause | R: Prism | S: Double Slit",
            10.0,
            20.0,
            20.0,
            WHITE,
        );

        if frame_idx % 20 == 0 {
            let fps = get_fps();
            print!("FPS: {}        \r", fps);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }

        next_frame().await
    }
}

const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

const SIMULATION_FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform sampler2D TextureWall;
uniform vec2 resolution;
uniform float time;
uniform float sub_dt;

// Unpack [-1.0, 1.0] from 2x8-bit channels
float unpack(vec2 rg) {
    return (rg.r + rg.g / 255.0 - 0.5) * 2.0;
}

// Pack [-1.0, 1.0] into 2x8-bit channels
vec2 pack(float val) {
    float v = clamp(val * 0.5 + 0.5, 0.0, 1.0);
    float high = floor(v * 255.0) / 255.0;
    float low = (v * 255.0 - floor(v * 255.0));
    return vec2(high, low);
}

void main() {
    vec2 step = 1.0 / resolution;
    
    vec4 current = texture2D(Texture, uv);
    float h = unpack(current.rg);
    float v = unpack(current.ba);

    float speed = texture2D(TextureWall, uv).r;
    
    // Sample neighbors 
    float n = unpack(texture2D(Texture, vec2(uv.x, uv.y + step.y)).rg);
    float s = unpack(texture2D(Texture, vec2(uv.y, uv.y - step.y)).rg);
    float e = unpack(texture2D(Texture, vec2(uv.x + step.x, uv.y)).rg);
    float w = unpack(texture2D(Texture, vec2(uv.x - step.x, uv.y)).rg);
    
    float avg_h = (n + s + e + w) / 4.0;
    float f = 6.0 * (avg_h - h);
    
    float new_v = v + f * speed * sub_dt;
    float new_h = h + new_v * sub_dt;
    
    // Wave Oscillator - Use a higher magnitude as before
    if (time < 5.0) {
        float cx = 0.2;
        float cy = 0.5;
        float dx = uv.x - cx;
        float dy = (uv.y - cy); 
        float r2 = (dx*dx*resolution.x*resolution.x + dy*dy*resolution.y*resolution.y);
        float real_size = 40.0;
        float fade = exp(-r2 / (2.0 * real_size * real_size)) / real_size;
        float freq_spawn = 0.5;
        float t_sin = abs(sin(time * 5.0)); 
        float wave = fade * cos(freq_spawn * dx * resolution.x) * t_sin * 100.0;
        new_h += wave * sub_dt;
    }
    
    // Border absorption 
    float dist_x = min(uv.x, 1.0 - uv.x);
    float dist_y = min(uv.y, 1.0 - uv.y);
    float min_dist = min(dist_x, dist_y);
    float falloff = smoothstep(0.0, 0.005, min_dist);
    new_h *= falloff;
    new_v *= falloff;
    
    gl_FragColor = vec4(pack(new_h), pack(new_v));
}
"#;

const VIS_FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;

varying vec2 uv;
uniform sampler2D Texture;
uniform sampler2D TextureWall;

float unpack(vec2 rg) {
    return (rg.r + rg.g / 255.0 - 0.5) * 2.0;
}

void main() {
    vec4 current = texture2D(Texture, uv);
    float h = unpack(current.rg);
    
    float wall_speed = texture2D(TextureWall, uv).r;
    float amp_intensity = clamp(abs(h) * 2.0, 0.0, 1.0);
    
    vec3 color = vec3(0.0);
    
    if (wall_speed < 0.1) {
        // Wall 
        color = vec3(0.5);
    } else if (wall_speed < 0.9) {
        // Prism
        color = vec3(0.0, 1.0, amp_intensity * 0.5);
    } else {
        // Vacuum
        color = vec3(h, 0.0, amp_intensity);
    }
    
    gl_FragColor = vec4(color, 1.0);
}
"#;
