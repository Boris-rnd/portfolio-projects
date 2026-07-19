#![allow(unused, dead_code)]
use bevy::platform::collections::HashMap;
pub use bevy::{input::keyboard::KeyboardInput, prelude::*, window::WindowResolution};
pub use bevy::color::palettes::css::*;

pub use crate::render::world::RenderedWorld;
pub use crate::game::*;
pub use crate::render::*;
pub use crate::render::camera::{OVERLAY_LAYER, GameCamera, OverlayCamera};


pub mod render;
pub mod game;

pub const TILE_SIZE: u32 = 64; // 64x64 tiles


fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_message::<render::hotbar::SelectSlotMsg>()
        .insert_resource(RenderedWorld::new(HashMap::new(), 0))
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Initerse - Top down automation game".to_string(),
                resolution: WindowResolution::new(800, 600),
                ..default()
            }),
            ..default()
        }).set(AssetPlugin {
            file_path: "assets".into(),
            ..default()
        }),
        // --- External ---
        // bevy_prototype_lyon::prelude::ShapePlugin,
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        bevy_vector_shapes::prelude::Shape2dPlugin::new(bevy_vector_shapes::painter::ShapeConfig::default_2d()),
        // ---- Game ------
        ))
        .add_systems(Startup, (setup, render::camera::spawn_camera, render::hotbar::spawn_hotbar))
        .add_systems(Update, (update, camera::camera_movement, camera::camera_pan, render::hotbar::hover_preview_building, render::hotbar::interaction_hotbar, render::hotbar::escape_preview))
        // .add_systems(Update, (buildings::draw_dynamic_shapes_in_ui))

    .run();
}

pub fn setup(mut commands: Commands) {
}

pub fn update() {

}
