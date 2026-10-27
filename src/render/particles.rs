use bevy::prelude::*;

pub const MAX_ACTIVE_PARTICLES: usize = 256;

#[derive(Resource)]
pub struct ParticleAssets {
    steam_mesh: Handle<Mesh>,
    splash_mesh: Handle<Mesh>,
    spore_mesh: Handle<Mesh>,
    flame_mesh: Handle<Mesh>,
    steam_material: Handle<StandardMaterial>,
    splash_material: Handle<StandardMaterial>,
    spore_material: Handle<StandardMaterial>,
    flame_materials: [Handle<StandardMaterial>; 3],
}

impl FromWorld for ParticleAssets {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let steam_mesh = meshes.add(Sphere::new(0.11).mesh().uv(8, 6));
        let splash_mesh = meshes.add(crate::alchemy::surface_fluid::create_teardrop_droplet_mesh(
            0.045, 8, 6,
        ));
        let spore_mesh = meshes.add(Sphere::new(0.012).mesh().uv(8, 6));
        let flame_mesh = meshes.add(Sphere::new(0.045).mesh().uv(8, 6));
        drop(meshes);

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let steam_material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.88, 0.95, 1.0, 0.28),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let splash_material = materials.add(StandardMaterial {
            base_color: Color::LinearRgba(LinearRgba::new(0.28, 0.55, 0.75, 0.18)),
            alpha_mode: AlphaMode::Add,
            unlit: true,
            ..default()
        });
        let spore_material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.25, 0.88, 0.95, 0.65),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });
        let flame_materials = [
            materials.add(flame_material(Color::srgb(1.0, 0.42, 0.05))),
            materials.add(flame_material(Color::srgb(1.0, 0.56, 0.08))),
            materials.add(flame_material(Color::srgb(1.0, 0.68, 0.12))),
        ];

        Self {
            steam_mesh,
            splash_mesh,
            spore_mesh,
            flame_mesh,
            steam_material,
            splash_material,
            spore_material,
            flame_materials,
        }
    }
}

fn flame_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        emissive: Color::srgb(1.0, 0.35, 0.02).to_linear() * 3.8,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    }
}

