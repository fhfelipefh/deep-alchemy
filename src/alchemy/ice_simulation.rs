use super::{AlchemicalEffectKind, AlchemyTransientEmitter, PotionPuddle, ThermalGeyser};
use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

pub const ICE_GRID_SIZE: usize = 20;
pub const ICE_CELL_SIZE: f32 = 0.56;
pub const ICE_SURFACE_Y: f32 = 0.18;
const ICE_CENTER: Vec2 = Vec2::new(0.0, -8.5);
const ICE_RADIUS: f32 = 5.5;
const SOLID_FREEZE: f32 = 0.72;

#[derive(Resource, Debug, Clone)]
pub struct LakeFreezeSimulation {
    pub freeze: Vec<f32>,
    pub cold: Vec<f32>,
    pub lifetime: Vec<f32>,
    pub tiles: Vec<Option<Entity>>,
}

impl Default for LakeFreezeSimulation {
    fn default() -> Self {
        let count = ICE_GRID_SIZE * ICE_GRID_SIZE;
        Self {
            freeze: vec![0.0; count],
            cold: vec![0.0; count],
            lifetime: vec![0.0; count],
            tiles: vec![None; count],
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct IceSimulationCell {
    pub index: usize,
    pub solid: bool,
}

impl LakeFreezeSimulation {
    pub fn index_at(&self, world: Vec2) -> Option<usize> {
        let local = world - ICE_CENTER;
        let half = ICE_GRID_SIZE as f32 * ICE_CELL_SIZE * 0.5;
        let x = ((local.x + half) / ICE_CELL_SIZE).floor() as isize;
        let z = ((local.y + half) / ICE_CELL_SIZE).floor() as isize;
        if x < 0 || z < 0 || x >= ICE_GRID_SIZE as isize || z >= ICE_GRID_SIZE as isize {
            None
        } else {
            Some(z as usize * ICE_GRID_SIZE + x as usize)
        }
    }

    pub fn cell_position(index: usize) -> Vec2 {
        let x = index % ICE_GRID_SIZE;
        let z = index / ICE_GRID_SIZE;
        let half = ICE_GRID_SIZE as f32 * ICE_CELL_SIZE * 0.5;
        ICE_CENTER + Vec2::new(
            (x as f32 + 0.5) * ICE_CELL_SIZE - half,
            (z as f32 + 0.5) * ICE_CELL_SIZE - half,
        )
    }

    pub fn add_freeze_seed(&mut self, world: Vec2, intensity: f32) {
        for index in 0..self.freeze.len() {
            let pos = Self::cell_position(index);
            let distance = pos.distance(world);
            if distance <= 1.85 {
                let falloff = (1.0 - distance / 1.85).powi(2);
                self.cold[index] = (self.cold[index] + intensity * falloff).min(2.5);
                self.lifetime[index] = 0.0;
            }
        }
    }

    pub fn frozen_volume(&self) -> f32 {
        self.freeze.iter().sum::<f32>() * ICE_CELL_SIZE * ICE_CELL_SIZE
    }
}

fn valid_cell(index: usize) -> bool {
    LakeFreezeSimulation::cell_position(index).distance(ICE_CENTER) <= ICE_RADIUS
}

fn neighbors(index: usize) -> impl Iterator<Item = usize> {
    let x = index % ICE_GRID_SIZE;
    let z = index / ICE_GRID_SIZE;
    [
        x.checked_sub(1).map(|left| z * ICE_GRID_SIZE + left),
        (x + 1 < ICE_GRID_SIZE).then_some(z * ICE_GRID_SIZE + x + 1),
        z.checked_sub(1).map(|up| up * ICE_GRID_SIZE + x),
        (z + 1 < ICE_GRID_SIZE).then_some((z + 1) * ICE_GRID_SIZE + x),
    ]
    .into_iter()
    .flatten()
}

pub fn update_lake_freeze_simulation(
    time: Res<Time>,
    mut commands: Commands,
    mut simulation: ResMut<LakeFreezeSimulation>,
    puddles: Query<(&Transform, &PotionPuddle), Without<IceSimulationCell>>,
    geysers: Query<(&Transform, &ThermalGeyser), Without<IceSimulationCell>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tiles: Query<(&mut Transform, &mut IceSimulationCell, &MeshMaterial3d<StandardMaterial>)>,
) {
    let dt = time.delta_secs().min(0.05);
    if dt <= 0.0 {
        return;
    }
    let old_cold = simulation.cold.clone();
    let old_freeze = simulation.freeze.clone();
    for index in 0..simulation.freeze.len() {
        if !valid_cell(index) {
            continue;
        }
        let position = LakeFreezeSimulation::cell_position(index);
        let neighbor_cold = neighbors(index).map(|other| old_cold[other]).sum::<f32>() * 0.25;
        let neighbor_ice = neighbors(index).map(|other| old_freeze[other]).sum::<f32>() * 0.25;
        let irregularity = ((index as f32 * 19.73).sin() * 0.5 + 0.5) * 0.18;
        let mut heat = 0.0;
        for (transform, puddle) in puddles.iter() {
            if puddle.effects.iter().any(|effect| effect.kind == AlchemicalEffectKind::ThermalHeat) {
                let distance = position.distance(Vec2::new(transform.translation.x, transform.translation.z));
                heat += (1.0 - distance / (puddle.radius + 0.35)).max(0.0) * 1.3;
            }
        }
        for (transform, geyser) in geysers.iter() {
            if geyser.is_erupting {
                let distance = position.distance(Vec2::new(transform.translation.x, transform.translation.z));
                heat += (1.0 - distance / geyser.heat_radius).max(0.0) * 1.6;
            }
        }
        let incoming_cold = (old_cold[index] * 0.97 + neighbor_cold * 0.03 - heat * dt).max(0.0);
        simulation.cold[index] = incoming_cold;
        let growth = (incoming_cold * 0.62 + neighbor_ice * 0.18 - irregularity * 0.08 - heat * 0.62) * dt;
        simulation.freeze[index] = (old_freeze[index] + growth - 0.012 * dt).clamp(0.0, 1.0);
        simulation.lifetime[index] += dt;
    }
    for index in 0..simulation.freeze.len() {
        let freeze = simulation.freeze[index];
        let should_exist = valid_cell(index) && freeze > 0.035;
        let should_solid = freeze >= SOLID_FREEZE;
        if should_exist && simulation.tiles[index].is_none() {
            let position = LakeFreezeSimulation::cell_position(index);
            let thickness = (0.035 + freeze * 0.18).min(0.22);
            let material = materials.add(StandardMaterial {
                base_color: Color::srgba(0.48, 0.82, 0.96, (0.10 + freeze * 0.76).clamp(0.0, 0.88)),
                emissive: LinearRgba::new(0.04, 0.20, 0.32, freeze * 0.18),
                perceptual_roughness: 0.16 + (1.0 - freeze) * 0.45,
                metallic: 0.06,
                reflectance: 0.82,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                double_sided: true,
                depth_bias: 400.0,
                ..default()
            });
            let entity = commands
                .spawn((
                    Name::new("Progressive_Ice_Cell"),
                    IceSimulationCell { index, solid: should_solid },
                    Mesh3d(meshes.add(Cuboid::new(ICE_CELL_SIZE * 0.94, thickness, ICE_CELL_SIZE * 0.94))),
                    MeshMaterial3d(material),
                    Transform::from_xyz(position.x, ICE_SURFACE_Y + thickness * 0.5, position.y),
                ))
                .id();
            if should_solid {
                commands.entity(entity).insert((RigidBody::Static, Collider::cuboid(ICE_CELL_SIZE * 0.94, thickness, ICE_CELL_SIZE * 0.94)));
            }
            simulation.tiles[index] = Some(entity);
        } else if !should_exist {
            if let Some(entity) = simulation.tiles[index].take() {
                commands.entity(entity).despawn();
            }
        }
    }
    for (mut transform, mut cell, material) in tiles.iter_mut() {
        let freeze = simulation.freeze[cell.index];
        let thickness = (0.035 + freeze * 0.18).min(0.22);
        transform.translation.y = ICE_SURFACE_Y + thickness * 0.5;
        transform.scale.y = (thickness / 0.215).max(0.08);
        if let Some(mat) = materials.get_mut(&material.0) {
            mat.base_color = Color::srgba(0.48, 0.82, 0.96, (0.10 + freeze * 0.76).clamp(0.0, 0.88));
            mat.emissive = LinearRgba::new(0.04, 0.20, 0.32, freeze * 0.18);
        }
        let solid = freeze >= SOLID_FREEZE;
        let entity = simulation.tiles[cell.index].expect("ice cell entity must exist");
        if solid != cell.solid {
            cell.solid = solid;
            if solid {
                commands.entity(entity).insert((RigidBody::Static, Collider::cuboid(ICE_CELL_SIZE * 0.94, thickness, ICE_CELL_SIZE * 0.94)));
            } else {
                commands.entity(entity).remove::<(RigidBody, Collider)>();
                commands.spawn((
                    Transform::from_translation(transform.translation),
                    crate::render::particles::ParticleEmitter {
                        spawn_timer: 0.0,
                        spawn_rate: 0.05,
                        particle_type: crate::render::particles::ParticleType::WaterSplash,
                    },
                    AlchemyTransientEmitter { elapsed: 0.0, max_lifetime: 0.75 },
                ));
            }
        } else if solid {
            commands.entity(entity).insert(Collider::cuboid(
                ICE_CELL_SIZE * 0.94,
                thickness,
                ICE_CELL_SIZE * 0.94,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freeze_seed_is_local_and_preserves_unaffected_water() {
        let mut simulation = LakeFreezeSimulation::default();
        simulation.add_freeze_seed(ICE_CENTER, 1.0);
        let energized = simulation.cold.iter().filter(|cold| **cold > 0.0).count();
        assert!(energized > 1);
        assert!(energized < simulation.cold.len() / 2);
        assert_eq!(simulation.frozen_volume(), 0.0);
    }

    #[test]
    fn lattice_neighbors_do_not_wrap_across_rows() {
        let right_edge = ICE_GRID_SIZE - 1;
        assert!(!neighbors(right_edge).any(|index| index == ICE_GRID_SIZE));
        assert!(neighbors(right_edge).any(|index| index == right_edge - 1));
    }
}
