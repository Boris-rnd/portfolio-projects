use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
           .add_systems(Update, camera_movement_system);
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
        // Bloom {
        //     intensity: 0.1,
        //     ..default()
        // },
        bevy::core_pipeline::tonemapping::DebandDither::Enabled,
    ));
}

const PAN_SPEED: f32 = 500.0;
const ZOOM_SPEED: f32 = 0.1;

pub fn camera_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    kb: Res<crate::pause_menu::KeybindSettings>,
) {
    let Ok((mut transform, mut projection_enum)) = query.single_mut() else { return; };
    let Projection::Orthographic(ref mut projection) = *projection_enum else { return; };
    
    // Zooming
    for event in mouse_wheel_events.read() {
        let zoom_amount = -event.y * ZOOM_SPEED * projection.scale;
        projection.scale = (projection.scale + zoom_amount).clamp(0.1, 10.0);
    }

    let mut zoom_delta = 0.0;
    if keyboard_input.pressed(KeyCode::Minus) || keyboard_input.pressed(KeyCode::NumpadSubtract) {
        zoom_delta -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Equal) || keyboard_input.pressed(KeyCode::NumpadAdd) {
        zoom_delta += 1.0;
    }
    if zoom_delta != 0.0 {
        let zoom_amount = -zoom_delta * ZOOM_SPEED * 5.0 * projection.scale * time.delta_secs();
        projection.scale = (projection.scale + zoom_amount).clamp(0.1, 10.0);
    }
    
    // Keyboard panning
    let mut pan_dir = Vec2::ZERO;
    if keyboard_input.pressed(kb.pan_up) || keyboard_input.pressed(KeyCode::ArrowUp) {
        pan_dir.y += 1.0;
    }
    if keyboard_input.pressed(kb.pan_down) || keyboard_input.pressed(KeyCode::ArrowDown) {
        pan_dir.y -= 1.0;
    }
    if keyboard_input.pressed(kb.pan_left) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        pan_dir.x -= 1.0;
    }
    if keyboard_input.pressed(kb.pan_right) || keyboard_input.pressed(KeyCode::ArrowRight) {
        pan_dir.x += 1.0;
    }
    
    if pan_dir != Vec2::ZERO {
        let velocity = pan_dir.normalize() * PAN_SPEED * projection.scale * time.delta_secs();
        transform.translation += velocity.extend(0.0);
    }
    
    // Mouse drag panning (Middle Mouse Button)
    if mouse_input.pressed(MouseButton::Middle) {
        let mut drag_delta = Vec2::ZERO;
        for event in mouse_motion_events.read() {
            drag_delta += event.delta;
        }
        transform.translation.x -= drag_delta.x * projection.scale;
        transform.translation.y += drag_delta.y * projection.scale;
    } else {
        mouse_motion_events.clear();
    }
}

