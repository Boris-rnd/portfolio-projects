use bevy::prelude::*;
use crate::grid::{GridPos, GridSize};

#[derive(Resource, Default)]
pub struct PlacementState {
    pub selected_building: Option<BuildingType>,
    pub ghost_entity: Option<Entity>,
    pub is_drag_selecting: bool,
    pub selection_start: Option<Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum BuildingType {
    GluonGenerator,
    PlusBasin,
    MinusBasin,
    ProtonCreator,
    HydrogenAssembler,
    ArtificialStar,
    PhotonGenerator,
    PhotonDecay,
    Extractor,
    // Future:
    DronePort,
}

impl BuildingType {
    pub fn size(&self) -> GridSize {
        match self {
            Self::GluonGenerator => GridSize { w: 2, h: 2 },
            Self::PlusBasin => GridSize { w: 2, h: 2 },
            Self::MinusBasin => GridSize { w: 2, h: 2 },
            Self::ProtonCreator => GridSize { w: 3, h: 3 },
            Self::HydrogenAssembler => GridSize { w: 3, h: 3 },
            Self::ArtificialStar => GridSize { w: 5, h: 5 },
            Self::PhotonGenerator => GridSize { w: 2, h: 2 },
            Self::PhotonDecay => GridSize { w: 2, h: 2 },
            Self::Extractor => GridSize { w: 4, h: 2 },
            Self::DronePort => GridSize { w: 3, h: 3 },
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            Self::GluonGenerator => "Gluon gen\n-> up & down",
            Self::PlusBasin => "+ basin",
            Self::MinusBasin => "- basin",
            Self::ProtonCreator => "Proton\ncreator",
            Self::HydrogenAssembler => "Hydrogen atom",
            Self::ArtificialStar => "Artificial star\n(gravity/\nheat/fusion)",
            Self::PhotonGenerator => "Photons gen\n(virtual)",
            Self::PhotonDecay => "Photon decay\n-> e- & e+",
            Self::Extractor => "on star explosion\nor extractor",
            Self::DronePort => "Drone port",
        }
    }
}

pub struct PlacementPlugin;

impl Plugin for PlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_palette_selection, ghost_follow_mouse, place_on_click));
    }
}

fn handle_palette_selection(
    mut placement: ResMut<PlacementState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Keys 1-9 select building type
    if keys.just_pressed(KeyCode::Digit1) { placement.selected_building = Some(BuildingType::GluonGenerator); }
    if keys.just_pressed(KeyCode::Digit2) { placement.selected_building = Some(BuildingType::PlusBasin); }
    if keys.just_pressed(KeyCode::Digit3) { placement.selected_building = Some(BuildingType::MinusBasin); }
    if keys.just_pressed(KeyCode::Digit4) { placement.selected_building = Some(BuildingType::ProtonCreator); }
    if keys.just_pressed(KeyCode::Digit5) { placement.selected_building = Some(BuildingType::HydrogenAssembler); }
    if keys.just_pressed(KeyCode::Digit6) { placement.selected_building = Some(BuildingType::ArtificialStar); }
    // etc
}

fn ghost_follow_mouse() {
    // TODO: TASK-005 - implement ghost entity that snaps to grid
}

fn place_on_click() {
    // TODO: TASK-005
}
