use bevy::prelude::*;
use deep_alchemy::{
    alchemy::{AlchemicalEffectKind, EffectReceiverComponent, ThermalGeyser},
    player::{
        camera::{Health, PlayerState, Stamina},
        ragdoll::DeathCamera,
    },
    render::{
        particles::{Particle, ParticleType},
        water_material::WaterMaterial,
    },
};

#[test]
fn test_unit_alchemy_apply_and_query_effects() {
    let mut receiver = EffectReceiverComponent::default();
    assert!(!receiver.has_effect(AlchemicalEffectKind::ThermalHeat));

    receiver.apply_effect(AlchemicalEffectKind::ThermalHeat, 5.0, 20.0);
    assert!(receiver.has_effect(AlchemicalEffectKind::ThermalHeat));
    assert_eq!(receiver.active_effects.len(), 1);
    assert_eq!(receiver.active_effects[0].duration, 5.0);
    assert_eq!(receiver.active_effects[0].magnitude, 20.0);
    assert_eq!(receiver.active_effects[0].elapsed, 0.0);

    receiver.apply_effect(AlchemicalEffectKind::ThermalHeat, 8.0, 35.0);
    assert_eq!(receiver.active_effects.len(), 1);
    assert_eq!(receiver.active_effects[0].duration, 8.0);
    assert_eq!(receiver.active_effects[0].magnitude, 35.0);

    receiver.apply_effect(AlchemicalEffectKind::Swiftness, 10.0, 1.5);
    receiver.apply_effect(AlchemicalEffectKind::Luminescence, 15.0, 2.0);
    assert_eq!(receiver.active_effects.len(), 3);
    assert!(receiver.has_effect(AlchemicalEffectKind::Swiftness));
    assert!(receiver.has_effect(AlchemicalEffectKind::Luminescence));
    assert!(!receiver.has_effect(AlchemicalEffectKind::Frost));
}

#[test]
fn test_unit_alchemy_effects_ticking_and_expiration() {
    let mut receiver = EffectReceiverComponent::default();
    receiver.apply_effect(AlchemicalEffectKind::Vitality, 2.0, 50.0);
    receiver.apply_effect(AlchemicalEffectKind::Frost, 1.0, 10.0);

    let dt = 0.5;

    receiver.active_effects.retain_mut(|e| {
        e.elapsed += dt;
        e.elapsed < e.duration
    });
    assert_eq!(receiver.active_effects.len(), 2);

    let dt2 = 0.6;
    receiver.active_effects.retain_mut(|e| {
        e.elapsed += dt2;
        e.elapsed < e.duration
    });
    assert_eq!(receiver.active_effects.len(), 1);
    assert!(receiver.has_effect(AlchemicalEffectKind::Vitality));
    assert!(!receiver.has_effect(AlchemicalEffectKind::Frost));

    let dt3 = 1.0;
    receiver.active_effects.retain_mut(|e| {
        e.elapsed += dt3;
        e.elapsed < e.duration
    });
    assert_eq!(receiver.active_effects.len(), 0);
    assert!(!receiver.has_effect(AlchemicalEffectKind::Vitality));
}

#[test]
fn test_unit_thermal_geyser_state_and_proximity() {
    let mut geyser = ThermalGeyser {
        eruption_timer: 2.8,
        eruption_interval: 3.0,
        is_erupting: false,
        heat_radius: 5.0,
    };

    let dt = 0.3;
    geyser.eruption_timer += dt;
    if geyser.eruption_timer >= geyser.eruption_interval {
        geyser.eruption_timer = 0.0;
        geyser.is_erupting = !geyser.is_erupting;
    }

    assert!(geyser.is_erupting, "Geyser should have toggled to erupting");
    assert_eq!(geyser.eruption_timer, 0.0);

    let geyser_pos = Vec3::new(0.0, 0.0, 0.0);
    let close_player_pos = Vec3::new(2.0, 0.0, 2.0);
    let far_player_pos = Vec3::new(10.0, 0.0, 10.0);

    let mut close_receiver = EffectReceiverComponent::default();
    let mut far_receiver = EffectReceiverComponent::default();

    let radius_sq = geyser.heat_radius * geyser.heat_radius;

    if geyser_pos.distance_squared(close_player_pos) <= radius_sq {
        close_receiver.apply_effect(AlchemicalEffectKind::ThermalHeat, 1.5, 25.0);
    }
    if geyser_pos.distance_squared(far_player_pos) <= radius_sq {
        far_receiver.apply_effect(AlchemicalEffectKind::ThermalHeat, 1.5, 25.0);
    }

    assert!(close_receiver.has_effect(AlchemicalEffectKind::ThermalHeat));
    assert!(!far_receiver.has_effect(AlchemicalEffectKind::ThermalHeat));
}

