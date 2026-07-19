#![allow(unused, dead_code)]
use bevy::prelude::*;

pub struct SuUiPlugin;

impl Plugin for SuUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PaletteState>()
            .add_systems(Update, (palette_ui, properties_panel, excalidraw_toolbar));
    }
}

#[derive(Resource, Default)]
pub struct PaletteState {
    pub selected: Option<su_grid::placement::BuildingType>,
}

/// Bottom bar like Excalidraw - shows building palette
fn palette_ui(
    mut gizmos: Gizmos,
    state: Res<PaletteState>,
) {
    // TODO: Replace with egui or custom UI
    // For now draw placeholder bar at bottom of screen
    // Will be egui panel
}

fn properties_panel() {
    // TODO: When select building, show in right sidebar: buffers, rates, label editing
    // Like Excalidraw stroke options panel
}

fn excalidraw_toolbar() {
    // Top toolbar: select, hand, text, etc.
}
