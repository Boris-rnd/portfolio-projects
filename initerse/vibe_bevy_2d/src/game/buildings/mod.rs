use crate::*;

pub mod collector;
pub use collector::*;

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Collector>()
            .register_type::<Storage>()
            .register_type::<Base>()
            .register_type::<Turret>()
            .register_type::<GoldNode>()
            .register_type::<BuildingMarker>()
            // Load images as early as possible
            // .add_systems(PreStartup, load_building_assets)

            .add_systems(Update, (collector_tick_system, item_movement_system, spawn_gold_nodes, storage_tint_system));
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Building {
    FoamExtractor,
    GluonGenerator
}
impl Building {
    pub fn spawn(&self) -> impl Scene {
        match self {
            Building::FoamExtractor => {
                    bsn! {
                        Sprite {
                            color: Color::linear_rgb(0.5, 1.0, 0.2),
                            custom_size: Vec2::new(crate::grid::TILE_SIZE * 0.8, crate::grid::TILE_SIZE * 0.8),
                        }
                    }
            },
            Building::GluonGenerator => todo!(),
        }
    }
}


#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct BuildingMarker;

/// Resource that holds loaded asset handles
// #[derive(Resource)]
// pub struct BuildingAssets {
//     pub collector: Box<dyn Scene>,
//     pub storage: Box<dyn Scene>,
//     pub base: Box<dyn Scene>,
//     pub turret: Box<dyn Scene>,
//     pub enemy: Box<dyn Scene>,
//     pub gold: Box<dyn Scene>,
//     pub gold_node: Box<dyn Scene>,
// }

// pub fn load_building_assets(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
// ) {
//     use bevy_prototype_lyon::prelude::*;
//     let shape = shapes::RegularPolygon {
//         sides: 6,
//         feature: shapes::RegularPolygonFeature::Radius(200.0),
//         ..shapes::RegularPolygon::default()
//     };
//     commands.spawn(
//         ShapeBuilder::with(&shape)
//             .fill(DARK_CYAN)
//             .stroke((BLACK, 10.0))
//             .build(),
//     );
//     // commands.insert_resource(BuildingAssets {
//     //     collector: asset_server.load("collector.png"),
//     //     storage: asset_server.load("storage.png"),
//     //     base: asset_server.load("collector.png"),
//     //     turret: asset_server.load("repeater.png"),
//     //     enemy: asset_server.load("datawing.png"),
//     //     gold: asset_server.load("gold.png"),
//     //     gold_node: asset_server.load("gold_node.png"),
//     // });
// }

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



pub fn storage_tint_system(
    mut storages: bevy::prelude::Query<(&Storage, &mut bevy::prelude::Sprite)>,
) {
    for (storage, mut sprite) in &mut storages {
        let fill = (storage.current_amount / storage.max_capacity).clamp(0.0, 1.0);
        // Tint the image from neutral (white) toward a cyan glow as it fills
        let brightness = 1.0 + fill * 0.4;
        sprite.color = bevy::prelude::Color::srgb(
            (1.0_f32).min(brightness * (1.0 - fill * 0.3)),
            (1.0_f32).min(brightness),
            (1.0_f32).min(brightness),
        );
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
