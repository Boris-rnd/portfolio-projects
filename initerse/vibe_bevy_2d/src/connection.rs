use bevy::prelude::*;

pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Connection>()
           .register_type::<ItemMovement>()
           .add_systems(Update, (
               connection_visuals_system,
               update_connection_lines,
           ));
    }
}

/// A component that explicitly links this building to a target building.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Connection {
    pub targets: Vec<Entity>,
}

/// A visual item spawned from a collector moving toward its target storage.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ItemMovement {
    pub target_entity: Entity,
    pub amount: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct ConnectionLine {
    pub source: Entity,
    pub target: Entity,
}

fn connection_visuals_system(
    mut commands: Commands,
    // Track which (source, target) pairs already have a ConnectionLine
    existing_lines: Query<&ConnectionLine>,
    // All connections currently in the world
    connections: Query<(Entity, &Connection)>,
    transforms: Query<&GlobalTransform>,
) {
    for (source_entity, connection) in &connections {
        for &target_entity in &connection.targets {
            // Check if a line already exists for this (source, target) pair
            if existing_lines.iter().any(|line| line.source == source_entity && line.target == target_entity) {
                continue;
            }

            let Ok(source_transform) = transforms.get(source_entity) else { continue };
            let Ok(target_transform) = transforms.get(target_entity) else { continue };

            let start = source_transform.translation().truncate();
            let end = target_transform.translation().truncate();
            let diff = end - start;
            let length = diff.length();
            let angle = diff.y.atan2(diff.x);

            commands.spawn((
                Sprite {
                    color: Color::srgb(0.7, 0.7, 0.7),
                    custom_size: Some(Vec2::new(length, 4.0)),
                    ..default()
                },
                Transform::from_translation(((start + end) / 2.0).extend(0.5))
                    .with_rotation(Quat::from_rotation_z(angle)),
                ConnectionLine {
                    source: source_entity,
                    target: target_entity,
                },
            ));
        }
    }
}


fn update_connection_lines(
    mut commands: Commands,
    mut lines: Query<(Entity, &mut Transform, &mut Sprite, &ConnectionLine)>,
    transforms: Query<&GlobalTransform>,
) {
    for (line_entity, mut transform, mut sprite, line) in &mut lines {
        if let (Ok(source_transform), Ok(target_transform)) = (
            transforms.get(line.source),
            transforms.get(line.target),
        ) {
            let start = source_transform.translation().truncate();
            let end = target_transform.translation().truncate();
            let diff = end - start;
            let length = diff.length();
            let angle = diff.y.atan2(diff.x);

            sprite.custom_size = Some(Vec2::new(length, 4.0));
            transform.translation = ((start + end) / 2.0).extend(0.5);
            transform.rotation = Quat::from_rotation_z(angle);
        } else {
            // One of the buildings is gone, despawn the line
            commands.entity(line_entity).despawn();
        }
    }
}
