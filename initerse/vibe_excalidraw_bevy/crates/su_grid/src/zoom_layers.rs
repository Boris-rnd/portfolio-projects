use bevy::prelude::*;

/// Seamless zoom layers system
/// Idea: Each layer has its own entity set. When zoom crosses threshold,
/// we fade out current layer and show parent as single building.

#[derive(Resource)]
pub struct ZoomLayerConfig {
    pub thresholds: Vec<f32>, // zoom levels where layer switches
}

impl Default for ZoomLayerConfig {
    fn default() -> Self {
        Self {
            thresholds: vec![0.5, 0.1, 0.02], // quantum -> atomic -> stellar -> galactic
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealityLayer {
    String = 0,
    Quark = 1,
    Atomic = 2,
    Stellar = 3,
    Galactic = 4,
    Multiverse = 5,
}

pub struct ZoomLayersPlugin;

impl Plugin for ZoomLayersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomLayerConfig>()
            .add_systems(Update, update_visible_layer);
    }
}

fn update_visible_layer(
    camera_state: Res<super::camera::CanvasCameraState>,
    config: Res<ZoomLayerConfig>,
    mut query: Query<(&RealityLayer, &mut Visibility)>,
) {
    let zoom = camera_state.current_zoom;
    let current_layer = get_layer_for_zoom(zoom, &config);
    
    for (layer, mut vis) in query.iter_mut() {
        // Simple: only show current layer +-1 with fade
        // TODO: make spectacular seamless transition
        let layer_diff = (*layer as i32 - current_layer as i32).abs();
        *vis = if layer_diff <= 1 { Visibility::Visible } else { Visibility::Hidden };
    }
}

fn get_layer_for_zoom(zoom: f32, config: &ZoomLayerConfig) -> RealityLayer {
    if zoom < config.thresholds[2] { RealityLayer::Galactic }
    else if zoom < config.thresholds[1] { RealityLayer::Stellar }
    else if zoom < config.thresholds[0] { RealityLayer::Atomic }
    else { RealityLayer::Quark }
}
