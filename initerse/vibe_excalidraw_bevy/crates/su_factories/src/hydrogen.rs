use bevy::prelude::*;
use crate::tick_factory;
pub struct HydrogenPlugin;
impl Plugin for HydrogenPlugin { fn build(&self, app: &mut App) { app.add_systems(FixedUpdate, tick_factory); } }