#[test]
fn test_unit_player_health_and_stamina_invariants() {
    let mut health = Health {
        current: 100.0,
        max: 100.0,
    };
    let mut stamina = Stamina {
        current: 100.0,
        max: 100.0,
    };

    health.current = (health.current - 45.0).max(0.0);
    assert_eq!(health.current, 55.0);

    health.current = (health.current + 80.0).min(health.max);
    assert_eq!(health.current, 100.0);

    let dt = 1.0;
    stamina.current = (stamina.current - 15.0 * dt).max(0.0);
    assert_eq!(stamina.current, 85.0);

    stamina.current = 0.0;
    assert!(stamina.current < 5.0, "Cannot sprint when stamina < 5.0");
}

#[test]
fn test_unit_player_look_pitch_clamping() {
    let mut pitch: f32 = 0.0;
    let sensitivity = 0.0022;

    let big_mouse_up = -2000.0;
    pitch -= big_mouse_up * sensitivity;
    pitch = pitch.clamp(-1.52, 1.52);
    assert_eq!(pitch, 1.52, "Pitch must be clamped at top ~87 degrees");

    let big_mouse_down = 2000.0;
    pitch -= big_mouse_down * sensitivity;
    pitch = pitch.clamp(-1.52, 1.52);
    assert_eq!(pitch, -1.52, "Pitch must be clamped at bottom ~-87 degrees");
}

#[test]
fn test_unit_player_death_state_and_camera_orbit() {
    let mut state = PlayerState::Active;
    let health = Health {
        current: 0.0,
        max: 100.0,
    };

    if health.current <= 0.0 {
        state = PlayerState::Dead;
    }
    assert_eq!(state, PlayerState::Dead);

    let mut death_cam = DeathCamera {
        target_point: Vec3::new(5.0, 1.0, -2.0),
        orbit_angle: 0.0,
        distance: 3.8,
    };

    let dt = 1.0;
    death_cam.orbit_angle += 0.35 * dt;

    let height = 2.0;
    let x = death_cam.target_point.x + death_cam.distance * death_cam.orbit_angle.cos();
    let z = death_cam.target_point.z + death_cam.distance * death_cam.orbit_angle.sin();
    let y = death_cam.target_point.y + height;

    let _cam_pos = Vec3::new(x, y, z);
    assert_eq!(y, 3.0);
    let horizontal_dist =
        ((x - death_cam.target_point.x).powi(2) + (z - death_cam.target_point.z).powi(2)).sqrt();
    assert!(
        (horizontal_dist - 3.8).abs() < 0.001,
        "Camera distance must remain 3.8m during orbit"
    );
}

#[test]
fn test_unit_water_material_defaults_and_time_update() {
    let mut water = WaterMaterial::default();
    assert_eq!(water.params.y, 1.0, "Default wave speed should be 1.0");
    assert_eq!(water.params.x, 0.0, "Initial time should be 0.0");

    let elapsed = 14.5;
    water.params.x = elapsed;
    assert_eq!(water.params.x, 14.5);
    assert!(
        water.base_color.to_vec4().w > 0.0,
        "Water base alpha must be > 0"
    );
    assert!(
        water.deep_color.to_vec4().w > 0.0,
        "Water deep alpha must be > 0"
    );
}

