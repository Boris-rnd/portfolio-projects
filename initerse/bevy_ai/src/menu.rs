use bevy::prelude::*;
use crate::AppState;
use crate::save_load::LoadGameEvent;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
           .add_systems(OnExit(AppState::MainMenu), cleanup_state_scoped::<MainMenuRoot>)
           .add_systems(Update, main_menu_button_system.run_if(in_state(AppState::MainMenu)));
    }
}

#[derive(Component)]
struct MainMenuRoot;

#[derive(Component)]
enum MainMenuButton {
    NewGame,
    LoadGame,
    Quit,
}

fn cleanup_state_scoped<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for e in &query { commands.entity(e).despawn(); }
}

fn setup_main_menu(mut commands: Commands) {
    // Full-screen backdrop
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.04, 0.97)),
        MainMenuRoot,
    ))
    .with_children(|root| {
        // Title
        root.spawn((
            Text::new("INITERSE"),
            TextFont { font_size: 72.0, ..default() },
            TextColor(Color::linear_rgba(0.2, 1.5, 2.5, 1.0)),
            Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
        ));
        root.spawn((
            Text::new("Factory · Defense · Survive"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::linear_rgba(0.5, 0.8, 1.0, 0.7)),
            BorderColor::all(Color::linear_rgba(0.0, 0.8, 2.0, 0.6)),
            Node { margin: UiRect::bottom(Val::Px(60.0)), ..default() },
        ));

        // Buttons
        for (label, action, color) in [
            ("  ▶  NEW GAME", MainMenuButton::NewGame, Color::linear_rgba(0.2, 2.0, 0.8, 1.0)),
            ("  ⬆  LOAD GAME", MainMenuButton::LoadGame, Color::linear_rgba(0.3, 0.9, 2.5, 1.0)),
            ("  ✕  QUIT",     MainMenuButton::Quit,    Color::linear_rgba(2.0, 0.3, 0.3, 1.0)),
        ] {
            root.spawn((
                Button,
                Node {
                    width: Val::Px(280.0),
                    height: Val::Px(52.0),
                    margin: UiRect::all(Val::Px(8.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(color),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.06, 0.9)),
                action,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new(label),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(color),
                ));
            });
        }
    });
}

fn main_menu_button_system(
    interaction_q: Query<(&Interaction, &MainMenuButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut load_events: MessageWriter<LoadGameEvent>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button) in &interaction_q {
        if *interaction != Interaction::Pressed { continue; }
        match button {
            MainMenuButton::NewGame  => next_state.set(AppState::ConfigScreen),
            MainMenuButton::LoadGame => {
                load_events.write(LoadGameEvent);
                next_state.set(AppState::InGame);
            }
            MainMenuButton::Quit => { exit.write(AppExit::default()); }
        }
    }
}
