use bevy::prelude::*;

#[derive(Resource)]
pub struct GridConfig {
    pub cell_size: f32, // 64px default but zoom dependent?
    pub show_dots: bool,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self { cell_size: 64.0, show_dots: true }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
    pub layer: i32, // for zoom layers: 0 = quantum, 1 = atomic, 2 = stellar etc
}

#[derive(Component, Debug, Clone, Copy)]
pub struct GridSize {
    pub w: i32,
    pub h: i32,
}

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_draw_grid);
    }
}

fn debug_draw_grid(
    mut gizmos: Gizmos,
    config: Res<GridConfig>,
    cameras: Query<(&Transform, &OrthographicProjection), With<Camera2d>>,
) {
    // TODO: Replace with dot-grid shader (TASK-002)
    // For now simple dots
    if !config.show_dots { return; }
    
    if let Ok((cam_transform, proj)) = cameras.get_single() {
        let area = proj.area;
        let center = cam_transform.translation.truncate();
        let half_w = area.width() / 2.0;
        let half_h = area.height() / 2.0;
        
        let min_x = (center.x - half_w) / config.cell_size;
        let max_x = (center.x + half_w) / config.cell_size;
        let min_y = (center.y - half_h) / config.cell_size;
        let max_y = (center.y + half_h) / config.cell_size;
        
        for x in (min_x.floor() as i32)..=(max_x.ceil() as i32) {
            for y in (min_y.floor() as i32)..=(max_y.ceil() as i32) {
                let pos = Vec2::new(x as f32 * config.cell_size, y as f32 * config.cell_size);
                gizmos.circle_2d(pos, 2.0, Color::srgba(0.0, 0.0, 0.0, 0.15));
            }
        }
    }
}
