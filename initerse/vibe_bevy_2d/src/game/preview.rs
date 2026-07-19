use crate::*;

#[derive(Resource)]
pub struct PreviewBuilding {
    building: Option<Building>,
    entity: Entity,
}

pub fn escape_preview(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut preview_building: ResMut<PreviewBuilding>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) || mouse_input.just_pressed(MouseButton::Right) {
        preview_building.building = None;
        commands.entity(preview_building.entity).insert(Visibility::Hidden);
    }
}

pub fn get_mouse_world_pos(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (camera, camera_transform) = *camera_query;
    camera.viewport_to_world_2d(camera_transform, window.cursor_position()?).ok()
}



pub fn hover_preview_building(
    hotbar: Res<Hotbar>,
    mut preview_building: ResMut<PreviewBuilding>,
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) {
    // dbg!(&hotbar);
    if let Some(selected_slot) = hotbar.selected_slots {
        if let Some(ref building) = hotbar.slots[selected_slot as usize].item {
            if let Some(mouse_pos) = get_mouse_world_pos(window, camera_query) {
                // round the mouse position to the nearest tile
                let preview_building_position = Vec2::new(
                    (mouse_pos.x / TILE_SIZE as f32).round() * TILE_SIZE as f32,
                    (mouse_pos.y / TILE_SIZE as f32).round() * TILE_SIZE as f32,
                );
                preview_building.building = Some(building.clone());
                commands.entity(preview_building.entity).entry::<Transform>().and_modify(move |mut transform| {
                    transform.translation = preview_building_position.extend(10.0);
                });
                commands.entity(preview_building.entity).insert(Visibility::Visible);
            }
        }
    }
}




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
    mut ghost_q: Query<(Entity, &mut Transform, &mut Sprite, &mut Visibility), With<PreviewBuilding>>,
    rendered_grid: Res<RenderedGrid>,
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
            PreviewBuilding,
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
            can_place=true;

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
