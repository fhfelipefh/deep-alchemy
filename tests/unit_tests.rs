use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use deep_alchemy::{
    alchemy::{AlchemicalEffectKind, EffectReceiverComponent, ThermalGeyser},
    player::{
        camera::{Health, PlayerState, Stamina},
        ragdoll::DeathCamera,
    },
    render::{
        particles::{Particle, ParticleType},
        planar_reflection::{
            create_planar_reflection_image, sync_planar_reflection_to_water,
            update_planar_reflection_camera, PlanarReflectionCamera, PlanarReflectionTexture,
        },
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
fn test_unit_water_planar_reflection_parameters_and_bounds() {
    let water = WaterMaterial::default();
    assert_eq!(water.reflection_params.x, 0.18, "Water height should default to 0.18");
    assert!((water.reflection_params.y - 0.045).abs() < 1e-3, "Distortion strength should be ~0.045");
    assert!((water.reflection_params.z - 0.85).abs() < 1e-3, "Edge foam threshold should be ~0.85");
    assert!((water.reflection_params.w - 0.70).abs() < 1e-3, "Reflection blend should be ~0.70");
    assert!(water.reflection_texture.is_none(), "Initial reflection texture should be None fallback");
}

#[test]
fn test_unit_planar_reflection_camera_mirrored_across_water_plane() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
    app.init_asset::<Image>();

    let image_handle = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(create_planar_reflection_image(512, 512))
    };

    app.insert_resource(PlanarReflectionTexture {
        texture: image_handle,
        water_height: 0.18,
    });

    let player_tf = Transform::from_xyz(3.0, 2.0, -5.0)
        .with_rotation(Quat::from_euler(EulerRot::YXZ, 0.8, -0.4, 0.1));

    app.world_mut().spawn((
        deep_alchemy::player::camera::PlayerCamera,
        player_tf,
        GlobalTransform::from(player_tf),
    ));

    let refl_cam = app.world_mut().spawn((
        PlanarReflectionCamera,
        Transform::default(),
    )).id();

    app.add_systems(Update, update_planar_reflection_camera);
    app.update();

    let refl_tf = app.world().get::<Transform>(refl_cam).unwrap();
    let expected_y = 2.0 * 0.18 - 2.0;
    assert!((refl_tf.translation.y - expected_y).abs() < 1e-3, "Planar reflection Y must be inverted across 0.18");
    assert_eq!(refl_tf.translation.x, 3.0);
    assert_eq!(refl_tf.translation.z, -5.0);

    let (yaw, pitch, roll) = refl_tf.rotation.to_euler(EulerRot::YXZ);
    assert!((yaw - 0.8).abs() < 1e-3, "Yaw must match player look direction");
    assert!((pitch - 0.4).abs() < 1e-3, "Pitch must be inverted");
    assert!((roll - (-0.1)).abs() < 1e-3, "Roll must be inverted");
}

#[test]
fn test_unit_planar_reflection_lake_sync_and_fallback() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
    app.init_asset::<Image>();
    app.init_asset::<WaterMaterial>();

    let image_handle = {
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        images.add(create_planar_reflection_image(512, 512))
    };

    app.insert_resource(PlanarReflectionTexture {
        texture: image_handle.clone(),
        water_height: 0.18,
    });

    let water_mat_handle = {
        let mut mats = app.world_mut().resource_mut::<Assets<WaterMaterial>>();
        mats.add(WaterMaterial::default())
    };

    app.world_mut().spawn((
        deep_alchemy::dungeon::modular_kit::LakeWaterPool,
        MeshMaterial3d(water_mat_handle.clone()),
    ));

    app.add_systems(Update, sync_planar_reflection_to_water);
    app.update();

    let mats = app.world().resource::<Assets<WaterMaterial>>();
    let mat = mats.get(&water_mat_handle).unwrap();
    assert_eq!(mat.reflection_texture, Some(image_handle), "Water material must receive planar reflection texture");
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

    let irregular_block = deep_alchemy::alchemy::create_irregular_ice_block_mesh(2.65, 1.45, 24);
    assert!(
        irregular_block.contains_attribute(Mesh::ATTRIBUTE_POSITION),
        "Irregular ice block mesh must have positions"
    );
    assert!(
        irregular_block.contains_attribute(Mesh::ATTRIBUTE_NORMAL),
        "Irregular ice block mesh must have normals"
    );
    assert!(
        irregular_block.contains_attribute(Mesh::ATTRIBUTE_UV_0),
        "Irregular ice block mesh must have uvs"
    );
    assert!(
        irregular_block.contains_attribute(Mesh::ATTRIBUTE_COLOR),
        "Irregular ice block mesh must have vertex colors"
    );
    assert!(
        irregular_block.indices().is_some(),
        "Irregular ice block mesh must have indices"
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
    assert!((loco.coyote_timer - 0.05).abs() < 1e-3);

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
    assert!((shake.trauma - 0.65).abs() < 1e-3);
    assert!((shake.direction - Vec3::X).length() < 1e-3);

    shake.add_trauma(0.50, Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(shake.trauma, 1.0);
    assert!(shake.direction.is_normalized());

    let dt = 0.1;
    shake.trauma = (shake.trauma - 1.8 * dt).max(0.0);
    assert!((shake.trauma - 0.82).abs() < 1e-3);

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
    assert!((bead_radius - 0.042).abs() < 1e-3);

    loco.throw_recoil = 1.0;
    loco.throw_windup = 0.0;
    assert_eq!(loco.throw_recoil, 1.0);
    assert_eq!(loco.throw_windup, 0.0);

    loco.throw_recoil = (loco.throw_recoil - 6.0 * dt).max(0.0);
    assert!((loco.throw_recoil - 0.7).abs() < 1e-3);
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
                entered_water: false,
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

#[test]
fn test_unit_grass_bending_spring_damping_and_player_contact() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(20),
    ));
    app.add_systems(Update, deep_alchemy::dungeon::modular_kit::update_grass_bending);

    let player_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::from_translation(Vec3::ZERO),
        ))
        .id();

    let grass_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::modular_kit::TallGrassClump,
            deep_alchemy::dungeon::modular_kit::GrassBending {
                base_rotation: Quat::IDENTITY,
                current_bend: Vec2::ZERO,
                bend_velocity: Vec2::ZERO,
            },
            Transform::from_xyz(0.35, 0.0, 0.0),
            GlobalTransform::from_translation(Vec3::new(0.35, 0.0, 0.0)),
        ))
        .id();

    app.update();
    app.update();

    let grass = app
        .world()
        .get::<deep_alchemy::dungeon::modular_kit::GrassBending>(grass_entity)
        .unwrap();
    assert!(grass.current_bend.x > 0.0);

    let mut player_tf = app.world_mut().get_mut::<Transform>(player_entity).unwrap();
    player_tf.translation = Vec3::new(10.0, 0.0, 0.0);
    let mut player_gt = app.world_mut().get_mut::<GlobalTransform>(player_entity).unwrap();
    *player_gt = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));

    for _ in 0..40 {
        app.update();
    }

    let grass_returned = app
        .world()
        .get::<deep_alchemy::dungeon::modular_kit::GrassBending>(grass_entity)
        .unwrap();
    assert!(grass_returned.current_bend.length() < 0.08);
}

