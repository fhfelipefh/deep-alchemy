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
    mut emitters: Query<(&mut ParticleEmitter, &Transform)>,
) {
    let dt = time.delta_secs();
    for (mut emitter, tf) in emitters.iter_mut() {
        emitter.spawn_timer += dt;
        if emitter.spawn_timer >= emitter.spawn_rate {
            emitter.spawn_timer = 0.0;
            let pos = tf.translation;
            match emitter.particle_type {
                ParticleType::Steam => {
                    let vel = Vec3::new(
                        (fastrand::f32() - 0.5) * 0.4,
                        0.8 + fastrand::f32() * 0.5,
                        (fastrand::f32() - 0.5) * 0.4,
                    );
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.12).mesh().uv(8, 6))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.9, 0.95, 1.0, 0.35),
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
                        Mesh3d(meshes.add(Sphere::new(0.22).mesh().uv(8, 6))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::LinearRgba(LinearRgba::new(2.2, 4.2, 5.0, 0.40)),
                            alpha_mode: AlphaMode::Blend,
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
                    let vel = Vec3::new(
                        (fastrand::f32() - 0.5) * 0.3,
                        (fastrand::f32() - 0.5) * 0.3,
                        (fastrand::f32() - 0.5) * 0.3,
                    );
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.04).mesh().uv(6, 4))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::LinearRgba(LinearRgba::new(3.0, 6.0, 7.0, 0.85)),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(pos),
                        Particle {
                            lifetime: 0.0,
                            max_lifetime: 4.0,
                            velocity: vel,
                            p_type: ParticleType::GlowSpore,
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
                let scale = 1.0 + progress * 2.2;
                tf.scale = Vec3::splat(scale);
                p.velocity.y += 0.2 * dt; // Billowing gentle upward expansion
            }
            ParticleType::GlowSpore => {
                let wave = (p.lifetime * 3.0).sin() * 0.1;
                tf.translation.y += wave * dt;
            }
        }
    }
}

// Simple fast pseudo-random generator
mod fastrand {
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

        // Spawn emitter for each particle type
        world.spawn((
            ParticleEmitter {
                spawn_timer: 1.0,
                spawn_rate: 0.1, // will trigger spawn on first tick
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

        // Check that particles were spawned
        let count = world.query::<&Particle>().iter(&world).count();
        assert_eq!(count, 3, "All 3 emitters must have spawned a particle");

        // Advance time beyond max_lifetime to test despawning (run update_particles directly)
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_secs(10));
        let mut update_schedule = Schedule::default();
        update_schedule.add_systems(update_particles);
        update_schedule.run(&mut world);

        let remaining = world.query::<&Particle>().iter(&world).count();
        assert_eq!(remaining, 0, "Expired particles must be despawned");
    }
}