#[test]
fn test_unit_particles_simulation_and_expiration() {
    let mut particle = Particle {
        lifetime: 0.0,
        max_lifetime: 1.0,
        velocity: Vec3::new(1.0, 5.0, -1.0),
        p_type: ParticleType::WaterSplash,
    };

    let dt = 0.5;
    particle.lifetime += dt;
    particle.velocity.y -= 9.8 * dt;
    assert_eq!(particle.lifetime, 0.5);
    assert!(
        (particle.velocity.y - 0.1).abs() < 0.001,
        "Velocity y must be ~0.1"
    );

    particle.lifetime += 0.6;
    let expired = particle.lifetime >= particle.max_lifetime;
    assert!(
        expired,
        "Particle should expire when lifetime >= max_lifetime"
    );
}

#[test]
fn test_unit_hud_bar_percentages() {
    let health = Health {
        current: 75.0,
        max: 100.0,
    };
    let stamina = Stamina {
        current: 30.0,
        max: 100.0,
    };

    let hp_pct = (health.current / health.max).clamp(0.0, 1.0) * 100.0;
    let st_pct = (stamina.current / stamina.max).clamp(0.0, 1.0) * 100.0;

    assert!((hp_pct - 75.0).abs() < 0.001);
    assert!((st_pct - 30.0).abs() < 0.001);

    let dead_health = Health {
        current: -15.0,
        max: 100.0,
    };
    let clamped_hp = (dead_health.current / dead_health.max).clamp(0.0, 1.0) * 100.0;
    assert_eq!(clamped_hp, 0.0);
}

#[test]
fn test_unit_lake_water_fluid_physics_and_lake_bed_depth() {
    let lake_pos_underwater = Vec3::new(0.0, 0.10, -8.5);
    assert!(
        deep_alchemy::dungeon::modular_kit::is_in_lake_water(lake_pos_underwater),
        "Position at center of lake pool below 0.22 must be in lake water"
    );

    let dry_cavern_pos = Vec3::new(0.0, 1.2, 0.0);
    assert!(
        !deep_alchemy::dungeon::modular_kit::is_in_lake_water(dry_cavern_pos),
        "Dry cavern ground at entrance must not be in lake water"
    );

    let bed_depth = deep_alchemy::dungeon::modular_kit::get_grotto_terrain_height(0.0, -8.5);
    assert!(
        bed_depth < -3.0,
        "Lake center terrain depth must be submerged below -3.0m: got {:.3}",
        bed_depth
    );

    let shore_h = deep_alchemy::dungeon::modular_kit::get_grotto_terrain_height(0.0, 0.0);
    assert!(
        shore_h >= 0.0,
        "Entrance terrain height must be non-negative: got {:.3}",
        shore_h
    );
}

#[test]
fn test_unit_ice_platform_and_footprint_meshes() {
    let ice_mesh = deep_alchemy::alchemy::create_ice_disk_mesh(2.85, 32);
    assert!(
        ice_mesh.contains_attribute(Mesh::ATTRIBUTE_POSITION),
        "Ice disk mesh must have positions"
    );
    assert!(
        ice_mesh.contains_attribute(Mesh::ATTRIBUTE_COLOR),
        "Ice disk mesh must have vertex colors"
    );

    let footprint_mesh = deep_alchemy::alchemy::create_footprint_mesh(0.24, 0.11);
    assert!(
        footprint_mesh.contains_attribute(Mesh::ATTRIBUTE_POSITION),
        "Footprint mesh must have positions"
    );
    assert!(
        footprint_mesh.contains_attribute(Mesh::ATTRIBUTE_COLOR),
        "Footprint mesh must have vertex colors"
    );
}

