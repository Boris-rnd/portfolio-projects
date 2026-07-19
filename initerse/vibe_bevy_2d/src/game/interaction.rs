use crate::{
    *,
    connection::{Connection, ItemMovement},
    grid::{world_to_grid, grid_to_world, TILE_SIZE},
    ui::SelectedBuilding,
    GlobalInventory,
};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DragState>()
           .add_systems(Update, (
               building_placement_system,
               connection_drag_system,
               ghost_preview_system,
               keyboard_interaction_system,
           ));
    }
}

/// Tracks the state of drag and drop for connections
#[derive(Resource, Default)]
pub struct DragState {
    /// The entity we started dragging from (a Collector)
    pub connecting_from: Option<Entity>,
    /// The last grid position processed for hold-to-place
    pub last_placed_grid: Option<IVec2>,
}


/// The live line while the user is dragging a connection
#[derive(Component)]
pub struct DragLine;


fn connection_drag_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut drag_state: ResMut<DragState>,
    buildings: Query<(Entity, &GlobalTransform, &GridPosition, Option<&Collector>, Option<&Storage>)>,
    mut collectors: Query<&mut Connection>,
    mut drag_line_q: Query<(Entity, &mut Transform, &mut Sprite), With<DragLine>>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_q.single() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };

    if cursor_pos.y > window.resolution.height() - 100.0 {
        // Clean up drag line if in UI
        for (line_e, _, _) in &drag_line_q {
            commands.entity(line_e).despawn();
        }
        drag_state.connecting_from = None;
        return;
    }

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };
    let grid_pos = world_to_grid(world_pos);

    // Right-click drag to start connection from a Collector
    if mouse_input.just_pressed(MouseButton::Right) {
        if drag_state.connecting_from.is_none() {
            // Find an entity that HAS a Collector at this grid position
            if let Some((entity, _, _, _, _)) = buildings.iter().find(|(_, _, p, collector, _)| p.0 == grid_pos && collector.is_some()) {
                drag_state.connecting_from = Some(entity);
            }
        }
    }

    // While dragging: update the live preview line
    if let Some(source_entity) = drag_state.connecting_from {
        if mouse_input.pressed(MouseButton::Right) {
            if let Ok((_, source_transform, _, _, _)) = buildings.get(source_entity) {
                let start = source_transform.translation().truncate();

                // Snap end to storage center if hovering one
                let end = buildings
                    .iter()
                    .find(|(_, _, p, _, storage)| p.0 == grid_pos && storage.is_some())
                    .map(|(_, t, _, _, _)| t.translation().truncate())
                    .unwrap_or(world_pos);

                let diff = end - start;
                let length = diff.length();
                let angle = diff.y.atan2(diff.x);
                let mid = (start + end) / 2.0;

                if drag_line_q.is_empty() {
                    // Spawn drag line
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.9, 0.9, 0.3, 0.8),
                            custom_size: Some(Vec2::new(length, 3.0)),
                            ..default()
                        },
                        Transform::from_translation(mid.extend(2.5))
                            .with_rotation(Quat::from_rotation_z(angle)),
                        DragLine,
                    ));
                } else {
                    let Ok((_, mut t, mut sp)) = drag_line_q.single_mut() else { return };
                    sp.custom_size = Some(Vec2::new(length, 3.0));
                    t.translation = mid.extend(2.5);
                    t.rotation = Quat::from_rotation_z(angle);
                }
            }
        } else {
            // Released
            // Remove the preview line
            for (line_e, _, _) in &drag_line_q {
                commands.entity(line_e).despawn();
            }

            // Try to make the connection
            if let Some((target_entity, _, _, _, _)) = buildings
                .iter()
                .find(|(e, _, p, _, storage)| p.0 == grid_pos && storage.is_some() && *e != source_entity)
            {
                if let Ok(mut connection) = collectors.get_mut(source_entity) {
                    if !connection.targets.contains(&target_entity) {
                        connection.targets.push(target_entity);
                    }
                }
            }

            drag_state.connecting_from = None;
        }
    }
}
