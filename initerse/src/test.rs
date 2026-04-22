use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

pub fn camera_movement_system(
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let Ok((mut transform, mut projection)) = query.get_single_mut() else { return; };
    if let Projection::Orthographic(ref mut ortho) = *projection {
        ortho.scale += 1.0;
    }
}
