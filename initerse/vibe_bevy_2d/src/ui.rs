use bevy::prelude::*;
use crate::GlobalInventory;
use crate::building::{Collector, Storage, Base};
use crate::connection::Connection;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedBuilding>()
           .init_resource::<InspectedEntity>()
           .add_systems(Startup, setup_ui)
           .add_systems(Update, (
               button_interaction_system,
               button_visuals_system,
               hotbar_keyboard_system,
               inventory_panel_system,
               inspection_system,
           ));
    }
}

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum SelectedBuilding {
    #[default]
    None,
    Collector,
    Storage,
    Turret,
    Destroy,
}

#[derive(Resource, Default)]
pub struct InspectedEntity(pub Option<Entity>);

#[derive(Component)]
pub struct BuildingButton(pub SelectedBuilding);

#[derive(Component)]
pub struct GlobalStatsText;

#[derive(Component)]
pub struct InspectionPanel;

#[derive(Component)]
pub struct InspectionText;

fn setup_ui(mut commands: Commands) {
    // ── Hotbar ──
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for i in 1..=10u32 {
                let building = match i {
                    1 => SelectedBuilding::Collector,
                    2 => SelectedBuilding::Storage,
                    3 => SelectedBuilding::Turret,
                    10 => SelectedBuilding::Destroy,
                    _ => SelectedBuilding::None,
                };

                let inner_color = match building {
                    SelectedBuilding::Collector => Color::srgb(0.8, 0.4, 0.2),
                    SelectedBuilding::Storage   => Color::srgb(0.2, 0.4, 0.8),
                    SelectedBuilding::Turret    => Color::srgb(0.5, 0.5, 0.9),
                    SelectedBuilding::Destroy   => Color::srgb(0.8, 0.1, 0.1),
                    SelectedBuilding::None      => Color::srgba(0.0, 0.0, 0.0, 0.0),
                };

                let label = if i == 10 { "0".to_string() } else { i.to_string() };

                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(64.0),
                            height: Val::Px(64.0),
                            margin: UiRect::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(3.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.1, 0.1, 0.1)),
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                        BuildingButton(building),
                    ))
                    .with_children(|slot| {
                        slot.spawn((
                            Node {
                                width: Val::Px(44.0),
                                height: Val::Px(44.0),
                                ..default()
                            },
                            BackgroundColor(inner_color),
                        ));

                        slot.spawn((
                            Text::new(label),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(2.0),
                                left: Val::Px(4.0),
                                ..default()
                            },
                        ));
                    });
            }
        });

    // ── Global Stats (top-left) ──
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Global Resources"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.4)),
            ));
            parent.spawn((
                Text::new("Loading..."),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                GlobalStatsText,
            ));
        });

    // ── Inspection Panel (bottom-left) ──
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(120.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                min_width: Val::Px(200.0),
                ..default()
            },
            Visibility::Hidden,
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.8)),
            InspectionPanel,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Building Info"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.6, 0.8, 1.0)),
            ));
            parent.spawn((
                Text::new(""),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                InspectionText,
            ));
        });
}

fn inspection_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    selected_building: Res<SelectedBuilding>,
    buildings: Query<(Entity, &crate::building::GridPosition), With<crate::building::BuildingMarker>>,
    mut inspected: ResMut<InspectedEntity>,
    mut panel_q: Query<&mut Visibility, With<InspectionPanel>>,
    mut text_q: Query<&mut Text, With<InspectionText>>,
    collector_q: Query<(&Collector, &Connection)>,
    storage_q: Query<(&Storage, Option<&Base>)>,
) {
    if *selected_building != SelectedBuilding::None {
        if let Ok(mut v) = panel_q.single_mut() { *v = Visibility::Hidden; }
        inspected.0 = None;
        return;
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else { return };
        let Ok((camera, camera_transform)) = camera_q.single() else { return };
        let Some(cursor_pos) = window.cursor_position() else { return };
        let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };
        let grid_pos = crate::grid::world_to_grid(world_pos);

        if let Some((entity, _)) = buildings.iter().find(|(_, p)| p.0 == grid_pos) {
            inspected.0 = Some(entity);
        } else {
            inspected.0 = None;
        }
    }

    let Ok(mut visibility) = panel_q.single_mut() else { return };
    let Ok(mut text) = text_q.single_mut() else { return };

    if let Some(entity) = inspected.0 {
        *visibility = Visibility::Visible;
        let mut info = Vec::new();

        if let Ok((collector, conn)) = collector_q.get(entity) {
            info.push("Type: Gold Collector".to_string());
            info.push(format!("Speed: {:.1} gold/s", collector.items_per_second));
            info.push(format!("Connections: {}", conn.targets.len()));
        } else if let Ok((storage, base)) = storage_q.get(entity) {
            if base.is_some() {
                info.push("Type: Main Base".to_string());
            } else {
                info.push("Type: Gold Storage".to_string());
            }
            info.push(format!("Stored: {:.1} / {:.0}", storage.current_amount, storage.max_capacity));
        }

        **text = info.join("\n");
    } else {
        *visibility = Visibility::Hidden;
    }
}

fn inventory_panel_system(
    inventory: Res<GlobalInventory>,
    mut text_q: Query<&mut Text, With<GlobalStatsText>>,
) {
    let Ok(mut text) = text_q.single_mut() else { return };
    **text = format!(
        "Total Gold: {:.1}\nCollection: {:.1}/s",
        inventory.total_gold,
        inventory.total_collection_rate
    );
}

fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &BuildingButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut selected_building: ResMut<SelectedBuilding>,
) {
    for (interaction, button) in &mut interaction_query {
        if *interaction == Interaction::Pressed && button.0 != SelectedBuilding::None {
            if *selected_building == button.0 {
                *selected_building = SelectedBuilding::None;
            } else {
                *selected_building = button.0;
            }
        }
    }
}

fn button_visuals_system(
    mut button_query: Query<(&BuildingButton, &mut BorderColor, &mut BackgroundColor, &Interaction)>,
    selected_building: Res<SelectedBuilding>,
) {
    for (button, mut border, mut bg, interaction) in &mut button_query {
        let is_selected = *selected_building == button.0 && button.0 != SelectedBuilding::None;

        *border = BorderColor::all(if is_selected {
            Color::srgb(0.8, 0.9, 0.3)
        } else {
            Color::srgb(0.1, 0.1, 0.1)
        });

        *bg = BackgroundColor(match *interaction {
            Interaction::Pressed => Color::srgb(0.35, 0.35, 0.35),
            Interaction::Hovered => Color::srgb(0.25, 0.25, 0.25),
            Interaction::None    => Color::srgb(0.15, 0.15, 0.15),
        });
    }
}

fn hotbar_keyboard_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected_building: ResMut<SelectedBuilding>,
) {
    let mut toggle = |target: SelectedBuilding| {
        if *selected_building == target {
            *selected_building = SelectedBuilding::None;
        } else {
            *selected_building = target;
        }
    };

    if keyboard_input.just_pressed(KeyCode::Digit1) { toggle(SelectedBuilding::Collector); }
    if keyboard_input.just_pressed(KeyCode::Digit2) { toggle(SelectedBuilding::Storage); }
    if keyboard_input.just_pressed(KeyCode::Digit3) { toggle(SelectedBuilding::Turret); }
    if keyboard_input.just_pressed(KeyCode::Digit0) { toggle(SelectedBuilding::Destroy); }
    
    if keyboard_input.just_pressed(KeyCode::Escape) {
        *selected_building = SelectedBuilding::None;
    }
}

