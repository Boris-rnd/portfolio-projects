use bevy::prelude::*;
use crate::tick_factory;
pub struct PhotonPlugin;
impl Plugin for PhotonPlugin { fn build(&self, app: &mut App) { app.add_systems(FixedUpdate, tick_factory); } }