#[derive(Component)]
pub struct ParticleEmitter {
    pub spawn_timer: f32,
    pub spawn_rate: f32,
    pub particle_type: ParticleType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleType {
    Steam,
    WaterSplash,
    GlowSpore,
    FireFlame,
}

#[derive(Component)]
pub struct Particle {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub velocity: Vec3,
    pub p_type: ParticleType,
}

pub fn spawn_particles(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<ParticleAssets>,
    mut emitters: Query<(&mut ParticleEmitter, Option<&GlobalTransform>, &Transform)>,
    particles: Query<&Particle>,
) {
    let dt = time.delta_secs();
    let mut active_count = particles.iter().count();
    for (mut emitter, global_tf, tf) in emitters.iter_mut() {
        emitter.spawn_timer += dt;
        if emitter.spawn_timer >= emitter.spawn_rate {
            emitter.spawn_timer = 0.0;
            if active_count >= MAX_ACTIVE_PARTICLES {
                continue;
            }
            active_count += 1;
            let pos = global_tf.map(|g| g.translation()).unwrap_or(tf.translation);
            match emitter.particle_type {
                ParticleType::Steam => {
                    let vel = Vec3::new(
                        (fastrand::f32() - 0.5) * 0.7,
                        0.9 + fastrand::f32() * 0.9,
                        (fastrand::f32() - 0.5) * 0.7,
                    );
                    commands.spawn((
                        Mesh3d(assets.steam_mesh.clone()),
                        MeshMaterial3d(assets.steam_material.clone()),
                        Transform::from_translation(pos),
                        Particle {
                            lifetime: 0.0,
                            max_lifetime: 2.6 + fastrand::f32() * 0.8,
                            velocity: vel,
                            p_type: ParticleType::Steam,
                        },
                    ));
                }
                ParticleType::WaterSplash => {
                    let vel = Vec3::new(
                        (fastrand::f32() - 0.5) * 1.8,
                        0.6 + fastrand::f32() * 0.9,
                        (fastrand::f32() - 0.5) * 1.8,
                    );
                    commands.spawn((
                        Mesh3d(assets.splash_mesh.clone()),
                        MeshMaterial3d(assets.splash_material.clone()),
                        Transform::from_translation(pos),
                        Particle {
                            lifetime: 0.0,
                            max_lifetime: 1.4,
                            velocity: vel,
                            p_type: ParticleType::WaterSplash,
                        },
                    ));
                }
                ParticleType::GlowSpore => {
                    let angle = fastrand::f32() * std::f32::consts::TAU;
                    let radius_offset = fastrand::f32().sqrt() * 0.40;
                    let spawn_pos = pos
                        + Vec3::new(
                            angle.cos() * radius_offset,
                            -0.03 * fastrand::f32(),
                            angle.sin() * radius_offset,
                        );
                    let vel = Vec3::new(
                        (fastrand::f32() - 0.5) * 0.08,
                        -0.07 - fastrand::f32() * 0.06,
                        (fastrand::f32() - 0.5) * 0.08,
                    );
                    commands.spawn((
                        Mesh3d(assets.spore_mesh.clone()),
                        MeshMaterial3d(assets.spore_material.clone()),
                        Transform::from_translation(spawn_pos),
                        Particle {
                            lifetime: 0.0,
                            max_lifetime: 4.2,
                            velocity: vel,
                            p_type: ParticleType::GlowSpore,
                        },
                    ));
                }
                ParticleType::FireFlame => {
                    let angle = fastrand::f32() * std::f32::consts::TAU;
                    let radius_offset = fastrand::f32().sqrt() * 0.22;
                    let spawn_pos = pos
                        + Vec3::new(
                            angle.cos() * radius_offset,
                            fastrand::f32() * 0.12,
                            angle.sin() * radius_offset,
                        );
                    let vel = Vec3::new(
                        (fastrand::f32() - 0.5) * 0.35,
                        0.9 + fastrand::f32() * 0.75,
                        (fastrand::f32() - 0.5) * 0.35,
                    );
                    let flame_index =
                        ((fastrand::f32() * assets.flame_materials.len() as f32) as usize)
                            .min(assets.flame_materials.len() - 1);
                    let fire_material = assets.flame_materials[flame_index].clone();
                    commands.spawn((
                        Mesh3d(assets.flame_mesh.clone()),
                        MeshMaterial3d(fire_material),
                        Transform::from_translation(spawn_pos),
                        Particle {
                            lifetime: 0.0,
                            max_lifetime: 0.85 + fastrand::f32() * 0.35,
                            velocity: vel,
                            p_type: ParticleType::FireFlame,
                        },
                    ));
                }
            }
        }
    }
}

pub fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Particle, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut p, mut tf) in particles.iter_mut() {
        p.lifetime += dt;
        if p.lifetime >= p.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        tf.translation += p.velocity * dt;

        match p.p_type {
            ParticleType::Steam => {
                let progress = p.lifetime / p.max_lifetime;
                let expansion = 1.0 + (progress * std::f32::consts::PI * 0.5).sin() * 4.8;
                tf.scale = Vec3::splat(expansion);
                tf.translation.y += 0.35 * dt * (1.0 - progress * 0.5);
                p.velocity.y *= (1.0 - 0.35 * dt).max(0.0);
            }
            ParticleType::WaterSplash => {
                let progress = p.lifetime / p.max_lifetime;
                let speed = p.velocity.length();
                if speed > 0.05 {
                    tf.rotation = Quat::from_rotation_arc(Vec3::NEG_Y, p.velocity / speed);
                    let stretch = 1.0 + (speed * 0.22).min(1.4);
                    let lateral = (1.0 / stretch.sqrt()) * (1.0 - progress * 0.35);
                    tf.scale = Vec3::new(lateral, stretch * (1.0 - progress * 0.25), lateral);
                } else {
                    let scale = 1.0 - progress * 0.5;
                    tf.scale = Vec3::splat(scale.max(0.1));
                }
                p.velocity.y += 0.2 * dt;
            }
            ParticleType::GlowSpore => {
                let entity_seed = entity.index() as f32 * 0.37;
                let sway_x = (p.lifetime * 1.5 + entity_seed).sin() * 0.025;
                let sway_z = (p.lifetime * 1.2 + entity_seed).cos() * 0.025;
                tf.translation.x += sway_x * dt;
                tf.translation.z += sway_z * dt;

                let progress = p.lifetime / p.max_lifetime;
                let scale = (1.0 - progress * 0.75).max(0.1);
                tf.scale = Vec3::splat(scale);
            }
            ParticleType::FireFlame => {
                let progress = p.lifetime / p.max_lifetime;
                let scale = (1.2 - progress * 0.95).max(0.05);
                tf.scale = Vec3::splat(scale);
                p.velocity.y += 0.25 * dt;
                let flutter = ((p.lifetime * 14.0).sin()) * 0.06;
                tf.translation.x += flutter * dt;
            }
        }
    }
}