#[test]
fn test_unit_water_ripple_radial_expansion_and_fade() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(50),
    ));
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_systems(Update, deep_alchemy::dungeon::modular_kit::update_water_ripples);

    let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
    let mat_handle = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.96, 1.0, 0.8),
        ..default()
    });

    let ripple_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::modular_kit::WaterRipple {
                elapsed: 0.0,
                max_lifetime: 1.0,
                initial_radius: 0.2,
                max_radius: 1.6,
                initial_alpha: 0.8,
            },
            Transform::from_xyz(0.0, 0.20, 0.0).with_scale(Vec3::splat(0.2)),
            MeshMaterial3d(mat_handle.clone()),
        ))
        .id();

    app.update();
    app.update();

    let tf = app.world().get::<Transform>(ripple_entity).unwrap();
    assert!(tf.scale.x > 0.2);

    for _ in 0..25 {
        app.update();
    }

    assert!(app.world().get_entity(ripple_entity).is_err());
}

#[test]
fn test_unit_bioluminescent_mushroom_respiration_and_surge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(50),
    ));
    app.add_systems(Update, deep_alchemy::dungeon::modular_kit::update_bioluminescent_mushrooms);

    let shroom_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::modular_kit::BioluminescentMushroom {
                base_intensity: 1000.0,
                base_color: Color::srgb(0.12, 0.96, 0.88),
                phase: 0.0,
                surge_timer: 0.0,
            },
            PointLight {
                intensity: 1000.0,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::from_translation(Vec3::ZERO),
        ))
        .id();

    app.update();
    app.update();

    let shroom = app
        .world()
        .get::<deep_alchemy::dungeon::modular_kit::BioluminescentMushroom>(shroom_entity)
        .unwrap();
    assert_eq!(shroom.surge_timer, 0.0);
    assert!(shroom.phase > 0.0);

    let _player_close = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(0.8, 0.0, 0.0),
            GlobalTransform::from_translation(Vec3::new(0.8, 0.0, 0.0)),
        ))
        .id();

    app.update();

    let shroom_surging = app
        .world()
        .get::<deep_alchemy::dungeon::modular_kit::BioluminescentMushroom>(shroom_entity)
        .unwrap();
    assert!(shroom_surging.surge_timer > 0.0);

    let light = app.world().get::<PointLight>(shroom_entity).unwrap();
    assert!(light.intensity > 0.0);
}

#[test]
fn test_unit_hotbar_pop_animation_and_spring_damping() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_secs_f32(0.016)));
    app.init_resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>();
    app.add_systems(
        Update,
        (
            deep_alchemy::ui::hud::update_hotbar_selection_visuals,
            deep_alchemy::ui::hud::update_hotbar_pop_animations,
        ),
    );

    let slot_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::ui::hud::HotbarSlotUi { slot_index: 2 },
            deep_alchemy::ui::hud::HotbarPopAnimation::default(),
            Node::default(),
            Transform::default(),
            BorderColor(deep_alchemy::ui::hud::COLOR_BORDER_GOLD),
        ))
        .id();

    app.update();
    app.update();

    let anim_initial = app
        .world()
        .get::<deep_alchemy::ui::hud::HotbarPopAnimation>(slot_entity)
        .unwrap();
    assert_eq!(anim_initial.scale, 1.0);

    app.world_mut().resource_mut::<deep_alchemy::ui::hud::ActiveHotbarSlot>().0 = 2;
    app.update();

    let anim_popped = *app
        .world()
        .get::<deep_alchemy::ui::hud::HotbarPopAnimation>(slot_entity)
        .unwrap();
    assert!(anim_popped.scale > 1.0);
    assert!(anim_popped.flash_timer > 0.0);
    let popped_scale = anim_popped.scale;

    for _ in 0..10 {
        app.update();
    }

    let anim_settling = app
        .world()
        .get::<deep_alchemy::ui::hud::HotbarPopAnimation>(slot_entity)
        .unwrap();
    assert!(anim_settling.scale < popped_scale);
}

