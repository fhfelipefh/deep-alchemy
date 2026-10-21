use super::ActiveEffect;
use crate::render::surface_fluid_material::SurfaceFluidMaterial;
use avian3d::prelude::*;
use bevy::prelude::*;

pub const SURFACE_FLUID_GRID: usize = 16;
pub const SURFACE_FLUID_CELLS: usize = SURFACE_FLUID_GRID * SURFACE_FLUID_GRID;

#[derive(Debug, Clone, Copy)]
pub struct PotionFluidProfile {
    pub viscosity: f32,
    pub density: f32,
    pub surface_tension: f32,
    pub adhesion: f32,
    pub transparency: f32,
    pub emissive: f32,
    pub initial_volume: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct PotionFluidProfiles(pub [PotionFluidProfile; 5]);

impl Default for PotionFluidProfiles {
    fn default() -> Self {
        Self([
            PotionFluidProfile { viscosity: 0.72, density: 1.14, surface_tension: 0.68, adhesion: 0.82, transparency: 0.72, emissive: 0.58, initial_volume: 1.00 },
            PotionFluidProfile { viscosity: 0.48, density: 0.94, surface_tension: 0.43, adhesion: 0.55, transparency: 0.55, emissive: 0.84, initial_volume: 1.06 },
            PotionFluidProfile { viscosity: 0.88, density: 1.26, surface_tension: 0.80, adhesion: 0.90, transparency: 0.66, emissive: 0.46, initial_volume: 0.94 },
            PotionFluidProfile { viscosity: 0.64, density: 1.07, surface_tension: 0.63, adhesion: 0.76, transparency: 0.78, emissive: 0.70, initial_volume: 1.02 },
            PotionFluidProfile { viscosity: 0.79, density: 1.18, surface_tension: 0.76, adhesion: 0.88, transparency: 0.84, emissive: 0.65, initial_volume: 0.98 },
        ])
    }
}

impl PotionFluidProfiles {
    pub fn for_slot(&self, slot: usize) -> PotionFluidProfile {
        self.0[slot.saturating_sub(1).min(self.0.len() - 1)]
    }
}

#[derive(Component, Debug, Clone)]
pub struct SurfaceFluidPatch {
    pub profile: PotionFluidProfile,
    pub color: Color,
    pub effects: Vec<ActiveEffect>,
    pub host: Option<Entity>,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub thickness: [f32; SURFACE_FLUID_CELLS],
    pub flow: Vec2,
    pub volume: f32,
    pub age: f32,
    pub max_age: f32,
    pub radius: f32,
    pub surface_adhesion: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SurfaceFluidReceiver {
    pub adhesion: f32,
    pub roughness: f32,
    pub retention: f32,
}

impl Default for SurfaceFluidReceiver {
    fn default() -> Self {
        Self { adhesion: 1.0, roughness: 0.5, retention: 1.0 }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SurfaceFluidDroplet {
    pub profile: PotionFluidProfile,
    pub color: Color,
    pub effects: Vec<ActiveEffect>,
    pub volume: f32,
    pub age: f32,
}

pub fn surface_basis(normal: Vec3) -> (Vec3, Vec3) {
    let axis = if normal.y.abs() < 0.94 { Vec3::Y } else { Vec3::X };
    let tangent = normal.cross(axis).normalize_or_zero();
    (tangent, normal.cross(tangent).normalize_or_zero())
}

pub fn tangential_gravity(normal: Vec3) -> Vec3 {
    let gravity = Vec3::new(0.0, -9.81, 0.0);
    gravity - normal * gravity.dot(normal)
}

pub fn fluid_volume(cells: &[f32; SURFACE_FLUID_CELLS]) -> f32 {
    cells.iter().sum()
}

pub fn simulate_surface_flow(patch: &mut SurfaceFluidPatch, dt: f32) -> f32 {
    let dt = dt.clamp(0.0, 1.0 / 20.0);
    let slope = tangential_gravity(patch.normal);
    let downhill = Vec2::new(slope.dot(patch.tangent), slope.dot(patch.bitangent));
    let slope_strength = downhill.length();
    let adhesion = (patch.profile.adhesion * patch.surface_adhesion).clamp(0.05, 1.5);
    let viscous_rate = (0.18 + slope_strength * 0.085) / (0.25 + patch.profile.viscosity * 1.75 + adhesion * 0.20);
    let cohesion = 0.012 + patch.profile.surface_tension * 0.035 + adhesion * 0.006;
    let mut next = patch.thickness;
    let mut released = 0.0;

    for y in 0..SURFACE_FLUID_GRID {
        for x in 0..SURFACE_FLUID_GRID {
            let index = y * SURFACE_FLUID_GRID + x;
            let amount = patch.thickness[index];
            if amount <= 0.000_001 {
                continue;
            }
            let noise = ((x as f32 * 12.9898 + y as f32 * 78.233).sin() * 43_758.547).fract().abs();
            let direction = downhill + Vec2::new(noise - 0.5, 0.5 - noise) * (1.0 - adhesion.min(1.0)) * 0.22;
            let outflow = (amount * (viscous_rate + cohesion * amount.sqrt()) * dt).min(amount * 0.42);
            if outflow <= 0.0 {
                continue;
            }
            let target_x = x as isize + direction.x.signum() as isize;
            let target_y = y as isize + direction.y.signum() as isize;
            next[index] -= outflow;
            if target_x >= 0 && target_x < SURFACE_FLUID_GRID as isize && target_y >= 0 && target_y < SURFACE_FLUID_GRID as isize {
                let target = target_y as usize * SURFACE_FLUID_GRID + target_x as usize;
                next[target] += outflow;
            } else {
                released += outflow;
            }
        }
    }

    let diffusion = (0.002 + (1.0 - slope_strength.min(1.0)) * 0.010) * (1.0 - patch.profile.surface_tension * 0.55);
    for y in 1..SURFACE_FLUID_GRID - 1 {
        for x in 1..SURFACE_FLUID_GRID - 1 {
            let i = y * SURFACE_FLUID_GRID + x;
            let neighbor_mean = (patch.thickness[i - 1] + patch.thickness[i + 1] + patch.thickness[i - SURFACE_FLUID_GRID] + patch.thickness[i + SURFACE_FLUID_GRID]) * 0.25;
            next[i] = (next[i] + (neighbor_mean - patch.thickness[i]) * diffusion).max(0.0);
        }
    }

    let retained_volume = (fluid_volume(&patch.thickness) - released).max(0.0);
    let next_volume = fluid_volume(&next);
    if next_volume > 0.000_001 {
        let scale = retained_volume / next_volume;
        for cell in &mut next {
            *cell *= scale;
        }
    }
    patch.thickness = next;
    patch.volume = fluid_volume(&patch.thickness);
    patch.flow = patch.flow.lerp(downhill * (0.05 + slope_strength * 0.018), (dt * 8.0).min(1.0));
    released
}

pub fn spawn_surface_fluid_patch(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<SurfaceFluidMaterial>,
    position: Vec3,
    normal: Vec3,
    host: Option<Entity>,
    profile: PotionFluidProfile,
    color: Color,
    effects: Vec<ActiveEffect>,
    volume: f32,
) {
    let normal = normal.normalize_or_zero();
    let (tangent, bitangent) = surface_basis(normal);
    let mut thickness = [0.0; SURFACE_FLUID_CELLS];
    for y in 0..SURFACE_FLUID_GRID {
        for x in 0..SURFACE_FLUID_GRID {
            let dx = x as f32 / (SURFACE_FLUID_GRID - 1) as f32 * 2.0 - 1.0;
            let dy = y as f32 / (SURFACE_FLUID_GRID - 1) as f32 * 2.0 - 1.0;
            let weight = (1.0 - (dx * dx + dy * dy).min(1.0)).powi(2);
            thickness[y * SURFACE_FLUID_GRID + x] = weight;
        }
    }
    let sum = fluid_volume(&thickness).max(0.000_1);
    for cell in &mut thickness {
        *cell *= volume / sum;
    }
    let radius = (volume.sqrt() * 0.52).clamp(0.16, 1.35);
    let material = materials.add(SurfaceFluidMaterial {
        color: color.to_linear().with_alpha(profile.transparency),
        flow_data: Vec4::ZERO,
        film_data: Vec4::new((volume / 1.2).min(1.0), 0.0, profile.emissive, 0.0),
    });
    let mesh = if normal.y.abs() < 0.65 {
        super::create_wall_splatter_mesh(radius, 32)
    } else {
        super::create_irregular_puddle_mesh(radius, 32)
    };
    commands.spawn((
        SurfaceFluidPatch {
            profile,
            color,
            effects,
            host,
            normal,
            tangent,
            bitangent,
            thickness,
            flow: Vec2::ZERO,
            volume,
            age: 0.0,
            max_age: 75.0,
            radius,
            surface_adhesion: 1.0,
        },
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::from_translation(position + normal * 0.012).with_rotation(Quat::from_rotation_arc(Vec3::Y, normal)),
    ));
}

pub fn update_surface_fluid_patches(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    receivers: Query<&SurfaceFluidReceiver>,
    mut patches: Query<(Entity, &mut SurfaceFluidPatch, &mut Transform, &MeshMaterial3d<SurfaceFluidMaterial>)>,
    mut materials: ResMut<Assets<SurfaceFluidMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let filter = SpatialQueryFilter::default();
    for (entity, mut patch, mut transform, material) in patches.iter_mut() {
        patch.age += dt;
        if patch.age >= patch.max_age || patch.volume <= 0.000_1 {
            commands.entity(entity).despawn();
            continue;
        }
        let released = simulate_surface_flow(&mut patch, dt);
        let downhill = tangential_gravity(patch.normal).normalize_or_zero();
        transform.translation += downhill * patch.flow.length() * dt;
        let probe_origin = transform.translation + patch.normal * 0.10;
        let hit = spatial_query.cast_ray(probe_origin, Dir3::new(-patch.normal).unwrap_or(Dir3::NEG_Y), 0.25, true, &filter)
            .or_else(|| spatial_query.cast_ray(transform.translation + Vec3::Y * 0.08, Dir3::NEG_Y, 0.60, true, &filter));
        if let Some(hit) = hit {
            transform.translation = if hit.normal.dot(patch.normal) > 0.35 {
                probe_origin - patch.normal * hit.distance
            } else {
                transform.translation + Vec3::Y * 0.08 - Vec3::Y * hit.distance
            };
            patch.host = Some(hit.entity);
            patch.surface_adhesion = receivers.get(hit.entity).map(|receiver| receiver.adhesion).unwrap_or(1.0);
            patch.normal = hit.normal.normalize_or_zero();
            (patch.tangent, patch.bitangent) = surface_basis(patch.normal);
            transform.rotation = Quat::from_rotation_arc(Vec3::Y, patch.normal);
        }
        let detachment = (0.028 + patch.profile.surface_tension * 0.045) * patch.profile.adhesion;
        if released > detachment {
            let drop_volume = released;
            let velocity = patch.tangent * patch.flow.x + patch.bitangent * patch.flow.y + Vec3::new(0.0, -0.65, 0.0);
            let drop_radius = (drop_volume * 0.11).cbrt().clamp(0.025, 0.10);
            commands.spawn((
                SurfaceFluidDroplet { profile: patch.profile, color: patch.color, effects: patch.effects.clone(), volume: drop_volume, age: 0.0 },
                RigidBody::Dynamic,
                Collider::sphere(drop_radius),
                LinearVelocity(velocity),
                Mesh3d(meshes.add(Sphere::new(drop_radius).mesh().uv(10, 8))),
                MeshMaterial3d(material.0.clone()),
                Transform::from_translation(transform.translation + patch.normal * 0.03),
            ));
        } else if released > 0.0 {
            patch.thickness[0] += released;
            patch.volume = fluid_volume(&patch.thickness);
        }
        if let Some(mat) = materials.get_mut(&material.0) {
            mat.flow_data = Vec4::new(patch.flow.x, patch.flow.y, 0.0, 0.0);
            mat.film_data = Vec4::new((patch.volume / 1.2).min(1.0), patch.profile.viscosity, patch.profile.emissive, time.elapsed_secs());
            mat.color = patch.color.to_linear().with_alpha(patch.profile.transparency);
        }
    }
}

pub fn merge_surface_fluid_patches(
    mut commands: Commands,
    mut patches: ParamSet<(
        Query<(Entity, &Transform, &SurfaceFluidPatch)>,
        Query<(Entity, &mut SurfaceFluidPatch)>,
    )>,
) {
    let candidates: Vec<(Entity, Vec3, Option<Entity>, f32)> = patches
        .p0()
        .iter()
        .map(|(entity, transform, patch)| (entity, transform.translation, patch.host, patch.radius))
        .collect();
    for (index, (a_entity, a_position, a_host, a_radius)) in candidates.iter().enumerate() {
        for (b_entity, b_position, b_host, b_radius) in candidates.iter().skip(index + 1) {
            if a_host != b_host || a_position.distance(*b_position) > (a_radius + b_radius) * 0.55 {
                continue;
            }
            if let Ok([(_, mut a), (_, mut b)]) = patches.p1().get_many_mut([*a_entity, *b_entity]) {
                let transferred = b.volume;
                if transferred <= 0.0 {
                    continue;
                }
                for (target, source) in a.thickness.iter_mut().zip(b.thickness.iter()) {
                    *target += *source;
                }
                a.volume = fluid_volume(&a.thickness);
                a.radius = (a.radius * a.radius + b.radius * b.radius).sqrt().min(1.5);
                b.volume = 0.0;
                commands.entity(*b_entity).despawn();
            }
        }
    }
}

pub fn capture_surface_fluid_droplets(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut droplets: Query<(Entity, &mut SurfaceFluidDroplet, &Transform, &LinearVelocity)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SurfaceFluidMaterial>>,
    mut commands: Commands,
) {
    let filter = SpatialQueryFilter::default();
    for (entity, mut droplet, transform, velocity) in droplets.iter_mut() {
        droplet.age += time.delta_secs();
        if droplet.age > 8.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let speed = velocity.0.length();
        if speed <= 0.05 {
            continue;
        }
        if let Some(hit) = spatial_query.cast_ray(transform.translation, Dir3::new(velocity.0).unwrap_or(Dir3::NEG_Y), speed * time.delta_secs() + 0.05, true, &filter) {
            let point = transform.translation + velocity.0.normalize_or_zero() * hit.distance;
            spawn_surface_fluid_patch(&mut commands, &mut meshes, &mut materials, point, hit.normal, Some(hit.entity), droplet.profile, droplet.color, droplet.effects.clone(), droplet.volume);
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patch(normal: Vec3) -> SurfaceFluidPatch {
        let profile = PotionFluidProfiles::default().for_slot(1);
        let (tangent, bitangent) = surface_basis(normal);
        let mut thickness = [0.0; SURFACE_FLUID_CELLS];
        thickness[SURFACE_FLUID_GRID / 2 * SURFACE_FLUID_GRID + SURFACE_FLUID_GRID / 2] = 1.0;
        SurfaceFluidPatch { profile, color: Color::WHITE, effects: Vec::new(), host: None, normal, tangent, bitangent, thickness, flow: Vec2::ZERO, volume: 1.0, age: 0.0, max_age: 75.0, radius: 0.5, surface_adhesion: 1.0 }
    }

    #[test]
    fn tangential_gravity_has_no_normal_component() {
        let normal = Vec3::new(0.0, 0.6, 0.8).normalize();
        assert!(tangential_gravity(normal).dot(normal).abs() < 0.0001);
    }

    #[test]
    fn surface_flow_conserves_volume_without_an_edge_release() {
        let mut patch = patch(Vec3::new(0.0, 0.82, 0.57).normalize());
        let before = fluid_volume(&patch.thickness);
        let released = simulate_surface_flow(&mut patch, 1.0 / 60.0);
        assert!(released < 0.0001);
        assert!((before - patch.volume).abs() < 0.0001);
    }

    #[test]
    fn surface_flow_moves_on_a_vertical_surface() {
        let mut patch = patch(Vec3::Z);
        simulate_surface_flow(&mut patch, 1.0 / 30.0);
        assert!(patch.flow.length() > 0.001);
    }

    #[test]
    fn edge_release_preserves_total_volume() {
        let mut patch = patch(Vec3::Z);
        patch.thickness = [0.0; SURFACE_FLUID_CELLS];
        patch.thickness[SURFACE_FLUID_CELLS - SURFACE_FLUID_GRID / 2 - 1] = 1.0;
        patch.volume = 1.0;
        let released = simulate_surface_flow(&mut patch, 1.0 / 20.0);
        assert!(released > 0.0);
        assert!((patch.volume + released - 1.0).abs() < 0.0001);
    }
}