pub(crate) mod fastrand {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(88172645463325252);
    pub fn f32() -> f32 {
        let mut s = SEED.load(Ordering::Relaxed);
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        SEED.store(s, Ordering::Relaxed);
        (s & 0xFFFFFF) as f32 / 16777216.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_particles_and_update_lifecycle() {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        world.insert_resource(time);
        world.insert_resource(Assets::<Mesh>::default());
        world.insert_resource(Assets::<StandardMaterial>::default());
        world.init_resource::<ParticleAssets>();
        let mesh_count = world.resource::<Assets<Mesh>>().len();
        let material_count = world.resource::<Assets<StandardMaterial>>().len();

        world.spawn((
            ParticleEmitter {
                spawn_timer: 1.0,
                spawn_rate: 0.1,
                particle_type: ParticleType::Steam,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        world.spawn((
            ParticleEmitter {
                spawn_timer: 1.0,
                spawn_rate: 0.1,
                particle_type: ParticleType::WaterSplash,
            },
            Transform::from_xyz(1.0, 0.0, 0.0),
        ));

        world.spawn((
            ParticleEmitter {
                spawn_timer: 1.0,
                spawn_rate: 0.1,
                particle_type: ParticleType::GlowSpore,
            },
            Transform::from_xyz(2.0, 0.0, 0.0),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems((spawn_particles, update_particles));
        schedule.run(&mut world);

        let count = world.query::<&Particle>().iter(&world).count();
        assert_eq!(count, 3, "All 3 emitters must have spawned a particle");
        assert_eq!(world.resource::<Assets<Mesh>>().len(), mesh_count);
        assert_eq!(world.resource::<Assets<StandardMaterial>>().len(), material_count);

        world
            .resource_mut::<Time<()>>()
            .advance_by(std::time::Duration::from_secs(10));
        let mut update_schedule = Schedule::default();
        update_schedule.add_systems(update_particles);
        update_schedule.run(&mut world);

        let remaining = world.query::<&Particle>().iter(&world).count();
        assert_eq!(remaining, 0, "Expired particles must be despawned");
    }

    #[test]
    fn test_particle_count_is_capped() {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        world.insert_resource(time);
        world.insert_resource(Assets::<Mesh>::default());
        world.insert_resource(Assets::<StandardMaterial>::default());
        world.init_resource::<ParticleAssets>();

        for _ in 0..MAX_ACTIVE_PARTICLES {
            world.spawn(Particle {
                lifetime: 0.0,
                max_lifetime: 1.0,
                velocity: Vec3::ZERO,
                p_type: ParticleType::Steam,
            });
        }
        world.spawn((
            ParticleEmitter {
                spawn_timer: 1.0,
                spawn_rate: 0.1,
                particle_type: ParticleType::Steam,
            },
            Transform::default(),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_particles);
        schedule.run(&mut world);

        let count = world.query::<&Particle>().iter(&world).count();
        assert_eq!(count, MAX_ACTIVE_PARTICLES);
    }
}
