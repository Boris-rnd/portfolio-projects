use bevy::{prelude::*, render::render_resource::StorageBuffer};
use bevy_flycam::prelude::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(PlayerPlugin)
    .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
    .add_plugins(iyes_perf_ui::PerfUiPlugin)
    .add_systems(Startup, setup)
    .run()
    ;
}

pub const PARTICLE_COUNT: usize = 100;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut meshes_let: ResMut<Assets<MeshletMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(iyes_perf_ui::prelude::PerfUiCompleteBundle::default());
    // commands.spawn(Camera3dBundle { ..Default::default() });

    let mesh = meshes.add(Sphere {
        radius: 1.,
    });

    let mut pos = Vec::with_capacity(PARTICLE_COUNT);
    for i in 0..PARTICLE_COUNT {
        let x = fastrand::f32()*100.;
        let y = fastrand::f32()*100.;
        let z = fastrand::f32()*100.;
        pos.push(Vec3::new(x,y,z));
    }
    let buffer = bevy::render::render_resource::StorageBuffer::from(&pos);


    let mat = materials.add(Color::srgb_u8(255, 255, 255));

    for pos in pos {
        commands.spawn(MaterialMeshBundle {
            mesh: mesh.clone(),
            material: mat.clone(),
            transform: Transform::from_translation(pos),
            ..default()
        });
    }
}
