#![allow(unused, dead_code)]

use bevy::prelude::*;

pub mod rough;
pub mod materials;

pub struct SuRenderExcaliPlugin;

impl Plugin for SuRenderExcaliPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(rough::RoughRenderPlugin)
            .add_plugins(materials::ExcaliMaterialPlugin);
    }
}
