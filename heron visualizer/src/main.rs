// #![feature(generic_arg_infer)]
#![feature(vec_push_within_capacity)]
#![allow(unused, dead_code)]
// Temporary code to allow static mutable references
#![allow(static_mut_refs)]
#![allow(ambiguous_glob_reexports)]
use std::{cell::OnceCell, ops::RangeInclusive};

pub use bevy::prelude::*;
use bevy::{input::{keyboard::KeyboardInput, mouse::MouseButtonInput}, math::DVec2, render::{extract_resource::ExtractResourcePlugin, render_asset::RenderAssets, storage::GpuShaderStorageBuffer}};
pub use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::WHITE,
    prelude::*,
    render::{
        batching::NoAutomaticBatching,
        render_resource::AsBindGroup,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages, ShaderType},
        storage::ShaderStorageBuffer,        
    },
};
use rug::Complex;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                watch_for_changes_override: Some(true),
                ..Default::default()
            }),
            bevy::sprite_render::Material2dPlugin::<FractalMaterial>::default(),
        ))
        .insert_resource(CameraState {
            center: Vec2::new(-9.625868e-5, 0.), // heron
            // center: Vec2::new(-0.54702383, -0.56356114), // mandelbrot
            zoom: Vec2::new(1.0, 1.0),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control)
        .run();
}

#[derive(Resource)]
struct CameraState {
    center: Vec2,
    zoom: Vec2,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct FractalMaterial {
    #[uniform(0)]
    cam_center_pos: Vec2,
    #[uniform(1)]
    zoom: Vec2,
    #[storage(2)]
    ref_orbit_length: Handle<ShaderStorageBuffer>,
}

impl bevy::sprite_render::Material2d for FractalMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/fractal_visualizer.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FractalMaterial>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    commands.spawn((Camera2d, Transform::default()));

    // Fullscreen quad with shader material
    commands.spawn((
        Mesh2d(meshes.add(Mesh::from(Rectangle::default()))),
        MeshMaterial2d(materials.add(FractalMaterial {
            cam_center_pos: Vec2::ZERO, // will be set in camera control, overriden
            zoom: Vec2::new(1.0, 1.0),
            ref_orbit_length: buffers.add(ShaderStorageBuffer::from(compute_reference_orbit(Complex::new(256), 1000))),
        })),
        Transform::from_scale(Vec3::new(1920.0, 1080.0, 1.0)),
    ));
}

fn camera_control(
    mut state: ResMut<CameraState>,
    mut materials: ResMut<Assets<FractalMaterial>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut motion_evr: MessageReader<bevy::input::mouse::MouseMotion>,
    mut scroll_evr: MessageReader<bevy::input::mouse::MouseWheel>,
    time: Res<Time>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>
) {
    let mut center_delta = DVec2::ZERO;
    // Handle mouse drag
    if mouse_input.pressed(MouseButton::Left) {
        for ev in motion_evr.read() {
            center_delta -= Into::<DVec2>::into((ev.delta)) * Into::<DVec2>::into(state.zoom) / 2.0;
        }
    }

    // Handle WASD movement
    let mut movement = DVec2::ZERO;
    let speed = 50.0;
    if keyboard.pressed(KeyCode::KeyW) { movement.y += speed; }
    if keyboard.pressed(KeyCode::KeyS) { movement.y -= speed; }
    if keyboard.pressed(KeyCode::KeyA) { movement.x -= speed; }
    if keyboard.pressed(KeyCode::KeyD) { movement.x += speed; }
    center_delta += movement * Into::<DVec2>::into(state.zoom) * time.delta().as_secs_f64();


    state.center += vec2(center_delta.x as _, center_delta.y as _);
    let prev_zoom = state.zoom;
    // Handle zoom with scroll wheel
    for ev in scroll_evr.read() {
        let zoom_factor = if ev.y > 0.0 { 0.9 } else { 1.1 };
        state.zoom *= zoom_factor;
    }
    state.zoom *= 0.99;
    if center_delta != DVec2::ZERO || prev_zoom != state.zoom {
        print!("Center: {:?}, Zoom: {:?}               \r", state.center, state.zoom);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        buffers.get_mut(&materials.iter_mut().next().unwrap().1.ref_orbit_length).unwrap().set_data(&compute_reference_orbit(Complex::with_val(256, (state.center.x, state.center.y)), 1000));
    }
    // Update material uniforms
    for material in materials.iter_mut() {
        material.1.cam_center_pos = state.center;
        material.1.zoom = state.zoom;
    }
}


fn compute_reference_orbit(center: Complex, max_iter: usize) -> Vec<Vec2> {
    let mut z = Complex::new(256); // 256-bit precision
    let mut orbit = Vec::new();
    
    for _ in 0..max_iter {
        orbit.push(Vec2::new(z.real().to_f32(), z.imag().to_f32()));
        z = z.clone() * z.clone() + &center; // TODO remove clone
    }
    orbit
}

