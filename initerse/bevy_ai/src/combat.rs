use bevy::prelude::*;
use crate::building::{Turret, BuildingAssets};
use crate::world_config::WorldConfig;
use crate::AppState;
use rand::Rng;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            spawn_enemies_system,
            enemy_movement_system,
            turret_targeting_system,
            bullet_movement_system,
            despawn_dead_system,
        ).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct Bullet {
    pub target: Entity,
    pub speed: f32,
    pub damage: f32,
}

fn spawn_enemies_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<f32>,
    building_assets: Res<BuildingAssets>,
    config: Res<WorldConfig>,
) {
    if building_assets.enemy == Handle::default() { return; }

    *timer -= time.delta_secs();
    if *timer <= 0.0 {
        *timer = config.enemy_spawn_rate;

        let mut rng = rand::rng();
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let dist = 800.0;
        let x = angle.cos() * dist;
        let y = angle.sin() * dist;

        commands.spawn((
            Sprite {
                image: building_assets.enemy.clone(),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 2.0),
            Enemy { health: config.enemy_health, speed: 100.0 },
        ));
    }
}

fn enemy_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut enemies: Query<(Entity, &mut Transform, &Enemy)>,
) {
    for (entity, mut transform, enemy) in &mut enemies {
        let current_pos = transform.translation.truncate();
        let dist_to_center = current_pos.length();

        if dist_to_center < 16.0 {
            // Hit the base!
            commands.entity(entity).despawn();
            continue;
        }

        let direction = -current_pos.normalize_or_zero();
        transform.translation += direction.extend(0.0) * enemy.speed * time.delta_secs();
        
        let angle = direction.y.atan2(direction.x);
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

fn turret_targeting_system(
    mut commands: Commands,
    time: Res<Time>,
    mut turrets: Query<(&GlobalTransform, &mut Turret)>,
    enemies: Query<(Entity, &GlobalTransform), With<Enemy>>,
    _building_assets: Res<BuildingAssets>,
) {
    for (turret_transform, mut turret) in &mut turrets {
        turret.timer.tick(time.delta());
        if turret.timer.just_finished() {
            let turret_pos = turret_transform.translation().truncate();
            
            // Find nearest enemy in range
            let nearest = enemies.iter()
                .filter(|(_, et)| et.translation().truncate().distance(turret_pos) < turret.range)
                .min_by(|(_, a), (_, b)| {
                    a.translation().truncate().distance_squared(turret_pos)
                        .partial_cmp(&b.translation().truncate().distance_squared(turret_pos))
                        .unwrap()
                });

            if let Some((target_entity, _)) = nearest {
                // Shoot!
                commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, 1.0, 0.5),
                        custom_size: Some(Vec2::new(8.0, 8.0)),
                        ..default()
                    },
                    Transform::from_translation(turret_pos.extend(2.5)),
                    Bullet {
                        target: target_entity,
                        speed: 400.0,
                        damage: 2.0,
                    },
                ));
            }
        }
    }
}

fn bullet_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &mut Transform, &Bullet)>,
    mut enemies: Query<(&GlobalTransform, &mut Enemy)>,
) {
    for (bullet_entity, mut transform, bullet) in &mut bullets {
        if let Ok((enemy_transform, mut enemy)) = enemies.get_mut(bullet.target) {
            let target_pos = enemy_transform.translation().truncate();
            let current_pos = transform.translation.truncate();
            let direction = (target_pos - current_pos).normalize_or_zero();
            
            let dist = current_pos.distance(target_pos);
            let move_amt = bullet.speed * time.delta_secs();

            if dist < move_amt {
                // Hit!
                enemy.health -= bullet.damage;
                commands.entity(bullet_entity).despawn();
            } else {
                transform.translation += direction.extend(0.0) * move_amt;
            }
        } else {
            // Target gone
            commands.entity(bullet_entity).despawn();
        }
    }
}

fn despawn_dead_system(
    mut commands: Commands,
    enemies: Query<(Entity, &Enemy)>,
) {
    for (entity, enemy) in &enemies {
        if enemy.health <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
