#![allow(unused, dead_code)]
use bevy::prelude::*;

pub mod camera;
pub mod grid;
pub mod placement;
pub mod zoom_layers;

pub struct SuGridPlugin;

impl Plugin for SuGridPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<camera::CanvasCameraState>()
            .init_resource::<grid::GridConfig>()
            .init_resource::<placement::PlacementState>()
            .add_plugins(camera::CanvasCameraPlugin)
            .add_plugins(grid::GridPlugin)
            .add_plugins(placement::PlacementPlugin)
            .add_plugins(zoom_layers::ZoomLayersPlugin);
    }
}
