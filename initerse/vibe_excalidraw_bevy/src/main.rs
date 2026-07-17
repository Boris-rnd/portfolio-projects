use bevy::prelude::*;
use su_core::SuCorePlugin;
use su_grid::SuGridPlugin;
use su_render_excali::SuRenderExcaliPlugin;
use su_logistics::SuLogisticsPlugin;
use su_factories::SuFactoriesPlugin;
use su_ui::SuUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sketched Universe - Excalidraw Factorio".to_string(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        // Core pure logic first
        .add_plugins(SuCorePlugin)
        // Hand-drawn rendering
        .add_plugins(SuRenderExcaliPlugin)
        // Grid + infinite canvas camera
        .add_plugins(SuGridPlugin)
        // Logistics: hybrid arrows + drone system
        .add_plugins(SuLogisticsPlugin)
        // Factories from diagram
        .add_plugins(SuFactoriesPlugin)
        // UI: palette at bottom, properties on right
        .add_plugins(SuUiPlugin)
        .add_systems(Startup, spawn_initial_dot_grid)
        .run();
}

fn spawn_initial_dot_grid(mut commands: Commands) {
    // Placeholder - TASK-002 will replace with shader dots
    commands.spawn(Camera2d);
    info!("Sketched Universe booted - infinite canvas ready");
}
