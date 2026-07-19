use crate::*;

#[derive(Resource, Debug)]
pub struct RenderedWorld {
    pub buildings: bevy::platform::collections::HashMap<WorldPosition, (Building, Entity)>,
    pub seed: usize,
}
impl RenderedWorld {
    pub fn new(buildings: bevy::platform::collections::HashMap<WorldPosition, (Building, Entity)>, seed: usize) -> Self {
        RenderedWorld { buildings, seed }
    }
    pub fn insert_building(&mut self, position: WorldPosition, building: Building, mut commands: &mut Commands) -> Entity {
        let entity = commands.spawn((
        Sprite {
                color: Color::linear_rgb(0.5, 0.5, 1.0),
                custom_size: Some(Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32)),
                ..default()
            },
            Transform::from_xyz(
                position.x as f32 * TILE_SIZE as f32,
                position.y as f32 * TILE_SIZE as f32,
                0.0,
            ),
        )).id();
        self.buildings.insert(position, (building, entity));
        entity
    }
    pub fn remove_building(&mut self, position: WorldPosition, mut commands: Commands) {
        if let Some((_, entity)) = self.buildings.remove(&position) {
            commands.entity(entity).despawn();
        }
    }
}