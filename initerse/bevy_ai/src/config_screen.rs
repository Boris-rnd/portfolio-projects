use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use crate::AppState;
use crate::world_config::{Preset, WorldConfig};

pub struct ConfigScreenPlugin;

impl Plugin for ConfigScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConfigState>()
           .add_systems(OnEnter(AppState::ConfigScreen), setup_config_screen)
           .add_systems(OnExit(AppState::ConfigScreen), cleanup_config_screen)
           .add_systems(Update, (
               preset_button_system,
               config_adjust_system,
               start_game_button_system,
               back_button_system,
               update_config_display,
           ).run_if(in_state(AppState::ConfigScreen)));
    }
}

#[derive(Resource)]
struct ConfigState {
    seed: u64,
    preset: Preset,
    enemy_spawn_rate: f32,
    enemy_health: f32,
    gold_node_frequency: f32,
    building_health: f32,
}

impl Default for ConfigState {
    fn default() -> Self {
        let cfg = WorldConfig::from_preset(Preset::Normal, 1337);
        Self {
            seed: cfg.seed,
            preset: Preset::Normal,
            enemy_spawn_rate: cfg.enemy_spawn_rate,
            enemy_health: cfg.enemy_health,
            gold_node_frequency: cfg.gold_node_frequency,
            building_health: cfg.building_health,
        }
    }
}

#[derive(Component)]
struct ConfigScreenRoot;

#[derive(Component)]
enum PresetButton { Easy, Normal, Hard }

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct BackButton;

#[derive(Component, Clone, Copy)]
enum AdjustParam {
    SpawnRate,
    EnemyHealth,
    GoldFreq,
    BuildingHealth,
    Seed,
}

#[derive(Component, Clone, Copy)]
struct AdjustButton { param: AdjustParam, delta: f32 }

#[derive(Component)]
struct ConfigValueText(AdjustParam);

fn cleanup_config_screen(mut commands: Commands, q: Query<Entity, With<ConfigScreenRoot>>) {
    for e in &q { commands.entity(e).despawn(); }
}

fn row(card: &mut RelatedSpawnerCommands<ChildOf>, label: &str, param: AdjustParam) {
    card.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        margin: UiRect::bottom(Val::Px(8.0)),
        ..default()
    })
    .with_children(|row: &mut RelatedSpawnerCommands<ChildOf>| {
        // label
        row.spawn((
            Text::new(label),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.8, 0.9, 1.0)),
            Node { width: Val::Px(180.0), ..default() },
        ));

        // [-] button
        let neg_delta = match param {
            AdjustParam::SpawnRate => -0.25,
            AdjustParam::EnemyHealth => -1.0,
            AdjustParam::GoldFreq => -0.05,
            AdjustParam::BuildingHealth => -10.0,
            AdjustParam::Seed => -1.0,
        };
        small_btn(row, "−", AdjustButton { param, delta: neg_delta });

        // value label
        row.spawn((
            Text::new("..."),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::linear_rgba(0.5, 2.0, 1.0, 1.0)),
            Node { width: Val::Px(80.0), justify_content: JustifyContent::Center, ..default() },
            ConfigValueText(param),
        ));

        // [+] button
        let pos_delta = -neg_delta;
        small_btn(row, "+", AdjustButton { param, delta: pos_delta });
    });
}

fn small_btn(parent: &mut RelatedSpawnerCommands<ChildOf>, label: &str, marker: AdjustButton) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(32.0),
            height: Val::Px(32.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::horizontal(Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::linear_rgba(0.3, 1.0, 1.5, 0.7)),
        BackgroundColor(Color::srgba(0.0, 0.05, 0.1, 0.9)),
        marker,
    ))
    .with_children(|mut b: &mut RelatedSpawnerCommands<ChildOf>| {
        b.spawn((
            Text::new(label),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::linear_rgba(0.3, 1.5, 2.0, 1.0)),
        ));
    });
}

