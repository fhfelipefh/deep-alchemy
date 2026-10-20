use bevy::prelude::*;

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut emitters: Query<(&mut ParticleEmitter, Option<&GlobalTransform>, &Transform)>,
) {
    let dt = time.delta_secs();
    for (mut emitter, global_tf, tf) in emitters.iter_mut() {
        emitter.spawn_timer += dt;
        if emitter.spawn_timer >= emitter.spawn_rate {
            emitter.spawn_timer = 0.0;
            let pos = global_tf.map(|g| g.translation()).unwrap_or(tf.translation);
            match emitter.particle_type {
                ParticleType::Steam => {
                    let vel = Vec3::new(
                        (fastrand::f32() - 0.5) * 0.4,
                        0.8 + fastrand::f32() * 0.5,
                        (fastrand::f32() - 0.5) * 0.4,
                    );
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.08).mesh().uv(8, 6))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.85, 0.94, 1.0, 0.16),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(pos),
                        Particle {
                            lifetime: 0.0,
                            max_lifetime: 2.2,
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
                        Mesh3d(meshes.add(Sphere::new(0.09).mesh().uv(8, 6))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::LinearRgba(LinearRgba::new(0.28, 0.55, 0.75, 0.18)),
                            alpha_mode: AlphaMode::Add,
                            unlit: true,
                            ..default()
                        })),
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
                        Mesh3d(meshes.add(Sphere::new(0.012).mesh().uv(8, 6))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.25, 0.88, 0.95, 0.65),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        })),
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
                    let fire_color = Color::srgba(1.0, 0.42 + fastrand::f32() * 0.28, 0.05, 0.82);
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.045).mesh().uv(8, 6))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: fire_color,
                            emissive: Color::srgb(1.0, 0.35, 0.02).to_linear() * 3.8,
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        })),
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
                let scale = 1.0 + progress * 2.5;
                tf.scale = Vec3::splat(scale);
            }
            ParticleType::WaterSplash => {
                let progress = p.lifetime / p.max_lifetime;
                let scale = 1.0 + progress * 1.1;
                tf.scale = Vec3::splat(scale);
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

        world
            .resource_mut::<Time<()>>()
            .advance_by(std::time::Duration::from_secs(10));
        let mut update_schedule = Schedule::default();
        update_schedule.add_systems(update_particles);
        update_schedule.run(&mut world);

        let remaining = world.query::<&Particle>().iter(&world).count();
        assert_eq!(remaining, 0, "Expired particles must be despawned");
    }
}
