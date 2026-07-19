use bevy::prelude::*;
use su_core::resources::ResourceType;

#[derive(Component, Debug)]
pub struct Connection {
    pub from: Entity,
    pub to: Entity,
    pub resource_filter: Option<ResourceType>, // None = any
    pub throughput: f32, // items/sec
    pub curve: BezierCurve,
    pub particles: Vec<FlowParticle>,
}

#[derive(Debug, Clone)]
pub struct BezierCurve {
    pub p0: Vec2,
    pub p1: Vec2, // control
    pub p2: Vec2, // control
    pub p3: Vec2,
}

impl BezierCurve {
    pub fn straight(a: Vec2, b: Vec2) -> Self {
        let mid = (a+b)/2.0;
        let offset = Vec2::new(0.0, 30.0); // slight curve for hand-drawn
        Self { p0: a, p1: mid+offset, p2: mid-offset, p3: b }
    }
    
    pub fn at(&self, t: f32) -> Vec2 {
        // Cubic bezier
        let u = 1.0 - t;
        self.p0 * u*u*u + self.p1 * 3.0*u*u*t + self.p2 * 3.0*u*t*t + self.p3 * t*t*t
    }
}

#[derive(Debug, Clone)]
pub struct FlowParticle {
    pub t: f32, // 0..1 along curve
    pub resource: ResourceType,
}

pub struct ConnectionsPlugin;

impl Plugin for ConnectionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, tick_connections)
            .add_systems(Update, draw_connections)
            ;
    }
}

fn tick_connections(
    time: Res<Time>,
    mut query: Query<&mut Connection>,
) {
    for mut conn in query.iter_mut() {
        let throughput = conn.throughput;
        for p in conn.particles.iter_mut() {
            p.t += time.delta_secs() * 0.5 * throughput;
            if p.t > 1.0 { p.t = 0.0; } // loop for now, later deliver
        }
    }
}

fn draw_connections(
    mut gizmos: Gizmos,
    query: Query<&Connection>,
) {
    for conn in query.iter() {
        // Draw arrow with hand-drawn style
        let steps = 20;
        for i in 0..steps {
            let t0 = i as f32 / steps as f32;
            let t1 = (i+1) as f32 / steps as f32;
            let p0 = conn.curve.at(t0);
            let p1 = conn.curve.at(t1);
            // Sketch style based on resource
            let color = if let Some(rt) = conn.resource_filter {
                rt.color()
            } else {
                Color::BLACK
            };
            gizmos.line_2d(p0, p1, color);
        }
        // Draw particles
        for particle in &conn.particles {
            let pos = conn.curve.at(particle.t);
            gizmos.circle_2d(pos, 5.0, particle.resource.color());
        }
        // Arrow head
        dbg!(&conn);
        let end = conn.curve.at(1.0);
        dbg!(&end);
        let dir = (conn.curve.at(0.99) - end).normalize_or_zero();
        dbg!(&dir);
        let left = end + dir * 15.0 + Vec2::new(-dir.y, dir.x) * 8.0;
        dbg!(&left);
        let right = end + dir * 15.0 + Vec2::new(dir.y, -dir.x) * 8.0;
        dbg!(&right);
        gizmos.line_2d(end, left, Color::BLACK);
        gizmos.line_2d(end, right, Color::BLACK);
    }
}
