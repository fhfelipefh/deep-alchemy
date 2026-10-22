use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegetationProfile {
    pub id: String,
    pub wind_speed: f32,
    pub wind_gust_frequency: f32,
    pub primary_sway_amplitude: f32,
    pub secondary_sway_amplitude: f32,
    pub player_push_radius: f32,
    pub player_push_strength: f32,
    pub spring_stiffness: f32,
    pub spring_damping: f32,
    pub wind_direction: (f32, f32),
}

impl Default for VegetationProfile {
    fn default() -> Self {
        Self {
            id: "default_grotto_breeze".to_string(),
            wind_speed: 0.65,
            wind_gust_frequency: 0.20,
            primary_sway_amplitude: 0.018,
            secondary_sway_amplitude: 0.008,
            player_push_radius: 0.70,
            player_push_strength: 0.16,
            spring_stiffness: 28.0,
            spring_damping: 9.0,
            wind_direction: (0.866, 0.5),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VegetationDatabase {
    pub clumps: Vec<VegetationProfile>,
}

#[derive(Resource, Debug, Clone)]
pub struct VegetationAnimationConfig {
    pub profile: VegetationProfile,
}

impl Default for VegetationAnimationConfig {
    fn default() -> Self {
        Self {
            profile: VegetationProfile::default(),
        }
    }
}

pub fn load_vegetation_animations_from_disk<P: AsRef<Path>>(path: P) -> VegetationAnimationConfig {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(database) = ron::from_str::<VegetationDatabase>(&content) {
            if let Some(first) = database.clumps.into_iter().next() {
                return VegetationAnimationConfig { profile: first };
            }
        }
    }
    VegetationAnimationConfig::default()
}

#[derive(Component, Debug, Clone)]
pub struct FoliageMeshDeformer {
    pub base_positions: Vec<[f32; 3]>,
    pub min_y: f32,
    pub max_y: f32,
    pub phase_offset: f32,
    pub current_deflection: Vec2,
    pub deflection_velocity: Vec2,
    pub last_applied_deflection: Vec2,
}

impl FoliageMeshDeformer {
    pub fn new(positions: &[[f32; 3]], phase_offset: f32) -> Option<Self> {
        if positions.is_empty() {
            return None;
        }
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        for p in positions {
            if p[1] < min_y {
                min_y = p[1];
            }
            if p[1] > max_y {
                max_y = p[1];
            }
        }
        if max_y - min_y < 0.05 {
            return None;
        }
        Some(Self {
            base_positions: positions.to_vec(),
            min_y,
            max_y,
            phase_offset,
            current_deflection: Vec2::ZERO,
            deflection_velocity: Vec2::ZERO,
            last_applied_deflection: Vec2::new(999.0, 999.0),
        })
    }
}

pub fn update_vegetation_foliage_animation(
    time: Res<Time>,
    config: Res<VegetationAnimationConfig>,
    player_query: Query<&GlobalTransform, (With<crate::player::camera::Player>, Without<FoliageMeshDeformer>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut deformers: Query<(Entity, &Mesh3d, &GlobalTransform, &mut FoliageMeshDeformer)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 || dt > 0.1 {
        return;
    }
    let elapsed = time.elapsed_secs();
    let player_pos_opt = player_query.get_single().map(|gt| gt.translation()).ok();

    let prof = &config.profile;
    let wind_dir = Vec2::new(prof.wind_direction.0, prof.wind_direction.1).normalize_or_zero();
    let mut updated_meshes = std::collections::HashSet::new();

    for (_entity, mesh_handle, global_tf, mut deformer) in deformers.iter_mut() {
        let clump_world = global_tf.translation();
        let clump_p = Vec2::new(clump_world.x, clump_world.z);

        if let Some(player_pos) = player_pos_opt {
            if clump_world.distance(player_pos) > 16.0 {
                continue;
            }
        }

        let gust = (elapsed * prof.wind_gust_frequency + deformer.phase_offset * 1.5).sin() * 0.4 + 0.6;
        let breeze_phase = elapsed * prof.wind_speed + deformer.phase_offset;
        let primary_breeze = breeze_phase.sin() * prof.primary_sway_amplitude * gust;
        let secondary_breeze = (breeze_phase * 1.73).cos() * prof.secondary_sway_amplitude;
        let breeze_offset = wind_dir * (primary_breeze + secondary_breeze);

        let mut push_target = Vec2::ZERO;
        if let Some(player_pos) = player_pos_opt {
            let player_p = Vec2::new(player_pos.x, player_pos.z);
            let to_clump = clump_p - player_p;
            let dist = to_clump.length();
            if dist < prof.player_push_radius && dist > 1e-4 {
                let factor = (1.0 - dist / prof.player_push_radius).clamp(0.0, 1.0);
                let push_dir = to_clump / dist;
                push_target = push_dir * (factor * prof.player_push_strength);
            }
        }

        let target = breeze_offset + push_target;
        let diff = target - deformer.current_deflection;
        let spring_acc = diff * prof.spring_stiffness - deformer.deflection_velocity * prof.spring_damping;
        let next_vel = deformer.deflection_velocity + spring_acc * dt;
        deformer.deflection_velocity = next_vel;
        deformer.current_deflection += next_vel * dt;

        let height = deformer.max_y - deformer.min_y;
        if height <= 0.001 {
            continue;
        }

        let delta_sq = (deformer.current_deflection - deformer.last_applied_deflection).length_squared();
        if delta_sq < 0.00002 {
            continue;
        }

        if !updated_meshes.insert(mesh_handle.0.id()) {
            continue;
        }

        let Some(mesh) = meshes.get_mut(&mesh_handle.0) else {
            continue;
        };

        deformer.last_applied_deflection = deformer.current_deflection;

        if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            let disp = deformer.current_deflection;
            for (curr, base) in positions.iter_mut().zip(&deformer.base_positions) {
                let t = ((base[1] - deformer.min_y) / height).clamp(0.0, 1.0);
                let weight = t * t;
                curr[0] = base[0] + disp.x * weight;
                curr[1] = base[1] - (disp.x.abs() + disp.y.abs()) * 0.08 * weight;
                curr[2] = base[2] + disp.y * weight;
            }
        }
    }
}
