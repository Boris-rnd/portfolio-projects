use bevy::prelude::*;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_grid);
    }
}

pub const TILE_SIZE: f32 = 64.0;
pub const GRID_SIZE: i32 = 50; // Grid from -50 to 50 in both axes

pub fn setup_grid(mut commands: Commands) {
    let line_color = Color::srgb(0.15, 0.15, 0.15);
    let line_thickness = 2.0;
    let grid_extents = TILE_SIZE * GRID_SIZE as f32;

    for i in -GRID_SIZE..=GRID_SIZE {
        let x = i as f32 * TILE_SIZE;
        commands.spawn((
            Sprite {
                color: line_color,
                custom_size: Some(Vec2::new(line_thickness, grid_extents * 2.0)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, -10.0),
        ));

        let y = i as f32 * TILE_SIZE;
        commands.spawn((
            Sprite {
                color: line_color,
                custom_size: Some(Vec2::new(grid_extents * 2.0, line_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, y, -10.0),
        ));
    }
}

pub fn world_to_grid(world_pos: Vec2) -> IVec2 {
    let x = (world_pos.x / TILE_SIZE).floor() as i32;
    let y = (world_pos.y / TILE_SIZE).floor() as i32;
    IVec2::new(x, y)
}

pub fn grid_to_world(grid_pos: IVec2) -> Vec2 {
    Vec2::new(grid_pos.x as f32 * TILE_SIZE + TILE_SIZE / 2.0, grid_pos.y as f32 * TILE_SIZE + TILE_SIZE / 2.0)
}


use bevy::platform::collections::HashMap;

use crate::*;
#[derive(Default, Resource)]
pub struct RenderedGrid {
    grid: HashMap<IVec2, (Building, Entity)>,
    seed: usize,
}
impl RenderedGrid {
    pub fn new(seed: usize) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }
    // Returns the spawned entity if was able to insert the building, or 
    pub fn insert_building(&mut self, pos: IVec2, building: Building, commands: &mut Commands) -> (Entity, Option<(Building, Entity)>) {
        let prev_b = self.grid.remove_entry(&pos).map(|entry| entry.1);
        todo!();
        let ent = commands.spawn_scene(bsn! {
            
        }).id();
        self.grid.insert(pos, (building, ent));
        (ent, prev_b)
    }
    pub fn get_building(&self, pos: &IVec2) -> Option<&Building> {
        self.get(pos).map(|(b,e)|b)
    }
    pub fn get(&self, pos: &IVec2) -> Option<&(Building, Entity)> {
        self.grid.get(pos)
    }
}