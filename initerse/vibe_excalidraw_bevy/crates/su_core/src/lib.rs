use bevy::prelude::*;

/// Pure logic, no rendering. This is what LLMs can test easily.
pub mod resources;
pub mod recipes;
pub mod buffers;

pub struct SuCorePlugin;

impl Plugin for SuCorePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<resources::ResourceRegistry>()
            .init_resource::<recipes::RecipeRegistry>()
            .add_systems(FixedUpdate, buffers::tick_buffers);
    }
}

/// TASK-006 Acceptancce:
/// Define ResourceType enum based on diagram
pub use resources::ResourceType;
pub use recipes::Recipe;
