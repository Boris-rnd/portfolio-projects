use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct WorldConfigPlugin;

impl Plugin for WorldConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldConfig>()
           .register_type::<WorldConfig>();
    }
}

/// Governs how the world is generated and how difficult combat is.
/// Set from the Config Screen before transitioning to InGame.
#[derive(Resource, Debug, Clone, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct WorldConfig {
    /// Seed for procedural gold-node generation.
    pub seed: u64,
    /// Seconds between enemy spawns (lower = harder).
    pub enemy_spawn_rate: f32,
    /// Starting health for each enemy.
    pub enemy_health: f32,
    /// Noise threshold above which a gold node spawns (higher = fewer nodes).
    pub gold_node_frequency: f32,
    /// HP for buildings (reserved for future use).
    pub building_health: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self::from_preset(Preset::Normal, 42)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Easy,
    Normal,
    Hard,
}

impl WorldConfig {
    pub fn from_preset(preset: Preset, seed: u64) -> Self {
        match preset {
            Preset::Easy => Self {
                seed,
                enemy_spawn_rate: 4.0,
                enemy_health: 6.0,
                gold_node_frequency: 0.25,
                building_health: 150.0,
            },
            Preset::Normal => Self {
                seed,
                enemy_spawn_rate: 2.0,
                enemy_health: 10.0,
                gold_node_frequency: 0.4,
                building_health: 100.0,
            },
            Preset::Hard => Self {
                seed,
                enemy_spawn_rate: 0.8,
                enemy_health: 20.0,
                gold_node_frequency: 0.55,
                building_health: 75.0,
            },
        }
    }
}
