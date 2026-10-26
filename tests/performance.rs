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


#[test]
fn test_perf_dungeon_wfc_procedural_generation_latency() {
    use deep_alchemy::dungeon::generator::{DungeonConfig, DungeonGenerator};

    let seeds = [42, 1337, 2026, 9999, 54321];
    let config = DungeonConfig::default();
    let mut total_duration = std::time::Duration::ZERO;
    let mut total_cells = 0;

    for &seed in &seeds {
        let mut cfg = config.clone();
        cfg.seed = seed;
        let start = Instant::now();
        let dungeon = DungeonGenerator::generate(cfg);
        let elapsed = start.elapsed();
        total_duration += elapsed;
        total_cells += dungeon.dual_cells.len();

        let ms = elapsed.as_secs_f64() * 1000.0;
        println!(
            "[BOTTLENECK] Dungeon WFC Seed {}: generated {} dual cells in {:.3} ms ({:.2} µs/cell)",
            seed,
            dungeon.dual_cells.len(),
            ms,
            (elapsed.as_micros() as f64) / (dungeon.dual_cells.len().max(1) as f64)
        );
        assert!(
            ms < 35.0,
            "Dungeon generation bottleneck detected: took {:.3} ms",
            ms
        );
    }

    let avg_ms = (total_duration.as_secs_f64() * 1000.0) / (seeds.len() as f64);
    println!(
        "[BOTTLENECK REPORT] Average Dungeon Generation Time: {:.3} ms across {} runs ({} total cells)",
        avg_ms,
        seeds.len(),
        total_cells
    );
    assert!(avg_ms < 25.0, "Average generation exceeded budget");
}

#[test]
fn test_perf_water_ripples_and_puddles_simulation_bottleneck() {
    use deep_alchemy::dungeon::modular_kit::{update_water_ripples, WaterRipple};
    use deep_alchemy::dungeon::water_drips::{
        update_falling_water_drops, update_puddle_player_interaction, CavernWaterPuddle,
        FallingWaterDrop, WaterPuddleState,
    };
    use deep_alchemy::player::camera::Player;

    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    for i in 0..30 {
        let angle = (i as f32 / 30.0) * std::f32::consts::TAU;
        let r = 1.0 + (i as f32 % 5.0) * 0.4;
        app.world_mut().spawn((
            CavernWaterPuddle {
                state: WaterPuddleState::Liquid,
                radius: 1.2,
                original_radius: 1.2,
                base_y: 0.02,
                freeze_timer: 0.0,
                boil_timer: 0.0,
                regen_timer: 0.0,
                step_ripple_cooldown: 0.0,
            },
            Transform::from_xyz(angle.cos() * r, 0.02, angle.sin() * r),
            Visibility::Inherited,
        ));
    }

    for i in 0..150 {
        let t = i as f32 * 0.05;
        app.world_mut().spawn((
            WaterRipple {
                elapsed: t % 1.5,
                max_lifetime: 1.5,
                initial_radius: 0.1,
                max_radius: 1.2,
                initial_alpha: 0.7,
            },
            Transform::from_xyz((i as f32 % 10.0) - 5.0, 0.03, (i as f32 / 10.0) - 5.0),
            MeshMaterial3d(Handle::<StandardMaterial>::default()),
        ));
    }

    for i in 0..50 {
        app.world_mut().spawn((
            FallingWaterDrop {
                velocity_y: -2.5,
                target_y: 0.02,
                origin: Vec3::new((i as f32 % 8.0) - 4.0, 4.5, (i as f32 / 8.0) - 4.0),
                puddle_entity: None,
            },
            Transform::from_xyz((i as f32 % 8.0) - 4.0, 2.5, (i as f32 / 8.0) - 4.0),
        ));
    }

    let mut schedule = Schedule::default();
    schedule.add_systems((
        update_water_ripples,
        update_falling_water_drops,
        update_puddle_player_interaction,
    ));

    let iterations = 500;
    let start = Instant::now();

    for _ in 0..iterations {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        schedule.run(app.world_mut());
    }

    let elapsed = start.elapsed();
    let per_tick_us = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "[BOTTLENECK] Water Simulation (150 ripples, 30 puddles, 50 drops): {:.2} µs/tick ({:.4} ms/frame, theoretical {:.0} FPS)",
        per_tick_us,
        per_tick_us / 1000.0,
        1_000_000.0 / per_tick_us.max(1.0)
    );

    assert!(
        per_tick_us < 1200.0,
        "Water simulation bottleneck detected: took {:.2} µs/tick",
        per_tick_us
    );
}

