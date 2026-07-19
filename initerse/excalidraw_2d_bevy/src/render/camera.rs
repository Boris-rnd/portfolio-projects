use bevy::camera::visibility::RenderLayers;

use crate::*;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct OverlayCamera;

pub const OVERLAY_LAYER: RenderLayers = RenderLayers::layer(1);
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 100.0), Msaa::Sample4, GameCamera));
    // commands.spawn((
    //         Camera2d,OverlayCamera,
    //         Camera {
    //             order: 1,
    //             clear_color: ClearColorConfig::None,
    //             ..default()
    //         },
    //         OVERLAY_LAYER,
    //     ));
}


pub fn camera_pan(
    input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: MessageReader<bevy::input::mouse::MouseWheel>,
    mut cam_transform: Single<&mut Transform, With<GameCamera>>,
) {
    // if input.pressed(MouseButton::Middle) {
        let mut delta = Vec2::ZERO;
        for event in mouse_motion_events.read() {
            delta += Into::<Vec2>::into((event.x, event.y));
        }
        cam_transform.translation.x -= delta.x;
        cam_transform.translation.y += delta.y;
    // }
}

pub fn camera_movement(
    input: Res<ButtonInput<KeyCode>>,           
    mouse_input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,                            
    mut cam_transform: Single<&mut Transform, With<GameCamera>>, 
) {
    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::ArrowLeft) || input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::ArrowRight) || input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::ArrowUp) || input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::ArrowDown) || input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }


    if direction != Vec2::ZERO {
        let speed = 300.0; // pixels per second
        let delta = direction.normalize() * speed * time.delta_secs();
        cam_transform.translation.x += delta.x;
        cam_transform.translation.y += delta.y;
    }
}
