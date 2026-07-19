use crate::*;
pub use world::GlobalInventory;
pub use buildings::*;
pub use crate::connection::*;
pub use grid::*;

pub mod buildings;
pub mod combat;
pub mod connection;
pub mod grid;
pub mod interaction;
pub mod world;
pub mod preview;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            world::WorldPlugin,
            buildings::BuildingPlugin,
            combat::CombatPlugin,
            connection::ConnectionPlugin,
            grid::GridPlugin,
            interaction::InteractionPlugin,
        ));
    }
}