#[test]
fn test_unit_critical_health_heartbeat_pulse_and_vignette() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_secs_f32(0.016)));
    app.add_systems(Update, deep_alchemy::ui::hud::update_critical_health_pulse);

    let player_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::Health {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    let heart_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::ui::hud::HealthHeartFill { index: 0 },
            Transform::default(),
        ))
        .id();

    let vignette_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::ui::hud::CriticalVignette,
            Visibility::Hidden,
            BorderColor(Color::NONE),
            BackgroundColor(Color::NONE),
        ))
        .id();

    app.update();
    app.update();

    let tf_normal = app.world().get::<Transform>(heart_entity).unwrap();
    assert_eq!(tf_normal.scale, Vec3::ONE);
    let vis_normal = app.world().get::<Visibility>(vignette_entity).unwrap();
    assert_eq!(*vis_normal, Visibility::Hidden);

    app.world_mut()
        .get_mut::<deep_alchemy::player::camera::Health>(player_entity)
        .unwrap()
        .current = 20.0;

    for _ in 0..15 {
        app.update();
    }

    let vis_crit = app.world().get::<Visibility>(vignette_entity).unwrap();
    assert_eq!(*vis_crit, Visibility::Inherited);
}

#[test]
fn test_unit_ballistic_aim_dynamic_droplets_and_splash_ring() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(avian3d::prelude::PhysicsPlugins::default());
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_secs_f32(0.016)));
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>();
    app.init_resource::<deep_alchemy::alchemy::PotionCooldowns>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.world_mut().spawn((
        deep_alchemy::player::camera::PlayerCamera,
        Camera::default(),
        Transform::from_xyz(0.0, 1.6, 0.0),
        GlobalTransform::from_translation(Vec3::new(0.0, 1.6, 0.0)),
    ));

    app.world_mut().spawn((
        deep_alchemy::player::camera::Player,
        deep_alchemy::player::camera::PlayerState::Active,
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::from_translation(Vec3::ZERO),
    ));

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Right);

    let mut schedule = Schedule::default();
    schedule.add_systems(deep_alchemy::alchemy::update_potion_aiming);
    schedule.run(app.world_mut());

    let mut indicators_query = app
        .world_mut()
        .query::<(&deep_alchemy::alchemy::TrajectoryAimIndicator, &Transform)>();
    let indicator_count = indicators_query.iter(app.world()).count();
    assert!(indicator_count > 10);
}

#[test]
fn test_unit_flammable_combustion_and_despawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_systems(Update, deep_alchemy::alchemy::update_flammable_combustion);

    let flammable_ent = app
        .world_mut()
        .spawn((
            Name::new("TestFlammableProp"),
            Transform::from_xyz(1.0, 0.0, 1.0),
            deep_alchemy::alchemy::Flammable::with_duration(0.3),
        ))
        .id();

    let flammable_before = app.world().get::<deep_alchemy::alchemy::Flammable>(flammable_ent);
    assert!(flammable_before.is_some());
    assert!(!flammable_before.unwrap().is_burning);

    app.world_mut()
        .get_mut::<deep_alchemy::alchemy::Flammable>(flammable_ent)
        .unwrap()
        .ignite();

    app.update();
    app.update();

    let flammable_burning = app.world().get::<deep_alchemy::alchemy::Flammable>(flammable_ent);
    assert!(flammable_burning.is_some());
    assert!(flammable_burning.unwrap().is_burning);
    assert!(flammable_burning.unwrap().elapsed > 0.0);

    for _ in 0..15 {
        app.update();
    }

    let flammable_after = app.world().get_entity(flammable_ent);
    assert!(flammable_after.is_err());
}

#[test]
fn test_unit_fire_potion_impact_ignites_flammable() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
        avian3d::prelude::PhysicsPlugins::default(),
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(25)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.insert_resource(deep_alchemy::player::camera::CameraShake::default());
    app.add_systems(Update, deep_alchemy::alchemy::update_potion_projectiles);

    let flammable_ent = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.0, 0.0, 1.0),
            deep_alchemy::alchemy::Flammable::with_duration(2.0),
        ))
        .id();

    let _proj_ent = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.2, 0.1, 1.2),
            deep_alchemy::alchemy::PotionProjectile {
                velocity: Vec3::ZERO,
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                    duration: 5.0,
                    magnitude: 20.0,
                    elapsed: 0.0,
                }],
                lifetime: 8.0,
                color: Color::srgb(0.95, 0.12, 0.16),
                slot: 2,
                hitstop_timer: 0.001,
                entered_water: false,
            },
        ))
        .id();

    app.update();
    app.update();

    let flammable = app.world().get::<deep_alchemy::alchemy::Flammable>(flammable_ent);
    assert!(flammable.is_some());
    assert!(flammable.unwrap().is_burning);
}

#[test]
fn test_unit_menu_start_button_click_does_not_fire_potion_and_pauses_game() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::state::app::StatesPlugin,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_state::<deep_alchemy::ui::GameState>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<deep_alchemy::alchemy::PotionCooldowns>();
    app.add_systems(
        OnEnter(deep_alchemy::ui::GameState::MainMenu),
        deep_alchemy::ui::menu::on_enter_menu_state,
    );
    app.add_systems(
        OnEnter(deep_alchemy::ui::GameState::InGame),
        deep_alchemy::ui::menu::on_enter_ingame_state,
    );
    app.add_systems(Update, deep_alchemy::alchemy::spawn_potion_projectile);

    app.world_mut().spawn((
        deep_alchemy::player::camera::Player,
        deep_alchemy::player::camera::PlayerState::Active,
        deep_alchemy::player::camera::LocomotionState::default(),
    ));

    app.world_mut().spawn((
        deep_alchemy::player::camera::PlayerCamera,
        Transform::from_xyz(0.0, 1.75, 0.0),
        GlobalTransform::from_translation(Vec3::new(0.0, 1.75, 0.0)),
    ));

    app.update();

    let state = app
        .world()
        .resource::<State<deep_alchemy::ui::GameState>>();
    assert_eq!(*state.get(), deep_alchemy::ui::GameState::MainMenu);

    let virtual_time = app.world().resource::<Time<Virtual>>();
    assert!(virtual_time.is_paused());

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    let proj_count_menu = app
        .world_mut()
        .query::<&deep_alchemy::alchemy::PotionProjectile>()
        .iter(app.world())
        .count();
    assert_eq!(proj_count_menu, 0);

    app.world_mut()
        .resource_mut::<NextState<deep_alchemy::ui::GameState>>()
        .set(deep_alchemy::ui::GameState::InGame);
    app.update();

    let state_ingame = app
        .world()
        .resource::<State<deep_alchemy::ui::GameState>>();
    assert_eq!(*state_ingame.get(), deep_alchemy::ui::GameState::InGame);

    let virtual_time_ingame = app.world().resource::<Time<Virtual>>();
    assert!(!virtual_time_ingame.is_paused());

    let proj_count_transition = app
        .world_mut()
        .query::<&deep_alchemy::alchemy::PotionProjectile>()
        .iter(app.world())
        .count();
    assert_eq!(proj_count_transition, 0);

    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();

    let proj_count_fire = app
        .world_mut()
        .query::<&deep_alchemy::alchemy::PotionProjectile>()
        .iter(app.world())
        .count();
    assert_eq!(proj_count_fire, 1);
}

