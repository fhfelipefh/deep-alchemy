use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    alchemy::{AlchemicalEffectKind, PotionProjectile},
    dungeon::asset_loader::SceneAssetRegistry,
    player::camera::Player,
    render::particles::{ParticleEmitter, ParticleType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureSpecies {
    Sproutling,
    Emberling,
    Glaceling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureState {
    Patrolling,
    Curious,
    Cautious,
    Dodging,
    Fleeing,
    Frozen,
    Melting,
    Exhausted,
    DancingHappy,
}

#[derive(Component, Debug, Clone)]
pub struct DungeonCreature {
    pub species: CreatureSpecies,
    pub state: CreatureState,
    pub state_timer: f32,
    pub patrol_waypoints: Vec<Vec3>,
    pub current_waypoint_idx: usize,
    pub learned_player_threat: bool,
    pub has_been_hit: bool,
    pub hit_count: u32,
    pub dodge_cooldown: f32,
    pub dodge_velocity: Vec3,
    pub dodge_duration: f32,
    pub hop_phase: f32,
    pub hop_speed: f32,
    pub base_scale: Vec3,
    pub move_speed: f32,
    pub dist_to_exit: f32,
    pub turn_speed: f32,
}

impl DungeonCreature {
    pub fn new(species: CreatureSpecies, waypoints: Vec<Vec3>, dist_to_exit: f32) -> Self {
        let (base_scale, move_speed) = match species {
            CreatureSpecies::Sproutling => (Vec3::splat(1.1), 2.2),
            CreatureSpecies::Emberling => (Vec3::splat(1.0), 2.6),
            CreatureSpecies::Glaceling => (Vec3::splat(1.05), 1.9),
        };

        Self {
            species,
            state: CreatureState::Patrolling,
            state_timer: 0.0,
            patrol_waypoints: waypoints,
            current_waypoint_idx: 0,
            learned_player_threat: false,
            has_been_hit: false,
            hit_count: 0,
            dodge_cooldown: 0.0,
            dodge_velocity: Vec3::ZERO,
            dodge_duration: 0.0,
            hop_phase: 0.0,
            hop_speed: 6.28,
            base_scale,
            move_speed,
            dist_to_exit,
            turn_speed: 6.0,
        }
    }

    pub fn can_dodge(&self) -> bool {
        matches!(
            self.state,
            CreatureState::Patrolling | CreatureState::Curious | CreatureState::Cautious
        )
    }

    pub fn is_incapacitated(&self) -> bool {
        matches!(
            self.state,
            CreatureState::Frozen | CreatureState::Melting | CreatureState::Exhausted
        )
    }
}

pub fn spawn_dungeon_creature_instance(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &SceneAssetRegistry,
    species: CreatureSpecies,
    position: Vec3,
    waypoints: Vec<Vec3>,
    dist_to_exit: f32,
) -> Entity {
    let asset_key = match species {
        CreatureSpecies::Sproutling => "creature_sproutling",
        CreatureSpecies::Emberling => "creature_emberling",
        CreatureSpecies::Glaceling => "creature_glaceling",
    };

    let scene_handle = registry.load_scene(asset_server, asset_key);

    let creature = DungeonCreature::new(species, waypoints, dist_to_exit);
    let base_scale = creature.base_scale;

    commands
        .spawn((
            Name::new(format!("DungeonCreature_{:?}", species)),
            SceneRoot(scene_handle),
            Transform::from_translation(position).with_scale(base_scale),
            RigidBody::Dynamic,
            Collider::sphere(0.45),
            LockedAxes::ROTATION_LOCKED,
            Friction::new(0.6),
            Restitution::new(0.2),
            creature,
        ))
        .id()
}

pub fn update_creature_patrol_and_movement(
    time: Res<Time>,
    mut creatures: Query<(&mut Transform, &mut LinearVelocity, &mut DungeonCreature)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut velocity, mut creature) in creatures.iter_mut() {
        if creature.dodge_cooldown > 0.0 {
            creature.dodge_cooldown = (creature.dodge_cooldown - dt).max(0.0);
        }

        if creature.is_incapacitated() {
            creature.state_timer -= dt;
            velocity.x = 0.0;
            velocity.z = 0.0;

            match creature.state {
                CreatureState::Frozen => {
                    transform.scale = creature.base_scale * Vec3::new(1.05, 1.05, 1.05);
                }
                CreatureState::Melting => {
                    transform.scale = creature.base_scale * Vec3::new(1.4, 0.25, 1.4);
                }
                CreatureState::Exhausted => {
                    transform.scale = creature.base_scale * Vec3::new(1.15, 0.6, 1.15);
                }
                _ => {}
            }

            if creature.state_timer <= 0.0 {
                creature.state = if creature.learned_player_threat {
                    CreatureState::Cautious
                } else {
                    CreatureState::Patrolling
                };
                transform.scale = creature.base_scale;
            }
            continue;
        }

        if creature.state == CreatureState::Dodging {
            creature.dodge_duration -= dt;
            velocity.x = creature.dodge_velocity.x;
            velocity.z = creature.dodge_velocity.z;
            velocity.y = creature.dodge_velocity.y;

            let squash = (creature.dodge_duration * 12.0).sin() * 0.25;
            transform.scale = creature.base_scale * Vec3::new(1.0 + squash, 1.0 - squash, 1.0 + squash);

            if creature.dodge_duration <= 0.0 {
                creature.state = CreatureState::Cautious;
                transform.scale = creature.base_scale;
                velocity.x = 0.0;
                velocity.z = 0.0;
            }
            continue;
        }

        if creature.state == CreatureState::DancingHappy {
            creature.state_timer -= dt;
            transform.rotate_y(dt * 8.0);
            let hop = (time.elapsed_secs() * 10.0).sin().abs() * 0.2;
            transform.scale = creature.base_scale * Vec3::new(1.0 - hop * 0.5, 1.0 + hop, 1.0 - hop * 0.5);
            velocity.x = 0.0;
            velocity.z = 0.0;

            if creature.state_timer <= 0.0 {
                creature.state = CreatureState::Patrolling;
                transform.scale = creature.base_scale;
            }
            continue;
        }

        if creature.state == CreatureState::Fleeing {
            creature.state_timer -= dt;
            let panic_hop = (time.elapsed_secs() * 16.0).sin().abs();
            transform.scale = creature.base_scale * Vec3::new(1.0 - panic_hop * 0.3, 1.0 + panic_hop * 0.4, 1.0 - panic_hop * 0.3);

            let wander_angle = (time.elapsed_secs() * 7.0).sin() * 1.5;
            let forward = transform.forward();
            let flee_dir = Quat::from_rotation_y(wander_angle) * forward;

            velocity.x = flee_dir.x * (creature.move_speed * 1.8);
            velocity.z = flee_dir.z * (creature.move_speed * 1.8);

            if creature.state_timer <= 0.0 {
                creature.state = CreatureState::Cautious;
                transform.scale = creature.base_scale;
            }
            continue;
        }

        if creature.patrol_waypoints.is_empty() {
            continue;
        }

        let target = creature.patrol_waypoints[creature.current_waypoint_idx];
        let diff = target - transform.translation;
        let dist = Vec2::new(diff.x, diff.z).length();

        if dist < 0.6 {
            creature.current_waypoint_idx =
                (creature.current_waypoint_idx + 1) % creature.patrol_waypoints.len();
        } else {
            let dir = Vec3::new(diff.x, 0.0, diff.z).normalize();
            let current_speed = if creature.state == CreatureState::Cautious {
                creature.move_speed * 0.75
            } else {
                creature.move_speed
            };

            velocity.x = dir.x * current_speed;
            velocity.z = dir.z * current_speed;

            let target_rot = Quat::from_rotation_arc(Vec3::Z, dir);
            transform.rotation = transform.rotation.slerp(target_rot, dt * creature.turn_speed);

            creature.hop_phase += dt * creature.hop_speed;
            let hop_offset = creature.hop_phase.sin().abs();
            let stretch = (creature.hop_phase * 2.0).sin() * 0.12;
            transform.scale = creature.base_scale * Vec3::new(1.0 - stretch, 1.0 + stretch + hop_offset * 0.15, 1.0 - stretch);
        }
    }
}

pub fn update_creature_ai_perception_and_aim_dodge(
    time: Res<Time>,
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut creatures: Query<(Entity, &mut Transform, &mut DungeonCreature), Without<Player>>,
) {
    let Ok(player_tf) = player_query.get_single() else {
        return;
    };

    let player_pos = player_tf.translation;
    let player_forward = player_tf.forward();

    for (creature_ent, mut transform, mut creature) in creatures.iter_mut() {
        if creature.is_incapacitated() || creature.state == CreatureState::Dodging {
            continue;
        }

        let creature_pos = transform.translation;
        let to_creature = creature_pos - player_pos;
        let dist = to_creature.length();

        if dist < 16.0 && dist > 1.2 {
            let to_creature_dir = to_creature.normalize();
            let dot = player_forward.dot(to_creature_dir);

            if creature.learned_player_threat {
                if dot > 0.94 && creature.dodge_cooldown <= 0.0 && creature.can_dodge() {
                    let right = player_forward.cross(Vec3::Y).normalize();
                    let side = if ((creature_ent.index() + (time.elapsed_secs() * 10.0) as u32) % 2) == 0 {
                        1.0
                    } else {
                        -1.0
                    };

                    creature.dodge_velocity = right * (side * 7.2) + Vec3::new(0.0, 3.6, 0.0);
                    creature.dodge_duration = 0.35;
                    creature.dodge_cooldown = 1.8;
                    creature.state = CreatureState::Dodging;

                    commands.spawn((
                        Transform::from_translation(creature_pos + Vec3::new(0.0, 0.2, 0.0)),
                        ParticleEmitter {
                            spawn_timer: 0.0,
                            spawn_rate: 0.02,
                            particle_type: ParticleType::GlowSpore,
                        },
                        crate::alchemy::AlchemyTransientEmitter {
                            elapsed: 0.0,
                            max_lifetime: 0.6,
                        },
                    ));
                    continue;
                }

                if dist < 4.2 && creature.state != CreatureState::Fleeing {
                    creature.state = CreatureState::Cautious;
                    let away_dir = (creature_pos - player_pos).normalize();
                    transform.translation += away_dir * (time.delta_secs() * 1.5);
                }
            } else if dist < 5.0 && creature.state == CreatureState::Patrolling {
                creature.state = CreatureState::Curious;
                let look_target = Vec3::new(player_pos.x, creature_pos.y, player_pos.z);
                let look_dir = (look_target - creature_pos).normalize();
                if look_dir.length_squared() > 0.01 {
                    let target_rot = Quat::from_rotation_arc(Vec3::Z, look_dir);
                    transform.rotation = transform.rotation.slerp(target_rot, time.delta_secs() * 4.0);
                }
            }
        }
    }
}

pub fn update_creature_potion_interactions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &PotionProjectile)>,
    mut creatures: Query<(Entity, &Transform, &mut DungeonCreature)>,
    mut sound_events: Option<ResMut<Events<crate::audio::SoundEffectEvent>>>,
) {
    for (_proj_ent, proj_tf, proj) in projectiles.iter() {
        let proj_pos = proj_tf.translation;
        let is_fire = proj.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat) || proj.slot == 2;
        let is_frost = proj.effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost) || proj.slot == 1;

        for (creature_ent, creature_tf, mut creature) in creatures.iter_mut() {
            let creature_pos = creature_tf.translation;
            let dist = (proj_pos - creature_pos).length();

            if dist < 2.5 {
                creature.has_been_hit = true;
                creature.learned_player_threat = true;
                creature.hit_count += 1;

                if let Some(ref mut events) = sound_events {
                    events.send(crate::audio::SoundEffectEvent::new(crate::audio::SoundEffectKind::CreatureChirp));
                }

                match creature.species {
                    CreatureSpecies::Sproutling => {
                        if is_fire {
                            creature.state = CreatureState::Fleeing;
                            creature.state_timer = 5.0;
                            commands.spawn((
                                Transform::from_translation(creature_pos + Vec3::new(0.0, 0.4, 0.0)),
                                ParticleEmitter {
                                    spawn_timer: 0.0,
                                    spawn_rate: 0.03,
                                    particle_type: ParticleType::FireFlame,
                                },
                                crate::alchemy::AlchemyTransientEmitter {
                                    elapsed: 0.0,
                                    max_lifetime: 1.5,
                                },
                            ));
                        } else if is_frost {
                            creature.state = CreatureState::Frozen;
                            creature.state_timer = 6.0;
                            commands.spawn((
                                Transform::from_translation(creature_pos),
                                ParticleEmitter {
                                    spawn_timer: 0.0,
                                    spawn_rate: 0.04,
                                    particle_type: ParticleType::GlowSpore,
                                },
                                crate::alchemy::AlchemyTransientEmitter {
                                    elapsed: 0.0,
                                    max_lifetime: 1.0,
                                },
                            ));
                        } else {
                            creature.state = CreatureState::DancingHappy;
                            creature.state_timer = 6.0;
                        }
                    }
                    CreatureSpecies::Emberling => {
                        if is_frost {
                            creature.state = CreatureState::Exhausted;
                            creature.state_timer = 7.0;
                            commands.spawn((
                                Transform::from_translation(creature_pos + Vec3::new(0.0, 0.4, 0.0)),
                                ParticleEmitter {
                                    spawn_timer: 0.0,
                                    spawn_rate: 0.02,
                                    particle_type: ParticleType::Steam,
                                },
                                crate::alchemy::AlchemyTransientEmitter {
                                    elapsed: 0.0,
                                    max_lifetime: 1.8,
                                },
                            ));
                        } else if is_fire {
                            creature.state = CreatureState::Patrolling;
                            creature.move_speed = 3.6;
                            creature.dodge_cooldown = 0.0;
                        } else {
                            creature.state = CreatureState::Cautious;
                            creature.state_timer = 3.0;
                        }
                    }
                    CreatureSpecies::Glaceling => {
                        if is_fire {
                            creature.state = CreatureState::Melting;
                            creature.state_timer = 6.0;
                            commands.spawn((
                                Transform::from_translation(creature_pos + Vec3::new(0.0, 0.2, 0.0)),
                                ParticleEmitter {
                                    spawn_timer: 0.0,
                                    spawn_rate: 0.03,
                                    particle_type: ParticleType::WaterSplash,
                                },
                                crate::alchemy::AlchemyTransientEmitter {
                                    elapsed: 0.0,
                                    max_lifetime: 1.2,
                                },
                            ));
                        } else if is_frost {
                            creature.state = CreatureState::Patrolling;
                            creature.dodge_cooldown = 0.0;
                        } else {
                            creature.state = CreatureState::Cautious;
                            creature.state_timer = 3.0;
                        }
                    }
                }

                alert_nearby_creatures(creature_ent, creature_pos, &mut creatures);
                break;
            }
        }
    }
}

fn alert_nearby_creatures(
    source_ent: Entity,
    source_pos: Vec3,
    creatures: &mut Query<(Entity, &Transform, &mut DungeonCreature)>,
) {
    for (other_ent, other_tf, mut other_creature) in creatures.iter_mut() {
        if other_ent != source_ent {
            let dist = (other_tf.translation - source_pos).length();
            if dist < 12.0 {
                other_creature.learned_player_threat = true;
                if other_creature.state == CreatureState::Curious || other_creature.state == CreatureState::Patrolling {
                    other_creature.state = CreatureState::Cautious;
                }
            }
        }
    }
}

pub struct DungeonCreaturesPlugin;

impl Plugin for DungeonCreaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_creature_patrol_and_movement,
                update_creature_ai_perception_and_aim_dodge.after(update_creature_patrol_and_movement),
                update_creature_potion_interactions.after(update_creature_ai_perception_and_aim_dodge),
            ),
        );
    }
}
