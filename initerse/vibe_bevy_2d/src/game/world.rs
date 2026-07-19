use crate::*;
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct GlobalInventory {
    pub total_gold: f32,
    pub total_collection_rate: f32,
}

pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalInventory::default())
                // Storage color tint (image tint based on fill)
        // .add_systems(Update, ())

        ;
    }
}