#[test]
fn test_unit_lissajous_head_bob_and_weight_shifting_roll() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, deep_alchemy::player::controller::player_camera_effects);

    let mut locomotion = deep_alchemy::player::camera::LocomotionState::default();
    locomotion.head_bob_timer = std::f32::consts::FRAC_PI_2;
    locomotion.head_bob_intensity = 1.0;

    app.world_mut().spawn((
        deep_alchemy::player::camera::Player,
        locomotion,
    ));

    let camera = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::PlayerCamera,
            Transform::from_xyz(0.0, 0.55, 0.0),
            Projection::Perspective(PerspectiveProjection {
                fov: 75.0_f32.to_radians(),
                ..default()
            }),
        ))
        .id();

    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_millis(16));
    app.insert_resource(time);
    app.update();

    let cam_tf = app.world().get::<Transform>(camera).unwrap();
    assert!(
        cam_tf.translation.x > 0.015,
        "Lissajous bobbing at PI/2 should deflect camera laterally (X > 0)"
    );
    let (_, _, roll) = cam_tf.rotation.to_euler(EulerRot::YXZ);
    assert!(
        roll.abs() > 0.003 && roll.abs() < 0.015,
        "Weight shifting roll should tilt camera by ~0.4 deg (0.007 rad)"
    );
}

#[test]
fn test_unit_weapon_sway_spring_damper_dynamics() {
    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_millis(50));
    app.insert_resource(time);
    app.add_systems(
        Update,
        deep_alchemy::player::controller::update_first_person_potion_sway,
    );

    let mut locomotion = deep_alchemy::player::camera::LocomotionState::default();
    locomotion.mouse_look_delta = Vec2::new(40.0, -25.0);

    let player = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::PlayerState::Active,
            locomotion,
        ))
        .id();

    let potion = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::FirstPersonHeldPotion::default(),
            Transform::from_xyz(0.24, -0.20, -0.42),
            Visibility::default(),
        ))
        .id();

    app.update();

    let potion_sway = app
        .world()
        .get::<deep_alchemy::player::camera::FirstPersonHeldPotion>(potion)
        .unwrap();
    assert!(
        potion_sway.sway_velocity.x < 0.0,
        "Mouse right movement must induce opposing lateral sway velocity on the held weapon"
    );
    assert!(
        potion_sway.sway_offset.x < 0.0,
        "Held weapon sway position offset must lag opposite to camera turn"
    );

    app.world_mut()
        .get_mut::<deep_alchemy::player::camera::LocomotionState>(player)
        .unwrap()
        .mouse_look_delta = Vec2::ZERO;

    for _ in 0..60 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        app.update();
    }

    let settled_sway = app
        .world()
        .get::<deep_alchemy::player::camera::FirstPersonHeldPotion>(potion)
        .unwrap();
    assert!(
        settled_sway.sway_offset.length() < 0.005,
        "Spring-damper must restore weapon to center equilibrium when movement stops"
    );
}

#[test]
fn test_unit_ragdoll_death_transition_and_viewmodel_hidden() {
    let mut app = App::new();
    app.add_systems(
        Update,
        deep_alchemy::player::ragdoll::check_player_death,
    );

    let camera = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::PlayerCamera,
            Transform::from_xyz(0.0, 1.75, 0.0),
        ))
        .id();

    let head = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::HeadHatPart,
            bevy::render::view::RenderLayers::layer(1),
        ))
        .id();

    let limb_arm = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::RagdollLimb {
                limb_name: "RightArm".to_string(),
            },
            Transform::from_xyz(0.35, 1.4, 0.0),
        ))
        .id();

    let held_potion = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::FirstPersonHeldPotion::default(),
            Visibility::Inherited,
        ))
        .id();

    let player = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::PlayerState::Active,
            deep_alchemy::player::camera::Health {
                current: 0.0,
                max: 100.0,
            },
            Transform::from_xyz(0.0, 1.2, 0.0),
        ))
        .id();

    app.update();

    let state = app
        .world()
        .get::<deep_alchemy::player::camera::PlayerState>(player)
        .unwrap();
    assert_eq!(*state, deep_alchemy::player::camera::PlayerState::Dead);

    let player_rb = app.world().get::<avian3d::prelude::RigidBody>(player);
    assert!(player_rb.is_some());
    assert_eq!(*player_rb.unwrap(), avian3d::prelude::RigidBody::Dynamic);

    let arm_rb = app.world().get::<avian3d::prelude::RigidBody>(limb_arm);
    assert!(arm_rb.is_some());
    assert_eq!(*arm_rb.unwrap(), avian3d::prelude::RigidBody::Dynamic);

    let potion_vis = app.world().get::<Visibility>(held_potion).unwrap();
    assert_eq!(*potion_vis, Visibility::Hidden);

    let head_layers = app
        .world()
        .get::<bevy::render::view::RenderLayers>(head)
        .unwrap();
    assert!(head_layers.intersects(&bevy::render::view::RenderLayers::layer(0)));

    let death_cam = app
        .world()
        .get::<deep_alchemy::player::ragdoll::DeathCamera>(camera);
    assert!(death_cam.is_some());
}

