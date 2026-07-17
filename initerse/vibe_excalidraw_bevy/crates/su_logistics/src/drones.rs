use bevy::prelude::*;
use su_core::resources::ResourceType;

/// Vectorio-like drone system
/// Building that produces X broadcasts in area
/// Building that wants X requests, drones fetch

#[derive(Component, Debug)]
pub struct DronePort {
    pub range: f32,
    pub drone_count: usize,
    pub speed: f32,
}

#[derive(Component, Debug)]
pub struct Drone {
    pub carrying: Option<ResourceType>,
    pub from: Entity,
    pub to: Entity,
    pub progress: f32,
}

#[derive(Component, Debug)]
pub struct Provider {
    pub provides: ResourceType,
    pub amount: f32,
}

#[derive(Component, Debug)]
pub struct Requester {
    pub requests: ResourceType,
    pub amount_needed: f32,
    pub priority: f32,
}

pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (match_requests, tick_drones))
            .add_systems(Update, draw_drones);
    }
}

fn match_requests(
    // TODO: TASK-016 - Implement request/provide matching within range
) {
}

fn tick_drones(
    time: Res<Time>,
    mut drones: Query<&mut Drone>,
) {
    for mut drone in drones.iter_mut() {
        drone.progress += time.delta_secs() * 0.5;
    }
}

fn draw_drones(
    mut gizmos: Gizmos,
    drones: Query<(&Drone, &Transform)>,
) {
    for (drone, transform) in drones.iter() {
        let pos = transform.translation.truncate();
        // Hand-drawn little bird/drone - for now circle with rotor lines
        gizmos.circle_2d(pos, 8.0, Color::BLACK);
        if drone.carrying.is_some() {
            gizmos.circle_2d(pos + Vec2::new(0.0, 10.0), 3.0, drone.carrying.unwrap().color());
        }
    }
}
