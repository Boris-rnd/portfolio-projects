pub use bevy::prelude::*;
pub use bevy::color::palettes::css::*;
pub use bevy::window::PrimaryWindow;

use smallvec::SmallVec;

pub mod render;
pub mod game;

pub use render::*;
pub use game::*;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Initerse - Top down automation".into(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            }),
            // ----- External plugins --
            bevy_prototype_lyon::prelude::ShapePlugin,
            bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
            bevy_inspector_egui::quick::WorldInspectorPlugin::default(),

            // ----- Game plugins ------
            game::GamePlugin,
            render::RenderPlugin,
        ))
        .run();
}
