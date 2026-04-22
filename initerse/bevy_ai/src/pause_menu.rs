use bevy::prelude::*;

use crate::AppState;
use crate::save_load::{SaveGameEvent, LoadGameEvent};

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeybindSettings>()
           .init_resource::<WaitingForRebind>()
           .add_systems(OnEnter(AppState::Paused), setup_pause_menu)
           .add_systems(OnExit(AppState::Paused), cleanup_pause_menu)
           .add_systems(Update, toggle_pause_system.run_if(in_state(AppState::Paused).or(in_state(AppState::InGame))))
           .add_systems(Update, (
               pause_button_system,
               keybind_rebind_system,
           ).run_if(in_state(AppState::Paused)));
    }
}

// ─── Keybind settings ────────────────────────────────────────────────────────

#[derive(Resource, Clone)]
pub struct KeybindSettings {
    pub pan_up:    KeyCode,
    pub pan_down:  KeyCode,
    pub pan_left:  KeyCode,
    pub pan_right: KeyCode,
    pub slot_1:    KeyCode,
    pub slot_2:    KeyCode,
    pub slot_3:    KeyCode,
    pub slot_destroy: KeyCode,
}

impl Default for KeybindSettings {
    fn default() -> Self {
        Self {
            pan_up:      KeyCode::KeyW,
            pan_down:    KeyCode::KeyS,
            pan_left:    KeyCode::KeyA,
            pan_right:   KeyCode::KeyD,
            slot_1:      KeyCode::Digit1,
            slot_2:      KeyCode::Digit2,
            slot_3:      KeyCode::Digit3,
            slot_destroy: KeyCode::Digit0,
        }
    }
}

// ─── Components ──────────────────────────────────────────────────────────────

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct KeybindPanel;

#[derive(Component)]
struct KeybindRow(KeybindAction);

#[derive(Component)]
struct KeylabelMarker;

#[derive(Clone, Copy, PartialEq, Eq, Component)]
pub enum KeybindAction {
    PanUp, PanDown, PanLeft, PanRight,
    Slot1, Slot2, Slot3, SlotDestroy,
}

impl std::fmt::Display for KeybindAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::PanUp      => "Pan Up",
            Self::PanDown    => "Pan Down",
            Self::PanLeft    => "Pan Left",
            Self::PanRight   => "Pan Right",
            Self::Slot1      => "Build: Collector",
            Self::Slot2      => "Build: Storage",
            Self::Slot3      => "Build: Turret",
            Self::SlotDestroy=> "Destroy Mode",
        })
    }
}

#[derive(Resource, Default)]
pub struct WaitingForRebind(pub Option<KeybindAction>);

#[derive(Component)]
enum PauseButton {
    Resume,
    Save,
    Load,
    Keybinds,
    QuitToMenu,
}

// ─── Toggle ──────────────────────────────────────────────────────────────────

fn toggle_pause_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next: ResMut<NextState<AppState>>,
    hotbar: Res<crate::ui::Hotbar>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if hotbar.selected != crate::ui::SelectedBuilding::None && state.get() == &AppState::InGame {
            return; // Let the UI system deselect first
        }
        match state.get() {
            AppState::InGame => next.set(AppState::Paused),
            AppState::Paused => next.set(AppState::InGame),
            _ => {}
        }
    }
}

// ─── Pause menu UI ───────────────────────────────────────────────────────────

fn cleanup_pause_menu(mut commands: Commands, q: Query<Entity, With<PauseMenuRoot>>) {
    for e in &q { commands.entity(e).despawn(); }
}

fn key_label(key: KeyCode) -> &'static str {
    match key {
        KeyCode::KeyW      => "W",
        KeyCode::KeyA      => "A",
        KeyCode::KeyS      => "S",
        KeyCode::KeyD      => "D",
        KeyCode::Digit0    => "0",
        KeyCode::Digit1    => "1",
        KeyCode::Digit2    => "2",
        KeyCode::Digit3    => "3",
        KeyCode::Digit4    => "4",
        KeyCode::Digit5    => "5",
        KeyCode::ArrowUp   => "↑",
        KeyCode::ArrowDown => "↓",
        KeyCode::ArrowLeft => "←",
        KeyCode::ArrowRight=> "→",
        _ => "?",
    }
}

fn action_current_key(action: KeybindAction, kb: &KeybindSettings) -> KeyCode {
    match action {
        KeybindAction::PanUp       => kb.pan_up,
        KeybindAction::PanDown     => kb.pan_down,
        KeybindAction::PanLeft     => kb.pan_left,
        KeybindAction::PanRight    => kb.pan_right,
        KeybindAction::Slot1       => kb.slot_1,
        KeybindAction::Slot2       => kb.slot_2,
        KeybindAction::Slot3       => kb.slot_3,
        KeybindAction::SlotDestroy => kb.slot_destroy,
    }
}

