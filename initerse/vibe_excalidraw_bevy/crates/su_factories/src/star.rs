use bevy::prelude::*;
use crate::{Factory, tick_factory};
use su_core::buffers::Buffer;

pub struct StarPlugin;

impl Plugin for StarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (tick_factory, star_gravity, star_expansion))
            .add_systems(Update, draw_star);
    }
}

#[derive(Component, Debug)]
pub struct ArtificialStar {
    pub mass: f32, // hydrogen count = mass
    pub temperature: f32,
    pub age: f32,
}

// Gravity: attract nearby Hydrogen buffers towards star
fn star_gravity(
    time: Res<Time>,
    stars: Query<(&ArtificialStar, &Transform)>,
    mut hydrogen: Query<(&mut Transform, &Buffer), Without<ArtificialStar>>,
) {
    for (star, star_transform) in stars.iter() {
        let star_pos = star_transform.translation.truncate();
        let gravity_strength = star.mass * 0.001;
        
        for (mut h_transform, buffer) in hydrogen.iter_mut() {
            // Only attract if buffer has hydrogen
            if buffer.amount(su_core::resources::ResourceType::Hydrogen) > 0.0 {
                let diff = star_pos - h_transform.translation.truncate();
                let dist = diff.length().max(10.0);
                let force = gravity_strength / (dist * dist) * time.delta_secs();
                h_transform.translation += (diff.normalize_or_zero() * force).extend(0.0);
            }
        }
    }
}

fn star_expansion(
    time: Res<Time>,
    mut stars: Query<(&mut ArtificialStar, &mut Transform, &Buffer)>,
) {
    for (mut star, mut transform, buffer) in stars.iter_mut() {
        // Mass grows with hydrogen stored
        let h_amount = buffer.amount(su_core::resources::ResourceType::Hydrogen);
        star.mass = h_amount;
        // Scale visual with mass: starts 5x5, grows log
        let base = 5.0;
        let scale = base + (star.mass / 100.0).log10().max(0.0) * 0.5;
        transform.scale = Vec3::splat(scale);
        // Temperature rises with mass (fusion)
        star.temperature = 1e6 + star.mass * 1000.0;
        star.age += time.delta_secs();
    }
}

fn draw_star(
    mut gizmos: Gizmos,
    stars: Query<(&ArtificialStar, &Transform)>,
) {
    for (star, transform) in stars.iter() {
        let pos = transform.translation.truncate();
        // Hand-drawn star: wobbly circle + glow if hot
        let radius = 20.0 * transform.scale.x;
        gizmos.circle_2d(pos, radius, Color::srgb(1.0, 0.8, 0.2));
        if star.temperature > 2e6 {
            gizmos.circle_2d(pos, radius * 1.2, Color::srgba(1.0, 0.9, 0.5, 0.3));
        }
    }
}
