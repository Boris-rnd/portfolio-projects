use bevy::{scene::SceneBox, window::PrimaryWindow};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Building {
    FoamExtractor{level: u32},
    GluonGenerator
}

#[derive(Component, Default, Clone)]
pub struct UiCanvasTargetFoamExtractor;
#[derive(Component, Default, Clone)]
pub struct UiCanvasTargetGluonGenerator;

impl Building {
    pub fn preview_scene(&self) -> Box<dyn Scene> {
        match self {
            Building::FoamExtractor { level } => {
                
                (bsn! {
                    #Button
                    Node {
                        width: percent(100.),
                        height: percent(100.),
                        position_type: PositionType::Relative,
                    }
                    BackgroundColor(Color::linear_rgb(0.5, 1., 0.2))
                    UiCanvasTargetFoamExtractor
                    on(|_press: On<Pointer<Press>>| {
                        dbg!();
                    })
                }).into()
            },
            Building::GluonGenerator => {
                bsn! {
                    #Button
                    Node {
                        width: percent(100.),
                        height: percent(100.),
                        position_type: PositionType::Relative,
                    }
                    BackgroundColor(Color::linear_rgb(0.5, 1., 0.2))
                    UiCanvasTargetGluonGenerator
                    on(|_press: On<Pointer<Press>>| {
                        dbg!();
                    })
                }.into()
            },
        }
    }
}

pub fn draw_dynamic_shapes_in_ui(
    query: Query<(&ComputedNode, &UiGlobalTransform), With<UiCanvasTargetFoamExtractor>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut painter: bevy_vector_shapes::painter::ShapePainter,
    camera: Single<(&Camera, &GlobalTransform), With<OverlayCamera>>,
) {
    for (computed_node, transform) in query.iter() {
        let size = computed_node.size();
        let mut ui_pos = transform.translation;
        let pos = camera.0.viewport_to_world_2d(camera.1, ui_pos).unwrap();
        ui_pos += camera.1.translation().xy();
        // pos.y = -pos.y;
        // pos -= window.size()/2.;
        dbg!(pos);
        // painter.transform.translation.z = 0.0;
        painter.render_layers = Some(bevy::camera::visibility::RenderLayers::layer(1));
        // painter.transform.translation.z = 10.;
        painter.set_translation(pos.extend(10.0));
        painter.color = Color::linear_rgb(0.0, 1.0, 1.0);
        use bevy_vector_shapes::shapes::DiscPainter;
        painter.circle(400.0); 
    }
}
