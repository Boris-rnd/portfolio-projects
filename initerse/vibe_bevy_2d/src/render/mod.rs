use crate::*;

pub mod camera;
pub mod ui;
pub mod hotbar;
pub mod world;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((            
            camera::CameraPlugin,
            ui::UiPlugin,
        ));
    }
}
