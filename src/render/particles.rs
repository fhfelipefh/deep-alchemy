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
                        (fastrand::f32() - 0.5) * 2.0,
                        1.5 + fastrand::f32() * 1.5,
                        (fastrand::f32() - 0.5) * 2.0,
                    );
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.08).mesh().uv(6, 4))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgba(0.85, 0.95, 1.0, 0.7),
                            alpha_mode: AlphaMode::Blend,
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(pos),
                        Particle {
                            lifetime: 0.0,
                            max_lifetime: 1.0,
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
                            base_color: Color::srgb(0.1, 0.9, 1.0),
                            emissive: LinearRgba::new(0.2, 1.8, 2.0, 1.0),
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
                p.velocity.y -= 9.8 * dt; // Gravity
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
