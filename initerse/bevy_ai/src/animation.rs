use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<JuiceScale>()
           .add_systems(Update, juice_scale_system);
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct JuiceScale {
    pub stiffness: f32,
    pub damping: f32,
    pub target: Vec2,
    pub velocity: Vec2,
    pub current: Vec2,
}

impl Default for JuiceScale {
    fn default() -> Self {
        Self {
            stiffness: 200.0,
            damping: 15.0,
            target: Vec2::ONE,
            velocity: Vec2::ZERO,
            current: Vec2::ONE,
        }
    }
}

impl JuiceScale {
    pub fn new(target: Vec2, stiffness: f32, damping: f32) -> Self {
        Self {
            stiffness,
            damping,
            target,
            velocity: Vec2::ZERO,
            current: target,
        }
    }

    pub fn punch(&mut self, amount: Vec2) {
        self.velocity += amount;
    }
}

fn juice_scale_system(
    time: Res<Time>,
    mut query: Query<(&mut JuiceScale, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (mut juice, mut transform) in &mut query {
        let (stiffness, damping, target, current, velocity) = (juice.stiffness, juice.damping, juice.target, juice.current, juice.velocity);
        
        // Simple spring physics
        let displacement = target - current;
        let spring_force = displacement * stiffness;
        let damping_force = velocity * damping;
        let acceleration = spring_force - damping_force;
        
        let new_velocity = velocity + acceleration * dt;
        let new_current = current + new_velocity * dt;
        
        juice.velocity = new_velocity;
        juice.current = new_current;
        
        transform.scale = new_current.extend(1.0);
    }
}