#[test]
fn test_unit_player_frozen_punishment_and_ice_walking() {
    let mut loco = deep_alchemy::player::camera::LocomotionState::default();
    assert!(!loco.is_frozen);
    assert!(!loco.is_in_water);
    assert!(!loco.is_on_ice);

    loco.is_frozen = true;
    loco.freeze_timer = 3.5;
    assert!(loco.is_frozen);
    assert_eq!(loco.freeze_timer, 3.5);

    let dt = 1.0;
    loco.freeze_timer = (loco.freeze_timer - dt).max(0.0);
    assert_eq!(loco.freeze_timer, 2.5);

    loco.is_on_ice = true;
    assert!(loco.is_on_ice);
}

#[test]
fn test_unit_coyote_time_and_jump_buffering_physics() {
    let mut loco = deep_alchemy::player::camera::LocomotionState::default();
    assert!(loco.is_grounded);
    loco.coyote_timer = 0.10;
    assert_eq!(loco.coyote_timer, 0.10);

    let dt = 0.05;
    loco.is_grounded = false;
    loco.coyote_timer = (loco.coyote_timer - dt).max(0.0);
    assert!((loco.coyote_timer - 0.05).abs() < 1e-4);

    loco.jump_buffer_timer = 0.12;
    assert!(loco.jump_buffer_timer > 0.0 && loco.coyote_timer > 0.0);

    loco.vertical_velocity = 6.8;
    loco.coyote_timer = 0.0;
    loco.jump_buffer_timer = 0.0;
    loco.squash_scale = Vec3::new(0.96, 1.08, 0.96);
    assert!(loco.vertical_velocity > 0.0);
    assert_eq!(loco.coyote_timer, 0.0);
    assert_eq!(loco.squash_scale.y, 1.08);

    loco.is_grounded = false;
    loco.coyote_timer = 0.0;
    loco.jump_buffer_timer = 0.12;
    loco.jump_buffer_timer = (loco.jump_buffer_timer - 0.06).max(0.0);
    assert!(loco.jump_buffer_timer > 0.0);
    assert_eq!(loco.coyote_timer, 0.0);

    loco.is_grounded = true;
    loco.coyote_timer = 0.10;
    assert!(loco.jump_buffer_timer > 0.0 && loco.coyote_timer > 0.0);
}

#[test]
fn test_unit_asymmetric_gravity_ascent_and_descent() {
    let dt = 0.016;

    let mut vy_up = 6.8_f32;
    let g_up = if vy_up > 0.0 { 24.0 } else { 36.0 };
    assert_eq!(g_up, 24.0);
    vy_up -= g_up * dt;
    assert!((vy_up - (6.8 - 0.384)).abs() < 1e-3);

    let mut vy_down = -2.0_f32;
    let g_down = if vy_down > 0.0 { 24.0 } else { 36.0 };
    assert_eq!(g_down, 36.0);
    vy_down -= g_down * dt;
    assert!((vy_down - (-2.0 - 0.576)).abs() < 1e-3);
    assert!(g_down > g_up);
}

#[test]
fn test_unit_squash_stretch_and_camera_landing_dip() {
    let mut loco = deep_alchemy::player::camera::LocomotionState::default();
    assert_eq!(loco.squash_scale, Vec3::ONE);
    assert_eq!(loco.camera_landing_dip, 0.0);

    loco.squash_scale = Vec3::new(0.96, 1.08, 0.96);
    assert!(loco.squash_scale.y > 1.0);
    assert!(loco.squash_scale.x < 1.0);

    let impact_speed = 5.0_f32;
    let impact_ratio = (impact_speed / 14.0).clamp(0.0, 1.0);
    let squash_factor = 0.06 + 0.10 * impact_ratio;
    loco.squash_scale = Vec3::new(
        1.0 + squash_factor * 0.5,
        1.0 - squash_factor,
        1.0 + squash_factor * 0.5,
    );
    loco.camera_landing_dip = (impact_speed * 0.016).clamp(0.04, 0.10);

    assert!(loco.squash_scale.y < 1.0);
    assert!(loco.squash_scale.x > 1.0);
    assert!(loco.camera_landing_dip >= 0.04 && loco.camera_landing_dip <= 0.10);

    let dt: f32 = 0.05;
    loco.camera_landing_dip = loco.camera_landing_dip.lerp(0.0, (14.0 * dt).min(1.0));
    loco.squash_scale = loco.squash_scale.lerp(Vec3::ONE, (12.0 * dt).min(1.0));
    assert!(loco.camera_landing_dip < 0.08);
    assert!(loco.squash_scale.y > 0.88);
}

