use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct CanvasCameraState {
    pub target_position: Vec2,
    pub target_zoom: f32,
    pub current_zoom: f32,
}

pub struct CanvasCameraPlugin;

impl Plugin for CanvasCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (pan_system, zoom_system, lerp_camera));
    }
}

fn pan_system(
    mut camera_state: ResMut<CanvasCameraState>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
) {
    let mut delta = Vec2::ZERO;
    let speed = 10.0;
    if keys.pressed(KeyCode::KeyW) { delta.y += speed; }
    if keys.pressed(KeyCode::KeyS) { delta.y -= speed; }
    if keys.pressed(KeyCode::KeyA) { delta.x -= speed; }
    if keys.pressed(KeyCode::KeyD) { delta.x += speed; }
    
    // Middle mouse drag pan (Excalidraw style)
    if mouse_button.pressed(MouseButton::Middle) {
        for motion in mouse_motion.read() {
            delta += Vec2::new(-motion.delta.x, motion.delta.y);
        }
    }
    
    camera_state.target_position += delta;
}

fn zoom_system(
    mut scroll: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_state: ResMut<CanvasCameraState>,
) {
    for event in scroll.read() {
        let zoom_factor = if event.y > 0.0 { 1.1 } else { 0.9 };
        camera_state.target_zoom = (camera_state.target_zoom * zoom_factor).clamp(0.05, 20.0);
    }
}

// Smooth lerp like Excalidraw
fn lerp_camera(
    mut camera_state: ResMut<CanvasCameraState>,
    mut query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    // Exponential smoothing
    camera_state.current_zoom = Lerp::lerp(
        camera_state.current_zoom,
        camera_state.target_zoom,
        1.0 - (-10.0 * time.delta_secs()).exp()
    ).clamp(0.05, 20.0);
    
    for mut transform in query.iter_mut() {
        transform.translation.x = camera_state.target_position.x;
        transform.translation.y = camera_state.target_position.y;
        transform.scale = Vec3::splat(1.0 / camera_state.current_zoom).clamp_length(0.05, 20.0);
    }
}

impl Default for CanvasCameraPlugin {
    fn default() -> Self { Self }
}

pub trait Lerp {
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(self, other: Self, t: f32) -> Self {
        self * (1.0 - t) + other * t
    }
}