#[test]
fn test_perf_alchemy_spatial_propagation_and_flammables_bottleneck() {
    use deep_alchemy::alchemy::{ActiveEffect, AlchemicalEffectKind, Flammable, PotionPuddle};

    let mut world = World::new();

    for i in 0..500 {
        let x = (i as f32 % 25.0) - 12.0;
        let z = (i as f32 / 25.0) - 10.0;
        world.spawn((
            Flammable::new(8.0),
            Transform::from_xyz(x, 0.0, z),
        ));
    }

    for i in 0..50 {
        let x = (i as f32 % 10.0) * 2.0 - 10.0;
        let z = (i as f32 / 10.0) * 2.0 - 5.0;
        world.spawn((
            PotionPuddle {
                color: Color::srgb(1.0, 0.4, 0.0),
                elapsed: 0.0,
                max_lifetime: 30.0,
                radius: 1.8,
                effects: vec![ActiveEffect {
                    kind: AlchemicalEffectKind::ThermalHeat,
                    duration: 15.0,
                    elapsed: 0.0,
                    magnitude: 1.0,
                }],
                is_wall: false,
            },
            Transform::from_xyz(x, 0.0, z),
        ));
    }

    let iterations = 200;
    let start = Instant::now();
    let mut total_checks = 0;

    for _ in 0..iterations {
        let mut puddles_query = world.query::<(&Transform, &PotionPuddle)>();
        let puddle_data: Vec<(Vec3, f32)> = puddles_query
            .iter(&world)
            .map(|(tf, p)| (tf.translation, p.radius))
            .collect();

        let mut flammables_query = world.query::<(&Transform, &mut Flammable)>();
        for (f_tf, mut flammable) in flammables_query.iter_mut(&mut world) {
            let f_pos = f_tf.translation;
            for &(p_pos, p_radius) in &puddle_data {
                total_checks += 1;
                let dx = f_pos.x - p_pos.x;
                let dz = f_pos.z - p_pos.z;
                let dist_sq = dx * dx + dz * dz;
                let range = p_radius + 0.5;
                if dist_sq <= range * range {
                    flammable.temperature += 0.5;
                }
            }
        }
    }

    let elapsed = start.elapsed();
    let per_tick_us = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "[BOTTLENECK] Alchemy 500 Flammables x 50 Puddles ({} total checks): {:.2} µs/tick ({:.3} ms, {:.0} checks/ms)",
        total_checks,
        per_tick_us,
        per_tick_us / 1000.0,
        (total_checks as f64) / (elapsed.as_millis().max(1) as f64)
    );

    assert!(
        per_tick_us < 2500.0,
        "Alchemy spatial collision bottleneck: took {:.2} µs/tick",
        per_tick_us
    );
}

#[test]
fn test_perf_vegetation_wind_oscillation_throughput() {
    let count = 2_000;
    let mut transforms: Vec<Transform> = Vec::with_capacity(count);
    let mut phases: Vec<f32> = Vec::with_capacity(count);

    for i in 0..count {
        transforms.push(Transform::from_xyz((i as f32) * 0.1, 0.0, (i as f32) * 0.1));
        phases.push((i as f32) * 0.037);
    }

    let iterations = 500;
    let start = Instant::now();

    for tick in 0..iterations {
        let t = tick as f32 * 0.016;
        for i in 0..count {
            let wave = (t * 2.4 + phases[i]).sin() * 0.08 + (t * 5.1 + phases[i] * 1.5).cos() * 0.03;
            transforms[i].rotation = Quat::from_rotation_z(wave);
        }
    }

    let elapsed = start.elapsed();
    let per_tick_us = elapsed.as_micros() as f64 / iterations as f64;
    let per_entity_ns = (elapsed.as_nanos() as f64) / (iterations as f64 * count as f64);

    println!(
        "[BOTTLENECK] 2,000 Vegetation Foliage Wind Oscillations: {:.2} µs/tick ({:.2} ns/entity, {:.3} ms/frame)",
        per_tick_us,
        per_entity_ns,
        per_tick_us / 1000.0
    );

    assert!(
        per_tick_us < 1800.0,
        "Vegetation wind oscillation bottleneck: took {:.2} µs/tick",
        per_tick_us
    );
}

