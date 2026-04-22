use tracing::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::building::{
    GridPosition, Collector, Storage, Turret, BuildingMarker, Base, GoldNode, BuildingAssets,
};
use crate::connection::{Connection, ConnectionLine, ItemMovement};
use crate::world_config::WorldConfig;
use crate::GlobalInventory;
use crate::animation::JuiceScale;
use crate::grid::{grid_to_world, TILE_SIZE};

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveGameEvent>()
           .add_message::<LoadGameEvent>()
           .init_resource::<PendingLoad>()
           .add_systems(Update, (
               handle_save_event,
               handle_load_event,
               apply_load_system,
           ));
    }
}

#[derive(Message, Default)]
pub struct SaveGameEvent;

#[derive(Message, Default)]
pub struct LoadGameEvent;

fn save_path() -> PathBuf {
    PathBuf::from("saves/save.json")
}

// ─── Serialisable data structures ───────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorldConfigData {
    pub seed: u64,
    pub enemy_spawn_rate: f32,
    pub enemy_health: f32,
    pub gold_node_frequency: f32,
    pub building_health: f32,
}

impl From<&WorldConfig> for WorldConfigData {
    fn from(c: &WorldConfig) -> Self {
        Self {
            seed: c.seed,
            enemy_spawn_rate: c.enemy_spawn_rate,
            enemy_health: c.enemy_health,
            gold_node_frequency: c.gold_node_frequency,
            building_health: c.building_health,
        }
    }
}

