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
    pub accumulated_drip: f32,
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
    pub base_radius: f32,
}

pub fn create_teardrop_droplet_mesh(radius: f32, rings: usize, sectors: usize) -> Mesh {
    let rings = rings.max(6);
    let sectors = sectors.max(6);
    let mut positions = Vec::with_capacity((rings + 1) * (sectors + 1));
    let mut normals = Vec::with_capacity((rings + 1) * (sectors + 1));
    let mut uvs = Vec::with_capacity((rings + 1) * (sectors + 1));
    let mut indices = Vec::with_capacity(rings * sectors * 6);

    for i in 0..=rings {
        let t = i as f32 / rings as f32;
        let y = radius * (1.5 * (1.0 - t).powf(1.35) - t);
        let r = radius * (t * std::f32::consts::PI).sin() * (0.55 + 0.45 * t);

        let dt = 0.001;
        let t_next = (t + dt).min(1.0);
        let t_prev = (t - dt).max(0.0);
        let y_next = radius * (1.5 * (1.0 - t_next).powf(1.35) - t_next);
        let y_prev = radius * (1.5 * (1.0 - t_prev).powf(1.35) - t_prev);
        let dy = (y_next - y_prev) / (t_next - t_prev);

        let r_next = radius * (t_next * std::f32::consts::PI).sin() * (0.55 + 0.45 * t_next);
        let r_prev = radius * (t_prev * std::f32::consts::PI).sin() * (0.55 + 0.45 * t_prev);
        let dr = (r_next - r_prev) / (t_next - t_prev);

        for j in 0..=sectors {
            let u = j as f32 / sectors as f32;
            let theta = u * std::f32::consts::TAU;
            let cos_theta = theta.cos();
            let sin_theta = theta.sin();

            let x = r * cos_theta;
            let z = r * sin_theta;
            positions.push([x, y, z]);
            uvs.push([u, t]);

            if i == 0 {
                normals.push([0.0, 1.0, 0.0]);
            } else if i == rings {
                normals.push([0.0, -1.0, 0.0]);
            } else {
                let norm = Vec3::new(-dy * cos_theta, dr, -dy * sin_theta).normalize_or_zero();
                normals.push([norm.x, norm.y, norm.z]);
            }
        }
    }

    let stride = (sectors + 1) as u32;
    for i in 0..rings as u32 {
        for j in 0..sectors as u32 {
            let top_left = i * stride + j;
            let top_right = top_left + 1;
            let bottom_left = (i + 1) * stride + j;
            let bottom_right = bottom_left + 1;

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh
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
    let in_grotto_ramp = position.x.abs() > 2.5 && position.x.abs() < 7.5 && position.z > -16.0 && position.z < 3.8;
    let normal = if in_grotto_ramp {
        let terr_y = crate::dungeon::modular_kit::get_grotto_terrain_height(position.x, position.z);
        if (position.y - terr_y).abs() <= 0.65 {
            crate::dungeon::modular_kit::get_grotto_terrain_normal(position.x, position.z)
        } else {
            normal.normalize_or_zero()
        }
    } else {
        normal.normalize_or_zero()
    };
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
    if crate::dungeon::modular_kit::is_in_lake_water(position) && effects.iter().any(|e| e.kind == super::AlchemicalEffectKind::Frost) {
        return;
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
            accumulated_drip: 0.0,
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
    mut patches: Query<(Entity, &mut SurfaceFluidPatch, &mut Transform, &MeshMaterial3d<SurfaceFluidMaterial>, Option<&mut Mesh3d>)>,
    droplets: Query<(), With<SurfaceFluidDroplet>>,
    mut materials: ResMut<Assets<SurfaceFluidMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let filter = SpatialQueryFilter::default();
    let current_droplets = droplets.iter().count();
    let mut can_spawn_droplets = current_droplets < 12;
    let mut despawned: std::collections::HashSet<Entity> = std::collections::HashSet::new();

    for (entity, mut patch, mut transform, material, _mesh_handle) in patches.iter_mut() {
        patch.age += dt;
        if patch.age >= patch.max_age || patch.volume <= 0.000_1 {
            if despawned.insert(entity) {
                commands.queue(move |world: &mut World| {
                    if let Ok(ent) = world.get_entity_mut(entity) {
                        ent.despawn();
                    }
                });
            }
            continue;
        }
        let was_wall = patch.normal.y.abs() < 0.65;
        let prev_pos = transform.translation;
        let released = simulate_surface_flow(&mut patch, dt);
        let downhill = tangential_gravity(patch.normal).normalize_or_zero();
        let creep_speed = if was_wall {
            ((0.025 + (1.0 - patch.profile.adhesion).max(0.0) * 0.035) / (1.0 + patch.profile.viscosity)).clamp(0.015, 0.06)
        } else {
            0.0
        };
        let flow_factor = (patch.flow.length() * 0.35).clamp(0.2, 1.0);
        transform.translation += downhill * (creep_speed * flow_factor) * dt;

        let probe_dir = Dir3::new(-patch.normal).unwrap_or(Dir3::NEG_Y);
        let probe_origin = transform.translation + patch.normal * 0.08;
        if let Some(hit) = spatial_query.cast_ray(probe_origin, probe_dir, 0.28, true, &filter) {
            if hit.distance > 0.001 && hit.normal.length_squared() > 0.25 {
                let contact = probe_origin + probe_dir.as_vec3() * hit.distance;
                let hit_norm = hit.normal.normalize_or_zero();
                let in_grotto_ramp = contact.x.abs() > 2.5 && contact.x.abs() < 7.5 && contact.z > -16.0 && contact.z < 3.8;
                let final_normal = if in_grotto_ramp {
                    let terr_y = crate::dungeon::modular_kit::get_grotto_terrain_height(contact.x, contact.z);
                    if (contact.y - terr_y).abs() <= 0.65 {
                        crate::dungeon::modular_kit::get_grotto_terrain_normal(contact.x, contact.z)
                    } else {
                        hit_norm
                    }
                } else {
                    hit_norm
                };
                transform.translation = contact + final_normal * 0.012;
                patch.host = Some(hit.entity);
                patch.surface_adhesion = receivers.get(hit.entity).map(|receiver| receiver.adhesion).unwrap_or(1.0);
                patch.normal = final_normal;
                (patch.tangent, patch.bitangent) = surface_basis(patch.normal);
                transform.rotation = Quat::from_rotation_arc(Vec3::Y, patch.normal);
            }
        }

        if was_wall {
            let floor_probe = spatial_query.cast_ray(transform.translation + patch.normal * 0.04 + Vec3::Y * 0.04, Dir3::NEG_Y, patch.radius * 0.8 + 0.12, true, &filter);
            if let Some(floor_hit) = floor_probe {
                if floor_hit.normal.y > 0.65 {
                    let floor_y = transform.translation.y + 0.04 - floor_hit.distance;
                    let min_y = floor_y + patch.radius * 0.45;
                    if transform.translation.y < min_y {
                        transform.translation.y = min_y;
                    }
                }
            }
        } else if patch.normal.y > 0.65 && transform.translation.y > prev_pos.y + 0.02 {
            transform.translation.y = prev_pos.y;
        }

        if was_wall {
            patch.accumulated_drip += released;
            let detachment = (0.014 + patch.profile.surface_tension * 0.018) * patch.profile.adhesion;
            if patch.accumulated_drip >= detachment && patch.volume > 0.025 {
                let drop_volume = detachment.min(patch.volume * 0.25).max(0.012);
                patch.accumulated_drip -= drop_volume;
                patch.volume = (patch.volume - drop_volume).max(0.0);
                let current_vol = fluid_volume(&patch.thickness);
                if current_vol > 0.000_1 {
                    let scale = patch.volume / current_vol;
                    for cell in &mut patch.thickness {
                        *cell *= scale;
                    }
                }
                if can_spawn_droplets {
                    can_spawn_droplets = false;
                    let drop_offset = -Vec3::Y * (patch.radius * 0.70) + patch.normal * 0.03;
                    let drop_pos = transform.translation + drop_offset;
                    let drop_radius = (drop_volume * 0.08).cbrt().clamp(0.022, 0.046);
                    let drop_vel = Vec3::new(0.0, -1.8, 0.0) + patch.normal * 0.04;
                    let drop_mesh = meshes.add(create_teardrop_droplet_mesh(drop_radius, 10, 8));
                    commands.spawn((
                        SurfaceFluidDroplet { profile: patch.profile, color: patch.color, effects: patch.effects.clone(), volume: drop_volume, age: 0.0, base_radius: drop_radius },
                        RigidBody::Dynamic,
                        Collider::sphere(drop_radius * 0.75),
                        LinearVelocity(drop_vel),
                        Mesh3d(drop_mesh),
                        MeshMaterial3d(material.0.clone()),
                        Transform::from_translation(drop_pos),
                    ));
                }
            }
        } else {
            patch.accumulated_drip = 0.0;
        }

        if let Some(mat) = materials.get_mut(&material.0) {
            mat.flow_data = Vec4::new(patch.flow.x, patch.flow.y, 0.0, 0.0);
            mat.film_data = Vec4::new((patch.volume / 1.2).min(1.0), patch.profile.viscosity, patch.profile.emissive, time.elapsed_secs());
            let alpha_factor = (patch.volume / patch.profile.initial_volume).clamp(0.25, 1.0);
            mat.color = patch.color.to_linear().with_alpha(patch.profile.transparency * alpha_factor);
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
    let mut despawned: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    let candidates: Vec<(Entity, Vec3, Option<Entity>, f32, Vec3)> = patches
        .p0()
        .iter()
        .map(|(entity, transform, patch)| (entity, transform.translation, patch.host, patch.radius, patch.normal))
        .collect();
    for (index, (a_entity, a_position, a_host, a_radius, a_normal)) in candidates.iter().enumerate() {
        if despawned.contains(a_entity) {
            continue;
        }
        for (b_entity, b_position, b_host, b_radius, b_normal) in candidates.iter().skip(index + 1) {
            if despawned.contains(b_entity) {
                continue;
            }
            if a_host != b_host || a_normal.dot(*b_normal) < 0.75 || a_position.distance(*b_position) > (a_radius + b_radius) * 0.55 {
                continue;
            }
            if let Ok([(_, mut a), (_, b)]) = patches.p1().get_many_mut([*a_entity, *b_entity]) {
                let transferred = b.volume;
                if transferred <= 0.0 {
                    continue;
                }
                for (target, source) in a.thickness.iter_mut().zip(b.thickness.iter()) {
                    *target += *source;
                }
                a.volume = fluid_volume(&a.thickness);
                a.radius = (a.radius * a.radius + b.radius * b.radius).sqrt().min(1.5);
                let b_ent = *b_entity;
                despawned.insert(b_ent);
                commands.queue(move |world: &mut World| {
                    if let Ok(ent) = world.get_entity_mut(b_ent) {
                        ent.despawn();
                    }
                });
            }
        }
    }
}

pub fn update_surface_fluid_droplet_physics(
    time: Res<Time>,
    mut droplets: Query<(
        &SurfaceFluidDroplet,
        &mut Transform,
        &mut LinearVelocity,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    for (droplet, mut transform, mut velocity) in droplets.iter_mut() {
        let speed = velocity.0.length();
        if speed <= 0.001 {
            continue;
        }

        let terminal_speed = 5.2 + droplet.base_radius * 40.0;
        let drag_factor = (speed / terminal_speed).powi(2) * 9.81;
        let drag_accel = -velocity.0.normalize() * drag_factor * dt;
        velocity.0 += drag_accel;

        velocity.0.x *= (1.0 - 1.5 * dt).max(0.0);
        velocity.0.z *= (1.0 - 1.5 * dt).max(0.0);

        let travel_dir = velocity.0.normalize();
        let target_rot = Quat::from_rotation_arc(Vec3::NEG_Y, travel_dir);
        transform.rotation = transform.rotation.slerp(target_rot, (dt * 18.0).min(1.0));

        let stretch_ratio = 1.0 + (speed / terminal_speed).min(1.5) * 0.55;

        let omega = 24.0 * (droplet.profile.surface_tension / 0.7).sqrt();
        let damping = 1.8 + droplet.profile.viscosity * 5.0;
        let decay = (-droplet.age * damping).exp();
        let wobble = (droplet.age * omega).sin() * decay * 0.22;

        let scale_y = (stretch_ratio * (1.0 + wobble)).clamp(0.35, 2.8);
        let scale_lateral = 1.0 / scale_y.sqrt();

        transform.scale = Vec3::new(scale_lateral, scale_y, scale_lateral);
    }
}

pub fn capture_surface_fluid_droplets(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut droplets: Query<(Entity, &mut SurfaceFluidDroplet, &Transform, &LinearVelocity)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SurfaceFluidMaterial>>,
    mut std_materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut commands: Commands,
) {
    let mut despawned: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for (entity, mut droplet, transform, velocity) in droplets.iter_mut() {
        droplet.age += time.delta_secs();
        if droplet.age > 4.0 {
            if despawned.insert(entity) {
                commands.queue(move |world: &mut World| {
                    if let Ok(ent) = world.get_entity_mut(entity) {
                        ent.despawn();
                    }
                });
            }
            continue;
        }
        let speed = velocity.0.length();
        if speed <= 0.05 {
            continue;
        }
        let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
        if let Some(hit) = spatial_query.cast_ray(transform.translation, Dir3::new(velocity.0).unwrap_or(Dir3::NEG_Y), speed * time.delta_secs() + 0.08, true, &filter) {
            let point = transform.translation + velocity.0.normalize_or_zero() * hit.distance;
            if crate::dungeon::modular_kit::is_in_lake_water(point) {
                if let Some(ref mut std_mats) = std_materials {
                    crate::dungeon::modular_kit::spawn_water_ripple(
                        &mut commands,
                        &mut meshes,
                        std_mats,
                        Vec3::new(point.x, 0.09, point.z),
                        0.04,
                        0.42,
                        0.85,
                        0.70,
                    );
                }
            } else if droplet.volume >= 0.010 {
                spawn_surface_fluid_patch(&mut commands, &mut meshes, &mut materials, point, hit.normal, Some(hit.entity), droplet.profile, droplet.color, droplet.effects.clone(), droplet.volume);
            }
            if despawned.insert(entity) {
                commands.queue(move |world: &mut World| {
                    if let Ok(ent) = world.get_entity_mut(entity) {
                        ent.despawn();
                    }
                });
            }
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
        SurfaceFluidPatch { profile, color: Color::WHITE, effects: Vec::new(), host: None, normal, tangent, bitangent, thickness, flow: Vec2::ZERO, volume: 1.0, age: 0.0, max_age: 75.0, radius: 0.5, surface_adhesion: 1.0, accumulated_drip: 0.0 }
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

    #[test]
    fn test_patch_flowing_down_wall_does_not_rise_to_infinity() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::transform::TransformPlugin,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin::default(),
            avian3d::prelude::PhysicsPlugins::default(),
        ));
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(16),
        ));
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<SurfaceFluidMaterial>>();
        app.add_systems(Update, (update_surface_fluid_patches, update_surface_fluid_droplet_physics, capture_surface_fluid_droplets, merge_surface_fluid_patches));

        app.world_mut().spawn((
            Transform::from_xyz(0.0, -5.0, 0.0),
            RigidBody::Static,
            Collider::cuboid(40.0, 10.0, 40.0),
        ));

        app.world_mut().spawn((
            Transform::from_xyz(0.0, 5.0, -10.0),
            RigidBody::Static,
            Collider::cuboid(40.0, 10.0, 20.0),
        ));

        let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(crate::alchemy::create_wall_splatter_mesh(0.5, 16));
        let mut mats = app.world_mut().resource_mut::<Assets<SurfaceFluidMaterial>>();
        let mat = mats.add(SurfaceFluidMaterial::default());

        let patch_entity = app.world_mut().spawn((
            patch(Vec3::Z),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_xyz(0.0, 2.0, 0.02),
        )).id();

        for _ in 0..80 {
            app.update();
        }

        let tf = app.world().get::<Transform>(patch_entity).unwrap();
        let p = app.world().get::<SurfaceFluidPatch>(patch_entity).unwrap();
        assert!(p.normal.y.abs() < 0.65);
        assert!(tf.translation.y <= 2.0 && tf.translation.y >= 0.0);
        assert!(p.volume <= 1.001);
    }
}
