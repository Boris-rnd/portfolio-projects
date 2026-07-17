use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

mod building;
mod camera;
mod combat;
mod connection;
mod grid;
mod interaction;
mod ui;

#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct GlobalInventory {
    pub total_gold: f32,
    pub total_collection_rate: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Initerse Factory Defense".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<GlobalInventory>()
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_plugins((
            camera::CameraPlugin,
            grid::GridPlugin,
            building::BuildingPlugin,
            combat::CombatPlugin,
            ui::UiPlugin,
            interaction::InteractionPlugin,
            connection::ConnectionPlugin,
        ))
        // Load images as early as possible
        .add_systems(PreStartup, building::load_building_assets)
        // Storage color tint (image tint based on fill)
        .add_systems(Update, (storage_tint_system, update_global_stats_system, setup_base))
        .run();
}

use building::{Storage, Base, BuildingMarker, GridPosition, Collector, BuildingAssets};

fn setup_base(
    mut commands: Commands,
    mut already_spawned: Local<bool>,
    building_assets: Res<BuildingAssets>,
) {
    if *already_spawned { return; }
    
    // Check if assets are loaded (they should be since it's Update, but let's be safe)
    if building_assets.base == Handle::default() { return; }

    commands.spawn((
        Sprite {
            image: building_assets.base.clone(),
            custom_size: Some(Vec2::new(grid::TILE_SIZE * 1.5, grid::TILE_SIZE * 1.5)),
            ..default()
        },
        Transform::from_xyz(grid::TILE_SIZE / 2.0, grid::TILE_SIZE / 2.0, 1.0),
        GridPosition(IVec2::new(0, 0)),
        BuildingMarker,
        Base,
        Storage { current_amount: 100.0, max_capacity: 1000.0 }, // Starting with 10 gold
    ));

    *already_spawned = true;
}

fn update_global_stats_system(
    mut inventory: ResMut<GlobalInventory>,
    storages: Query<&Storage>,
    collectors: Query<&Collector>,
) {
    let mut total_gold = 0.0;
    for storage in &storages {
        total_gold += storage.current_amount;
    }
    inventory.total_gold = total_gold;

    let mut total_rate = 0.0;
    for collector in &collectors {
        total_rate += collector.items_per_second;
    }
    inventory.total_collection_rate = total_rate;
}

fn storage_tint_system(
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