fn setup_pause_menu(mut commands: Commands, mut waiting: ResMut<WaitingForRebind>, kb: Res<KeybindSettings>) {
    waiting.0 = None;

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        PauseMenuRoot,
    ))
    .with_children(|root| {
        // ── Left panel: pause actions ──────────────────────────────────
        root.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(30.0)),
                margin: UiRect::right(Val::Px(30.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(240.0),
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(Color::linear_rgba(0.0, 0.8, 2.0, 0.6)),
            BackgroundColor(Color::srgba(0.01, 0.01, 0.08, 0.92)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("PAUSED"),
                TextFont { font_size: 36.0, ..default() },
                TextColor(Color::linear_rgba(0.3, 1.5, 2.5, 1.0)),
                Node { margin: UiRect::bottom(Val::Px(24.0)), ..default() },
            ));

            for (label, action, color) in [
                ("▶  Resume",       PauseButton::Resume,     Color::linear_rgba(0.3, 2.5, 0.8, 1.0)),
                ("💾  Save",         PauseButton::Save,       Color::linear_rgba(0.8, 2.0, 0.3, 1.0)),
                ("📂  Load",         PauseButton::Load,       Color::linear_rgba(0.3, 1.0, 2.5, 1.0)),
                ("⌨  Keybinds",     PauseButton::Keybinds,  Color::srgb(0.8, 0.7, 1.0)),
                ("✕  Quit to Menu", PauseButton::QuitToMenu, Color::linear_rgba(2.0, 0.3, 0.3, 1.0)),
            ] {
                panel.spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(44.0),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(color),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.06, 0.9)),
                    action,
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(color),
                    ));
                });
            }
        });

        // ── Right panel: keybinds ──────────────────────────────────────
        root.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(24.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(340.0),
                ..default()
            },
            BorderColor::all(Color::linear_rgba(0.5, 0.3, 2.0, 0.5)),
            BackgroundColor(Color::srgba(0.02, 0.01, 0.08, 0.92)),
            KeybindPanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("KEYBINDS  (click to rebind)"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.7, 0.6, 1.0)),
                Node { margin: UiRect::bottom(Val::Px(16.0)), ..default() },
            ));

            for action in [
                KeybindAction::PanUp,
                KeybindAction::PanDown,
                KeybindAction::PanLeft,
                KeybindAction::PanRight,
                KeybindAction::Slot1,
                KeybindAction::Slot2,
                KeybindAction::Slot3,
                KeybindAction::SlotDestroy,
            ] {
                let current_key = key_label(action_current_key(action, &kb));
                panel.spawn((
                    Button,
                    Node {
                        flex_direction: FlexDirection::Row,
                        width: Val::Percent(100.0),
                        height: Val::Px(36.0),
                        margin: UiRect::bottom(Val::Px(6.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::horizontal(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.4, 0.3, 0.8, 0.5)),
                    BackgroundColor(Color::srgba(0.03, 0.02, 0.08, 0.8)),
                    KeybindRow(action),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(action.to_string()),
                        TextFont { font_size: 15.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.9)),
                    ));
                    row.spawn((
                        Text::new(current_key),
                        TextFont { font_size: 15.0, ..default() },
                        TextColor(Color::linear_rgba(0.5, 2.0, 1.0, 1.0)),
                        KeylabelMarker,
                    ));
                });
            }
        });
    });
}

fn pause_button_system(
    q: Query<(&Interaction, &PauseButton), (Changed<Interaction>, With<Button>)>,
    mut next: ResMut<NextState<AppState>>,
    mut save_ev: MessageWriter<SaveGameEvent>,
    mut load_ev: MessageWriter<LoadGameEvent>,
) {
    for (interaction, button) in &q {
        if *interaction != Interaction::Pressed { continue; }
        match button {
            PauseButton::Resume     => next.set(AppState::InGame),
            PauseButton::Save       => { save_ev.write(SaveGameEvent); }
            PauseButton::Load       => {
                load_ev.write(LoadGameEvent);
                next.set(AppState::InGame);
            }
            PauseButton::Keybinds   => { /* panel is always visible */ }
            PauseButton::QuitToMenu => next.set(AppState::MainMenu),
        }
    }
}

fn keybind_rebind_system(
    mut waiting: ResMut<WaitingForRebind>,
    mut kb: ResMut<KeybindSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    click_q: Query<(&Interaction, &KeybindRow), (Changed<Interaction>, With<Button>)>,
    mut keybind_text_q: Query<(&mut Text, &ChildOf), With<KeylabelMarker>>,
    keybind_row_q: Query<(Entity, &KeybindRow)>,
) {
    // Click on a row → start waiting for new key
    for (interaction, row) in &click_q {
        if *interaction == Interaction::Pressed {
            waiting.0 = Some(row.0);
        }
    }

    // Capture next pressed key
    if let Some(action) = waiting.0 {
        if let Some(key) = keyboard.get_just_pressed().next() {
            match action {
                KeybindAction::PanUp       => kb.pan_up    = *key,
                KeybindAction::PanDown     => kb.pan_down  = *key,
                KeybindAction::PanLeft     => kb.pan_left  = *key,
                KeybindAction::PanRight    => kb.pan_right = *key,
                KeybindAction::Slot1       => kb.slot_1    = *key,
                KeybindAction::Slot2       => kb.slot_2    = *key,
                KeybindAction::Slot3       => kb.slot_3    = *key,
                KeybindAction::SlotDestroy => kb.slot_destroy = *key,
            }
            waiting.0 = None;

            // Update label in the UI: find the row entity, then find child Text
            for (row_entity, row_marker) in &keybind_row_q {
                if row_marker.0 == action {
                    for (mut text, parent) in &mut keybind_text_q {
                        if parent.0 == row_entity {
                            // The second child is the key label
                            let current = action_current_key(action, &kb);
                            **text = key_label(current).to_string();
                        }
                    }
                    break;
                }
            }
        }
    }
}