fn setup_config_screen(mut commands: Commands, mut cfg: ResMut<ConfigState>) {
    // Reset to normal on each entry
    *cfg = ConfigState::default();

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
        ConfigScreenRoot,
    ))
    .with_children(|root: &mut RelatedSpawnerCommands<ChildOf>| {
        root.spawn((
            Text::new("NEW GAME – CONFIGURATION"),
            TextFont { font_size: 42.0, ..default() },
            TextColor(Color::linear_rgba(0.3, 1.5, 2.5, 1.0)),
            Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
        ));

        // Card panel
        root.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(28.0)),
                border: UiRect::all(Val::Px(2.0)),
                min_width: Val::Px(520.0),
                ..default()
            },
            BorderColor::all(Color::linear_rgba(0.0, 0.8, 2.0, 0.5)),
            BackgroundColor(Color::srgba(0.01, 0.02, 0.08, 0.9)),
        ))
        .with_children(|card: &mut RelatedSpawnerCommands<ChildOf>| {
            // ── Preset row ────────────────────────────────────────────
            card.spawn((
                Text::new("Preset"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.7, 0.8, 1.0)),
                Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
            ));
            card.spawn(Node {
                flex_direction: FlexDirection::Row,
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            })
            .with_children(|row: &mut RelatedSpawnerCommands<ChildOf>| {
                for (label, preset, color) in [
                    ("Easy",   PresetButton::Easy,   Color::linear_rgba(0.3, 2.0, 0.5, 1.0)),
                    ("Normal", PresetButton::Normal, Color::linear_rgba(0.3, 1.0, 2.5, 1.0)),
                    ("Hard",   PresetButton::Hard,   Color::linear_rgba(2.0, 0.4, 0.3, 1.0)),
                ] {
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(36.0),
                            margin: UiRect::right(Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(color),
                        BackgroundColor(Color::srgba(0.0, 0.03, 0.08, 0.9)),
                        preset,
                    ))
                    .with_children(|b: &mut RelatedSpawnerCommands<ChildOf>| {
                        b.spawn((
                            Text::new(label),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(color),
                        ));
                    });
                }
            });

            // ── Parameter rows ────────────────────────────────────────
            row(card, "Seed", AdjustParam::Seed);
            row(card, "Enemy Spawn Rate (s)", AdjustParam::SpawnRate);
            row(card, "Enemy Health", AdjustParam::EnemyHealth);
            row(card, "Gold Node Density", AdjustParam::GoldFreq);
            row(card, "Building Health", AdjustParam::BuildingHealth);
        });

        // ── Bottom buttons ─────────────────────────────────────────────
        root.spawn(Node {
            flex_direction: FlexDirection::Row,
            margin: UiRect::top(Val::Px(24.0)),
            ..default()
        })
        .with_children(|row: &mut RelatedSpawnerCommands<ChildOf>| {
            // Back
            row.spawn((
                Button,
                Node {
                    width: Val::Px(140.0),
                    height: Val::Px(48.0),
                    margin: UiRect::right(Val::Px(16.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.06, 0.9)),
                BackButton,
            ))
            .with_children(|mut b: &mut RelatedSpawnerCommands<ChildOf>| {
                b.spawn((
                    Text::new("◀  Back"),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            });

            // Start
            row.spawn((
                Button,
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(48.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::linear_rgba(0.3, 2.5, 0.8, 1.0)),
                BackgroundColor(Color::srgba(0.0, 0.06, 0.03, 0.9)),
                StartButton,
            ))
            .with_children(|mut b: &mut RelatedSpawnerCommands<ChildOf>| {
                b.spawn((
                    Text::new("▶  START GAME"),
                    TextFont { font_size: 20.0, ..default() },
                    TextColor(Color::linear_rgba(0.3, 2.5, 0.8, 1.0)),
                ));
            });
        });
    });
}

fn preset_button_system(
    q: Query<(&Interaction, &PresetButton), (Changed<Interaction>, With<Button>)>,
    mut cfg: ResMut<ConfigState>,
) {
    for (interaction, btn) in &q {
        if *interaction != Interaction::Pressed { continue; }
        let preset = match btn {
            PresetButton::Easy   => Preset::Easy,
            PresetButton::Normal => Preset::Normal,
            PresetButton::Hard   => Preset::Hard,
        };
        let seed = cfg.seed;
        let base = WorldConfig::from_preset(preset, seed);
        cfg.preset = preset;
        cfg.enemy_spawn_rate = base.enemy_spawn_rate;
        cfg.enemy_health = base.enemy_health;
        cfg.gold_node_frequency = base.gold_node_frequency;
        cfg.building_health = base.building_health;
    }
}

fn config_adjust_system(
    q: Query<(&Interaction, &AdjustButton), (Changed<Interaction>, With<Button>)>,
    mut cfg: ResMut<ConfigState>,
) {
    for (interaction, btn) in &q {
        if *interaction != Interaction::Pressed { continue; }
        match btn.param {
            AdjustParam::SpawnRate => {
                cfg.enemy_spawn_rate = (cfg.enemy_spawn_rate + btn.delta).max(0.25);
            }
            AdjustParam::EnemyHealth => {
                cfg.enemy_health = (cfg.enemy_health + btn.delta).max(1.0);
            }
            AdjustParam::GoldFreq => {
                cfg.gold_node_frequency = (cfg.gold_node_frequency + btn.delta).clamp(0.05, 0.85);
            }
            AdjustParam::BuildingHealth => {
                cfg.building_health = (cfg.building_health + btn.delta).max(10.0);
            }
            AdjustParam::Seed => {
                cfg.seed = cfg.seed.wrapping_add(btn.delta as u64);
            }
        }
    }
}

fn update_config_display(
    cfg: Res<ConfigState>,
    mut texts: Query<(&mut Text, &ConfigValueText)>,
) {
    if !cfg.is_changed() { return; }
    for (mut text, label) in &mut texts {
        **text = match label.0 {
            AdjustParam::SpawnRate     => format!("{:.2}s", cfg.enemy_spawn_rate),
            AdjustParam::EnemyHealth   => format!("{:.0}", cfg.enemy_health),
            AdjustParam::GoldFreq      => format!("{:.2}", cfg.gold_node_frequency),
            AdjustParam::BuildingHealth=> format!("{:.0}", cfg.building_health),
            AdjustParam::Seed          => format!("{}", cfg.seed),
        };
    }
}

fn start_game_button_system(
    q: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    cfg: Res<ConfigState>,
    mut world_config: ResMut<WorldConfig>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &q {
        if *interaction != Interaction::Pressed { continue; }
        *world_config = WorldConfig {
            seed: cfg.seed,
            enemy_spawn_rate: cfg.enemy_spawn_rate,
            enemy_health: cfg.enemy_health,
            gold_node_frequency: cfg.gold_node_frequency,
            building_health: cfg.building_health,
        };
        next_state.set(AppState::InGame);
    }
}

fn back_button_system(
    q: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::MainMenu);
        }
    }
}
