use crate::*;
pub mod material;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::storage::ShaderBuffer;
pub use material::*;
pub mod camera;
pub use camera::*;

pub use bevy::render::*;
pub use bevy::shader::*;

#[derive(Resource, ExtractResource, Clone)]
pub struct AccumulatedTexture(pub Handle<ShaderBuffer>, pub Handle<ShaderBuffer>);


pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    window_query: Single<&Window, With<bevy::window::PrimaryWindow>>,
    mut materials: ResMut<Assets<PassthroughMaterial>>,
    mut buffers: ResMut<Assets<bevy::render::storage::ShaderBuffer>>,
    mut imgs: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    game_world: Res<GameWorld>,
    camera: Res<FragCamera>,
    accumulated_tex: Res<AccumulatedTexture>,
) {
    let center = vec3(-10., 10., -10.);

    let image_dimensions = window_query.resolution.size();
    // commands.spawn((
    //     Mesh2d(meshes.add(Rectangle::default())),
    //     MeshMaterial2d(materials.add(PassthroughMaterial {
    //         camera: camera.clone(),
    //         accumulated_tex: accumulated_tex.0.0.clone(),
    //         // accumulated_tex2: accumulated_tex.0.1.clone(),
    //     })),
    //     Transform::default().with_scale(image_dimensions.extend(0.0)),
    // ));

    commands.spawn(Camera2d);
}