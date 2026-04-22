use bevy::prelude::*;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_grid)
           .add_systems(Update, pulse_grid_system);
    }
}

pub const TILE_SIZE: f32 = 64.0;
pub const GRID_SIZE: i32 = 50; // Grid from -50 to 50 in both axes

#[derive(Component)]
pub struct GridLine;

fn setup_grid(mut commands: Commands) {
    let line_color = Color::linear_rgba(0.0, 0.1, 0.2, 1.0); // Subtle dark blue base
    let line_thickness = 1.5;
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
            GridLine,
        ));

        let y = i as f32 * TILE_SIZE;
        commands.spawn((
            Sprite {
                color: line_color,
                custom_size: Some(Vec2::new(grid_extents * 2.0, line_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, y, -10.0),
            GridLine,
        ));
    }
}

fn pulse_grid_system(
    time: Res<Time>,
    mut query: Query<&mut Sprite, With<GridLine>>,
) {
    let t = time.elapsed_secs() * 0.5;
    let pulse = (t.sin() * 0.5 + 0.5) * 0.15 + 0.1; // Oscillate between 0.1 and 0.25
    
    for mut sprite in &mut query {
        sprite.color = Color::linear_rgba(0.0, pulse * 0.5, pulse, 1.0);
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