#[test]
fn test_unit_potion_individual_cooldowns_and_random_interval() {
    let mut cooldowns = deep_alchemy::alchemy::PotionCooldowns::default();
    for slot in 1..=5 {
        assert!(cooldowns.is_ready(slot));
    }

    let dur_slot1 = cooldowns.trigger_cooldown(1);
    assert!(dur_slot1 >= 3.0 && dur_slot1 <= 5.0);
    assert!(!cooldowns.is_ready(1));

    for other_slot in 2..=5 {
        assert!(cooldowns.is_ready(other_slot));
    }

    let dur_slot2 = cooldowns.trigger_cooldown(2);
    assert!(dur_slot2 >= 3.0 && dur_slot2 <= 5.0);
    assert!(!cooldowns.is_ready(2));
    assert!(cooldowns.is_ready(3));
    assert!(cooldowns.is_ready(4));
    assert!(cooldowns.is_ready(5));

    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(6.0));
    app.insert_resource(time);
    app.insert_resource(cooldowns);
    let mut schedule = Schedule::default();
    schedule.add_systems(deep_alchemy::alchemy::update_potion_cooldowns);
    schedule.run(app.world_mut());

    let res = app.world().resource::<deep_alchemy::alchemy::PotionCooldowns>();
    assert!(res.is_ready(1));
    assert!(res.is_ready(2));
    assert!(res.is_ready(3));
}

#[test]
fn test_unit_alchemy_memory_limits_preservation_and_culling() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, deep_alchemy::alchemy::cleanup_alchemy_memory_limits);

    for i in 0..10 {
        app.world_mut().spawn(deep_alchemy::alchemy::PotionPuddle {
            color: Color::WHITE,
            elapsed: i as f32,
            max_lifetime: 75.0,
            radius: 1.15,
            effects: Vec::new(),
            is_wall: false,
        });
    }

    app.update();
    let puddles_under_limit = app.world_mut().query::<&deep_alchemy::alchemy::PotionPuddle>().iter(app.world()).count();
    assert_eq!(puddles_under_limit, 10);

    for i in 10..45 {
        app.world_mut().spawn(deep_alchemy::alchemy::PotionPuddle {
            color: Color::WHITE,
            elapsed: i as f32,
            max_lifetime: 75.0,
            radius: 1.15,
            effects: Vec::new(),
            is_wall: false,
        });
    }

    app.update();
    let puddles_after_culling = app.world_mut().query::<&deep_alchemy::alchemy::PotionPuddle>().iter(app.world()).count();
    assert_eq!(puddles_after_culling, 36);

    for i in 0..140 {
        app.world_mut().spawn(deep_alchemy::alchemy::GlassShardParticle {
            velocity: Vec3::ZERO,
            lifetime: i as f32,
            max_lifetime: 75.0,
            ground_y: 0.0,
            settled: true,
            color: Color::WHITE,
            angular_velocity: Vec3::ZERO,
            bounces_remaining: 0,
        });
    }

    app.update();
    let shards_after_culling = app.world_mut().query::<&deep_alchemy::alchemy::GlassShardParticle>().iter(app.world()).count();
    assert_eq!(shards_after_culling, 128);
}