#[test]
fn test_unit_gamepad_hotbar_cycling_r1_l1() {
    let mut app = App::new();
    app.init_resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>();
    app.add_event::<bevy::input::mouse::MouseWheel>();
    app.add_systems(Update, deep_alchemy::ui::hud::handle_hotbar_input);

    let mut gamepad = Gamepad::default();
    gamepad.digital_mut().press(GamepadButton::RightTrigger);
    let gp_entity = app.world_mut().spawn(gamepad).id();

    app.update();
    assert_eq!(app.world().resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>().0, 2);

    {
        let mut gp = app.world_mut().get_mut::<Gamepad>(gp_entity).unwrap();
        gp.digital_mut().clear();
        gp.digital_mut().press(GamepadButton::LeftTrigger);
    }
    app.update();
    assert_eq!(app.world().resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>().0, 1);
}

#[test]
fn test_unit_gamepad_aiming_l2_and_throwing_r2() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_plugins(avian3d::prelude::PhysicsPlugins::default());
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<deep_alchemy::alchemy::PotionCooldowns>();
    app.init_resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>();

    let mut gamepad = Gamepad::default();
    gamepad.digital_mut().press(GamepadButton::LeftTrigger2);
    let gp_entity = app.world_mut().spawn(gamepad).id();

    app.world_mut().spawn((
        deep_alchemy::player::camera::Player,
        deep_alchemy::player::camera::PlayerState::Active,
        deep_alchemy::player::camera::LocomotionState::default(),
    ));

    app.world_mut().spawn((
        deep_alchemy::player::camera::PlayerCamera,
        Transform::from_xyz(0.0, 1.75, 0.0),
        GlobalTransform::from_translation(Vec3::new(0.0, 1.75, 0.0)),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems((
        deep_alchemy::alchemy::update_potion_aiming,
        deep_alchemy::alchemy::spawn_potion_projectile,
    ));
    schedule.run(app.world_mut());

    let mut indicator_query = app.world_mut().query::<&deep_alchemy::alchemy::TrajectoryAimIndicator>();
    let count = indicator_query.iter(app.world()).count();
    assert!(count >= 15);

    {
        let mut gp = app.world_mut().get_mut::<Gamepad>(gp_entity).unwrap();
        gp.digital_mut().clear();
        gp.digital_mut().press(GamepadButton::RightTrigger2);
    }
    schedule.run(app.world_mut());

    let mut proj_query = app.world_mut().query::<&deep_alchemy::alchemy::PotionProjectile>();
    assert_eq!(proj_query.iter(app.world()).count(), 1);
}

#[test]
fn test_unit_gamepad_analog_movement_and_look() {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.add_event::<bevy::input::mouse::MouseMotion>();

    let player = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::PlayerState::Active,
            deep_alchemy::player::camera::LocomotionState::default(),
            Transform::from_xyz(0.0, 1.2, 0.0),
            deep_alchemy::player::camera::Stamina {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    let _camera = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::PlayerCamera,
            Transform::from_xyz(0.0, 1.75, 0.0),
        ))
        .id();

    let mut gamepad = Gamepad::default();
    gamepad.analog_mut().set(GamepadAxis::LeftStickY, 1.0);
    gamepad.analog_mut().set(GamepadAxis::RightStickX, 1.0);
    app.world_mut().spawn(gamepad);

    let mut schedule = Schedule::default();
    schedule.add_systems((
        deep_alchemy::player::controller::player_movement,
        deep_alchemy::player::controller::player_look,
    ));

    for _ in 0..10 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        schedule.run(app.world_mut());
    }

    let player_tf = *app.world().get::<Transform>(player).unwrap();
    assert!(player_tf.translation.z.abs() > 0.01);
    let (yaw, _, _) = player_tf.rotation.to_euler(EulerRot::YXZ);
    assert!(yaw.abs() > 0.001);

    let loco = app.world().get::<deep_alchemy::player::camera::LocomotionState>(player).unwrap();
    assert!(loco.mouse_look_delta.x.abs() > 0.1);
}

#[test]
fn test_unit_gamepad_pause_toggle_start_button() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.init_state::<deep_alchemy::ui::menu::GameState>();
    app.init_resource::<deep_alchemy::ui::menu::MenuNavigation>();
    app.init_resource::<deep_alchemy::ui::menu::GameSettings>();
    app.add_systems(Update, deep_alchemy::ui::menu::handle_global_shortcuts);

    app.world_mut()
        .resource_mut::<NextState<deep_alchemy::ui::menu::GameState>>()
        .set(deep_alchemy::ui::menu::GameState::InGame);
    app.update();

    let mut gamepad = Gamepad::default();
    gamepad.digital_mut().press(GamepadButton::Start);
    app.world_mut().spawn(gamepad);

    app.update();
    app.update();
    let state = app.world().resource::<State<deep_alchemy::ui::menu::GameState>>();
    assert_eq!(*state.get(), deep_alchemy::ui::menu::GameState::MainMenu);
}

