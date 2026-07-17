use bevy::prelude::*;
use crate::connection::{Connection, ItemMovement};

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridPosition>()
            .register_type::<Collector>()
            .register_type::<Storage>()
            .register_type::<Base>()
            .register_type::<Turret>()
            .register_type::<GoldNode>()
            .register_type::<BuildingMarker>()
            .add_systems(Update, (collector_tick_system, item_movement_system, spawn_gold_nodes));
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct GridPosition(pub IVec2);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Collector {
    pub timer: Timer,
    pub items_per_second: f32,
    pub target_index: usize,
}

impl Default for Collector {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            items_per_second: 10.0,
            target_index: 0,
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Storage {
    pub current_amount: f32,
    pub max_capacity: f32,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Base;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Turret {
    pub timer: Timer,
    pub range: f32,
}

impl Default for Turret {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            range: 300.0,
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct GoldNode;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct BuildingMarker;

/// Resource that holds loaded asset handles
#[derive(Resource, Default)]
pub struct BuildingAssets {
    pub collector: Handle<Image>,
    pub storage: Handle<Image>,
    pub base: Handle<Image>,
    pub turret: Handle<Image>,
    pub enemy: Handle<Image>,
    pub gold: Handle<Image>,
    pub gold_node: Handle<Image>,
}

pub fn load_building_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(BuildingAssets {
        collector: asset_server.load("collector.png"),
        storage: asset_server.load("storage.png"),
        base: asset_server.load("collector.png"),
        turret: asset_server.load("repeater.png"),
        enemy: asset_server.load("datawing.png"),
        gold: asset_server.load("gold.png"),
        gold_node: asset_server.load("gold_node.png"),
    });
}

fn spawn_gold_nodes(
    mut commands: Commands,
    mut already_spawned: Local<bool>,
    building_assets: Res<BuildingAssets>,
) {
    if *already_spawned { return; }
    if building_assets.gold_node == Handle::default() { return; }

    use noiz::prelude::*;
    use noiz::prelude::common_noise::*;
    let mut noise = Noise::<Fbm<Simplex>>::default();
    noise.set_seed(42);
    noise.set_frequency(0.1);

    for x in -crate::grid::GRID_SIZE..crate::grid::GRID_SIZE {
        for y in -crate::grid::GRID_SIZE..crate::grid::GRID_SIZE {
            // Check distance from center to avoid nodes on top of base
            let dist = ((x*x + y*y) as f32).sqrt();
            if dist < 5.0 { continue; }

            let n: f32 = noise.sample(Vec2::new(x as f32, y as f32));
            if n > 0.4 {
                let world_pos = crate::grid::grid_to_world(IVec2::new(x, y));
                commands.spawn((
                    Sprite {
                        image: building_assets.gold_node.clone(),
                        custom_size: Some(Vec2::new(crate::grid::TILE_SIZE * 0.8, crate::grid::TILE_SIZE * 0.8)),
                        ..default()
                    },
                    Transform::from_translation(world_pos.extend(-5.0)),
                    GridPosition(IVec2::new(x, y)),
                    GoldNode,
                ));
            }
        }
    }

    *already_spawned = true;
}


fn collector_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    mut collectors: Query<(Entity, &mut Collector, &GlobalTransform, &Connection)>,
    storages: Query<(&GlobalTransform, &Storage)>,
    building_assets: Res<BuildingAssets>,
) {
    for (_entity, mut collector, transform, connection) in &mut collectors {
        collector.timer.tick(time.delta());
        if collector.timer.just_finished() {
            if connection.targets.is_empty() {
                continue;
            }

            // Round-robin distribution
            let num_targets = connection.targets.len();
            
            for i in 0..num_targets {
                let current_idx = (collector.target_index + i) % num_targets;
                let target_entity = connection.targets[current_idx];

                if let Ok((_target_transform, storage)) = storages.get(target_entity) {
                    // Check if storage has space
                    if storage.current_amount < storage.max_capacity {
                        // Spawn gold item
                        commands.spawn((
                            Sprite {
                                image: building_assets.gold.clone(),
                                custom_size: Some(Vec2::new(20.0, 20.0)),
                                ..default()
                            },
                            Transform::from_translation(transform.translation().truncate().extend(2.0)),
                            ItemMovement {
                                target_entity,
                                amount: 1.0,
                                speed: 150.0,
                            },
                        ));
                        
                        // Increment round-robin index for next tick
                        collector.target_index = (current_idx + 1) % num_targets;
                        break; // Sent one item, stop checking targets for this tick
                    }
                }
            }
        }
    }
}

fn item_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut items: Query<(Entity, &mut Transform, &ItemMovement)>,
    mut storages: Query<(&GlobalTransform, &mut Storage)>,
) {
    for (item_entity, mut transform, movement) in &mut items {
        if let Ok((target_transform, mut storage)) = storages.get_mut(movement.target_entity) {
            let target_pos = target_transform.translation().truncate();
            let current_pos = transform.translation.truncate();
            let direction = target_pos - current_pos;
            let distance = direction.length();

            let move_dist = movement.speed * time.delta_secs();

            if distance <= move_dist {
                // Arrived
                // Even though we checked on spawn, check again for overflow
                storage.current_amount = (storage.current_amount + movement.amount).min(storage.max_capacity);
                commands.entity(item_entity).despawn();
            } else {
                let velocity = direction.normalize() * move_dist;
                transform.translation += velocity.extend(0.0);
            }
        } else {
            // Target destroyed or invalid
            commands.entity(item_entity).despawn();
        }
    }
}

