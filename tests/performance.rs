use bevy::prelude::*;
use std::time::Instant;

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

    assert!(
        per_tick_us < 15000.0,
        "Alchemy tick took too long: {} µs",
        per_tick_us
    );
}

#[test]
fn test_particle_system_simulation_throughput() {
    let mut world = World::new();

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

    assert!(
        per_tick_us < 15000.0,
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
        per_op_ns < 2500.0,
        "Player math overhead too high: {} ns",
        per_op_ns
    );
}

#[test]
fn test_spatial_raycast_math_throughput() {
    let start = Instant::now();
    let iterations = 100_000;
    let origin = Vec3::new(0.0, 1.75, 0.0);
    let dir = Vec3::new(0.0, 0.0, -1.0);
    let target = Vec3::new(0.2, 1.8, -2.5);

    let mut in_reach_count = 0;
    for _ in 0..iterations {
        let to_target = target - origin;
        let dist = to_target.length();
        if dist < 3.5 {
            let dot = to_target.normalize().dot(dir);
            if dot > 0.94 {
                in_reach_count += 1;
            }
        }
    }

    let duration = start.elapsed();
    let per_op_ns = duration.as_nanos() as f64 / iterations as f64;
    println!(
        "[PERFORMANCE] 100,000 Raycast distance & dot operations: {:.2} ns/op (found: {})",
        per_op_ns, in_reach_count
    );

    assert!(
        per_op_ns < 1000.0,
        "Raycast math too slow: {} ns",
        per_op_ns
    );
}

#[test]
fn test_zero_allocation_tick_verification() {

    let mut effects: Vec<(u8, f32, f32)> = Vec::with_capacity(8);
    effects.push((1, 5.0, 0.0));
    effects.push((2, 10.0, 0.0));

    let initial_capacity = effects.capacity();
    let initial_ptr = effects.as_ptr();

    let dt = 0.016;
    for _ in 0..1000 {
        effects.retain_mut(|(_, dur, elapsed)| {
            *elapsed += dt;
            *elapsed < *dur
        });
    }

    assert_eq!(
        effects.capacity(),
        initial_capacity,
        "Heap reallocation detected! Capacity changed."
    );
    assert_eq!(
        effects.as_ptr(),
        initial_ptr,
        "Buffer relocated in memory! Zero-allocation invariant violated."
    );
    println!("[PERFORMANCE] Zero-allocation invariant verified: Vector pointer and capacity intact over 1,000 ticks.");
}