#[test]
fn test_unit_potion_fluid_material_and_sloshing_dynamics() {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>();
    app.init_resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>();
    app.init_resource::<deep_alchemy::alchemy::PotionCooldowns>();

    let _player = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::PlayerState::Active,
            deep_alchemy::player::camera::LocomotionState {
                mouse_look_delta: Vec2::new(12.0, -4.0),
                ..default()
            },
            Transform::from_xyz(0.0, 1.2, 0.0),
        ))
        .id();

    let _camera = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::PlayerCamera,
            Transform::from_rotation(Quat::from_rotation_y(0.25)),
        ))
        .id();

    let mut materials = app
        .world_mut()
        .resource_mut::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>();
    let mat_handle = materials.add(deep_alchemy::render::potion_fluid_material::PotionFluidMaterial::default());

    let potion_liquid = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::HeldPotionLiquid,
            MeshMaterial3d(mat_handle.clone()),
            deep_alchemy::render::potion_fluid_material::PotionSloshState::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(deep_alchemy::render::potion_fluid_material::update_potion_sloshing);

    for _ in 0..10 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        schedule.run(app.world_mut());
    }

    let slosh = app.world().get::<deep_alchemy::render::potion_fluid_material::PotionSloshState>(potion_liquid).unwrap();
    assert!(slosh.initialized);
    assert!(slosh.current_tilt.length() > 0.001);

    let materials_res = app.world().resource::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>();
    let mat = materials_res.get(&mat_handle).unwrap();
    assert!(mat.slosh_normal.y > 0.5);
    assert!(mat.slosh_normal.x.abs() > 0.0001 || mat.slosh_normal.z.abs() > 0.0001);
    assert!(mat.fluid_params.x > 0.0);
}

#[test]
fn test_unit_potion_fluid_color_and_surface_update_per_slot() {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>();
    app.init_resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>();
    app.init_resource::<deep_alchemy::alchemy::PotionCooldowns>();

    app.world_mut().resource_mut::<deep_alchemy::ui::hud::ActiveHotbarSlot>().0 = 2;

    let mut materials = app
        .world_mut()
        .resource_mut::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>();
    let mat_handle = materials.add(deep_alchemy::render::potion_fluid_material::PotionFluidMaterial::default());

    app.world_mut().spawn((
        deep_alchemy::player::camera::HeldPotionLiquid,
        MeshMaterial3d(mat_handle.clone()),
        deep_alchemy::render::potion_fluid_material::PotionSloshState::default(),
    ));

    let mut schedule = Schedule::default();
    schedule.add_systems(deep_alchemy::render::potion_fluid_material::update_potion_sloshing);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(16));
    schedule.run(app.world_mut());

    let materials_res = app.world().resource::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>();
    let mat = materials_res.get(&mat_handle).unwrap();
    assert!(mat.base_color.red > 0.8);
}

#[test]
fn test_unit_frost_potion_water_sinking_and_ice_puzzle_mechanics() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
        avian3d::prelude::PhysicsPlugins::default(),
    ));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(50),
    ));
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Mesh>>();
    app.add_systems(Update, deep_alchemy::alchemy::update_potion_projectiles);

    let shallow_pos = Vec3::new(4.8, 0.15, -8.5);
    let shallow_bed_y = deep_alchemy::dungeon::modular_kit::get_grotto_terrain_height(shallow_pos.x, shallow_pos.z);
    let shallow_water_depth = (0.18 - shallow_bed_y).max(0.0);
    assert!(shallow_water_depth <= 1.45, "Shallow lake location must have depth <= 1.45m");

    let shallow_proj = app
        .world_mut()
        .spawn((
            Transform::from_translation(shallow_pos),
            deep_alchemy::alchemy::PotionProjectile {
                velocity: Vec3::new(0.0, -1.0, 0.0),
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: deep_alchemy::alchemy::AlchemicalEffectKind::Frost,
                    duration: 5.0,
                    magnitude: 10.0,
                    elapsed: 0.0,
                }],
                lifetime: 8.0,
                color: Color::WHITE,
                slot: 4,
                hitstop_timer: 0.0,
                entered_water: true,
            },
        ))
        .id();

    for _ in 0..15 {
        app.update();
    }

    assert!(
        app.world().get_entity(shallow_proj).is_err(),
        "Shallow water projectile must burst upon reaching lake bed"
    );

    let mut ice_q = app.world_mut().query::<(&Transform, &deep_alchemy::alchemy::FrozenIceSheet)>();
    let shallow_ice = ice_q.iter(app.world()).find(|(_, s)| (s.center - Vec2::new(shallow_pos.x, shallow_pos.z)).length() < 0.2);
    assert!(shallow_ice.is_some(), "Shallow freeze must spawn FrozenIceSheet");
    let (_, s_sheet) = shallow_ice.unwrap();
    assert!(s_sheet.top_y >= 0.18, "Shallow water ice block must reach surface");

    let deep_pos = Vec3::new(0.0, 0.15, -8.5);
    let deep_bed_y = deep_alchemy::dungeon::modular_kit::get_grotto_terrain_height(deep_pos.x, deep_pos.z);
    let deep_water_depth = (0.18 - deep_bed_y).max(0.0);
    assert!(deep_water_depth > 1.45, "Deep lake center must have depth > 1.45m");

    let deep_proj = app
        .world_mut()
        .spawn((
            Transform::from_translation(deep_pos),
            deep_alchemy::alchemy::PotionProjectile {
                velocity: Vec3::new(0.0, -1.0, 0.0),
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: deep_alchemy::alchemy::AlchemicalEffectKind::Frost,
                    duration: 5.0,
                    magnitude: 10.0,
                    elapsed: 0.0,
                }],
                lifetime: 8.0,
                color: Color::WHITE,
                slot: 4,
                hitstop_timer: 0.0,
                entered_water: true,
            },
        ))
        .id();

    for _ in 0..50 {
        app.update();
    }

    assert!(
        app.world().get_entity(deep_proj).is_err(),
        "Deep water projectile must burst upon reaching lake bed"
    );

    let mut deep_ice_q = app.world_mut().query::<(&Transform, &deep_alchemy::alchemy::FrozenIceSheet)>();
    let deep_ice = deep_ice_q.iter(app.world()).find(|(_, s)| (s.center - Vec2::new(deep_pos.x, deep_pos.z)).length() < 0.2);
    assert!(deep_ice.is_some(), "Deep freeze must spawn FrozenIceSheet on bed");
    let (_, d_sheet) = deep_ice.unwrap();
    assert!(d_sheet.top_y <= -1.80, "Deep water ice block must remain submerged");
}

