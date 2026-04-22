use bevy::prelude::*;
use bevy::state::commands;
use crate::connection::{Connection, ItemMovement};
use crate::animation::JuiceScale;
use crate::world_config::WorldConfig;
use crate::save_load::NeedGoldNodeRespawn;
use crate::{AppState, GlobalInventory};
use rand::Rng;


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
            .add_systems(OnEnter(AppState::InGame), (
                spawn_gold_nodes
            ))
            .add_systems(Update, (
                // storage_tint_system,
                collector_tick_system,
                gluon_generator_system,
                gluon_movement_system,
                gluon_decay_system,
                quark_despawn_system,
                basin_system,
                proton_creator_system,
                // item_movement_system,
                // respawn_gold_nodes_on_load,
            ).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Resource, PartialEq, Eq, Clone, Copy, Debug)]
pub enum BuildingType {
    Collector,
    GluonGenerator,
    Basin,
    ProtonCreator,
    Storage,
    Base,
    Turret,
}
impl BuildingType {
    pub fn to_image(&self, building_assets: &BuildingAssets) -> Handle<Image> {
        match self {
            BuildingType::Collector => building_assets.collector.clone(),
            BuildingType::GluonGenerator => building_assets.gluon_generator.clone(),
            BuildingType::Basin => building_assets.basin.clone(),
            BuildingType::ProtonCreator => building_assets.proton_creator.clone(),
            BuildingType::Storage => building_assets.storage.clone(),
            BuildingType::Base => building_assets.base.clone(),
            BuildingType::Turret => building_assets.turret.clone(),
        }
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
            items_per_second: 1.0,
            target_index: 0,
        }
    }
}
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct GluonGenerator {
    pub timer: Timer,
    pub items_per_second: f32,
}

