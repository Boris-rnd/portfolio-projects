use bevy::{camera::visibility::Visibility::Visible, ecs::{event::Trigger, system::IntoResult}, input::keyboard, window::PrimaryWindow};
use smallvec::SmallVec;

use crate::{game::{Building, WorldPosition}, render::world::RenderedWorld, *};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct Hotbar {
    pub selected_slots: Option<u32>,
    pub slots: [Option<Building>; 10],
}

#[derive(Message, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelectSlotMsg(pub Option<u32>);

pub fn spawn_hotbar(mut commands: Commands) {
    let mut hotbar = Hotbar {
        selected_slots: None,
        slots: [
            Some(Building::FoamExtractor { level: 1 }),
            Some(Building::GluonGenerator),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ],
    };

    commands.spawn_scene(bsn! {
        #Node
        Node {
            width: percent(100.),
            // min_width: px(800.),
            // max_width: percent(100.),
            height: px(80),
            position_type: PositionType::Absolute,
            left: percent(0.),
            bottom: px(0),
            
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: px(8),
        }
        BackgroundColor(Color::linear_rgba(0.2, 0.2, 0.2, 0.9))
        Children [
            {(0..10).map(|slot| {
                let preview = hotbar.slots[slot as usize].as_ref().map(|b| {
                    b.preview_scene()
                });
                // let x_position = -350.0 + (slot as f32 * 70.0);
                (bsn! {
                #Button
                Pickable::default()
                Node {
                    width: px(70),
                    height: px(70),
                    position_type: PositionType::Relative,
                    margin: px(5.),
                }
                BackgroundColor(Color::linear_rgba(0., 0., 0., 0.5))
                on(move |_press: On<Pointer<Press>>, mut ev_writer: MessageWriter<SelectSlotMsg>| {
                    ev_writer.write(SelectSlotMsg(Some(slot)));
                })
                Children [
                    (Pickable::default()
                        preview)
                ]
            })
            }).collect::<SmallVec<[_; 10]>>()}
        ]
    });
    commands.insert_resource(hotbar);

    let entity = commands.spawn_scene(bsn! {
        #Button
        Sprite {
            custom_size: Vec2::new(TILE_SIZE as f32, TILE_SIZE as f32),
            color: Color::linear_rgb(0.5, 0.5, 1.0),
        }
        Pickable::default()
        Transform::from_xyz(0.0, 0.0, 10.0)
        Visibility::Hidden

    }).observe(move |_press: On<Pointer<Press>>, mut prev_build: ResMut<PreviewBuilding>, mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>, camera: Single<(&Camera, &GlobalTransform)>, mut rendered_world: ResMut<RenderedWorld>, mut visibility_query: Query<&mut Visibility>| {
        let mut vis = visibility_query.get_mut(prev_build.entity).unwrap();
        if Visibility::Visible == *vis {
            // Hide the preview building and build it in the world
            *vis = Visibility::Hidden;
            // commands.entity(prev_build.entity).entry::<Visibility>().and_modify(|vis| {*vis = Visibility::Hidden;});

            let building = prev_build.building.take().expect("No building even though preview was visible");
            let mouse_pos = get_mouse_world_pos(window, camera).unwrap();
            let world_pos = WorldPosition {
                x: (mouse_pos.x / TILE_SIZE as f32).round() as i32,
                y: (mouse_pos.y / TILE_SIZE as f32).round() as i32,
            };
            rendered_world.insert_building(world_pos, building, &mut commands);
        }
    }).id();
    commands.insert_resource(PreviewBuilding {
        building: None,
        entity,
    });
}

pub fn interaction_hotbar(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut hotbar: ResMut<Hotbar>,
    mut ev_reader: MessageReader<SelectSlotMsg>,
) {
    for event in ev_reader.read() {
        hotbar.selected_slots = event.0;
    }

    for i in 0..10 {
        let keycode = match i {
            0 => KeyCode::Digit0,
            1 => KeyCode::Digit1,
            2 => KeyCode::Digit2,
            3 => KeyCode::Digit3,
            4 => KeyCode::Digit4,
            5 => KeyCode::Digit5,
            6 => KeyCode::Digit6,
            7 => KeyCode::Digit7,
            8 => KeyCode::Digit8,
            9 => KeyCode::Digit9,
            _ => unreachable!(),
        };
        if keyboard_input.just_pressed(keycode) {
            hotbar.selected_slots = Some(i as u32);
        }
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        hotbar.selected_slots = None;
    }
}

#[derive(Resource, Clone)]
pub struct PreviewBuilding {
    building: Option<Building>,
    entity: Entity,
}

pub fn hover_preview_building(
    hotbar: Res<Hotbar>,
    mut preview_building: ResMut<PreviewBuilding>,
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) {
    // dbg!(&hotbar);
    if let Some(selected_slot) = hotbar.selected_slots {
        if let Some(ref building) = hotbar.slots[selected_slot as usize] {
            if let Some(mouse_pos) = get_mouse_world_pos(window, camera_query) {
                // round the mouse position to the nearest tile
                let preview_building_position = Vec2::new(
                    (mouse_pos.x / TILE_SIZE as f32).round() * TILE_SIZE as f32,
                    (mouse_pos.y / TILE_SIZE as f32).round() * TILE_SIZE as f32,
                );
                preview_building.building = Some(building.clone());
                commands.entity(preview_building.entity).entry::<Transform>().and_modify(move |mut transform| {
                    transform.translation = preview_building_position.extend(10.0);
                });
                commands.entity(preview_building.entity).insert(Visibility::Visible);
            }
        }
    }
}

pub fn escape_preview(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut preview_building: ResMut<PreviewBuilding>,
    mut commands: Commands,
    mut select_slot_msg: MessageWriter<SelectSlotMsg>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) || mouse_input.just_pressed(MouseButton::Right) {
        preview_building.building = None;
        commands.entity(preview_building.entity).insert(Visibility::Hidden);
        select_slot_msg.write(SelectSlotMsg(None));
    }
}

fn get_mouse_world_pos(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (camera, camera_transform) = *camera_query;
    camera.viewport_to_world_2d(camera_transform, window.cursor_position()?).ok()
}
