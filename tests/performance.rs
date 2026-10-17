use std::time::Instant;
use bevy::prelude::*;

// Direct struct tests to measure engine performance and zero allocations
#[derive(Component)]
struct TestEffectReceiver {
    effects: Vec<(u8, f32, f32)>,
}

#[derive(Component)]
struct TestParticle {
    pos: Vec3,
    vel: Vec3,
    life: f32,
}

#[test]
fn test_alchemy_systemic_effects_throughput() {
    let mut world = World::new();

    // Spawn 10,000 entities with effect receivers
    for _i in 0..10_000 {
        world.spawn(TestEffectReceiver {
            effects: vec![(1, 10.0, 5.0), (2, 5.0, 2.0)],
        });
    }

    let start = Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        let mut query = world.query::<&mut TestEffectReceiver>();
        let dt = 0.016;
        for mut receiver in query.iter_mut(&mut world) {
            receiver.effects.retain_mut(|(_, dur, elapsed)| {
                *elapsed += dt;
                *elapsed < *dur
            });
        }
    }

    let duration = start.elapsed();
    let per_tick_us = duration.as_micros() as f64 / iterations as f64;
    println!(
        "\n[PERFORMANCE] 10,000 Effect Receivers processed in {:.2} µs/tick ({:.3} ms)",
        per_tick_us,
        per_tick_us / 1000.0
    );

    // Assert that 10,000 entities take less than 2.0ms per tick (well below 16.6ms budget)
    assert!(
        per_tick_us < 2000.0,
        "Alchemy tick took too long: {} µs",
        per_tick_us
    );
}

#[test]
fn test_particle_system_simulation_throughput() {
    let mut world = World::new();

    // Spawn 10,000 active particles
    for _i in 0..10_000 {
        world.spawn(TestParticle {
            pos: Vec3::ZERO,
            vel: Vec3::new(0.1, 1.2, -0.3),
            life: 0.0,
        });
    }

    let start = Instant::now();
    let iterations = 100;
    let dt = 0.016;

    for _ in 0..iterations {
        let mut query = world.query::<&mut TestParticle>();
        for mut p in query.iter_mut(&mut world) {
            p.life += dt;
            let move_delta = p.vel * dt;
            p.pos += move_delta;
            p.vel.y -= 9.8 * dt;
        }
    }

    let duration = start.elapsed();
    let per_tick_us = duration.as_micros() as f64 / iterations as f64;
    println!(
        "[PERFORMANCE] 10,000 Particle simulations processed in {:.2} µs/tick ({:.3} ms)",
        per_tick_us,
        per_tick_us / 1000.0
    );

    // 10,000 particles should simulate in less than 1.5ms
    assert!(
        per_tick_us < 1500.0,
        "Particle tick took too long: {} µs",
        per_tick_us
    );
}

#[test]
fn test_player_controller_math_overhead() {
    let start = Instant::now();
    let iterations = 1_000_000;
    let mut rotation = Quat::IDENTITY;
    let delta = Vec2::new(1.5, -2.2);
    let sensitivity = 0.0022;

    for _ in 0..iterations {
        let (yaw, mut pitch, roll) = rotation.to_euler(EulerRot::YXZ);
        pitch -= delta.y * sensitivity;
        pitch = pitch.clamp(-1.52, 1.52);
        rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }

    let duration = start.elapsed();
    let per_op_ns = duration.as_nanos() as f64 / iterations as f64;
    println!(
        "[PERFORMANCE] 1,000,000 Player Look euler rotations: {:.2} ns/op",
        per_op_ns
    );

    assert!(
        per_op_ns < 100.0,
        "Player math overhead too high: {} ns",
        per_op_ns
    );
}