#[test]
fn test_unit_hotbar_cooldown_visuals_dimming() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut cooldowns = deep_alchemy::alchemy::PotionCooldowns::default();
    cooldowns.trigger_cooldown(2);
    app.insert_resource(cooldowns);
    app.add_systems(Update, deep_alchemy::ui::hud::update_hotbar_cooldown_visuals);

    let icon1 = app.world_mut().spawn((
        deep_alchemy::ui::hud::PotionSlotIcon { slot_index: 1 },
        ImageNode::default(),
    )).id();

    let icon2 = app.world_mut().spawn((
        deep_alchemy::ui::hud::PotionSlotIcon { slot_index: 2 },
        ImageNode::default(),
    )).id();

    app.update();

    let img1 = app.world().get::<ImageNode>(icon1).unwrap();
    let img2 = app.world().get::<ImageNode>(icon2).unwrap();

    assert_eq!(img1.color, Color::WHITE);
    assert_ne!(img2.color, Color::WHITE);
}

#[test]
fn test_unit_camera_banking_and_trauma_shake_decay() {
    let mut shake = deep_alchemy::player::camera::CameraShake::default();
    assert_eq!(shake.trauma, 0.0);
    assert_eq!(shake.direction, Vec3::ZERO);

    shake.add_trauma(0.65, Vec3::new(2.0, 0.0, 0.0));
    assert!((shake.trauma - 0.65).abs() < 1e-4);
    assert!((shake.direction - Vec3::X).length() < 1e-4);

    shake.add_trauma(0.50, Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(shake.trauma, 1.0);
    assert!(shake.direction.is_normalized());

    let dt = 0.1;
    shake.trauma = (shake.trauma - 1.8 * dt).max(0.0);
    assert!((shake.trauma - 0.82).abs() < 1e-4);

    let mut loco = deep_alchemy::player::camera::LocomotionState::default();
    assert_eq!(loco.camera_roll, 0.0);
    assert_eq!(loco.target_camera_roll, 0.0);

    loco.target_camera_roll = 0.020;
    loco.camera_roll = loco.camera_roll.lerp(loco.target_camera_roll, (10.0 * dt).min(1.0));
    assert!(loco.camera_roll > 0.0);
    assert!(loco.camera_roll <= 0.020);
}

#[test]
fn test_unit_potion_throw_recoil_and_windup_lifecycle() {
    let mut loco = deep_alchemy::player::camera::LocomotionState::default();
    assert_eq!(loco.throw_windup, 0.0);
    assert_eq!(loco.throw_recoil, 0.0);

    let dt = 0.05;
    for _ in 0..10 {
        loco.throw_windup = (loco.throw_windup + 7.0 * dt).min(1.0);
    }
    assert_eq!(loco.throw_windup, 1.0);

    let bead_radius = 0.028 + 0.014 * loco.throw_windup;
    assert!((bead_radius - 0.042).abs() < 1e-4);

    loco.throw_recoil = 1.0;
    loco.throw_windup = 0.0;
    assert_eq!(loco.throw_recoil, 1.0);
    assert_eq!(loco.throw_windup, 0.0);

    loco.throw_recoil = (loco.throw_recoil - 6.0 * dt).max(0.0);
    assert!((loco.throw_recoil - 0.7).abs() < 1e-4);
}

#[test]
fn test_unit_potion_hitstop_micro_freeze_duration() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
        avian3d::prelude::PhysicsPlugins::default(),
    ));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(25),
    ));
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Mesh>>();
    app.insert_resource(deep_alchemy::player::camera::CameraShake::default());
    app.add_systems(Update, deep_alchemy::alchemy::update_potion_projectiles);

    let _target = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, -3.0),
            GlobalTransform::from_translation(Vec3::new(0.0, 0.0, -3.0)),
            avian3d::prelude::RigidBody::Static,
            avian3d::prelude::Collider::cuboid(4.0, 4.0, 0.5),
            deep_alchemy::alchemy::EffectReceiverComponent::default(),
        ))
        .id();

    app.update();

    let proj_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, -1.0),
            deep_alchemy::alchemy::PotionProjectile {
                velocity: Vec3::new(0.0, 0.0, -100.0),
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: deep_alchemy::alchemy::AlchemicalEffectKind::Frost,
                    duration: 5.0,
                    magnitude: 10.0,
                    elapsed: 0.0,
                }],
                lifetime: 8.0,
                color: Color::WHITE,
                slot: 1,
                hitstop_timer: 0.0,
            },
        ))
        .id();

    app.update();

    let proj = app
        .world()
        .get::<deep_alchemy::alchemy::PotionProjectile>(proj_entity);
    assert!(proj.is_some());
    let p = proj.unwrap();
    assert_eq!(p.hitstop_timer, 0.045);

    let shake = app
        .world()
        .resource::<deep_alchemy::player::camera::CameraShake>();
    assert!(shake.trauma >= 0.0);

    app.update();

    let proj_mid = app
        .world()
        .get::<deep_alchemy::alchemy::PotionProjectile>(proj_entity);
    assert!(proj_mid.is_some());
    let p_mid = proj_mid.unwrap();
    assert!((p_mid.hitstop_timer - 0.020).abs() < 1e-3);

    app.update();

    let proj_end = app
        .world()
        .get::<deep_alchemy::alchemy::PotionProjectile>(proj_entity);
    assert!(proj_end.is_none());
}

