use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Generates hand-drawn wobbly boxes like Excalidraw
/// Based on Rough.js double-stroke algorithm

#[derive(Component)]
pub struct RoughBox {
    pub width: f32,
    pub height: f32,
    pub seed: u64,
    pub roughness: f32, // 0.0 perfect, 1.0 very wobbly
    pub stroke_width: f32,
}

impl Default for RoughBox {
    fn default() -> Self {
        Self { width: 100.0, height: 100.0, seed: 0, roughness: 1.0, stroke_width: 2.0 }
    }
}

impl RoughBox {
    /// Generate mesh points with jitter
    pub fn generate_points(&self) -> Vec<Vec2> {
        let mut rng = StdRng::seed_from_u64(self.seed);
        let hw = self.width / 2.0;
        let hh = self.height / 2.0;
        
        let mut points = Vec::new();
        let segments_per_edge = 3;
        
        // Top edge: -hw,-hh -> hw,-hh
        for i in 0..=segments_per_edge {
            let t = i as f32 / segments_per_edge as f32;
            let mut x = -hw + t * self.width;
            let mut y = -hh;
            if i > 0 && i < segments_per_edge {
                x += rng.gen_range(-self.roughness*4.0..self.roughness*4.0);
                y += rng.gen_range(-self.roughness*3.0..self.roughness*3.0);
            }
            points.push(Vec2::new(x,y));
        }
        // Right edge
        for i in 1..=segments_per_edge {
            let t = i as f32 / segments_per_edge as f32;
            let mut x = hw;
            let mut y = -hh + t * self.height;
            if i > 0 && i < segments_per_edge {
                x += rng.gen_range(-self.roughness*3.0..self.roughness*3.0);
                y += rng.gen_range(-self.roughness*4.0..self.roughness*4.0);
            }
            points.push(Vec2::new(x,y));
        }
        // Bottom edge
        for i in 1..=segments_per_edge {
            let t = i as f32 / segments_per_edge as f32;
            let mut x = hw - t * self.width;
            let mut y = hh;
            if i > 0 && i < segments_per_edge {
                x += rng.gen_range(-self.roughness*4.0..self.roughness*4.0);
                y += rng.gen_range(-self.roughness*3.0..self.roughness*3.0);
            }
            points.push(Vec2::new(x,y));
        }
        // Left edge
        for i in 1..segments_per_edge {
            let t = i as f32 / segments_per_edge as f32;
            let mut x = -hw;
            let mut y = hh - t * self.height;
            if i > 0 && i < segments_per_edge {
                x += rng.gen_range(-self.roughness*3.0..self.roughness*3.0);
                y += rng.gen_range(-self.roughness*4.0..self.roughness*4.0);
            }
            points.push(Vec2::new(x,y));
        }
        
        points
    }
}

pub struct RoughRenderPlugin;

impl Plugin for RoughRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_rough_boxes);
    }
}

fn draw_rough_boxes(
    mut gizmos: Gizmos,
    query: Query<(&RoughBox, &Transform)>,
) {
    for (rbox, transform) in query.iter() {
        let points = rbox.generate_points();
        let pos = transform.translation.truncate();
        // Draw wobbly outline
        for window in points.windows(2) {
            let a = pos + window[0];
            let b = pos + window[1];
            gizmos.line_2d(a, b, Color::BLACK);
        }
        // Close
        if let (Some(first), Some(last)) = (points.first(), points.last()) {
            gizmos.line_2d(pos + *last, pos + *first, Color::BLACK);
        }
        // Second stroke slightly offset for Excalidraw double-line
        for window in points.windows(2) {
            let offset = Vec2::new(1.2, -0.8);
            let a = pos + window[0] + offset;
            let b = pos + window[1] + offset;
            gizmos.line_2d(a, b, Color::srgba(0.0, 0.0, 0.0, 0.6));
        }
    }
}

pub struct ExcaliText;

pub fn spawn_excali_label(commands: &mut Commands, pos: Vec2, text: &str) {
    commands.spawn((
        Text2d::new(text),
        Transform::from_translation(pos.extend(0.1)),
        // TODO: load Virgil font
    ));
}