#[test]
fn test_perf_full_gameplay_tick_bottleneck_breakdown() {
    use deep_alchemy::dungeon::modular_kit::{update_bioluminescent_mushrooms, update_water_ripples};
    use deep_alchemy::dungeon::water_drips::{
        update_ceiling_drip_points, update_falling_water_drops, update_puddle_player_interaction,
        update_puddle_temperature_reactions, CavernWaterPuddle, CeilingDripPoint, WaterPuddleState,
    };
    use deep_alchemy::player::camera::Player;

    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.5, 0.0, 0.5),
    ));

    for i in 0..10 {
        let puddle_ent = app
            .world_mut()
            .spawn((
                CavernWaterPuddle {
                    state: WaterPuddleState::Liquid,
                    radius: 1.2,
                    original_radius: 1.2,
                    base_y: 0.02,
                    freeze_timer: 0.0,
                    boil_timer: 0.0,
                    regen_timer: 0.0,
                    step_ripple_cooldown: 0.0,
                },
                Transform::from_xyz((i as f32) * 1.5 - 7.5, 0.02, (i as f32) * 1.5 - 7.5),
                Visibility::Inherited,
                MeshMaterial3d(Handle::<StandardMaterial>::default()),
            ))
            .id();

        app.world_mut().spawn((
            CeilingDripPoint {
                timer: (i as f32) * 0.3,
                drip_interval: 2.5,
                puddle_entity: Some(puddle_ent),
                drop_fall_speed: 0.0,
            },
            Transform::from_xyz((i as f32) * 1.5 - 7.5, 5.0, (i as f32) * 1.5 - 7.5),
        ));
    }

    let mut schedule = Schedule::default();
    schedule.add_systems((
        update_water_ripples,
        update_bioluminescent_mushrooms,
        (
            update_ceiling_drip_points,
            update_falling_water_drops,
            update_puddle_player_interaction,
            update_puddle_temperature_reactions,
        ).chain(),
    ));

    for _ in 0..5 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        schedule.run(app.world_mut());
    }

    let frame_count = 300;
    let mut tick_times_us: Vec<f64> = Vec::with_capacity(frame_count);

    for _ in 0..frame_count {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));

        let tick_start = Instant::now();
        schedule.run(app.world_mut());
        let tick_us = tick_start.elapsed().as_micros() as f64;
        tick_times_us.push(tick_us);
    }

    tick_times_us.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_us = tick_times_us[frame_count / 2];
    let p95_us = tick_times_us[(frame_count as f64 * 0.95) as usize];
    let p99_us = tick_times_us[(frame_count as f64 * 0.99) as usize];
    let max_us = tick_times_us[frame_count - 1];
    let avg_us = tick_times_us.iter().sum::<f64>() / (frame_count as f64);

    println!("\n================================================================================");
    println!("📊 [DIAGNOSTIC BOTTLENECK REPORT] Full Gameplay Tick Benchmark ({} frames)", frame_count);
    println!("================================================================================");
    println!("   • Median Tick Latency (p50): {:.2} µs ({:.4} ms)", median_us, median_us / 1000.0);
    println!("   • Average Tick Latency:      {:.2} µs ({:.4} ms)", avg_us, avg_us / 1000.0);
    println!("   • 95th Percentile (p95):     {:.2} µs ({:.4} ms)", p95_us, p95_us / 1000.0);
    println!("   • 99th Percentile (p99):     {:.2} µs ({:.4} ms)", p99_us, p99_us / 1000.0);
    println!("   • Max Spike Duration:        {:.2} µs ({:.4} ms)", max_us, max_us / 1000.0);
    println!("   • Theoretical Throughput:    {:.0} FPS capability (median tick limit)", 1_000_000.0 / median_us.max(1.0));
    println!("================================================================================\n");

    assert!(
        p99_us < 4000.0,
        "Frame tick p99 bottleneck detected: took {:.2} µs",
        p99_us
    );
}
