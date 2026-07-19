#![allow(unused, dead_code)]
use bevy::prelude::*;

pub mod gluon;
pub mod proton;
pub mod hydrogen;
pub mod star;
pub mod photon;

pub struct SuFactoriesPlugin;

impl Plugin for SuFactoriesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(gluon::GluonPlugin)
            .add_plugins(proton::ProtonPlugin)
            .add_plugins(hydrogen::HydrogenPlugin)
            .add_plugins(star::StarPlugin)
            .add_plugins(photon::PhotonPlugin);
    }
}

// Generic Factory component used by all buildings from diagram
#[derive(Component, Debug)]
pub struct Factory {
    pub recipe_id: String,
    pub progress: f32, // 0..1
    pub active: bool,
}

impl Factory {
    pub fn new(recipe_id: &str) -> Self {
        Self { recipe_id: recipe_id.to_string(), progress: 0.0, active: true }
    }
}

pub fn tick_factory(
    time: Res<Time>,
    recipes: Res<su_core::recipes::RecipeRegistry>,
    mut query: Query<(&mut Factory, &mut su_core::buffers::Buffer)>,
) {
    for (mut factory, mut buffer) in query.iter_mut() {
        if !factory.active { continue; }
        if let Some(recipe) = recipes.recipes.get(&factory.recipe_id) {
            // Check inputs
            let mut can_produce = true;
            for (res, amount) in &recipe.inputs {
                if buffer.amount(*res) < *amount {
                    can_produce = false;
                    break;
                }
            }
            if can_produce {
                factory.progress += time.delta_secs() / recipe.time_secs;
                if factory.progress >= 1.0 {
                    factory.progress = 0.0;
                    // Consume inputs
                    for (res, amount) in &recipe.inputs {
                        buffer.remove(*res, *amount);
                    }
                    // Produce outputs
                    for (res, amount) in &recipe.outputs {
                        buffer.insert(*res, *amount);
                    }
                }
            }
        }
    }
}