impl From<WorldConfigData> for WorldConfig {
    fn from(d: WorldConfigData) -> Self {
        Self {
            seed: d.seed,
            enemy_spawn_rate: d.enemy_spawn_rate,
            enemy_health: d.enemy_health,
            gold_node_frequency: d.gold_node_frequency,
            building_health: d.building_health,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum BuildingKind {
    Base,
    Collector,
    Storage,
    Turret,
}

/// Connection targets serialised as grid positions (not entity IDs, which are session-specific)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BuildingSaveData {
    pub kind: BuildingKind,
    pub grid_x: i32,
    pub grid_y: i32,
    /// Used only for Storage / Base
    pub storage_amount: Option<f32>,
    /// Used only for Collector – targets stored as grid positions
    pub connection_targets: Vec<[i32; 2]>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveData {
    pub config: WorldConfigData,
    pub total_gold: f32,
    pub buildings: Vec<BuildingSaveData>,
}

// ─── Save ────────────────────────────────────────────────────────────────────

fn handle_save_event(
    mut events: MessageReader<SaveGameEvent>,
    config: Res<WorldConfig>,
    inventory: Res<GlobalInventory>,
    buildings: Query<
        (Entity, &GridPosition, Option<&Base>, Option<&Collector>, Option<&Storage>, Option<&Connection>, Option<&Turret>),
        With<BuildingMarker>,
    >,
    all_positions: Query<(Entity, &GridPosition), With<BuildingMarker>>,
) {
    for _ in events.read() {
        let config_data = WorldConfigData::from(config.as_ref());
        let mut building_list: Vec<BuildingSaveData> = Vec::new();

        for (entity, gp, base, collector, storage, connection, turret) in &buildings {
            let kind = if base.is_some() {
                BuildingKind::Base
            } else if collector.is_some() {
                BuildingKind::Collector
            } else if storage.is_some() {
                BuildingKind::Storage
            } else if turret.is_some() {
                BuildingKind::Turret
            } else {
                continue;
            };

            let storage_amount = storage.map(|s| s.current_amount);

            // Encode connection targets as grid positions
            let connection_targets: Vec<[i32; 2]> = if let Some(conn) = connection {
                conn.targets.iter().filter_map(|&target_entity| {
                    all_positions.get(target_entity).ok().map(|(_, p)| [p.0.x, p.0.y])
                }).collect()
            } else {
                Vec::new()
            };
            let _ = entity; // used implicitly above

            building_list.push(BuildingSaveData {
                kind,
                grid_x: gp.0.x,
                grid_y: gp.0.y,
                storage_amount,
                connection_targets,
            });
        }

        let save_data = SaveData {
            config: config_data,
            total_gold: inventory.total_gold,
            buildings: building_list,
        };

        if let Ok(json) = serde_json::to_string_pretty(&save_data) {
            let path = save_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&path, &json) {
                Ok(_) => info!("Game saved to {:?}", path),
                Err(e) => error!("Failed to save game: {}", e),
            }
        }
    }
}

// ─── Load ────────────────────────────────────────────────────────────────────

/// Holds deserialized data between the "trigger load" and "apply load" frames.
#[derive(Resource, Default)]
pub struct PendingLoad(pub Option<SaveData>);

fn handle_load_event(
    mut events: MessageReader<LoadGameEvent>,
    mut pending: ResMut<PendingLoad>,
    // Despawn existing world entities
    mut commands: Commands,
    building_ents: Query<Entity, Or<(
        With<BuildingMarker>,
        With<GoldNode>,
        With<ItemMovement>,
        With<ConnectionLine>,
    )>>,
) {
    for _ in events.read() {
        let path = save_path();
        match std::fs::read_to_string(&path) {
            Err(e) => { error!("Failed to read save file: {}", e); }
            Ok(text) => {
                match serde_json::from_str::<SaveData>(&text) {
                    Err(e) => { error!("Failed to parse save file: {}", e); }
                    Ok(data) => {
                        // Despawn everything
                        for e in &building_ents { commands.entity(e).despawn(); }
                        info!("Load triggered – {} buildings in save", data.buildings.len());
                        pending.0 = Some(data);
                    }
                }
            }
        }
    }
}

fn apply_load_system(
    mut commands: Commands,
    mut pending: ResMut<PendingLoad>,
    mut config: ResMut<WorldConfig>,
    mut gold_nodes_spawned: Local<bool>,
    building_assets: Option<Res<BuildingAssets>>,
) {
    let Some(data) = pending.0.clone() else { return };
    let Some(assets) = building_assets else { return };

    // Apply config
    *config = data.config.clone().into();
    *gold_nodes_spawned = false; // allow gold nodes to re-spawn

    // First pass: spawn all buildings, collect (grid_pos → entity) map
    let mut grid_to_entity: std::collections::HashMap<[i32; 2], Entity> =
        std::collections::HashMap::new();

    for bld in &data.buildings {
        let gp = IVec2::new(bld.grid_x, bld.grid_y);
        let world_pos = grid_to_world(gp);

        let image = match bld.kind {
            BuildingKind::Base      => assets.base.clone(),
            BuildingKind::Collector => assets.collector.clone(),
            BuildingKind::Storage   => assets.storage.clone(),
            BuildingKind::Turret    => assets.turret.clone(),
        };
        let custom_size = match bld.kind {
            BuildingKind::Base      => Vec2::new(TILE_SIZE * 1.5, TILE_SIZE * 1.5),
            _ => Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.9),
        };

        let mut ecmd = commands.spawn((
            Sprite {
                image: image.clone(),
                custom_size: Some(custom_size),
                color: Color::linear_rgba(1.2, 1.2, 1.2, 1.0),
                ..default()
            },
            Transform::from_translation(world_pos.extend(1.0)),
            GridPosition(gp),
            BuildingMarker,
            JuiceScale::default(),
        ));

        match bld.kind {
            BuildingKind::Base => {
                ecmd.insert(Base);
                ecmd.insert(Storage {
                    current_amount: bld.storage_amount.unwrap_or(0.0),
                    max_capacity: 1000.0,
                });
            }
            BuildingKind::Collector => {
                ecmd.insert(Collector::default());
                ecmd.insert(Connection { targets: Vec::new() }); // filled below
            }
            BuildingKind::Storage => {
                ecmd.insert(Storage {
                    current_amount: bld.storage_amount.unwrap_or(0.0),
                    max_capacity: 100.0,
                });
            }
            BuildingKind::Turret => {
                ecmd.insert(Turret::default());
            }
        }

        let entity = ecmd.id();
        grid_to_entity.insert([bld.grid_x, bld.grid_y], entity);
    }

    // Second pass: wire up connections  (needs entities to exist)
    // We do this by iterating again; connections_targets stored as a deferred command
    // We'll schedule a one-shot system via a local resource trick — simpler: just store
    // them in a resource and wire in next frame.
    commands.insert_resource(PendingConnections(
        data.buildings.iter().filter_map(|bld| {
            if !bld.connection_targets.is_empty() {
                let src = grid_to_entity.get(&[bld.grid_x, bld.grid_y]).copied()?;
                let targets: Vec<Entity> = bld.connection_targets.iter()
                    .filter_map(|t| grid_to_entity.get(t).copied())
                    .collect();
                Some((src, targets))
            } else {
                None
            }
        }).collect()
    ));

    // Signal gold nodes to respawn
    commands.insert_resource(NeedGoldNodeRespawn(true));

    pending.0 = None;
    info!("Load applied successfully");
}

/// Deferred connection wiring (resolved the frame after entities exist)
#[derive(Resource)]
pub struct PendingConnections(pub Vec<(Entity, Vec<Entity>)>);

/// Flag to re-trigger gold-node spawning after a load
#[derive(Resource)]
pub struct NeedGoldNodeRespawn(pub bool);

/// System run every frame to flush pending connection wiring
pub fn flush_pending_connections(
    mut commands: Commands,
    pending: Option<ResMut<PendingConnections>>,
    mut connections: Query<&mut Connection>,
) {
    let Some(mut p) = pending else { return };
    if p.0.is_empty() { return; }
    for (src, targets) in p.0.drain(..) {
        if let Ok(mut conn) = connections.get_mut(src) {
            conn.targets = targets;
        }
    }
    commands.remove_resource::<PendingConnections>();
}
