use bevy::prelude::*;
use crate::{
    building::{GridPosition, Collector, Storage, Turret, GoldNode, BuildingMarker, BuildingAssets},
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

#[derive(Component)]
pub struct GhostPreview;

/// The live line while the user is dragging a connection
#[derive(Component)]
pub struct DragLine;

fn keyboard_interaction_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected_building: ResMut<SelectedBuilding>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        *selected_building = SelectedBuilding::None;
    }
}

fn ghost_preview_system(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    selected_building: Res<SelectedBuilding>,
    existing_buildings: Query<&GridPosition, With<BuildingMarker>>,
    gold_nodes: Query<&GridPosition, With<GoldNode>>,
    mut ghost_q: Query<(Entity, &mut Transform, &mut Sprite, &mut Visibility), With<GhostPreview>>,
    building_assets: Option<Res<BuildingAssets>>,
    global_inventory: Res<GlobalInventory>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_q.single() else { return };

    // Ensure ghost entity exists
    if ghost_q.is_empty() {
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.9)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
            GhostPreview,
            Visibility::Hidden,
        ));
        return;
    }

    let Ok((_, mut transform, mut sprite, mut visibility)) = ghost_q.single_mut() else { return };

    let should_show = *selected_building != SelectedBuilding::None;

    if !should_show {
        *visibility = Visibility::Hidden;
        sprite.image = Handle::default();
        return;
    }

    let Some(cursor_pos) = window.cursor_position() else {
        *visibility = Visibility::Hidden;
        return;
    };

    if cursor_pos.y > window.resolution.height() - 100.0 {
        *visibility = Visibility::Hidden;
        return;
    }

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        *visibility = Visibility::Hidden;
        return;
    };

    let grid_pos = world_to_grid(world_pos);
    let world_snap = grid_to_world(grid_pos);
    transform.translation = world_snap.extend(3.0);
    *visibility = Visibility::Visible;

    let occupied = existing_buildings.iter().any(|pos| pos.0 == grid_pos);
    let can_afford = global_inventory.total_gold >= 5.0;
    let on_gold_node = gold_nodes.iter().any(|pos| pos.0 == grid_pos);

    match *selected_building {
        SelectedBuilding::Collector | SelectedBuilding::Storage | SelectedBuilding::Turret => {
            let mut can_place = !occupied && can_afford;
            if *selected_building == SelectedBuilding::Collector && !on_gold_node {
                can_place = false;
            }

            if !can_place {
                sprite.image = Handle::default();
                sprite.color = Color::srgba(0.9, 0.1, 0.1, 0.5);
            } else {
                if let Some(ref assets) = building_assets {
                    sprite.image = match *selected_building {
                        SelectedBuilding::Collector => assets.collector.clone(),
                        SelectedBuilding::Storage   => assets.storage.clone(),
                        SelectedBuilding::Turret    => assets.turret.clone(),
                        _ => Handle::default(),
                    };
                }
                sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.55);
            }
        }
        SelectedBuilding::Destroy => {
            sprite.image = Handle::default();
            sprite.color = Color::srgba(0.9, 0.1, 0.1, 0.45);
        }
        _ => {}
    }
}

fn building_placement_system(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    selected_building: Res<SelectedBuilding>,
    existing_buildings: Query<(Entity, &GridPosition), With<BuildingMarker>>,
    gold_nodes: Query<&GridPosition, With<GoldNode>>,
    mut drag_state: ResMut<DragState>,
    building_assets: Option<Res<BuildingAssets>>,
    global_inventory: Res<GlobalInventory>,
    mut storages: Query<&mut Storage>,
    mut connections: Query<&mut Connection>,
    items: Query<(Entity, &ItemMovement)>,
) {
    if *selected_building == SelectedBuilding::None {
        return;
    }

    // Only act when LEFT mouse is pressed or held
    let just_pressed = mouse_input.just_pressed(MouseButton::Left);
    let held = mouse_input.pressed(MouseButton::Left);

    // We need at least a press or hold
    if !held {
        drag_state.last_placed_grid = None;
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_q.single() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };

    if cursor_pos.y > window.resolution.height() - 100.0 {
        return;
    }

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };
    let grid_pos = world_to_grid(world_pos);

    // Avoid repeating on the same cell during a drag
    if !just_pressed {
        if drag_state.last_placed_grid == Some(grid_pos) {
            return;
        }
    }

    // Destroy mode
    if *selected_building == SelectedBuilding::Destroy {
        if let Some((entity, _)) = existing_buildings.iter().find(|(_, pos)| pos.0 == grid_pos) {
            // Cleanup connections pointing to this entity
            for mut conn in &mut connections {
                conn.targets.retain(|&t| t != entity);
            }
            // Cleanup items in flight to this entity
            for (item_entity, movement) in &items {
                if movement.target_entity == entity {
                    commands.entity(item_entity).despawn();
                }
            }
            commands.entity(entity).despawn();
        }
        drag_state.last_placed_grid = Some(grid_pos);
        return;
    }

    // Cost Check
    if global_inventory.total_gold < 5.0 {
        return;
    }

    // Check if occupied
    if existing_buildings.iter().any(|(_, pos)| pos.0 == grid_pos) {
        return;
    }

    // Gold Node Check for Collector
    if *selected_building == SelectedBuilding::Collector {
        if !gold_nodes.iter().any(|pos| pos.0 == grid_pos) {
            return;
        }
    }

    // Deduct cost from storages
    let mut remaining_to_deduct = 5.0f32;
    // Sort or prioritize base? Let's just iterate for now as per instructions
    for mut storage in &mut storages {
        let deduct = remaining_to_deduct.min(storage.current_amount);
        storage.current_amount -= deduct;
        remaining_to_deduct -= deduct;
        if remaining_to_deduct <= 0.0 {
            break;
        }
    }

    let world_spawn_pos = grid_to_world(grid_pos);

    let mut entity_cmd = if let Some(ref assets) = building_assets {
        let image = match *selected_building {
            SelectedBuilding::Collector => assets.collector.clone(),
            SelectedBuilding::Storage   => assets.storage.clone(),
            SelectedBuilding::Turret    => assets.turret.clone(),
            _ => Handle::default(),
        };
        commands.spawn((
            Sprite {
                image,
                custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.9)),
                ..default()
            },
            Transform::from_translation(world_spawn_pos.extend(1.0)),
            GridPosition(grid_pos),
            BuildingMarker,
        ))
    } else {
        commands.spawn((
            Sprite {
                color: match *selected_building {
                    SelectedBuilding::Collector => Color::srgb(0.8, 0.4, 0.2),
                    SelectedBuilding::Storage   => Color::srgb(0.2, 0.4, 0.8),
                    SelectedBuilding::Turret    => Color::srgb(0.5, 0.5, 0.9),
                    _ => Color::WHITE,
                },
                custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.9)),
                ..default()
            },
            Transform::from_translation(world_spawn_pos.extend(1.0)),
            GridPosition(grid_pos),
            BuildingMarker,
        ))
    };

    match *selected_building {
        SelectedBuilding::Collector => {
            entity_cmd.insert(Collector::default());
            entity_cmd.insert(Connection { targets: Vec::new() });
        }
        SelectedBuilding::Storage => {
            entity_cmd.insert(Storage { current_amount: 0.0, max_capacity: 100.0 });
        }
        SelectedBuilding::Turret => {
            entity_cmd.insert(Turret::default());
        }
        _ => {}
    }

    drag_state.last_placed_grid = Some(grid_pos);
}


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