#[test]
fn test_unit_heat_haze_thermal_sources_sync_and_projection() {
    use deep_alchemy::render::heat_haze::{
        project_world_to_viewport, sync_heat_sources_system, update_heat_haze_uniforms,
        HeatHazeSettings, HeatHazeSource,
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(
        Update,
        (sync_heat_sources_system, update_heat_haze_uniforms).chain(),
    );

    let geyser_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, -5.0),
            GlobalTransform::from_xyz(0.0, 0.0, -5.0),
            deep_alchemy::alchemy::ThermalGeyser {
                eruption_timer: 1.0,
                eruption_interval: 5.0,
                is_erupting: true,
                heat_radius: 3.5,
            },
        ))
        .id();

    let puddle_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(2.0, 0.0, -4.0),
            GlobalTransform::from_xyz(2.0, 0.0, -4.0),
            deep_alchemy::alchemy::PotionPuddle {
                radius: 2.0,
                color: Color::srgb(1.0, 0.3, 0.0),
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                    duration: 10.0,
                    magnitude: 15.0,
                    elapsed: 0.0,
                }],
                max_lifetime: 15.0,
                elapsed: 0.0,
                is_wall: false,
            },
        ))
        .id();

    let projectile_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(-2.0, 1.0, -3.0),
            GlobalTransform::from_xyz(-2.0, 1.0, -3.0),
            deep_alchemy::alchemy::PotionProjectile {
                velocity: Vec3::new(0.0, 0.0, -10.0),
                effects: vec![],
                lifetime: 5.0,
                color: Color::srgb(1.0, 0.2, 0.0),
                slot: 2,
                hitstop_timer: 0.0,
                entered_water: false,
            },
        ))
        .id();

    let flammable_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(1.0, 0.5, -6.0),
            GlobalTransform::from_xyz(1.0, 0.5, -6.0),
            deep_alchemy::alchemy::Flammable {
                burn_duration: 6.0,
                elapsed: 1.0,
                is_burning: true,
                fire_light: None,
                fire_emitter: None,
                initial_scale: None,
            },
        ))
        .id();

    let camera_entity = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            Camera {
                viewport: Some(bevy::render::camera::Viewport {
                    physical_position: UVec2::ZERO,
                    physical_size: UVec2::new(1280, 720),
                    depth: 0.0..1.0,
                }),
                ..default()
            },
            Transform::from_xyz(0.0, 1.5, 0.0).looking_at(Vec3::new(0.0, 1.0, -5.0), Vec3::Y),
            GlobalTransform::from(
                Transform::from_xyz(0.0, 1.5, 0.0).looking_at(Vec3::new(0.0, 1.0, -5.0), Vec3::Y),
            ),
            HeatHazeSettings::default(),
        ))
        .id();

    app.update();
    app.update();

    let geyser_haze = app.world().get::<HeatHazeSource>(geyser_entity);
    assert!(geyser_haze.is_some(), "ThermalGeyser must have HeatHazeSource attached");
    assert!(geyser_haze.unwrap().radius >= 3.0, "Erupting geyser must have wide thermal radius");

    let puddle_haze = app.world().get::<HeatHazeSource>(puddle_entity);
    assert!(puddle_haze.is_some(), "Thermal PotionPuddle must have HeatHazeSource attached");

    let proj_haze = app.world().get::<HeatHazeSource>(projectile_entity);
    assert!(proj_haze.is_some(), "Fire PotionProjectile must have HeatHazeSource attached");

    let flam_haze = app.world().get::<HeatHazeSource>(flammable_entity);
    assert!(flam_haze.is_some(), "Burning Flammable must have HeatHazeSource attached");

    let cam_settings = app.world().get::<HeatHazeSettings>(camera_entity).unwrap();
    assert!(cam_settings.count > 0, "HeatHazeSettings must track in-view thermal sources");
    assert!(cam_settings.count <= 16, "HeatHazeSettings count must be within bounds");

    for i in 0..(cam_settings.count as usize) {
        let src = cam_settings.sources[i];
        assert!(src.radius > 0.0, "Projected source radius must be positive");
        assert!(src.intensity > 0.0, "Projected source intensity must be positive");
    }

    let cam_ref = app.world().get::<Camera>(camera_entity).unwrap();
    let cam_gt = app.world().get::<GlobalTransform>(camera_entity).unwrap();
    let behind_pos = Vec3::new(0.0, 1.5, 10.0);
    let proj_behind = project_world_to_viewport(behind_pos, 2.0, cam_ref, cam_gt);
    assert!(proj_behind.is_none(), "Sources behind camera must be rejected");
}

#[test]
fn test_unit_radial_kinetic_impulse_on_dynamic_bodies_and_player() {
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

    let prop_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.5, -3.0),
            GlobalTransform::from_translation(Vec3::new(0.0, 0.5, -3.0)),
            avian3d::prelude::RigidBody::Dynamic,
            avian3d::prelude::Collider::sphere(0.25),
            avian3d::prelude::LinearVelocity(Vec3::ZERO),
        ))
        .id();

    let player_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::PlayerState::Active,
            deep_alchemy::player::camera::LocomotionState::default(),
            deep_alchemy::alchemy::EffectReceiverComponent::default(),
            Transform::from_xyz(0.0, 1.2, -1.0),
            GlobalTransform::from_translation(Vec3::new(0.0, 1.2, -1.0)),
        ))
        .id();

    let _proj_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.5, -2.0),
            deep_alchemy::alchemy::PotionProjectile {
                velocity: Vec3::ZERO,
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                    duration: 5.0,
                    magnitude: 10.0,
                    elapsed: 0.0,
                }],
                lifetime: 5.0,
                color: Color::srgb(1.0, 0.3, 0.1),
                slot: 2,
                hitstop_timer: 0.01,
                entered_water: false,
            },
        ))
        .id();

    app.update();
    app.update();

    let prop_vel = app.world().get::<avian3d::prelude::LinearVelocity>(prop_entity);
    assert!(prop_vel.is_some(), "Dynamic prop must have LinearVelocity");
    let speed = prop_vel.unwrap().0.length();
    assert!(speed > 0.5, "Explosion must apply radial impulse to dynamic prop: speed = {:.3}", speed);

    let player_loco = app.world().get::<deep_alchemy::player::camera::LocomotionState>(player_entity).unwrap();
    let p_knockback = player_loco.horizontal_velocity.length();
    assert!(p_knockback > 0.2, "Explosion must apply knockback to nearby player: knockback = {:.3}", p_knockback);
}