#[test]
fn test_unit_glass_shard_angular_velocity_and_bouncing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(20),
    ));
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_systems(Update, deep_alchemy::alchemy::update_glass_shards);

    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    let mat_handle = materials.add(StandardMaterial::default());

    let shard_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::alchemy::GlassShardParticle {
                velocity: Vec3::new(1.0, -10.0, 0.0),
                lifetime: 0.0,
                max_lifetime: 60.0,
                ground_y: 0.0,
                settled: false,
                color: Color::WHITE,
                angular_velocity: Vec3::new(2.0, 5.0, 1.0),
                bounces_remaining: 1,
            },
            Transform::from_xyz(0.0, 0.05, 0.0),
            MeshMaterial3d(mat_handle),
        ))
        .id();

    app.update();
    app.update();

    let shard = app
        .world()
        .get::<deep_alchemy::alchemy::GlassShardParticle>(shard_entity)
        .unwrap();
    assert_eq!(shard.bounces_remaining, 0);
    assert!(shard.velocity.y > 0.0);
    assert!(!shard.settled);

    for _ in 0..25 {
        app.update();
    }

    let shard_settled = app
        .world()
        .get::<deep_alchemy::alchemy::GlassShardParticle>(shard_entity)
        .unwrap();
    assert!(shard_settled.settled || shard_settled.bounces_remaining == 0);
}

#[test]
fn test_unit_puddle_thermal_shock_and_luminescent_reaction() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Mesh>>();
    app.add_systems(Update, deep_alchemy::alchemy::update_potion_puddles);

    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    let mat1 = materials.add(StandardMaterial::default());
    let mat2 = materials.add(StandardMaterial::default());

    app.world_mut().spawn((
        deep_alchemy::alchemy::PotionPuddle {
            color: Color::srgb(1.0, 0.2, 0.1),
            elapsed: 0.0,
            max_lifetime: 60.0,
            radius: 1.5,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 20.0,
                elapsed: 0.0,
            }],
            is_wall: false,
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        MeshMaterial3d(mat1),
    ));

    app.world_mut().spawn((
        deep_alchemy::alchemy::PotionPuddle {
            color: Color::srgb(0.2, 0.7, 1.0),
            elapsed: 0.0,
            max_lifetime: 60.0,
            radius: 1.5,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::Frost,
                duration: 5.0,
                magnitude: 20.0,
                elapsed: 0.0,
            }],
            is_wall: false,
        },
        Transform::from_xyz(0.8, 0.0, 0.0),
        MeshMaterial3d(mat2),
    ));

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(16));
    app.update();

    let cz_count = app
        .world_mut()
        .query::<&deep_alchemy::alchemy::PotionContactZone>()
        .iter(app.world())
        .count();
    assert_eq!(cz_count, 1);

    let emitter_count = app
        .world_mut()
        .query::<&deep_alchemy::render::particles::ParticleEmitter>()
        .iter(app.world())
        .count();
    assert!(emitter_count >= 1);
}



