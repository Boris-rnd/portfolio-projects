#![allow(unused, dead_code)]
use bevy::prelude::*;

pub mod connections;
pub mod drones;

pub struct SuLogisticsPlugin;

impl Plugin for SuLogisticsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(connections::ConnectionsPlugin)
            .add_plugins(drones::DronePlugin);
    }
}
