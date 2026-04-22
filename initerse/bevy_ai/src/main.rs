#![allow(dead_code, unused)]
pub use bevy::{prelude::*, window::WindowResolution};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub mod animation;
pub mod building;
pub mod camera;
pub mod combat;
pub mod config_screen;
pub mod connection;
pub mod grid;
pub mod interaction;
pub mod menu;
pub mod pause_menu;
pub mod save_load;
pub mod ui;
pub mod world_config;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    MainMenu,
    ConfigScreen,
    InGame,
    Paused,
}

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
                resolution: WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .init_resource::<GlobalInventory>()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::default())
        .add_plugins((
            camera::CameraPlugin,
            grid::GridPlugin,
            building::BuildingPlugin,
            // combat::CombatPlugin,
            ui::UiPlugin,
            interaction::InteractionPlugin,
            connection::ConnectionPlugin,
            animation::AnimationPlugin,
            world_config::WorldConfigPlugin,
            save_load::SaveLoadPlugin,
            config_screen::ConfigScreenPlugin,
            menu::MenuPlugin,
            pause_menu::PauseMenuPlugin,
        ))
        // Load images as early as possible
        .add_systems(PreStartup, building::load_building_assets)
        // Storage color tint (image tint based on fill)
        .add_systems(OnEnter(AppState::InGame), setup_base)
        .add_systems(Update, (
            update_global_stats_system,
            save_load::flush_pending_connections,
        ).run_if(in_state(AppState::InGame)))
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
    if building_assets.base == Handle::default() { dbg!("assets not loaded"); return; }

    commands.spawn((
        Sprite {
            image: building_assets.base.clone(),
            color: Color::linear_rgba(1.2, 1.2, 1.2, 1.0),
            custom_size: Some(Vec2::new(grid::TILE_SIZE * 3.0, grid::TILE_SIZE * 3.0)),
            ..default()
        },
        Transform::from_xyz(grid::TILE_SIZE / 2.0, grid::TILE_SIZE / 2.0, 1.0),
        GridPosition(IVec2::new(0, 0)),
        BuildingMarker,
        Base,
        Storage { current_amount: 100.0, max_capacity: 1000.0 }, // Starting with 10 gold
        crate::animation::JuiceScale::default(),
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
