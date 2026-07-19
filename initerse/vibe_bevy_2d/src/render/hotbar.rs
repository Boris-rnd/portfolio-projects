use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Resource)]
pub struct Hotbar {
    pub selected_slot: Option<u32>,
    pub slots: [Option<Building>; 10],
}

#[derive(Component, Debug, Default)]
pub struct SelectedSlot;

fn preview_hotbar_change(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    window: Res<PrimaryWindow>,
    select_slot_wrt: MessageWriter<SelectSlotMsg>,
) {
    
    let mut toggle = |target: SelectedBuilding| {
        if *selected_building == target {
            *selected_building = SelectedBuilding::None;
        } else {
            *selected_building = target;
        }
    };

    if keyboard_input.just_pressed(KeyCode::Digit1) {
        toggle(SelectedBuilding::Collector);
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        toggle(SelectedBuilding::Storage);
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        toggle(SelectedBuilding::Turret);
    }
    if keyboard_input.just_pressed(KeyCode::Digit0) {
        toggle(SelectedBuilding::Destroy);
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        *selected_building = SelectedBuilding::None;
    }
}
#[derive(Message, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelectSlotMsg(pub usize);


pub fn hotbar_select_reader(
    hotbar: ResMut<Hotbar>,
    select_slot_msg: MessageReader<SelectSlotMsg>
) {

}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct HotbarButton;

pub fn setup_hotbar(mut commands: Commands) {
    let mut hotbar = Hotbar {
        selected_slot: None,
        slots: [
            Some(Building::FoamExtractor),
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
        Node {
            width: percent(100.0),
            height: px(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            bottom: px(10.0),
        }
        Children [
            {(0..10).map(|i| {
                let inner_color = match hotbar.slots[i] {
                    Some(Building::FoamExtractor) => Color::srgb(0.8, 0.4, 0.2),
                    Some(Building::GluonGenerator) => Color::srgb(0.2, 0.4, 0.8),
                    None => Color::srgb(0.0, 0.0, 0.0)
                };

                bsn! {
                    #Button
                    Node {
                        width: px(64.0),
                        height: px(64.0),
                        margin: UiRect::all(px(4.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(px(3.0)),
                    }
                    BorderColor::all(Color::srgb(0.1, 0.1, 0.1))
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15))
                    HotbarButton
                    Children [
                        Node {
                            width: px(44.0),
                            height: px(44.0),
                        }
                        BackgroundColor(inner_color),
                    ]
                    on(|press: On<Pointer<Press>>| {
                        dbg!()
                    })
                }
            }).collect::<SmallVec<[_; 10]>>()}
        ]
    });
    commands.insert_resource(hotbar);

}
