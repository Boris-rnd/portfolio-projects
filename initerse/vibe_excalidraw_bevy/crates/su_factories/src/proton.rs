use bevy::prelude::*;
use crate::tick_factory;
pub struct ProtonPlugin;
impl Plugin for ProtonPlugin { fn build(&self, app: &mut App) { app.add_systems(FixedUpdate, tick_factory); } }