impl Default for GluonGenerator {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            items_per_second: 1.0,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Basin {
    pub timer: Timer,
    pub processed_per_second: f32,
    pub charge: i8, // 1 for positive, -1 for negative
}

impl Default for Basin {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            processed_per_second: 1.0,
            charge: 1,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ProtonCreator {
    pub timer: Timer,
    pub processed_per_second: f32,
}

impl Default for ProtonCreator {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            processed_per_second: 1.0,
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

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Quark {
    pub charge: i8,
    pub despawn_timer: Timer,
}

/// Resource that holds loaded asset handles
#[derive(Resource, Default)]
pub struct BuildingAssets {
    pub collector: Handle<Image>,
    pub gluon_generator: Handle<Image>,
    pub basin: Handle<Image>,
    pub proton_creator: Handle<Image>,
    pub storage: Handle<Image>,
    pub base: Handle<Image>,
    pub turret: Handle<Image>,
    pub enemy: Handle<Image>,
    pub gold: Handle<Image>,
    pub gold_node: Handle<Image>,
    pub none: Handle<Image>,
    pub destroy: Handle<Image>,
}

pub fn load_building_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(BuildingAssets {
        collector: asset_server.load("collector.png"),
        gluon_generator: asset_server.load("gluon_generator.png"),
        basin: asset_server.load("storage.png"),
        proton_creator: asset_server.load("gold_node.png"),
        storage: asset_server.load("storage.png"),
        base: asset_server.load("collector.png"),
        turret: asset_server.load("repeater.png"),
        enemy: asset_server.load("datawing.png"),
        gold: asset_server.load("gold.png"),
        gold_node: asset_server.load("gold_node.png"),
        destroy: asset_server.load("destroy.png"),
        none: asset_server.load("none.png"),
    });
}

fn do_spawn_gold_nodes(
    commands: &mut Commands,
    building_assets: &BuildingAssets,
    config: &WorldConfig,
) {
    use noise::{NoiseFn, Fbm, Simplex};
    let noise = Fbm::<Simplex>::new(config.seed as u32);

    for x in -crate::grid::GRID_SIZE..crate::grid::GRID_SIZE {
        for y in -crate::grid::GRID_SIZE..crate::grid::GRID_SIZE {
            let dist = ((x*x + y*y) as f32).sqrt();
            if dist < 5.0 { continue; }

            let n = noise.get([x as f64 * 0.15, y as f64 * 0.15]) as f32;
            if n > config.gold_node_frequency {
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
}

fn spawn_gold_nodes(
    mut commands: Commands,
    building_assets: Res<BuildingAssets>,
    config: Res<WorldConfig>,
) {
    do_spawn_gold_nodes(&mut commands, &building_assets, &config);
}

fn respawn_gold_nodes_on_load(
    mut commands: Commands,
    need_respawn: Option<Res<NeedGoldNodeRespawn>>,
    building_assets: Res<BuildingAssets>,
    config: Res<WorldConfig>,
    existing_nodes: Query<Entity, With<GoldNode>>,
) {
    let Some(flag) = need_respawn else { return };
    if !flag.0 { return; }

    // Despawn old nodes
    for e in &existing_nodes { commands.entity(e).despawn(); }
    do_spawn_gold_nodes(&mut commands, &building_assets, &config);
    commands.remove_resource::<NeedGoldNodeRespawn>();
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
                                color: Color::linear_rgba(1.5, 1.5, 0.5, 1.0),
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
    mut storages: Query<(&GlobalTransform, &mut Storage, Option<&mut JuiceScale>)>,
) {
    for (item_entity, mut transform, movement) in &mut items {
        if let Ok((target_transform, mut storage, juice)) = storages.get_mut(movement.target_entity) {
            let target_pos = target_transform.translation().truncate();
            let current_pos = transform.translation.truncate();
            let direction = target_pos - current_pos;
            let distance = direction.length();

            let move_dist = movement.speed * time.delta_secs();

            if distance <= move_dist {
                // Arrived
                storage.current_amount = (storage.current_amount + movement.amount).min(storage.max_capacity);
                if let Some(mut j) = juice {
                    j.punch(Vec2::splat(0.2)); // Brief scale up
                }
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



fn storage_tint_system(
    mut storages: bevy::prelude::Query<(&Storage, &mut bevy::prelude::Sprite)>,
) {
    for (storage, mut sprite) in &mut storages {
        let fill = (storage.current_amount / storage.max_capacity).clamp(0.0, 1.0);
        // Emissive cyan glow as it fills
        sprite.color = Color::linear_rgba(1.0 - fill * 0.5, 1.0 + fill * 0.5, 1.0 + fill * 1.5, 1.0);
    }
}

#[derive(Component)]
pub struct Gluon {
    pub decay_timer: Timer,
    pub color: Color,
}

fn gluon_generator_system(
    mut commands: Commands,
    mut generators: Query<(&mut GluonGenerator, &GlobalTransform)>, 
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (mut generator, transform) in &mut generators {
        generator.timer.tick(time.delta());
        if generator.timer.just_finished() {
            let color = Color::hsl(rand::rng().random_range(0.0..360.0), 0.8, 0.6);
            spawn_gluon(&mut commands, &mut meshes, &mut materials, transform.translation(), color);
        }
    }
}

fn gluon_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();
    }
}

fn gluon_decay_system(
    mut commands: Commands,
    time: Res<Time>,
    mut gluons: Query<(Entity, &mut Gluon, &Transform)>,
) {
    for (entity, mut gluon, transform) in &mut gluons {
        gluon.decay_timer.tick(time.delta());
        if gluon.decay_timer.just_finished() {
            // Transform into a Quark
            let charge = if rand::rng().random_bool(0.5) { 1 } else { -1 };
            let charge_color = if charge == 1 { Color::srgb(1.5, 0.5, 0.5) } else { Color::srgb(0.5, 0.5, 1.5) };

            commands.entity(entity)
                .remove::<Gluon>()
                .remove::<Mesh2d>()
                .remove::<MeshMaterial2d<ColorMaterial>>()
                .insert(Quark {
                    charge,
                    despawn_timer: Timer::from_seconds(5.0, TimerMode::Once),
                })
                .insert(Sprite {
                    color: charge_color,
                    custom_size: Some(Vec2::splat(12.0)), // Square shape
                    ..default()
                });
        }
    }
}

fn quark_despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Quark)>,
) {
    for (entity, mut quark) in &mut query {
        quark.despawn_timer.tick(time.delta());
        if quark.despawn_timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_gluon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec3,
    color: Color,
) {
    let mut rng = rand::rng();
    let velocity = Vec2::new(
        rng.random_range(-10.0..10.0),
        rng.random_range(-10.0..10.0),
    );

    // Ensure all components for Mesh2d are present for rendering
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(8.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from(color))),
        Transform::from_translation(position.truncate().extend(5.0)), // Use higher Z for visibility
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Gluon {
            decay_timer: Timer::from_seconds(1.5, TimerMode::Once),
            color,
        },
        Velocity(velocity),
    ));
}

fn basin_system(
    mut commands: Commands,
    time: Res<Time>,
    basins: Query<(&GlobalTransform, &Basin)>,
    mut quarks: Query<(Entity, &GlobalTransform, &mut Transform, &Quark, &mut Velocity)>,
) {
    for (q_entity, q_global, mut q_transform, quark, mut velocity) in &mut quarks {
        // Find nearest basin with matching charge
        let mut nearest_dist = f32::MAX;
        let mut target_dir = Vec2::ZERO;

        for (b_transform, basin) in &basins {
            if basin.charge == quark.charge {
                let dist_sq = b_transform.translation().truncate().distance_squared(q_global.translation().truncate());
                if dist_sq < nearest_dist {
                    nearest_dist = dist_sq;
                    target_dir = (b_transform.translation().truncate() - q_global.translation().truncate()).normalize_or_zero();
                }
            }
        }

        if nearest_dist < f32::MAX {
            // Attraction force
            let force = target_dir * 100.0;
            velocity.0 = velocity.0.lerp(force, time.delta_secs() * 2.0);

            // Consumption check
            if nearest_dist < 400.0 { // distance_squared, so dist < 20
                commands.entity(q_entity).despawn();
            }
        }
    }
}
fn proton_creator_system() {}

    


