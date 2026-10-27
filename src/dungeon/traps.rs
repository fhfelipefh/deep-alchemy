use avian3d::prelude::*;
use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    render::render_asset::RenderAssetUsages,
};

use crate::{
    alchemy::{AlchemicalEffectKind, PotionProjectile},
    dungeon::creatures::{CreatureState, DungeonCreature},
    player::camera::{CameraShake, Health, Player},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallNestKind {
    Snakes,
    Scorpions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallNestState {
    Intact,
    Neutralized,
    Triggered,
}

#[derive(Component, Debug, Clone)]
pub struct CrackedWallNest {
    pub kind: WallNestKind,
    pub state: WallNestState,
    pub trigger_radius: f32,
    pub damage: f32,
    pub wall_normal: Vec3,
}

impl CrackedWallNest {
    pub fn new(kind: WallNestKind, wall_normal: Vec3) -> Self {
        let damage = match kind {
            WallNestKind::Snakes => 22.0,
            WallNestKind::Scorpions => 28.0,
        };
        Self {
            kind,
            state: WallNestState::Intact,
            trigger_radius: 2.8,
            damage,
            wall_normal,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct EmergentCreature {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapMaterial {
    Wood,
    Metal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapKind {
    SpinningWheel,
    CrushingMechanism,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrapState {
    Active,
    Frozen { timer: f32, max_time: f32 },
    Glued { timer: f32, max_time: f32 },
    Burned,
}

#[derive(Component, Debug, Clone)]
pub struct DungeonTrap {
    pub material: TrapMaterial,
    pub kind: TrapKind,
    pub state: TrapState,
    pub base_position: Vec3,
    pub rotation_axis: Vec3,
    pub rotation_angle: f32,
    pub rotation_speed: f32,
    pub damage: f32,
    pub hazard_radius: f32,
    pub hit_cooldown: f32,
}

impl DungeonTrap {
    pub fn new(material: TrapMaterial, kind: TrapKind, base_pos: Vec3, axis: Vec3) -> Self {
        let (speed, damage, hazard_radius) = match (material, kind) {
            (TrapMaterial::Wood, TrapKind::SpinningWheel) => (3.5, 30.0, 1.6),
            (TrapMaterial::Wood, TrapKind::CrushingMechanism) => (2.5, 35.0, 1.8),
            (TrapMaterial::Metal, TrapKind::SpinningWheel) => (5.0, 40.0, 1.7),
            (TrapMaterial::Metal, TrapKind::CrushingMechanism) => (4.0, 45.0, 1.9),
        };

        Self {
            material,
            kind,
            state: TrapState::Active,
            base_position: base_pos,
            rotation_axis: axis.normalize_or_zero(),
            rotation_angle: 0.0,
            rotation_speed: speed,
            damage,
            hazard_radius,
            hit_cooldown: 0.0,
        }
    }

    pub fn is_operational(&self) -> bool {
        self.state == TrapState::Active
    }

    pub fn can_pass(&self) -> bool {
        matches!(
            self.state,
            TrapState::Frozen { .. } | TrapState::Glued { .. } | TrapState::Burned
        )
    }

    pub fn apply_alchemical_effect(
        &mut self,
        is_fire: bool,
        is_frost: bool,
        is_glue: bool,
    ) -> bool {
        if is_fire {
            match self.material {
                TrapMaterial::Wood => {
                    self.state = TrapState::Burned;
                    return true;
                }
                TrapMaterial::Metal => {
                    return false;
                }
            }
        }
        if is_frost {
            self.state = TrapState::Frozen {
                timer: 9.0,
                max_time: 9.0,
            };
            return true;
        }
        if is_glue {
            self.state = TrapState::Glued {
                timer: 9.0,
                max_time: 9.0,
            };
            return true;
        }
        false
    }
}

pub fn create_cracked_wall_fissure_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let hd = 0.08;

    let crack_segments = [
        ([-0.05, -0.9, hd + 0.01], [0.12, -0.4, hd + 0.01], [-0.08, 0.2, hd + 0.01]),
        ([-0.08, 0.2, hd + 0.01], [0.15, 0.6, hd + 0.01], [-0.02, 0.95, hd + 0.01]),
        ([-0.25, -0.2, hd + 0.01], [-0.08, 0.2, hd + 0.01], [0.22, 0.35, hd + 0.01]),
    ];

    for seg in &crack_segments {
        let base_idx = positions.len() as u32;
        let p0 = seg.0;
        let p1 = seg.1;
        let p2 = seg.2;

        let w = 0.035;
        positions.push([p0[0] - w, p0[1], p0[2]]);
        positions.push([p0[0] + w, p0[1], p0[2]]);
        positions.push([p1[0] + w, p1[1], p1[2]]);
        positions.push([p1[0] - w, p1[1], p1[2]]);

        positions.push([p1[0] - w, p1[1], p1[2]]);
        positions.push([p1[0] + w, p1[1], p1[2]]);
        positions.push([p2[0] + w, p2[1], p2[2]]);
        positions.push([p2[0] - w, p2[1], p2[2]]);

        for _ in 0..8 {
            normals.push([0.0, 0.0, 1.0]);
            uvs.push([0.5, 0.5]);
        }

        indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
            base_idx + 4,
            base_idx + 5,
            base_idx + 6,
            base_idx + 4,
            base_idx + 6,
            base_idx + 7,
        ]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_wooden_trap_wheel_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let hub_radius = 0.35;
    let rim_radius = 1.35;
    let thickness = 0.16;
    let spoke_count = 6;

    let center_front = [0.0, 0.0, thickness * 0.5];
    let center_back = [0.0, 0.0, -thickness * 0.5];

    positions.push(center_front);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([0.5, 0.5]);

    positions.push(center_back);
    normals.push([0.0, 0.0, -1.0]);
    uvs.push([0.5, 0.5]);

    for s in 0..spoke_count {
        let angle = (s as f32 / spoke_count as f32) * std::f32::consts::TAU;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let hub_x = cos_a * hub_radius;
        let hub_y = sin_a * hub_radius;

        let rim_x = cos_a * rim_radius;
        let rim_y = sin_a * rim_radius;

        let spike_angle = angle + 0.15;
        let spike_x = spike_angle.cos() * (rim_radius + 0.35);
        let spike_y = spike_angle.sin() * (rim_radius + 0.35);

        let base_idx = positions.len() as u32;

        positions.push([hub_x, hub_y, thickness * 0.5]);
        positions.push([rim_x, rim_y, thickness * 0.5]);
        positions.push([spike_x, spike_y, 0.0]);
        positions.push([hub_x, hub_y, -thickness * 0.5]);
        positions.push([rim_x, rim_y, -thickness * 0.5]);

        normals.push([0.0, 0.0, 1.0]);
        normals.push([0.0, 0.0, 1.0]);
        normals.push([cos_a, sin_a, 0.0]);
        normals.push([0.0, 0.0, -1.0]);
        normals.push([0.0, 0.0, -1.0]);

        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([0.5, 1.0]);
        uvs.push([0.0, 1.0]);
        uvs.push([1.0, 1.0]);

        indices.extend_from_slice(&[
            0,
            base_idx,
            base_idx + 1,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            1,
            base_idx + 4,
            base_idx + 3,
            base_idx + 4,
            base_idx + 2,
            base_idx + 3,
        ]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_metal_blade_trap_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let arm_len = 1.45;
    let arm_width = 0.22;
    let blade_len = 0.75;
    let th = 0.12;

    positions.push([-arm_width, -arm_len, th * 0.5]);
    positions.push([arm_width, -arm_len, th * 0.5]);
    positions.push([arm_width, arm_len, th * 0.5]);
    positions.push([-arm_width, arm_len, th * 0.5]);

    positions.push([-arm_width, -arm_len, -th * 0.5]);
    positions.push([arm_width, -arm_len, -th * 0.5]);
    positions.push([arm_width, arm_len, -th * 0.5]);
    positions.push([-arm_width, arm_len, -th * 0.5]);

    for _ in 0..4 {
        normals.push([0.0, 0.0, 1.0]);
    }
    for _ in 0..4 {
        normals.push([0.0, 0.0, -1.0]);
    }
    for _ in 0..2 {
        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([1.0, 1.0]);
        uvs.push([0.0, 1.0]);
    }

    indices.extend_from_slice(&[
        0, 1, 2, 0, 2, 3,
        4, 6, 5, 4, 7, 6,
    ]);

    let blade_offsets = [arm_len, -arm_len];
    for &offset_y in &blade_offsets {
        let b_idx = positions.len() as u32;
        let sign = if offset_y > 0.0 { 1.0 } else { -1.0 };
        let tip_y = offset_y + sign * blade_len;

        positions.push([-arm_width * 1.8, offset_y, 0.0]);
        positions.push([arm_width * 1.8, offset_y, 0.0]);
        positions.push([0.0, tip_y, 0.0]);

        normals.push([0.0, 0.0, 1.0]);
        normals.push([0.0, 0.0, 1.0]);
        normals.push([0.0, sign, 0.0]);

        uvs.push([0.0, 0.0]);
        uvs.push([1.0, 0.0]);
        uvs.push([0.5, 1.0]);

        indices.extend_from_slice(&[b_idx, b_idx + 1, b_idx + 2, b_idx, b_idx + 2, b_idx + 1]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_tiny_snake_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let segments = 8;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let z = (t - 0.5) * 0.9;
        let wave = (t * std::f32::consts::TAU).sin() * 0.14;
        let r = (1.0 - (t - 0.5).abs() * 1.5).max(0.04) * 0.06;

        let base_idx = positions.len() as u32;
        positions.push([wave - r, 0.0, z]);
        positions.push([wave + r, 0.0, z]);
        positions.push([wave, r * 1.4, z]);
        positions.push([wave, -r * 0.8, z]);

        normals.push([-1.0, 0.0, 0.0]);
        normals.push([1.0, 0.0, 0.0]);
        normals.push([0.0, 1.0, 0.0]);
        normals.push([0.0, -1.0, 0.0]);

        uvs.push([0.0, t]);
        uvs.push([1.0, t]);
        uvs.push([0.5, t]);
        uvs.push([0.5, t]);

        if i > 0 {
            let prev = base_idx - 4;
            indices.extend_from_slice(&[
                prev, prev + 1, base_idx + 1,
                prev, base_idx + 1, base_idx,
                prev + 2, prev, base_idx,
                prev + 2, base_idx, base_idx + 2,
            ]);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_tiny_scorpion_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let hw = 0.08;
    let hl = 0.12;
    let hh = 0.04;

    positions.push([-hw, -hh, -hl]);
    positions.push([hw, -hh, -hl]);
    positions.push([hw, hh, -hl]);
    positions.push([-hw, hh, -hl]);
    positions.push([-hw, -hh, hl]);
    positions.push([hw, -hh, hl]);
    positions.push([hw, hh, hl]);
    positions.push([-hw, hh, hl]);

    for _ in 0..4 {
        normals.push([0.0, 0.0, -1.0]);
    }
    for _ in 0..4 {
        normals.push([0.0, 0.0, 1.0]);
    }
    for _ in 0..8 {
        uvs.push([0.5, 0.5]);
    }

    indices.extend_from_slice(&[
        0, 1, 2, 0, 2, 3,
        4, 6, 5, 4, 7, 6,
        0, 4, 5, 0, 5, 1,
        2, 6, 7, 2, 7, 3,
    ]);

    let tail_idx = positions.len() as u32;
    positions.push([0.0, hh * 1.5, hl]);
    positions.push([0.0, hh * 4.0, hl + 0.12]);
    positions.push([0.0, hh * 5.0, hl + 0.06]);

    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 0.0, -1.0]);

    uvs.push([0.5, 0.0]);
    uvs.push([0.5, 0.5]);
    uvs.push([0.5, 1.0]);

    indices.extend_from_slice(&[
        tail_idx, tail_idx + 1, tail_idx + 2,
        tail_idx, tail_idx + 2, tail_idx + 1,
    ]);

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn update_cracked_wall_nests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut nests: Query<(Entity, &Transform, &mut CrackedWallNest), Without<Player>>,
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    mut shake_opt: Option<ResMut<CameraShake>>,
    mut sound_events: Option<ResMut<Events<crate::audio::SoundEffectEvent>>>,
) {
    let Ok((player_tf, mut health)) = player_query.get_single_mut() else {
        return;
    };

    let player_pos = player_tf.translation;

    for (_nest_ent, transform, mut nest) in nests.iter_mut() {
        if nest.state != WallNestState::Intact {
            continue;
        }

        let dist = transform.translation.distance(player_pos);
        if dist <= nest.trigger_radius {
            nest.state = WallNestState::Triggered;

            health.current = (health.current - nest.damage).max(0.0);
            if let Some(ref mut shake) = shake_opt {
                shake.trauma = (shake.trauma + 0.45).min(1.0);
            }

            if let Some(ref mut events) = sound_events {
                events.send(crate::audio::SoundEffectEvent::new(
                    crate::audio::SoundEffectKind::CreatureChirp,
                ));
            }

            let creature_mesh = match nest.kind {
                WallNestKind::Snakes => meshes.add(create_tiny_snake_mesh()),
                WallNestKind::Scorpions => meshes.add(create_tiny_scorpion_mesh()),
            };
            let creature_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.18, 0.12, 0.08),
                perceptual_roughness: 0.8,
                metallic: 0.2,
                ..default()
            });

            for i in 0..4 {
                let angle_offset = (i as f32 - 1.5) * 0.35;
                let outward = Quat::from_rotation_y(angle_offset) * nest.wall_normal;
                let lunge_vel = (outward * 4.0 + Vec3::new(0.0, 1.2, 0.0)).normalize() * 5.5;

                commands.spawn((
                    Mesh3d(creature_mesh.clone()),
                    MeshMaterial3d(creature_mat.clone()),
                    Transform::from_translation(transform.translation + nest.wall_normal * 0.25)
                        .with_scale(Vec3::splat(1.15)),
                    EmergentCreature {
                        velocity: lunge_vel,
                        lifetime: 0.0,
                        max_lifetime: 1.1,
                    },
                ));
            }
        }
    }
}

pub fn update_emergent_creatures(
    mut commands: Commands,
    time: Res<Time>,
    mut creatures: Query<(Entity, &mut Transform, &mut EmergentCreature)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut creature) in creatures.iter_mut() {
        creature.lifetime += dt;
        if creature.lifetime >= creature.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation += creature.velocity * dt;
        creature.velocity.y -= 9.8 * dt;

        let remain = 1.0 - (creature.lifetime / creature.max_lifetime);
        transform.scale = Vec3::splat(remain.max(0.01));
    }
}

pub fn update_dungeon_traps(
    time: Res<Time>,
    mut traps: Query<
        (&mut Transform, &mut DungeonTrap, Option<&mut MeshMaterial3d<StandardMaterial>>),
        Without<Player>,
    >,
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    mut shake_opt: Option<ResMut<CameraShake>>,
    mut sound_events: Option<ResMut<Events<crate::audio::SoundEffectEvent>>>,
) {
    let dt = time.delta_secs();
    let mut player_data = player_query.get_single_mut().ok();

    for (mut transform, mut trap, _mat_opt) in traps.iter_mut() {
        if trap.hit_cooldown > 0.0 {
            trap.hit_cooldown -= dt;
        }

        match trap.state {
            TrapState::Active => {
                trap.rotation_angle += trap.rotation_speed * dt;
                transform.rotation = Quat::from_axis_angle(trap.rotation_axis, trap.rotation_angle);

                if let Some((player_tf, ref mut health)) = player_data.as_mut() {
                    let dist = transform.translation.distance(player_tf.translation);
                    if dist <= trap.hazard_radius && trap.hit_cooldown <= 0.0 {
                        health.current = (health.current - trap.damage).max(0.0);
                        trap.hit_cooldown = 1.0;

                        if let Some(ref mut shake) = shake_opt {
                            shake.trauma = (shake.trauma + 0.50).min(1.0);
                        }

                        if let Some(ref mut events) = sound_events {
                            events.send(crate::audio::SoundEffectEvent::new(
                                crate::audio::SoundEffectKind::PotionShatter,
                            ));
                        }
                    }
                }
            }
            TrapState::Frozen { mut timer, max_time } => {
                timer -= dt;
                if timer <= 0.0 {
                    trap.state = TrapState::Active;
                    if let Some(ref mut events) = sound_events {
                        events.send(crate::audio::SoundEffectEvent::new(
                            crate::audio::SoundEffectKind::IceFreeze,
                        ));
                    }
                } else {
                    trap.state = TrapState::Frozen { timer, max_time };
                }
            }
            TrapState::Glued { mut timer, max_time } => {
                timer -= dt;
                if timer <= 0.0 {
                    trap.state = TrapState::Active;
                    if let Some(ref mut events) = sound_events {
                        events.send(crate::audio::SoundEffectEvent::new(
                            crate::audio::SoundEffectKind::PotionShatter,
                        ));
                    }
                } else {
                    trap.state = TrapState::Glued { timer, max_time };
                }
            }
            TrapState::Burned => {
                transform.scale = Vec3::ZERO;
            }
        }
    }
}

pub fn update_creatures_trap_tactics(
    mut commands: Commands,
    traps: Query<(&Transform, &DungeonTrap), Without<DungeonCreature>>,
    mut creatures: Query<(Entity, &Transform, &mut LinearVelocity, &mut DungeonCreature), Without<DungeonTrap>>,
    mut sound_events: Option<ResMut<Events<crate::audio::SoundEffectEvent>>>,
) {
    for (creature_ent, c_tf, mut c_vel, mut creature) in creatures.iter_mut() {
        if creature.state == CreatureState::Melting || creature.state == CreatureState::Exhausted {
            continue;
        }

        let c_pos = c_tf.translation;

        for (t_tf, trap) in traps.iter() {
            let dist = t_tf.translation.distance(c_pos);

            if trap.is_operational() {
                if dist <= trap.hazard_radius * 0.95 {
                    creature.state = CreatureState::Melting;
                    creature.state_timer = 1.0;
                    c_vel.x = 0.0;
                    c_vel.z = 0.0;

                    if let Some(ref mut events) = sound_events {
                        events.send(crate::audio::SoundEffectEvent::new(
                            crate::audio::SoundEffectKind::PotionShatter,
                        ));
                    }

                    commands.entity(creature_ent).despawn();
                    break;
                } else if dist <= trap.hazard_radius + 1.8 {
                    let away = (c_pos - t_tf.translation).normalize_or_zero();
                    c_vel.x = away.x * creature.move_speed;
                    c_vel.z = away.z * creature.move_speed;
                    creature.state = CreatureState::Cautious;
                }
            }
        }
    }
}

pub fn update_potion_traps_and_nests_interactions(
    projectiles: Query<(&Transform, &PotionProjectile)>,
    mut nests: Query<(Entity, &Transform, &mut CrackedWallNest)>,
    mut traps: Query<(Entity, &Transform, &mut DungeonTrap)>,
    mut sound_events: Option<ResMut<Events<crate::audio::SoundEffectEvent>>>,
) {
    for (proj_tf, proj) in projectiles.iter() {
        let proj_pos = proj_tf.translation;
        let is_fire = proj.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat) || proj.slot == 2;
        let is_frost = proj.effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost) || proj.slot == 4;
        let is_glue = proj.effects.iter().any(|e| e.kind == AlchemicalEffectKind::Swiftness) || proj.slot == 1;

        for (_nest_ent, nest_tf, mut nest) in nests.iter_mut() {
            if nest.state == WallNestState::Intact {
                let dist = proj_pos.distance(nest_tf.translation);
                if dist < 2.8 {
                    nest.state = WallNestState::Neutralized;
                    if let Some(ref mut events) = sound_events {
                        let sfx = if is_fire {
                            crate::audio::SoundEffectKind::TorchIgnite
                        } else if is_frost {
                            crate::audio::SoundEffectKind::IceFreeze
                        } else {
                            crate::audio::SoundEffectKind::PotionShatter
                        };
                        events.send(crate::audio::SoundEffectEvent::new(sfx));
                    }
                }
            }
        }

        for (_trap_ent, trap_tf, mut trap) in traps.iter_mut() {
            let dist = proj_pos.distance(trap_tf.translation);
            if dist < 2.8 {
                let affected = trap.apply_alchemical_effect(is_fire, is_frost, is_glue);
                if affected {
                    if let Some(ref mut events) = sound_events {
                        let sfx = match trap.state {
                            TrapState::Burned => crate::audio::SoundEffectKind::TorchIgnite,
                            TrapState::Frozen { .. } => crate::audio::SoundEffectKind::IceFreeze,
                            _ => crate::audio::SoundEffectKind::PotionShatter,
                        };
                        events.send(crate::audio::SoundEffectEvent::new(sfx));
                    }
                }
            }
        }
    }
}

pub fn spawn_dungeon_puzzles_and_traps(
    _commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<StandardMaterial>,
    _dungeon: &crate::dungeon::generator::ProceduralDungeon,
) {
}
