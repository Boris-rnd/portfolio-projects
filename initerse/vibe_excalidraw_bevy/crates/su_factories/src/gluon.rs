use bevy::prelude::*;
use crate::{Factory, tick_factory};
use su_core::buffers::Buffer;
use su_grid::placement::BuildingType;

pub struct GluonPlugin;

impl Plugin for GluonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, tick_factory)
           .add_systems(Update, spawn_gluon_generator_visuals);
    }
}

fn spawn_gluon_generator_visuals(
    mut commands: Commands,
    query: Query<Entity, Added<Factory>>,
) {
    // Visuals handled by rough renderer
}

// GluonGenerator specific logic: infinite source of gluons (vacuum)
#[derive(Component)]
pub struct GluonGenerator {
    pub rate: f32,
}

pub fn produce_gluons(
    time: Res<Time>,
    mut query: Query<(&GluonGenerator, &mut Buffer)>,
) {
    for (generator, mut buffer) in query.iter_mut() {
        buffer.insert(su_core::resources::ResourceType::Gluon, generator.rate * time.delta_secs());
    }
}