#[test]
fn test_unit_frost_potion_extinguishes_burning_flammable_and_spawns_steam() {
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

    let light_child = app.world_mut().spawn(PointLight::default()).id();
    let emitter_child = app.world_mut().spawn(Transform::default()).id();

    let flammable_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.5, -2.0),
            GlobalTransform::from_translation(Vec3::new(0.0, 0.5, -2.0)),
            deep_alchemy::alchemy::Flammable {
                burn_duration: 6.0,
                elapsed: 1.5,
                is_burning: true,
                fire_light: Some(light_child),
                fire_emitter: Some(emitter_child),
                initial_scale: Some(Vec3::ONE),
            },
        ))
        .id();

    let _proj_entity = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.5, -2.2),
            deep_alchemy::alchemy::PotionProjectile {
                velocity: Vec3::ZERO,
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: deep_alchemy::alchemy::AlchemicalEffectKind::Frost,
                    duration: 5.0,
                    magnitude: 15.0,
                    elapsed: 0.0,
                }],
                lifetime: 5.0,
                color: Color::srgb(0.2, 0.8, 1.0),
                slot: 4,
                hitstop_timer: 0.01,
                entered_water: false,
            },
        ))
        .id();

    app.update();
    app.update();

    let flam = app.world().get::<deep_alchemy::alchemy::Flammable>(flammable_entity).unwrap();
    assert!(!flam.is_burning, "Frost potion impact must extinguish burning flammable");
    assert!(flam.fire_light.is_none(), "Extinguished flammable must clear fire light");
    assert!(flam.fire_emitter.is_none(), "Extinguished flammable must clear fire emitter");
}

#[test]
fn test_unit_held_potion_5_distinct_slot_geometries() {
    for slot in 1..=5 {
        let flask = deep_alchemy::alchemy::create_held_flask_mesh(slot);
        assert!(flask.count_vertices() >= 40);
        assert!(flask.indices().is_some());
        assert!(flask.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(flask.attribute(Mesh::ATTRIBUTE_UV_0).is_some());

        let fluid = deep_alchemy::alchemy::create_held_fluid_mesh(slot);
        assert!(fluid.count_vertices() >= 30);
        assert!(fluid.indices().is_some());

        let stopper = deep_alchemy::alchemy::create_held_stopper_mesh(slot);
        assert!(stopper.count_vertices() >= 15);
        assert!(stopper.indices().is_some());
    }

    assert!(deep_alchemy::alchemy::create_held_accent_mesh(1).is_some());
    assert!(deep_alchemy::alchemy::create_held_accent_mesh(2).is_some());
    assert!(deep_alchemy::alchemy::create_held_accent_mesh(3).is_some());
    assert!(deep_alchemy::alchemy::create_held_accent_mesh(4).is_none());
    assert!(deep_alchemy::alchemy::create_held_accent_mesh(5).is_none());
}

#[test]
fn test_unit_wall_splatter_orthonormal_rotation_determinant_positive() {
    let normals = [
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Z,
        Vec3::NEG_Z,
        Vec3::new(1.0, 0.0, 1.0).normalize(),
        Vec3::new(-0.6, 0.2, 0.77).normalize(),
    ];

    let grav = Vec3::NEG_Y;
    for norm in normals {
        let down_proj = (grav - norm * grav.dot(norm)).normalize_or_zero();
        let down = if down_proj.length_squared() > 0.01 {
            down_proj
        } else {
            Vec3::NEG_Y
        };
        let right = down.cross(norm).normalize();
        let mat = Mat3::from_cols(right, norm, -down);
        let det = mat.determinant();
        assert!((det - 1.0).abs() < 1e-3, "det = {det} for norm = {norm:?}");
        let rot = Quat::from_mat3(&mat);
        assert!(rot.is_normalized());
    }
}

#[test]
fn test_unit_held_potion_switching_slots_rebuilds_visuals() {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>();
    app.init_resource::<deep_alchemy::alchemy::PotionCooldowns>();
    app.insert_resource(deep_alchemy::ui::hud::ActiveHotbarSlot(1));

    let held_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::FirstPersonHeldPotion::default(),
            Transform::from_xyz(0.24, -0.20, -0.42),
            Visibility::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(deep_alchemy::player::controller::update_held_potion_visuals);

    schedule.run(app.world_mut());

    let p1 = app.world().get::<deep_alchemy::player::camera::FirstPersonHeldPotion>(held_entity).unwrap();
    assert!(p1.visuals_spawned);
    assert_eq!(p1.current_slot, 1);

    app.world_mut().resource_mut::<deep_alchemy::ui::hud::ActiveHotbarSlot>().0 = 2;
    schedule.run(app.world_mut());

    let p2 = app.world().get::<deep_alchemy::player::camera::FirstPersonHeldPotion>(held_entity).unwrap();
    assert_eq!(p2.current_slot, 2);

    app.world_mut().resource_mut::<deep_alchemy::ui::hud::ActiveHotbarSlot>().0 = 3;
    schedule.run(app.world_mut());

    let p3 = app.world().get::<deep_alchemy::player::camera::FirstPersonHeldPotion>(held_entity).unwrap();
    assert_eq!(p3.current_slot, 3);
}
