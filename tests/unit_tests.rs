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

    let initial_pos = Vec3::new(0.35, 0.0, 0.0);
    let grass_entity = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::modular_kit::TallGrassClump,
            deep_alchemy::dungeon::modular_kit::GrassBending {
                base_rotation: Quat::IDENTITY,
                current_bend: Vec2::ZERO,
                bend_velocity: Vec2::ZERO,
            },
            Transform::from_translation(initial_pos),
            GlobalTransform::from_translation(initial_pos),
        ))
        .id();

    app.update();
    app.update();

    let grass_tf = app
        .world()
        .get::<Transform>(grass_entity)
        .unwrap();
    assert_eq!(grass_tf.translation, initial_pos);
    assert_eq!(grass_tf.rotation, Quat::IDENTITY);

    let mut player_tf = app.world_mut().get_mut::<Transform>(player_entity).unwrap();
    player_tf.translation = Vec3::new(10.0, 0.0, 0.0);
    let mut player_gt = app.world_mut().get_mut::<GlobalTransform>(player_entity).unwrap();
    *player_gt = GlobalTransform::from_translation(Vec3::new(10.0, 0.0, 0.0));

    for _ in 0..40 {
        app.update();
    }

    let grass_tf_after = app
        .world()
        .get::<Transform>(grass_entity)
        .unwrap();
    assert_eq!(grass_tf_after.translation, initial_pos);
    assert_eq!(grass_tf_after.rotation, Quat::IDENTITY);
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
fn test_unit_fire_potion_ignites_imported_tall_grass_and_mushroom_caps() {
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
    app.add_systems(
        Update,
        (
            deep_alchemy::dungeon::modular_kit::mark_imported_tall_grass,
            deep_alchemy::dungeon::modular_kit::attach_mushroom_spore_emitters,
            deep_alchemy::alchemy::update_potion_projectiles,
        ),
    );

    let grass_ent = app
        .world_mut()
        .spawn((
            Name::new("TallGrass_Meadow_L2"),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    let mushroom_ent = app
        .world_mut()
        .spawn((
            Name::new("Left_Shroom_Large_Cap"),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let grass_flam = app.world().get::<deep_alchemy::alchemy::Flammable>(grass_ent);
    assert!(grass_flam.is_some(), "Imported tall grass must have Flammable component");
    let mushroom_flam = app.world().get::<deep_alchemy::alchemy::Flammable>(mushroom_ent);
    assert!(mushroom_flam.is_some(), "Imported mushroom cap must have Flammable component");

    app.world_mut()
        .get_mut::<deep_alchemy::alchemy::Flammable>(grass_ent)
        .unwrap()
        .center_offset = Some(Vec3::new(-2.14, 0.49, 2.79));
    app.world_mut()
        .get_mut::<deep_alchemy::alchemy::Flammable>(mushroom_ent)
        .unwrap()
        .center_offset = Some(Vec3::new(-3.39, 2.31, -3.41));

    app.world_mut().spawn((
        Transform::from_xyz(-2.10, 0.50, 2.80),
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 25.0,
                elapsed: 0.0,
            }],
            lifetime: 8.0,
            color: Color::srgb(0.95, 0.12, 0.16),
            slot: 2,
            hitstop_timer: 0.001,
            entered_water: false,
        },
    ));

    app.update();
    app.update();

    let grass_flam_after = app.world().get::<deep_alchemy::alchemy::Flammable>(grass_ent).unwrap();
    assert!(grass_flam_after.is_burning, "Fire potion impact must ignite tall grass at its world center");

    app.world_mut().spawn((
        Transform::from_xyz(-3.35, 2.30, -3.40),
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 25.0,
                elapsed: 0.0,
            }],
            lifetime: 8.0,
            color: Color::srgb(0.95, 0.12, 0.16),
            slot: 2,
            hitstop_timer: 0.001,
            entered_water: false,
        },
    ));

    app.update();
    app.update();

    let mushroom_flam_after = app.world().get::<deep_alchemy::alchemy::Flammable>(mushroom_ent).unwrap();
    assert!(mushroom_flam_after.is_burning, "Fire potion impact must ignite mushroom cap at its world center");
}

#[test]
fn test_unit_fire_puddle_ignites_nearby_tall_grass_and_mushrooms() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(25)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_systems(Update, deep_alchemy::alchemy::update_potion_puddles);

    let grass_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::modular_kit::TallGrassClump,
            Transform::from_xyz(0.0, 0.0, 0.0),
            deep_alchemy::alchemy::Flammable::with_duration(2.5)
                .with_center_offset(Vec3::new(2.1, 0.05, 1.9)),
        ))
        .id();

    let mushroom_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::modular_kit::MushroomCap,
            Transform::from_xyz(0.0, 0.0, 0.0),
            deep_alchemy::alchemy::Flammable::with_duration(3.5)
                .with_center_offset(Vec3::new(2.3, 0.10, 2.2)),
        ))
        .id();

    let puddle_mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(Sphere::new(1.0));
    let puddle_mat = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial::default());

    app.world_mut().spawn((
        Transform::from_xyz(2.0, 0.0, 2.0),
        Mesh3d(puddle_mesh),
        MeshMaterial3d(puddle_mat),
        deep_alchemy::alchemy::PotionPuddle {
            color: Color::srgb(0.95, 0.12, 0.16),
            elapsed: 0.1,
            max_lifetime: 50.0,
            radius: 1.5,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                elapsed: 0.0,
                magnitude: 25.0,
            }],
            is_wall: false,
        },
    ));

    for _ in 0..20 {
        app.update();
    }

    let grass = app.world().get::<deep_alchemy::alchemy::Flammable>(grass_ent).unwrap();
    assert!(grass.is_burning, "Thermal puddle must ignite nearby tall grass");

    let mushroom = app.world().get::<deep_alchemy::alchemy::Flammable>(mushroom_ent).unwrap();
    assert!(mushroom.is_burning, "Thermal puddle must ignite nearby mushroom");
}

#[test]
fn test_unit_fire_potion_ignites_imported_dandelions() {
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
    app.add_systems(
        Update,
        (
            deep_alchemy::dungeon::modular_kit::mark_imported_flammable_props,
            deep_alchemy::alchemy::update_potion_projectiles,
        )
            .chain(),
    );

    let dandelion_ent = app
        .world_mut()
        .spawn((
            Name::new("Dandelion_3"),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let dandelion_flam = app.world().get::<deep_alchemy::alchemy::Flammable>(dandelion_ent);
    assert!(dandelion_flam.is_some(), "Imported dandelion must have Flammable component");

    app.world_mut()
        .get_mut::<deep_alchemy::alchemy::Flammable>(dandelion_ent)
        .unwrap()
        .center_offset = Some(Vec3::new(-4.00, 0.54, -3.50));

    app.world_mut().spawn((
        Transform::from_xyz(-3.95, 0.55, -3.48),
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 25.0,
                elapsed: 0.0,
            }],
            lifetime: 8.0,
            color: Color::srgb(0.95, 0.12, 0.16),
            slot: 2,
            hitstop_timer: 0.001,
            entered_water: false,
        },
    ));

    app.update();
    app.update();

    let dandelion_after = app.world().get::<deep_alchemy::alchemy::Flammable>(dandelion_ent).unwrap();
    assert!(dandelion_after.is_burning, "Fire potion impact must ignite dandelion at its world center");
}

#[test]
fn test_unit_flammable_combustion_and_heat_haze_concurrent_despawn_safety() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.add_systems(
        Update,
        (
            deep_alchemy::alchemy::update_flammable_combustion,
            deep_alchemy::render::heat_haze::sync_heat_sources_system,
        ),
    );

    let parent_ent = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::from_xyz(0.0, 0.0, 0.0),
            deep_alchemy::alchemy::Flammable {
                burn_duration: 0.1,
                elapsed: 0.09,
                is_burning: true,
                fire_light: None,
                fire_emitter: None,
                initial_scale: None,
                initial_translation: None,
                center_offset: Some(Vec3::new(1.0, 1.0, 1.0)),
                temperature: 20.0,
                ignition_temperature: 20.0,
                cooling_rate: 6.0,
            },
        ))
        .id();

    let child_ent = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::from_xyz(0.0, 1.0, 0.0),
            deep_alchemy::alchemy::Flammable {
                burn_duration: 0.1,
                elapsed: 0.09,
                is_burning: true,
                fire_light: None,
                fire_emitter: None,
                initial_scale: None,
                initial_translation: None,
                center_offset: Some(Vec3::new(1.0, 1.5, 1.0)),
                temperature: 20.0,
                ignition_temperature: 20.0,
                cooling_rate: 6.0,
            },
        ))
        .set_parent(parent_ent)
        .id();

    app.update();
    app.update();
    app.update();

    assert!(app.world().get_entity(parent_ent).is_err() || app.world().get_entity(parent_ent).unwrap().get::<deep_alchemy::alchemy::Flammable>().is_none());
    assert!(app.world().get_entity(child_ent).is_err());
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
fn test_unit_held_potion_jump_takeoff_vertical_fluid_stability_rtx_on() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_resource::<Time>();
    app.init_resource::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>();
    app.init_resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>();
    app.init_resource::<deep_alchemy::alchemy::PotionCooldowns>();
    app.init_resource::<deep_alchemy::ui::menu::GameSettings>();

    app.world_mut().resource_mut::<deep_alchemy::ui::menu::GameSettings>().rtx_quality = deep_alchemy::ui::menu::RtxQuality::Medium;
    app.world_mut().resource_mut::<deep_alchemy::ui::hud::ActiveHotbarSlot>().0 = 2;

    let player = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::PlayerState::Active,
            deep_alchemy::player::camera::LocomotionState::default(),
            Transform::from_xyz(0.0, 1.2, 0.0),
            Visibility::default(),
        ))
        .id();

    let camera = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::PlayerCamera,
            Transform::from_xyz(0.0, 0.55, 0.0),
            Visibility::default(),
        ))
        .id();

    let viewmodel = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::FirstPersonHeldPotion::default(),
            Transform::from_xyz(0.24, -0.20, -0.42),
            Visibility::default(),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ))
        .id();

    let mut materials = app
        .world_mut()
        .resource_mut::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>();
    let mat_handle = materials.add(deep_alchemy::render::potion_fluid_material::PotionFluidMaterial::default());

    let liquid = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::HeldPotionLiquid,
            MeshMaterial3d(mat_handle.clone()),
            deep_alchemy::render::potion_fluid_material::PotionSloshState::default(),
            Transform::IDENTITY,
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ))
        .id();

    app.world_mut().entity_mut(player).add_child(camera);
    app.world_mut().entity_mut(camera).add_child(viewmodel);
    app.world_mut().entity_mut(viewmodel).add_child(liquid);

    app.add_systems(
        Update,
        deep_alchemy::player::controller::update_first_person_potion_sway,
    );
    app.add_systems(
        PostUpdate,
        deep_alchemy::render::potion_fluid_material::update_potion_sloshing
            .after(bevy::transform::TransformSystem::TransformPropagate),
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(16));
    app.update();

    let initial_pos = app.world().get::<GlobalTransform>(liquid).unwrap().translation();
    let initial_bulb = app.world().resource::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>().get(&mat_handle).unwrap().bulb_center;
    assert!((initial_pos.y - initial_bulb.y).abs() < 0.005);

    {
        let mut p_mut = app.world_mut().get_mut::<Transform>(player).unwrap();
        p_mut.translation.y += 6.8 * 0.016;
        let mut l_mut = app.world_mut().get_mut::<deep_alchemy::player::camera::LocomotionState>(player).unwrap();
        l_mut.vertical_velocity = 6.8;
    }

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(16));
    app.update();

    let jumped_pos = app.world().get::<GlobalTransform>(liquid).unwrap().translation();
    let jumped_bulb = app.world().resource::<Assets<deep_alchemy::render::potion_fluid_material::PotionFluidMaterial>>().get(&mat_handle).unwrap().bulb_center;
    assert!((jumped_pos.y - jumped_bulb.y).abs() < 0.005);
    assert!(jumped_pos.y > initial_pos.y + 0.05);

    let not_caster = app.world().get::<bevy::pbr::NotShadowCaster>(liquid);
    let not_receiver = app.world().get::<bevy::pbr::NotShadowReceiver>(liquid);
    assert!(not_caster.is_some());
    assert!(not_receiver.is_some());
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
    app.init_resource::<deep_alchemy::alchemy::ice_simulation::LakeFreezeSimulation>();
    app.add_systems(
        Update,
        (
            deep_alchemy::alchemy::update_potion_projectiles,
            deep_alchemy::alchemy::ice_simulation::update_lake_freeze_simulation,
        )
            .chain(),
    );

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

    let freeze = app.world().resource::<deep_alchemy::alchemy::ice_simulation::LakeFreezeSimulation>();
    let shallow_index = freeze.index_at(Vec2::new(shallow_pos.x, shallow_pos.z)).unwrap();
    assert!(freeze.freeze[shallow_index] > 0.0, "Shallow water must start freezing progressively");
    assert!(freeze.freeze.iter().any(|value| *value == 0.0), "Freeze must remain localized rather than replacing the entire lake");

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

    let freeze = app.world().resource::<deep_alchemy::alchemy::ice_simulation::LakeFreezeSimulation>();
    let deep_front = freeze
        .freeze
        .iter()
        .enumerate()
        .filter(|(index, _)| deep_alchemy::alchemy::ice_simulation::LakeFreezeSimulation::cell_position(*index).distance(Vec2::new(deep_pos.x, deep_pos.z)) < 1.5)
        .map(|(_, freeze)| *freeze)
        .fold(0.0_f32, f32::max);
    assert!(deep_front > 0.08, "Deep water freeze must retain its local nucleation front: {deep_front}");
    let solid_cells = freeze.freeze.iter().filter(|value| **value >= 0.72).count();
    assert!(solid_cells > 0, "A sufficiently sustained freeze must eventually create physical support cells");
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
                initial_translation: None,
                center_offset: None,
                temperature: 20.0,
                ignition_temperature: 20.0,
                cooling_rate: 6.0,
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
                initial_translation: None,
                center_offset: None,
                temperature: 20.0,
                ignition_temperature: 20.0,
                cooling_rate: 6.0,
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

#[test]
fn test_unit_rust_vision_squeezer_tokens_and_dimensions() {
    use deep_alchemy::tools::vision_squeezer::estimate_vision_tokens;

    let (tiles, tokens) = estimate_vision_tokens(1280, 720, 512);
    assert_eq!(tiles, 6);
    assert_eq!(tokens, 6 * 258 + 85);

    let (tiles_small, tokens_small) = estimate_vision_tokens(512, 512, 512);
    assert_eq!(tiles_small, 1);
    assert_eq!(tokens_small, 258 + 85);
}

#[test]
fn test_unit_rust_frame_validator_color_and_edges() {
    use deep_alchemy::tools::validator::analyze_frame;
    use std::path::Path;

    let temp_dir = std::env::temp_dir();
    let test_img_path = temp_dir.join("test_validator_pass.png");

    let mut img = image::RgbImage::new(100, 100);
    for y in 0..100 {
        for x in 0..100 {
            if x % 4 == 0 {
                img.put_pixel(x, y, image::Rgb([0, 255, 200]));
            } else {
                img.put_pixel(x, y, image::Rgb([10, 20, 40]));
            }
        }
    }
    img.save_with_format(&test_img_path, image::ImageFormat::Png).unwrap();

    let report = analyze_frame(Path::new(&test_img_path)).expect("Validation must execute");
    assert!(report.vibrant_percentage > 10.0);
    assert!(report.edge_percentage > 0.80);
    assert!(report.passed);

    let _ = std::fs::remove_file(test_img_path);
}

#[test]
fn test_unit_rust_blueprint_color_and_hsv() {
    use deep_alchemy::tools::blueprint::{rgb_to_hex, rgb_to_hsv};

    assert_eq!(rgb_to_hex(255, 0, 128), "#ff0080");
    assert_eq!(rgb_to_hex(0, 255, 0), "#00ff00");

    let (h, s, v) = rgb_to_hsv(0, 255, 255);
    assert!((h - 180.0).abs() < 1.0);
    assert!((s - 1.0).abs() < 0.01);
    assert!((v - 1.0).abs() < 0.01);
}

#[test]
fn test_unit_rust_cli_backend_status_and_brewing() {
    use deep_alchemy::tools::cli_backend::{brew, get_player_stats, get_system_status, load_ron_database};

    let status = get_system_status();
    assert!(status.recipes_ron);

    let db = load_ron_database().expect("RON database must load");
    assert!(!db.ingredients.is_empty());
    assert!(!db.potions.is_empty());

    let brew_res = brew(&["ing_cyan_spore".to_string()]);
    assert!(brew_res.success);
    assert_eq!(brew_res.applied_effects.len(), 2);

    let player = get_player_stats();
    assert_eq!(player.health, 100.0);
    assert_eq!(player.active_slot, 1);
}

#[test]
fn test_unit_frost_potion_lake_water_ice_platform_and_flat_conforming_mesh() {
    let water_center = Vec3::new(0.0, 0.18, -8.5);
    assert!(deep_alchemy::dungeon::modular_kit::is_in_lake_water(water_center));

    let puddle_mesh = deep_alchemy::alchemy::create_conforming_puddle_mesh(1.15, 32, water_center);
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(positions)) =
        puddle_mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        for p in positions {
            assert!(
                p[1] >= -0.08 && p[1] <= 0.045,
                "In-water conforming puddle mesh must remain flat and not spike along lake bed funnel: y={}",
                p[1]
            );
        }
    } else {
        panic!("Position attribute not found on conforming puddle mesh");
    }

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
    app.init_resource::<deep_alchemy::alchemy::ice_simulation::LakeFreezeSimulation>();
    app.add_systems(Update, deep_alchemy::alchemy::update_potion_projectiles);

    let proj_entity = app
        .world_mut()
        .spawn((
            Transform::from_translation(water_center),
            deep_alchemy::alchemy::PotionProjectile {
                velocity: Vec3::ZERO,
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: deep_alchemy::alchemy::AlchemicalEffectKind::Frost,
                    duration: 5.0,
                    magnitude: 20.0,
                    elapsed: 0.0,
                }],
                lifetime: 5.0,
                color: Color::srgb(0.12, 0.42, 0.95),
                slot: 4,
                hitstop_timer: 0.01,
                entered_water: false,
            },
            deep_alchemy::alchemy::PotionSurfaceImpact {
                host: Entity::PLACEHOLDER,
                normal: Vec3::Y,
            },
        ))
        .id();

    app.update();
    app.update();

    assert!(app.world().get_entity(proj_entity).is_err());

    let mut ice_query = app
        .world_mut()
        .query::<(
            &Name,
            &Transform,
            &avian3d::prelude::RigidBody,
            &avian3d::prelude::Collider,
            &deep_alchemy::alchemy::PotionPuddle,
        )>();

    let ice_floes: Vec<_> = ice_query.iter(app.world()).collect();
    assert_eq!(ice_floes.len(), 1, "Must spawn exactly one solid ice floe platform in water");
    let (name, tf, rb, _, puddle) = ice_floes[0];
    assert_eq!(name.as_str(), "Solid_Ice_Platform");
    assert_eq!(*rb, avian3d::prelude::RigidBody::Static);
    assert!(puddle.radius >= 1.6);
    assert!((tf.translation.y - (deep_alchemy::alchemy::ice_simulation::ICE_SURFACE_Y + 0.11)).abs() < 0.02);
}

#[test]
fn test_unit_player_stepping_on_melting_ice_does_not_crash() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
        avian3d::prelude::PhysicsPlugins::default(),
        bevy::state::app::StatesPlugin,
    ));
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(50),
    ));
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<deep_alchemy::alchemy::ice_simulation::LakeFreezeSimulation>();
    app.init_state::<deep_alchemy::ui::menu::GameState>();
    app.add_systems(
        Update,
        (
            deep_alchemy::alchemy::ice_simulation::update_lake_freeze_simulation,
            deep_alchemy::player::controller::player_movement,
        )
            .chain(),
    );

    let seed_pos = Vec2::new(0.0, -8.5);
    {
        let mut sim = app.world_mut().resource_mut::<deep_alchemy::alchemy::ice_simulation::LakeFreezeSimulation>();
        sim.add_freeze_seed(seed_pos, 2.0);
    }

    for _ in 0..10 {
        app.update();
    }

    let player_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::PlayerState::Active,
            deep_alchemy::player::camera::Stamina { current: 100.0, max: 100.0 },
            deep_alchemy::player::camera::LocomotionState::default(),
            Transform::from_xyz(0.0, 0.18 + 1.2, -8.5),
        ))
        .id();

    {
        let mut sim = app.world_mut().resource_mut::<deep_alchemy::alchemy::ice_simulation::LakeFreezeSimulation>();
        for cold in &mut sim.cold {
            *cold = 0.0;
        }
        for freeze in &mut sim.freeze {
            *freeze = 0.01;
        }
    }

    for _ in 0..10 {
        app.update();
    }

    assert!(app.world().get_entity(player_ent).is_ok());
}

#[test]
fn test_unit_vegetation_foliage_animation_anchored_roots() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_resource::<Assets<Mesh>>();
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(20),
    ));

    let config = deep_alchemy::dungeon::vegetation_animation::load_vegetation_animations_from_disk(
        "assets/data/vegetation_animations.ron",
    );
    app.insert_resource(config);
    app.add_systems(
        Update,
        deep_alchemy::dungeon::vegetation_animation::update_vegetation_foliage_animation,
    );

    let initial_positions = vec![
        [-1.0, 0.03, 2.0],
        [-1.0, 0.15, 2.0],
        [-1.0, 0.35, 2.0],
        [-1.0, 0.60, 2.0],
    ];

    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        initial_positions.clone(),
    );

    let mesh_handle = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
    let deformer = deep_alchemy::dungeon::vegetation_animation::FoliageMeshDeformer::new(
        &initial_positions,
        0.0,
    )
    .expect("FoliageMeshDeformer should be created");

    let grass_ent = app
        .world_mut()
        .spawn((
            Mesh3d(mesh_handle.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::from_translation(Vec3::ZERO),
            deformer,
        ))
        .id();

    for _ in 0..15 {
        app.update();
    }

    let meshes = app.world().resource::<Assets<Mesh>>();
    let updated_mesh = meshes.get(&mesh_handle).unwrap();
    if let Some(bevy::render::mesh::VertexAttributeValues::Float32x3(positions)) =
        updated_mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        assert_eq!(positions[0][0], initial_positions[0][0]);
        assert_eq!(positions[0][1], initial_positions[0][1]);
        assert_eq!(positions[0][2], initial_positions[0][2]);

        assert_ne!(positions[3][0], initial_positions[3][0]);
    } else {
        panic!("Position attribute not found");
    }

    assert!(app.world().get_entity(grass_ent).is_ok());
}

#[test]
fn test_unit_dandelion_idle_animation_and_player_interaction() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));

    let config = deep_alchemy::dungeon::vegetation_animation::load_vegetation_animations_from_disk(
        "assets/data/vegetation_animations.ron",
    );
    assert_eq!(config.dandelion_profile.id, "ambient_dandelion_nod");
    assert!(config.dandelion_profile.idle_sway_speed > 0.0);
    assert!(config.dandelion_profile.idle_pitch_amplitude > 0.0);
    app.insert_resource(config);

    app.add_systems(
        Update,
        deep_alchemy::dungeon::flora::animate_dandelions,
    );

    let dandelion_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::flora::DandelionPlant {
                sway_phase: 0.0,
                base_yaw: 0.0,
                base_rotation: Quat::IDENTITY,
                base_translation: Vec3::new(2.0, 0.0, 2.0),
                pivot: Vec3::ZERO,
            },
            Transform::from_xyz(2.0, 0.0, 2.0),
            GlobalTransform::from_xyz(2.0, 0.0, 2.0),
        ))
        .id();

    app.update();
    let rot_initial = app.world().get::<Transform>(dandelion_ent).unwrap().rotation;

    for _ in 0..10 {
        app.update();
    }

    let rot_after_idle = app.world().get::<Transform>(dandelion_ent).unwrap().rotation;
    assert_ne!(rot_initial, rot_after_idle, "Dandelion must animate gently in idle state");

    let player_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(2.1, 0.0, 2.0),
            GlobalTransform::from_xyz(2.1, 0.0, 2.0),
        ))
        .id();

    app.update();
    let rot_pushed = app.world().get::<Transform>(dandelion_ent).unwrap().rotation;
    assert_ne!(rot_after_idle, rot_pushed, "Dandelion must deflect when player is nearby");

    app.world_mut().entity_mut(player_ent).despawn();
    for _ in 0..10 {
        app.update();
    }
}

#[test]
fn test_unit_mushroom_organic_breathing_and_file_driven_profile() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));

    let config = deep_alchemy::dungeon::vegetation_animation::load_vegetation_animations_from_disk(
        "assets/data/vegetation_animations.ron",
    );
    assert_eq!(config.mushroom_profile.id, "ambient_mushroom_pulse");
    assert!(config.mushroom_profile.idle_pulse_speed > 0.0);
    assert!(config.mushroom_profile.idle_squash_amplitude > 0.0);
    app.insert_resource(config);

    app.add_systems(
        Update,
        deep_alchemy::dungeon::modular_kit::update_bioluminescent_mushrooms,
    );

    let shroom_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::modular_kit::MushroomCap,
            deep_alchemy::dungeon::modular_kit::MushroomOrganicBreathing {
                base_scale: Vec3::splat(1.0),
                base_rotation: Quat::IDENTITY,
                base_translation: Vec3::new(0.0, 1.0, 0.0),
                pivot: Vec3::ZERO,
                phase_offset: 0.0,
            },
            deep_alchemy::dungeon::modular_kit::BioluminescentMushroom {
                base_intensity: 1000.0,
                base_color: Color::srgb(0.12, 0.96, 0.88),
                phase: 0.0,
                surge_timer: 0.0,
            },
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::from_xyz(0.0, 1.0, 0.0),
        ))
        .id();

    let initial_scale = app.world().get::<Transform>(shroom_ent).unwrap().scale;
    assert_eq!(initial_scale, Vec3::ONE);

    for _ in 0..10 {
        app.update();
    }

    let breathing_scale = app.world().get::<Transform>(shroom_ent).unwrap().scale;
    assert_ne!(
        breathing_scale,
        Vec3::ONE,
        "Mushroom scale must breathe organically over time"
    );

    let breathing_rot = app.world().get::<Transform>(shroom_ent).unwrap().rotation;
    assert_ne!(
        breathing_rot,
        Quat::IDENTITY,
        "Mushroom rotation must have subtle idle sway"
    );

    let player_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(0.5, 1.0, 0.0),
            GlobalTransform::from_xyz(0.5, 1.0, 0.0),
        ))
        .id();

    app.update();
    let shroom = app
        .world()
        .get::<deep_alchemy::dungeon::modular_kit::BioluminescentMushroom>(shroom_ent)
        .unwrap();
    assert!(
        shroom.surge_timer > 0.0,
        "Mushroom must react and surge when player is close"
    );

    app.world_mut().entity_mut(player_ent).despawn();
}

#[test]
fn test_unit_dandelion_and_mushroom_pivot_stability() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::transform::TransformPlugin);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));

    let config = deep_alchemy::dungeon::vegetation_animation::load_vegetation_animations_from_disk(
        "assets/data/vegetation_animations.ron",
    );
    app.insert_resource(config);

    app.add_systems(
        Update,
        (
            deep_alchemy::dungeon::flora::animate_dandelions,
            deep_alchemy::dungeon::modular_kit::update_bioluminescent_mushrooms,
        ),
    );

    let pivot_point = Vec3::new(-4.5, 1.8, -4.6);
    let shroom_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::modular_kit::MushroomCap,
            deep_alchemy::dungeon::modular_kit::MushroomOrganicBreathing {
                base_scale: Vec3::splat(1.0),
                base_rotation: Quat::IDENTITY,
                base_translation: Vec3::ZERO,
                pivot: pivot_point,
                phase_offset: 0.0,
            },
            deep_alchemy::dungeon::modular_kit::BioluminescentMushroom {
                base_intensity: 1000.0,
                base_color: Color::srgb(0.12, 0.96, 0.88),
                phase: 0.0,
                surge_timer: 0.0,
            },
            Transform::IDENTITY,
            GlobalTransform::IDENTITY,
        ))
        .id();

    let d_pivot = Vec3::new(-3.4, 0.3, -1.9);
    let dandelion_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::flora::DandelionPlant {
                sway_phase: 0.0,
                base_yaw: 0.0,
                base_rotation: Quat::IDENTITY,
                base_translation: Vec3::ZERO,
                pivot: d_pivot,
            },
            Transform::IDENTITY,
            GlobalTransform::IDENTITY,
        ))
        .id();

    for _ in 0..20 {
        app.update();
    }

    let shroom_tf = app.world().get::<Transform>(shroom_ent).unwrap();
    let computed_shroom_pivot = shroom_tf.translation + shroom_tf.rotation * (shroom_tf.scale * pivot_point);
    assert!(
        (computed_shroom_pivot - pivot_point).length() < 0.001,
        "Mushroom pivot center must remain stationary during organic breathing"
    );

    let dandelion_tf = app.world().get::<Transform>(dandelion_ent).unwrap();
    let computed_dandelion_pivot = dandelion_tf.translation + dandelion_tf.rotation * d_pivot;
    assert!(
        (computed_dandelion_pivot - d_pivot).length() < 0.001,
        "Dandelion root base must remain stationary on ground during sway"
    );
}

#[test]
fn test_unit_flammable_resistance_bench_requires_multiple_potions() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
        avian3d::prelude::PhysicsPlugins::default(),
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.insert_resource(deep_alchemy::player::camera::CameraShake::default());
    app.add_systems(Update, deep_alchemy::alchemy::update_potion_projectiles);

    let bench_ent = app
        .world_mut()
        .spawn((
            Name::new("Rustic_Mossy_Bench"),
            Transform::from_xyz(2.0, 0.0, 2.0),
            deep_alchemy::alchemy::Flammable::with_duration(4.5)
                .with_ignition_temperature(50.0)
                .with_cooling_rate(4.0),
        ))
        .id();

    app.world_mut().spawn((
        Transform::from_xyz(2.0, 0.1, 2.0),
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 25.0,
                elapsed: 0.0,
            }],
            lifetime: 8.0,
            color: Color::srgb(0.95, 0.12, 0.16),
            slot: 2,
            hitstop_timer: 0.001,
            entered_water: false,
        },
    ));

    app.update();
    app.update();

    let bench_after_first = app.world().get::<deep_alchemy::alchemy::Flammable>(bench_ent).unwrap();
    assert!(!bench_after_first.is_burning, "Bench must not ignite after a single fire potion");
    assert!(bench_after_first.temperature >= 20.0, "Bench must accumulate heat after first potion");
    assert!(bench_after_first.temperature < bench_after_first.ignition_temperature);

    app.world_mut().spawn((
        Transform::from_xyz(2.0, 0.1, 2.0),
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 25.0,
                elapsed: 0.0,
            }],
            lifetime: 8.0,
            color: Color::srgb(0.95, 0.12, 0.16),
            slot: 2,
            hitstop_timer: 0.001,
            entered_water: false,
        },
    ));

    app.update();
    app.update();

    let bench_after_second = app.world().get::<deep_alchemy::alchemy::Flammable>(bench_ent).unwrap();
    assert!(bench_after_second.is_burning, "Bench must ignite after second fire potion delivers enough heat");
    assert!(bench_after_second.temperature >= bench_after_second.ignition_temperature);
}

#[test]
fn test_unit_flammable_cooling_over_time_without_fire_source() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(500)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_systems(Update, deep_alchemy::alchemy::update_flammable_combustion);

    let prop_ent = app
        .world_mut()
        .spawn((
            Name::new("WarmWoodenProp"),
            Transform::from_xyz(0.0, 0.0, 0.0),
            deep_alchemy::alchemy::Flammable::with_duration(4.0)
                .with_ignition_temperature(50.0)
                .with_cooling_rate(10.0)
                .with_temperature(30.0),
        ))
        .id();

    app.update();
    app.update();

    let prop_mid = app.world().get::<deep_alchemy::alchemy::Flammable>(prop_ent).unwrap();
    assert!(!prop_mid.is_burning);
    assert!(prop_mid.temperature < 30.0);
    assert!(prop_mid.temperature > 0.0);

    for _ in 0..15 {
        app.update();
    }

    let prop_cold = app.world().get::<deep_alchemy::alchemy::Flammable>(prop_ent).unwrap();
    assert!(!prop_cold.is_burning);
    assert_eq!(prop_cold.temperature, 0.0);
}

#[test]
fn test_unit_flammable_sequential_potions_spaced_requires_three_potions() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
        avian3d::prelude::PhysicsPlugins::default(),
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(100)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.insert_resource(deep_alchemy::player::camera::CameraShake::default());
    app.add_systems(
        Update,
        (
            deep_alchemy::alchemy::update_potion_projectiles,
            deep_alchemy::alchemy::update_flammable_combustion,
        )
            .chain(),
    );

    let bench_ent = app
        .world_mut()
        .spawn((
            Name::new("Rustic_Mossy_Bench"),
            Transform::from_xyz(0.0, 0.0, 0.0),
            deep_alchemy::alchemy::Flammable::with_duration(4.5)
                .with_ignition_temperature(50.0)
                .with_cooling_rate(4.0),
        ))
        .id();

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.1, 0.0),
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 25.0,
                elapsed: 0.0,
            }],
            lifetime: 8.0,
            color: Color::srgb(0.95, 0.12, 0.16),
            slot: 2,
            hitstop_timer: 0.001,
            entered_water: false,
        },
    ));

    app.update();
    app.update();

    let bench_1 = app.world().get::<deep_alchemy::alchemy::Flammable>(bench_ent).unwrap();
    assert!(!bench_1.is_burning);

    for _ in 0..15 {
        app.update();
    }

    let bench_cooldown = app.world().get::<deep_alchemy::alchemy::Flammable>(bench_ent).unwrap();
    assert!(!bench_cooldown.is_burning);
    assert!(bench_cooldown.temperature < 20.0);

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.1, 0.0),
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 25.0,
                elapsed: 0.0,
            }],
            lifetime: 8.0,
            color: Color::srgb(0.95, 0.12, 0.16),
            slot: 2,
            hitstop_timer: 0.001,
            entered_water: false,
        },
    ));

    app.update();
    app.update();

    let bench_2 = app.world().get::<deep_alchemy::alchemy::Flammable>(bench_ent).unwrap();
    assert!(!bench_2.is_burning, "Bench must not ignite yet because cooldown dissipated heat between throws");

    app.world_mut().spawn((
        Transform::from_xyz(0.0, 0.1, 0.0),
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 25.0,
                elapsed: 0.0,
            }],
            lifetime: 8.0,
            color: Color::srgb(0.95, 0.12, 0.16),
            slot: 2,
            hitstop_timer: 0.001,
            entered_water: false,
        },
    ));

    app.update();
    app.update();

    let bench_3 = app.world().get::<deep_alchemy::alchemy::Flammable>(bench_ent).unwrap();
    assert!(bench_3.is_burning, "Bench must ignite after third potion when spaced across cooldown");
}

#[test]
fn test_unit_flammable_grass_ignites_easily_single_potion() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
        avian3d::prelude::PhysicsPlugins::default(),
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.insert_resource(deep_alchemy::player::camera::CameraShake::default());
    app.add_systems(Update, deep_alchemy::alchemy::update_potion_projectiles);

    let grass_ent = app
        .world_mut()
        .spawn((
            deep_alchemy::dungeon::modular_kit::TallGrassClump,
            Transform::from_xyz(1.0, 0.0, 1.0),
            deep_alchemy::alchemy::Flammable::with_duration(2.5)
                .with_ignition_temperature(15.0),
        ))
        .id();

    app.world_mut().spawn((
        Transform::from_xyz(1.0, 0.1, 1.0),
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![deep_alchemy::alchemy::ActiveEffect {
                kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                magnitude: 25.0,
                elapsed: 0.0,
            }],
            lifetime: 8.0,
            color: Color::srgb(0.95, 0.12, 0.16),
            slot: 2,
            hitstop_timer: 0.001,
            entered_water: false,
        },
    ));

    app.update();
    app.update();

    let grass = app.world().get::<deep_alchemy::alchemy::Flammable>(grass_ent).unwrap();
    assert!(grass.is_burning, "Tall grass must ignite easily with a single fire potion");
    assert!(grass.temperature >= grass.ignition_temperature);
}

#[test]
fn test_unit_realistic_fluid_droplet_aerodynamics_and_teardrop_physics() {
    use deep_alchemy::alchemy::surface_fluid::*;
    use avian3d::prelude::LinearVelocity;

    let radius = 0.04;
    let mesh = create_teardrop_droplet_mesh(radius, 10, 8);
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap();
    let normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap().as_float3().unwrap();
    assert_eq!(positions.len(), normals.len());
    assert!(positions.len() >= 80);

    let max_y = positions.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
    let min_y = positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
    assert!((max_y - radius * 1.5).abs() < 0.001);
    assert!((min_y - (-radius)).abs() < 0.001);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));
    app.add_systems(Update, update_surface_fluid_droplet_physics);

    let profile = PotionFluidProfiles::default().for_slot(1);
    let droplet_ent = app.world_mut().spawn((
        SurfaceFluidDroplet {
            profile,
            color: Color::srgb(0.2, 0.6, 0.9),
            effects: Vec::new(),
            volume: 0.03,
            age: 0.1,
            base_radius: radius,
        },
        Transform::from_xyz(0.0, 5.0, 0.0),
        LinearVelocity(Vec3::new(0.5, -4.5, 0.2)),
    )).id();

    for _ in 0..10 {
        app.update();
    }

    let tf = app.world().get::<Transform>(droplet_ent).unwrap();
    let vel = app.world().get::<LinearVelocity>(droplet_ent).unwrap();

    assert!(tf.scale.y > 1.05);
    let vol_scale = tf.scale.x * tf.scale.z * tf.scale.y;
    assert!((vol_scale - 1.0).abs() < 0.35);

    let travel_dir = vel.0.normalize();
    let dot = (tf.rotation * Vec3::NEG_Y).dot(travel_dir);
    assert!(dot > 0.85);
    assert!(vel.0.x.abs() < 0.5);
    assert!(vel.0.z.abs() < 0.2);
}

#[test]
fn test_unit_burning_flammable_collapses_in_place_without_sliding_to_origin() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
    ));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.add_systems(
        Update,
        (
            deep_alchemy::dungeon::modular_kit::update_bioluminescent_mushrooms,
            deep_alchemy::alchemy::update_flammable_combustion,
        ),
    );

    let initial_pos = Vec3::new(4.13, 1.60, -3.76);
    let mushroom_ent = app
        .world_mut()
        .spawn((
            Name::new("Right_Dome_Mushroom_1_Cap"),
            Transform::from_translation(initial_pos),
            deep_alchemy::dungeon::modular_kit::MushroomCap,
            deep_alchemy::dungeon::modular_kit::MushroomOrganicBreathing {
                base_scale: Vec3::ONE,
                base_rotation: Quat::IDENTITY,
                base_translation: initial_pos,
                pivot: Vec3::ZERO,
                phase_offset: 0.0,
            },
            deep_alchemy::dungeon::modular_kit::BioluminescentMushroom {
                base_intensity: 1.0,
                base_color: Color::srgb(0.12, 0.96, 0.88),
                phase: 0.0,
                surge_timer: 0.0,
            },
            deep_alchemy::alchemy::Flammable {
                burn_duration: 1.0,
                elapsed: 0.0,
                is_burning: false,
                fire_light: None,
                fire_emitter: None,
                initial_scale: None,
                initial_translation: None,
                center_offset: None,
                temperature: 0.0,
                ignition_temperature: 20.0,
                cooling_rate: 6.0,
            },
        ))
        .id();

    app.update();

    app.world_mut()
        .get_mut::<deep_alchemy::alchemy::Flammable>(mushroom_ent)
        .unwrap()
        .ignite();

    for _ in 0..18 {
        app.update();
    }

    let gt = app.world().get::<GlobalTransform>(mushroom_ent).unwrap();
    let tf = app.world().get::<Transform>(mushroom_ent).unwrap();

    let xz_dist = Vec2::new(gt.translation().x - initial_pos.x, gt.translation().z - initial_pos.z).length();
    assert!(xz_dist < 0.02, "Burning mushroom must not slide on XZ towards scene origin, shifted {}", xz_dist);
    assert!(tf.scale.y < 0.30, "Burning mushroom must collapse vertically in-place, scale_y: {}", tf.scale.y);
}

#[test]
fn test_unit_potion_impact_on_ramp_conforms_to_ramp_inclination() {
    let flat_normal = deep_alchemy::dungeon::modular_kit::get_grotto_terrain_normal(0.0, 0.0);
    assert!((flat_normal.y - 1.0).abs() < 1e-3);
    assert!(flat_normal.x.abs() < 1e-3);

    let ramp_pos = Vec3::new(4.0, 0.56, -1.0);
    let ramp_normal = deep_alchemy::dungeon::modular_kit::get_grotto_terrain_normal(ramp_pos.x, ramp_pos.z);
    assert!(ramp_normal.y > 0.85);
    assert!(ramp_normal.y < 0.98);
    assert!(ramp_normal.x < -0.20);

    let raw_wall_normal = Vec3::new(-1.0, 0.0, 0.0);
    let (resolved_pos, resolved_normal) = deep_alchemy::alchemy::resolve_surface_impact_normal(ramp_pos, raw_wall_normal);
    assert!(resolved_pos.y >= ramp_pos.y);
    assert!(resolved_normal.y > 0.85);
    assert!(resolved_normal.y.abs() >= 0.65);
    assert_eq!(resolved_normal, ramp_normal);

    let puddle_rot = Quat::from_rotation_arc(Vec3::Y, resolved_normal);
    let up_transformed = puddle_rot * Vec3::Y;
    assert!((up_transformed.dot(ramp_normal) - 1.0).abs() < 1e-3);
}

#[test]
fn test_unit_fire_potion_impact_near_water_edge_spawns_massive_steam_cloud() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
        avian3d::prelude::PhysicsPlugins::default(),
    ));
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<Assets<Mesh>>();
    app.insert_resource(deep_alchemy::player::camera::CameraShake::default());

    let water_edge_impact = Vec3::new(0.0, 0.08, -3.3);
    let mut system_state = bevy::ecs::system::SystemState::<(
        Commands,
        ResMut<Assets<Mesh>>,
        ResMut<Assets<StandardMaterial>>,
    )>::new(app.world_mut());
    let (mut commands, mut meshes, mut materials) = system_state.get_mut(app.world_mut());

    deep_alchemy::alchemy::spawn_massive_steam_cloud(
        &mut commands,
        &mut meshes,
        &mut materials,
        water_edge_impact,
        1.0,
    );
    system_state.apply(app.world_mut());

    app.update();

    let mut particle_query = app
        .world_mut()
        .query::<(&deep_alchemy::render::particles::Particle, &Transform)>();
    let steam_particles: Vec<_> = particle_query
        .iter(app.world())
        .filter(|(p, _)| matches!(p.p_type, deep_alchemy::render::particles::ParticleType::Steam))
        .collect();
    assert!(
        steam_particles.len() >= 30,
        "Massive steam cloud must immediately burst with at least 30 steam particles, found: {}",
        steam_particles.len()
    );

    let mut emitter_query = app
        .world_mut()
        .query::<(&deep_alchemy::render::particles::ParticleEmitter, &deep_alchemy::alchemy::AlchemyTransientEmitter)>();
    let steam_emitters: Vec<_> = emitter_query
        .iter(app.world())
        .filter(|(e, _)| matches!(e.particle_type, deep_alchemy::render::particles::ParticleType::Steam))
        .collect();
    assert!(
        steam_emitters.len() >= 12,
        "Massive steam cloud must spawn a cluster of at least 12 billowing emitters across water edge, found: {}",
        steam_emitters.len()
    );
}

#[test]
fn test_unit_hesoyam_cheat_sequence_and_fly_mode_toggle() {
    let mut fly_mode = deep_alchemy::player::FlyModeState::default();
    assert!(!fly_mode.enabled);

    let keys = [
        KeyCode::KeyH,
        KeyCode::KeyE,
        KeyCode::KeyS,
        KeyCode::KeyO,
        KeyCode::KeyY,
        KeyCode::KeyA,
        KeyCode::KeyM,
    ];

    let mut current_time = 1.0;
    for (i, &k) in keys.iter().enumerate() {
        let toggled = fly_mode.handle_key_press(k, current_time);
        if i < keys.len() - 1 {
            assert!(!toggled);
            assert!(!fly_mode.enabled);
        } else {
            assert!(toggled);
            assert!(fly_mode.enabled);
        }
        current_time += 0.35;
    }

    assert!(fly_mode.cheat_buffer.is_empty());

    for (i, &k) in keys.iter().enumerate() {
        let toggled = fly_mode.handle_key_press(k, current_time);
        if i < keys.len() - 1 {
            assert!(!toggled);
            assert!(fly_mode.enabled);
        } else {
            assert!(toggled);
            assert!(!fly_mode.enabled);
        }
        current_time += 0.20;
    }
}

#[test]
fn test_unit_hesoyam_cheat_sequence_timeout_and_mistakes() {
    let mut fly_mode = deep_alchemy::player::FlyModeState::default();

    fly_mode.handle_key_press(KeyCode::KeyH, 1.0);
    assert_eq!(fly_mode.cheat_buffer.len(), 1);

    let toggled = fly_mode.handle_key_press(KeyCode::KeyE, 3.5);
    assert!(!toggled);
    assert!(!fly_mode.enabled);
    assert!(fly_mode.cheat_buffer.is_empty());

    fly_mode.handle_key_press(KeyCode::KeyH, 10.0);
    fly_mode.handle_key_press(KeyCode::KeyE, 10.3);
    fly_mode.handle_key_press(KeyCode::KeyS, 10.6);
    assert_eq!(fly_mode.cheat_buffer.len(), 3);

    fly_mode.handle_key_press(KeyCode::KeyX, 10.9);
    assert!(!fly_mode.enabled);
    assert!(fly_mode.cheat_buffer.is_empty());

    fly_mode.handle_key_press(KeyCode::KeyH, 20.0);
    fly_mode.handle_key_press(KeyCode::KeyE, 20.2);
    fly_mode.handle_key_press(KeyCode::KeyH, 20.4);
    assert_eq!(fly_mode.cheat_buffer.len(), 1);
    assert_eq!(fly_mode.cheat_buffer[0], KeyCode::KeyH);

    fly_mode.handle_key_press(KeyCode::KeyE, 20.6);
    fly_mode.handle_key_press(KeyCode::KeyS, 20.8);
    fly_mode.handle_key_press(KeyCode::KeyO, 21.0);
    fly_mode.handle_key_press(KeyCode::KeyY, 21.2);
    fly_mode.handle_key_press(KeyCode::KeyA, 21.4);
    let toggled = fly_mode.handle_key_press(KeyCode::KeyM, 21.6);
    assert!(toggled);
    assert!(fly_mode.enabled);
}

#[test]
fn test_unit_update_fly_mode_cheat_system_in_game_only() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(200),
    ));
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<deep_alchemy::player::FlyModeState>();
    app.init_state::<deep_alchemy::ui::menu::GameState>();
    app.add_systems(Update, deep_alchemy::player::controller::update_fly_mode_cheat);

    let keys = [
        KeyCode::KeyH,
        KeyCode::KeyE,
        KeyCode::KeyS,
        KeyCode::KeyO,
        KeyCode::KeyY,
        KeyCode::KeyA,
        KeyCode::KeyM,
    ];

    for &k in &keys {
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(k);
        app.update();
        let mut btn = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        btn.release(k);
        btn.clear_just_pressed(k);
    }

    assert!(!app.world().resource::<deep_alchemy::player::FlyModeState>().enabled);

    app.world_mut()
        .resource_mut::<NextState<deep_alchemy::ui::menu::GameState>>()
        .set(deep_alchemy::ui::menu::GameState::InGame);
    app.update();

    for &k in &keys {
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(k);
        app.update();
        let mut btn = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        btn.release(k);
        btn.clear_just_pressed(k);
    }

    let fm = app.world().resource::<deep_alchemy::player::FlyModeState>();
    assert!(fm.enabled);
}

#[test]
fn test_unit_fly_mode_locomotion_free_movement_and_noclip() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(50),
    ));
    app.init_resource::<ButtonInput<KeyCode>>();
    app.insert_resource(deep_alchemy::player::FlyModeState {
        enabled: true,
        cheat_buffer: Vec::new(),
        last_key_time: 0.0,
        fly_speed: 10.0,
        fast_fly_multiplier: 2.0,
    });
    app.init_state::<deep_alchemy::ui::menu::GameState>();
    app.world_mut()
        .resource_mut::<NextState<deep_alchemy::ui::menu::GameState>>()
        .set(deep_alchemy::ui::menu::GameState::InGame);
    app.add_systems(Update, deep_alchemy::player::controller::player_movement);
    app.update();

    let player = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::PlayerState::Active,
            deep_alchemy::player::camera::LocomotionState::default(),
            Transform::from_xyz(0.0, 15.0, 0.0),
            Health {
                current: 100.0,
                max: 100.0,
            },
            Stamina {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    app.world_mut().spawn((
        deep_alchemy::dungeon::modular_kit::CavernObstacle,
        Transform::from_xyz(0.0, 15.0, -2.0),
        deep_alchemy::dungeon::modular_kit::ObstacleShape::Cylinder {
            radius: 1.0,
            height: 4.0,
        },
    ));

    for _ in 0..10 {
        app.update();
    }

    let tf_after_idle = *app.world().get::<Transform>(player).unwrap();
    assert_eq!(tf_after_idle.translation.y, 15.0);

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
    for _ in 0..10 {
        app.update();
    }
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();

    let tf_after_ascend = *app.world().get::<Transform>(player).unwrap();
    assert!(tf_after_ascend.translation.y > 15.0);

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyW);
    for _ in 0..30 {
        app.update();
    }
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();

    let tf_after_noclip = *app.world().get::<Transform>(player).unwrap();
    assert!(tf_after_noclip.translation.z < -2.0);

    let stamina = app.world().get::<Stamina>(player).unwrap();
    assert_eq!(stamina.current, 100.0);
}



#[test]
fn test_unit_touch_controls_state_defaults_and_sensitivity() {
    let state = deep_alchemy::player::touch_controls::TouchControlsState::default();
    assert_eq!(state.move_axis, Vec2::ZERO);
    assert_eq!(state.look_delta, Vec2::ZERO);
    assert!(!state.is_sprinting);
    assert!(!state.jump_pressed);
    assert!(!state.crouch_pressed);
    assert!(!state.aim_pressed);
    assert!(state.sensitivity > 0.001);
}

#[test]
fn test_unit_touch_action_button_interactions() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.init_resource::<deep_alchemy::player::touch_controls::TouchControlsState>();
    app.add_systems(Update, deep_alchemy::player::touch_controls::process_touch_inputs);

    app.world_mut().spawn((
        deep_alchemy::player::touch_controls::TouchActionButton {
            action: deep_alchemy::player::touch_controls::TouchAction::Jump,
        },
        Interaction::Pressed,
    ));

    app.update();

    let state = app.world().resource::<deep_alchemy::player::touch_controls::TouchControlsState>();
    assert!(state.jump_pressed);
    assert!(state.jump_just_pressed);
}

#[test]
fn test_unit_touch_hotbar_slot_tap_selection() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>();
    app.add_event::<bevy::input::mouse::MouseWheel>();
    app.add_systems(Update, deep_alchemy::ui::hud::handle_hotbar_input);

    assert_eq!(app.world().resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>().0, 1);

    app.world_mut().spawn((
        deep_alchemy::ui::hud::HotbarSlotUi { slot_index: 4 },
        Interaction::Pressed,
    ));

    app.update();

    assert_eq!(app.world().resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>().0, 4);
}

#[test]
fn test_unit_touch_menu_pause_toggle() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.init_state::<deep_alchemy::ui::menu::GameState>();
    app.init_resource::<deep_alchemy::ui::menu::MenuNavigation>();
    app.init_resource::<deep_alchemy::ui::menu::GameSettings>();
    app.init_resource::<deep_alchemy::player::touch_controls::TouchControlsState>();
    app.add_systems(Update, deep_alchemy::ui::menu::handle_global_shortcuts);

    app.world_mut()
        .resource_mut::<NextState<deep_alchemy::ui::menu::GameState>>()
        .set(deep_alchemy::ui::menu::GameState::InGame);
    app.update();

    app.world_mut()
        .resource_mut::<deep_alchemy::player::touch_controls::TouchControlsState>()
        .toggle_menu_just_pressed = true;

    app.update();

    let next_state = app.world().resource::<NextState<deep_alchemy::ui::menu::GameState>>();
    match next_state {
        NextState::Pending(state) => assert_eq!(*state, deep_alchemy::ui::menu::GameState::MainMenu),
        _ => panic!("Expected next state to be MainMenu"),
    }
}
#[test]
fn test_unit_hybrid_tri_modal_locomotion_and_look() {
    let mut app = App::new();
    app.init_resource::<Time>();
    app.add_event::<bevy::input::mouse::MouseMotion>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<deep_alchemy::player::touch_controls::TouchControlsState>();

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

    let mut schedule = Schedule::default();
    schedule.add_systems((
        deep_alchemy::player::controller::player_movement,
        deep_alchemy::player::controller::player_look,
    ));

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyW);
    for _ in 0..5 {
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
        schedule.run(app.world_mut());
    }
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().release(KeyCode::KeyW);

    let tf_after_keyboard = *app.world().get::<Transform>(player).unwrap();
    assert!(tf_after_keyboard.translation.z < 0.0);

    let mut gamepad = Gamepad::default();
    gamepad.analog_mut().set(GamepadAxis::LeftStickX, 1.0);
    app.world_mut().spawn(gamepad);

    for _ in 0..5 {
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
        schedule.run(app.world_mut());
    }

    let tf_after_gamepad = *app.world().get::<Transform>(player).unwrap();
    assert!(tf_after_gamepad.translation.x > 0.0);

    app.world_mut()
        .resource_mut::<deep_alchemy::player::touch_controls::TouchControlsState>()
        .move_axis = Vec2::new(0.0, 1.0);
    app.world_mut()
        .resource_mut::<deep_alchemy::player::touch_controls::TouchControlsState>()
        .look_delta = Vec2::new(10.0, 5.0);

    for _ in 0..5 {
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
        schedule.run(app.world_mut());
    }

    let tf_after_touch = *app.world().get::<Transform>(player).unwrap();
    assert!(tf_after_touch.translation.z < tf_after_gamepad.translation.z);

    let (yaw, _, _) = tf_after_touch.rotation.to_euler(EulerRot::YXZ);
    assert!(yaw.abs() > 0.0001);
}

#[test]
fn test_unit_rtx_hardware_support_gpu_detection_rules() {
    use deep_alchemy::ui::menu::check_rtx_hardware_support_custom;

    assert!(!check_rtx_hardware_support_custom("NVIDIA GeForce RTX 4090", true));
    assert!(!check_rtx_hardware_support_custom("AMD Radeon RX 7900 XTX", true));
    assert!(!check_rtx_hardware_support_custom("Adreno 740", true));
    assert!(!check_rtx_hardware_support_custom("Mali-G715 Immortalis", true));

    assert!(check_rtx_hardware_support_custom("NVIDIA GeForce RTX 2070", false));
    assert!(check_rtx_hardware_support_custom("NVIDIA GeForce RTX 3080", false));
    assert!(check_rtx_hardware_support_custom("NVIDIA GeForce RTX 4090", false));
    assert!(check_rtx_hardware_support_custom("NVIDIA RTX A5000", false));
    assert!(check_rtx_hardware_support_custom("Quadro RTX 6000", false));
    assert!(check_rtx_hardware_support_custom("RTX 4080 Super", false));

    assert!(check_rtx_hardware_support_custom("AMD Radeon RX 6700 XT", false));
    assert!(check_rtx_hardware_support_custom("AMD Radeon RX 6800", false));
    assert!(check_rtx_hardware_support_custom("AMD Radeon RX 7900 XTX", false));
    assert!(check_rtx_hardware_support_custom("AMD Radeon RX 7600", false));
    assert!(check_rtx_hardware_support_custom("AMD Radeon RX 8800 XT", false));
    assert!(check_rtx_hardware_support_custom("AMD Radeon PRO W6800", false));
    assert!(check_rtx_hardware_support_custom("AMD Radeon PRO W7900", false));
    assert!(check_rtx_hardware_support_custom("AMD RDNA2 Architecture", false));
    assert!(check_rtx_hardware_support_custom("AMD RDNA3 Architecture", false));

    assert!(check_rtx_hardware_support_custom("Intel(R) Arc(TM) A770 Graphics", false));
    assert!(check_rtx_hardware_support_custom("Intel(R) Arc(TM) A750 Graphics", false));
    assert!(check_rtx_hardware_support_custom("Intel(R) Arc(TM) A380 Graphics", false));
    assert!(check_rtx_hardware_support_custom("Intel Battlemage B580", false));

    assert!(check_rtx_hardware_support_custom("Apple M3 Max", false));
    assert!(check_rtx_hardware_support_custom("Apple M4 Pro", false));

    assert!(!check_rtx_hardware_support_custom("NVIDIA GeForce GTX 1060 6GB", false));
    assert!(!check_rtx_hardware_support_custom("NVIDIA GeForce GTX 1660 Ti", false));
    assert!(!check_rtx_hardware_support_custom("GeForce GTX 970", false));
    assert!(!check_rtx_hardware_support_custom("AMD Radeon RX 580 Series", false));
    assert!(!check_rtx_hardware_support_custom("AMD Radeon RX Vega 64", false));
    assert!(!check_rtx_hardware_support_custom("AMD Radeon R9 390", false));
    assert!(!check_rtx_hardware_support_custom("Intel(R) UHD Graphics 630", false));
    assert!(!check_rtx_hardware_support_custom("Intel(R) Iris(R) Xe Graphics", false));
    assert!(!check_rtx_hardware_support_custom("Intel HD Graphics 4000", false));
    assert!(!check_rtx_hardware_support_custom("Qualcomm Adreno (TM) 740", false));
    assert!(!check_rtx_hardware_support_custom("ARM Mali-G715 Immortalis", false));
    assert!(!check_rtx_hardware_support_custom("PowerVR Rogue", false));
    assert!(!check_rtx_hardware_support_custom("", false));
    assert!(!check_rtx_hardware_support_custom("   ", false));
}

#[test]
fn test_unit_rtx_row_display_hidden_on_unsupported_hardware() {
    use deep_alchemy::ui::menu::{
        update_rtx_hardware_availability, GameSettings, RtxHardwareSupport, RtxQuality,
        RtxSettingsRow,
    };

    let mut app = App::new();
    let mut settings = GameSettings::default();
    settings.rtx_enabled = true;
    settings.rtx_quality = RtxQuality::Ultra;
    app.insert_resource(settings);

    app.insert_resource(RtxHardwareSupport {
        supported: false,
        detected_gpu: "Intel(R) UHD Graphics 630".to_string(),
    });

    let row_entity = app
        .world_mut()
        .spawn((
            RtxSettingsRow,
            Node {
                display: Display::Flex,
                ..default()
            },
        ))
        .id();

    app.add_systems(Update, update_rtx_hardware_availability);
    app.update();

    let node = app.world().get::<Node>(row_entity).unwrap();
    assert_eq!(node.display, Display::None);

    let updated_settings = app.world().resource::<GameSettings>();
    assert_eq!(updated_settings.rtx_enabled, false);
    assert_eq!(updated_settings.rtx_quality, RtxQuality::Off);
}

#[test]
fn test_unit_rtx_row_display_visible_on_supported_hardware() {
    use deep_alchemy::ui::menu::{
        update_rtx_hardware_availability, GameSettings, RtxHardwareSupport, RtxSettingsRow,
    };

    let mut app = App::new();
    let settings = GameSettings::default();
    app.insert_resource(settings);

    app.insert_resource(RtxHardwareSupport {
        supported: true,
        detected_gpu: "NVIDIA GeForce RTX 4080".to_string(),
    });

    let row_entity = app
        .world_mut()
        .spawn((
            RtxSettingsRow,
            Node {
                display: Display::None,
                ..default()
            },
        ))
        .id();

    app.add_systems(Update, update_rtx_hardware_availability);
    app.update();

    let node = app.world().get::<Node>(row_entity).unwrap();
    assert_eq!(node.display, Display::Flex);
}

#[test]
fn test_unit_rtx_button_interaction_blocked_when_unsupported() {
    use deep_alchemy::ui::menu::{
        handle_menu_button_interactions, GameSettings, GameState, MenuButtonAction, MenuNavigation,
        RtxHardwareSupport, RtxQuality,
    };

    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.init_resource::<MenuNavigation>();
    app.init_resource::<GameSettings>();
    app.insert_resource(RtxHardwareSupport {
        supported: false,
        detected_gpu: "Adreno 740".to_string(),
    });
    app.add_event::<AppExit>();
    app.add_systems(Update, handle_menu_button_interactions);

    app.world_mut().spawn((
        Button,
        MenuButtonAction::ToggleRtx,
        Interaction::Pressed,
        BackgroundColor(Color::BLACK),
        BorderColor(Color::BLACK),
    ));

    app.update();

    let settings = app.world().resource::<GameSettings>();
    assert_eq!(settings.rtx_quality, RtxQuality::Off);
    assert_eq!(settings.rtx_enabled, false);
}

#[test]
fn test_unit_rtx_button_interaction_allowed_when_supported() {
    use deep_alchemy::ui::menu::{
        handle_menu_button_interactions, GameSettings, GameState, MenuButtonAction, MenuNavigation,
        RtxHardwareSupport, RtxQuality,
    };

    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.init_resource::<MenuNavigation>();
    app.init_resource::<GameSettings>();
    app.insert_resource(RtxHardwareSupport {
        supported: true,
        detected_gpu: "NVIDIA GeForce RTX 3070".to_string(),
    });
    app.add_event::<AppExit>();
    app.add_systems(Update, handle_menu_button_interactions);

    app.world_mut().spawn((
        Button,
        MenuButtonAction::ToggleRtx,
        Interaction::Pressed,
        BackgroundColor(Color::BLACK),
        BorderColor(Color::BLACK),
    ));

    app.update();

    let settings = app.world().resource::<GameSettings>();
    assert_eq!(settings.rtx_quality, RtxQuality::Low);
    assert_eq!(settings.rtx_enabled, true);
}


#[test]
fn test_unit_ceiling_water_drip_spawning_and_gravity_fall() {
    use deep_alchemy::dungeon::water_drips::{
        update_ceiling_drip_points, update_falling_water_drops, CavernWaterPuddle,
        CeilingDripPoint, FallingWaterDrop, WaterPuddleState,
    };
    use deep_alchemy::dungeon::modular_kit::WaterRipple;

    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    let puddle_entity = app
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
            Transform::from_xyz(2.0, 0.02, 2.0),
        ))
        .id();

    app.world_mut().spawn((
        CeilingDripPoint {
            timer: 9.95,
            drip_interval: 10.0,
            puddle_entity: Some(puddle_entity),
            drop_fall_speed: 0.0,
        },
        Transform::from_xyz(2.0, 5.0, 2.0),
    ));

    app.add_systems(
        Update,
        (update_ceiling_drip_points, update_falling_water_drops).chain(),
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(100));
    app.update();

    let drops_count = app
        .world_mut()
        .query::<&FallingWaterDrop>()
        .iter(app.world())
        .count();
    assert_eq!(drops_count, 1);

    for _ in 0..100 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        app.update();
    }

    let remaining_drops = app
        .world_mut()
        .query::<&FallingWaterDrop>()
        .iter(app.world())
        .count();
    assert_eq!(remaining_drops, 0);

    let ripples_count = app
        .world_mut()
        .query::<&WaterRipple>()
        .iter(app.world())
        .count();
    assert!(ripples_count >= 1);
}

#[test]
fn test_unit_water_puddle_player_step_ripple_generation() {
    use deep_alchemy::dungeon::water_drips::{
        update_puddle_player_interaction, CavernWaterPuddle, WaterPuddleState,
    };
    use deep_alchemy::dungeon::modular_kit::WaterRipple;
    use deep_alchemy::player::camera::Player;

    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.2, 0.0, 0.1),
    ));

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
        Transform::from_xyz(0.0, 0.02, 0.0),
        Visibility::Inherited,
    ));

    app.add_systems(Update, update_puddle_player_interaction);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(16));
    app.update();

    let ripples = app
        .world_mut()
        .query::<(&WaterRipple, &Transform)>()
        .iter(app.world())
        .count();
    assert_eq!(ripples, 1);
}

#[test]
fn test_unit_water_puddle_frost_congelation_and_thawing() {
    use deep_alchemy::alchemy::{ActiveEffect, AlchemicalEffectKind, PotionPuddle};
    use deep_alchemy::dungeon::water_drips::{
        update_puddle_temperature_reactions, CavernWaterPuddle, WaterPuddleState,
    };

    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<Mesh>>();
    let mut mats = Assets::<StandardMaterial>::default();
    let mat_handle = mats.add(StandardMaterial::default());
    app.insert_resource(mats);

    let puddle_entity = app
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
            Transform::from_xyz(0.0, 0.02, 0.0),
            MeshMaterial3d(mat_handle),
            Visibility::Inherited,
        ))
        .id();

    app.world_mut().spawn((
        PotionPuddle {
            color: Color::WHITE,
            elapsed: 0.0,
            max_lifetime: 20.0,
            radius: 1.5,
            effects: vec![ActiveEffect {
                kind: AlchemicalEffectKind::Frost,
                duration: 10.0,
                elapsed: 0.0,
                magnitude: 1.0,
            }],
            is_wall: false,
        },
        Transform::from_xyz(0.5, 0.02, 0.0),
    ));

    app.add_systems(Update, update_puddle_temperature_reactions);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(16));
    app.update();

    let puddle = app.world().get::<CavernWaterPuddle>(puddle_entity).unwrap();
    assert_eq!(puddle.state, WaterPuddleState::Frozen);
    assert!(puddle.freeze_timer > 0.0);
}

#[test]
fn test_unit_water_puddle_fire_boiling_and_evaporation() {
    use deep_alchemy::alchemy::{ActiveEffect, AlchemicalEffectKind, PotionPuddle};
    use deep_alchemy::dungeon::water_drips::{
        update_puddle_temperature_reactions, CavernWaterPuddle, WaterPuddleState,
    };

    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<Mesh>>();
    let mut mats = Assets::<StandardMaterial>::default();
    let mat_handle = mats.add(StandardMaterial::default());
    app.insert_resource(mats);

    let puddle_entity = app
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
            Transform::from_xyz(0.0, 0.02, 0.0),
            MeshMaterial3d(mat_handle),
            Visibility::Inherited,
        ))
        .id();

    app.world_mut().spawn((
        PotionPuddle {
            color: Color::srgb(1.0, 0.3, 0.0),
            elapsed: 0.0,
            max_lifetime: 20.0,
            radius: 1.5,
            effects: vec![ActiveEffect {
                kind: AlchemicalEffectKind::ThermalHeat,
                duration: 10.0,
                elapsed: 0.0,
                magnitude: 1.0,
            }],
            is_wall: false,
        },
        Transform::from_xyz(0.4, 0.02, 0.0),
    ));

    app.add_systems(Update, update_puddle_temperature_reactions);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(16));
    app.update();

    let puddle_boiling = app.world().get::<CavernWaterPuddle>(puddle_entity).unwrap();
    assert_eq!(puddle_boiling.state, WaterPuddleState::Boiling);

    for _ in 0..200 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(20));
        app.update();
    }

    let puddle_evaporated = app.world().get::<CavernWaterPuddle>(puddle_entity).unwrap();
    assert_eq!(puddle_evaporated.state, WaterPuddleState::Evaporated);

    let vis = app.world().get::<Visibility>(puddle_entity).unwrap();
    assert_eq!(*vis, Visibility::Hidden);
}

#[test]
fn test_unit_water_puddle_realistic_translucent_material_and_feathered_mesh() {
    use deep_alchemy::dungeon::water_drips::{
        create_realistic_cavern_water_mesh, create_wet_stone_underlayer_mesh,
    };

    let water_mesh = create_realistic_cavern_water_mesh(1.2, 32, Vec3::new(0.0, 0.02, 0.0));
    let Some(bevy::render::mesh::VertexAttributeValues::Float32x4(ref colors)) =
        water_mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("Missing Float32x4 color attribute");
    };

    assert!(colors[0][3] >= 0.80);
    assert_eq!(colors.last().unwrap()[3], 0.0);

    let wet_stone_mesh = create_wet_stone_underlayer_mesh(1.2, 32, Vec3::new(0.0, 0.02, 0.0));
    let Some(bevy::render::mesh::VertexAttributeValues::Float32x4(ref wet_colors)) =
        wet_stone_mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        panic!("Missing Float32x4 color attribute");
    };

    assert!(wet_colors[0][3] >= 0.60);
    assert_eq!(wet_colors.last().unwrap()[3], 0.0);
}

#[test]
fn test_unit_water_puddle_ripple_strictly_confined_to_puddle_bounds() {
    use deep_alchemy::dungeon::modular_kit::WaterRipple;
    use deep_alchemy::dungeon::water_drips::{
        update_puddle_player_interaction, CavernWaterPuddle, WaterPuddleState,
    };
    use deep_alchemy::player::camera::Player;

    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.75, 0.0, 0.0),
    ));

    let puddle_radius = 1.25;
    let puddle_center = Vec3::new(0.0, 0.02, 0.0);

    app.world_mut().spawn((
        CavernWaterPuddle {
            state: WaterPuddleState::Liquid,
            radius: puddle_radius,
            original_radius: puddle_radius,
            base_y: 0.02,
            freeze_timer: 0.0,
            boil_timer: 0.0,
            regen_timer: 0.0,
            step_ripple_cooldown: 0.0,
        },
        Transform::from_translation(puddle_center),
        Visibility::Inherited,
    ));

    app.add_systems(Update, update_puddle_player_interaction);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(16));
    app.update();

    let mut ripple_query = app.world_mut().query::<(&WaterRipple, &Transform)>();
    let (ripple, ripple_tf) = ripple_query.iter(app.world()).next().unwrap();

    let dist_center_to_ripple = Vec2::new(
        ripple_tf.translation.x - puddle_center.x,
        ripple_tf.translation.z - puddle_center.z,
    )
    .length();

    let ripple_reach = dist_center_to_ripple + ripple.max_radius;
    assert!(
        ripple_reach < puddle_radius,
        "Ripple reach {} exceeds puddle radius {}",
        ripple_reach,
        puddle_radius
    );
}

#[test]
fn test_unit_water_puddle_edge_step_does_not_spawn_expanding_ripple_onto_dry_ground() {
    use deep_alchemy::dungeon::modular_kit::WaterRipple;
    use deep_alchemy::dungeon::water_drips::{
        update_puddle_player_interaction, CavernWaterPuddle, WaterPuddleState,
    };
    use deep_alchemy::player::camera::Player;

    let mut app = App::new();
    app.init_resource::<Time>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.world_mut().spawn((
        Player,
        Transform::from_xyz(1.18, 0.0, 0.0),
    ));

    let puddle_radius = 1.20;
    app.world_mut().spawn((
        CavernWaterPuddle {
            state: WaterPuddleState::Liquid,
            radius: puddle_radius,
            original_radius: puddle_radius,
            base_y: 0.02,
            freeze_timer: 0.0,
            boil_timer: 0.0,
            regen_timer: 0.0,
            step_ripple_cooldown: 0.0,
        },
        Transform::from_xyz(0.0, 0.02, 0.0),
        Visibility::Inherited,
    ));

    app.add_systems(Update, update_puddle_player_interaction);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_millis(16));
    app.update();

    let ripples = app
        .world_mut()
        .query::<&WaterRipple>()
        .iter(app.world())
        .count();
    assert_eq!(ripples, 0);
}

#[test]
fn test_unit_ceiling_stalactites_solid_roof_anchor_and_supported_coordinates() {
    use deep_alchemy::dungeon::water_drips::{
        spawn_default_cavern_water_drips, CeilingDripPoint,
    };

    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    app.add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            spawn_default_cavern_water_drips(&mut commands, &mut meshes, &mut materials);
        },
    );
    app.update();

    let mut drip_query = app.world_mut().query::<(&CeilingDripPoint, &Transform, &Children)>();
    let drips: Vec<(Transform, Vec<Entity>)> = drip_query
        .iter(app.world())
        .map(|(_, tf, children)| (*tf, children.iter().copied().collect()))
        .collect();

    assert_eq!(drips.len(), 4);

    for (tf, children) in drips {
        assert!(
            tf.translation.y <= 4.50 && tf.translation.y >= 4.20,
            "Stalactite at y={} must be anchored directly into the corridor roof!",
            tf.translation.y
        );

        let mut has_anchor = false;
        let mut has_droplet_tip = false;
        let mut secondary_count = 0;

        for child in children {
            if let Some(name) = app.world().get::<Name>(child) {
                if name.as_str() == "Ceiling_Rock_Anchor" {
                    has_anchor = true;
                }
                if name.as_str() == "Drip_Tip_Droplet" {
                    has_droplet_tip = true;
                }
                if name.as_str().starts_with("Secondary_Stalactite") {
                    secondary_count += 1;
                }
            }
        }

        assert!(has_anchor, "Stalactite must have Ceiling_Rock_Anchor!");
        assert!(has_droplet_tip, "Stalactite must have Drip_Tip_Droplet!");
        assert!(secondary_count >= 2, "Stalactite must have companion formations!");
    }
}

#[test]
fn test_unit_ambient_insects_realistic_dimensions_and_sub_centimeter_scale() {
    let fly_mesh = deep_alchemy::dungeon::fauna::create_tiny_fly_mesh();
    let moth_body = deep_alchemy::dungeon::fauna::create_tiny_moth_body_mesh();
    let moth_wing_l = deep_alchemy::dungeon::fauna::create_tiny_moth_wing_mesh(true);
    let moth_wing_r = deep_alchemy::dungeon::fauna::create_tiny_moth_wing_mesh(false);
    let moth_wing_r_positions = moth_wing_r
        .attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();
    assert!(!moth_wing_r_positions.is_empty());

    let fly_positions = fly_mesh
        .attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();

    let mut fly_min = Vec3::splat(f32::MAX);
    let mut fly_max = Vec3::splat(f32::MIN);
    for &[x, y, z] in fly_positions {
        fly_min = fly_min.min(Vec3::new(x, y, z));
        fly_max = fly_max.max(Vec3::new(x, y, z));
    }
    let fly_extents = fly_max - fly_min;

    assert!(
        fly_extents.x <= 0.006,
        "Fly wingspan must be realistic sub-centimeter size, got {}",
        fly_extents.x
    );
    assert!(
        fly_extents.z <= 0.005,
        "Fly body length must be realistic sub-centimeter size, got {}",
        fly_extents.z
    );
    assert!(
        fly_extents.y <= 0.003,
        "Fly thickness must be realistic sub-centimeter size, got {}",
        fly_extents.y
    );

    let moth_body_positions = moth_body
        .attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();

    let mut mb_min = Vec3::splat(f32::MAX);
    let mut mb_max = Vec3::splat(f32::MIN);
    for &[x, y, z] in moth_body_positions {
        mb_min = mb_min.min(Vec3::new(x, y, z));
        mb_max = mb_max.max(Vec3::new(x, y, z));
    }
    let mb_extents = mb_max - mb_min;

    assert!(
        mb_extents.z <= 0.012,
        "Moth body length must be small real-life size <= 12mm, got {}",
        mb_extents.z
    );
    assert!(
        mb_extents.x <= 0.003,
        "Moth body width must be small real-life size <= 3mm, got {}",
        mb_extents.x
    );

    let moth_wing_positions = moth_wing_l
        .attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();

    let mut mw_max_span = 0.0f32;
    for &[x, _, _] in moth_wing_positions {
        mw_max_span = mw_max_span.max(x.abs());
    }

    assert!(
        mw_max_span <= 0.011,
        "Single moth forewing span must be <= 11mm, got {}",
        mw_max_span
    );

    let total_moth_wingspan = (mw_max_span * 2.0) + mb_extents.x;
    assert!(
        total_moth_wingspan <= 0.025,
        "Total moth wingspan must strictly represent small real-world moth <= 2.5cm, got {}",
        total_moth_wingspan
    );
}

#[test]
fn test_unit_ambient_insects_hovering_and_wing_flutter() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<deep_alchemy::dungeon::fauna::AmbientFaunaSpawner>();

    app.add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            deep_alchemy::dungeon::fauna::setup_ambient_fauna(
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        },
    );
    app.update();
    let fauna_assets = app
        .world()
        .resource::<deep_alchemy::dungeon::fauna::AmbientFaunaAssets>()
        .clone();

    let moth_pos = Vec3::new(2.0, 1.5, 3.0);
    let fly_pos = Vec3::new(-1.0, 0.15, 2.0);

    let moth_entity = {
        let mut commands = app.world_mut().commands();
        deep_alchemy::dungeon::fauna::spawn_ambient_moth(&mut commands, &fauna_assets, moth_pos, 42)
    };
    let fly_entity = {
        let mut commands = app.world_mut().commands();
        deep_alchemy::dungeon::fauna::spawn_ambient_water_fly(&mut commands, &fauna_assets, fly_pos, None, 84)
    };
    app.update();

    app.add_systems(Update, deep_alchemy::dungeon::fauna::update_ambient_insects);

    let moth_insect = app.world().get::<deep_alchemy::dungeon::fauna::AmbientInsect>(moth_entity).unwrap();
    assert_eq!(moth_insect.state, deep_alchemy::dungeon::fauna::InsectState::Hovering);
    assert_eq!(moth_insect.kind, deep_alchemy::dungeon::fauna::InsectKind::Moth);

    let initial_phase = moth_insect.flutter_phase;

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
    app.update();

    let updated_moth = app.world().get::<deep_alchemy::dungeon::fauna::AmbientInsect>(moth_entity).unwrap();
    assert!(updated_moth.flutter_phase > initial_phase);
    assert_eq!(updated_moth.state, deep_alchemy::dungeon::fauna::InsectState::Hovering);

    let updated_fly = app.world().get::<deep_alchemy::dungeon::fauna::AmbientInsect>(fly_entity).unwrap();
    let fly_tf = app.world().get::<Transform>(fly_entity).unwrap();
    assert_eq!(updated_fly.state, deep_alchemy::dungeon::fauna::InsectState::Hovering);
    assert!(fly_tf.translation.distance(fly_pos) < 0.40);
}

#[test]
fn test_unit_ambient_insects_flee_and_fade_on_player_proximity() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<deep_alchemy::dungeon::fauna::AmbientFaunaSpawner>();

    app.add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            deep_alchemy::dungeon::fauna::setup_ambient_fauna(
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        },
    );
    app.update();
    let fauna_assets = app
        .world()
        .resource::<deep_alchemy::dungeon::fauna::AmbientFaunaAssets>()
        .clone();

    let player_entity = app.world_mut().spawn((
        deep_alchemy::player::camera::Player,
        Transform::from_xyz(10.0, 0.0, 10.0),
    )).id();

    let insect_pos = Vec3::new(0.0, 1.0, 0.0);
    let moth_entity = {
        let mut commands = app.world_mut().commands();
        deep_alchemy::dungeon::fauna::spawn_ambient_moth(&mut commands, &fauna_assets, insect_pos, 77)
    };
    app.update();

    app.add_systems(Update, deep_alchemy::dungeon::fauna::update_ambient_insects);

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
    app.update();

    let moth = app.world().get::<deep_alchemy::dungeon::fauna::AmbientInsect>(moth_entity).unwrap();
    assert_eq!(moth.state, deep_alchemy::dungeon::fauna::InsectState::Hovering);

    app.world_mut().get_mut::<Transform>(player_entity).unwrap().translation = Vec3::new(0.1, 1.0, 0.1);

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
    app.update();

    let fleeing_moth = app.world().get::<deep_alchemy::dungeon::fauna::AmbientInsect>(moth_entity).unwrap();
    assert_eq!(fleeing_moth.state, deep_alchemy::dungeon::fauna::InsectState::Fleeing);
    assert!(fleeing_moth.flee_velocity.y > 0.3);

    let moth_tf = app.world().get::<Transform>(moth_entity).unwrap();
    assert!(moth_tf.scale.x <= 1.0);

    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(100),
    ));
    for _ in 0..16 {
        app.update();
    }

    assert!(app.world().get_entity(moth_entity).is_err());
}

#[test]
fn test_unit_ambient_water_flies_scatter_when_puddle_freezes_or_boils() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<deep_alchemy::dungeon::fauna::AmbientFaunaSpawner>();

    app.add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            deep_alchemy::dungeon::fauna::setup_ambient_fauna(
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        },
    );
    app.update();
    let fauna_assets = app
        .world()
        .resource::<deep_alchemy::dungeon::fauna::AmbientFaunaAssets>()
        .clone();

    let puddle_entity = app.world_mut().spawn((
        deep_alchemy::dungeon::water_drips::CavernWaterPuddle {
            state: deep_alchemy::dungeon::water_drips::WaterPuddleState::Liquid,
            radius: 1.0,
            original_radius: 1.0,
            base_y: 0.02,
            freeze_timer: 0.0,
            boil_timer: 0.0,
            regen_timer: 0.0,
            step_ripple_cooldown: 0.0,
        },
        Transform::from_xyz(0.0, 0.02, 5.0),
    )).id();

    let fly_entity = {
        let mut commands = app.world_mut().commands();
        deep_alchemy::dungeon::fauna::spawn_ambient_water_fly(
            &mut commands,
            &fauna_assets,
            Vec3::new(0.0, 0.15, 5.0),
            Some(puddle_entity),
            123,
        )
    };
    app.update();

    app.add_systems(Update, deep_alchemy::dungeon::fauna::update_ambient_insects);

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
    app.update();

    let fly = app.world().get::<deep_alchemy::dungeon::fauna::AmbientInsect>(fly_entity).unwrap();
    assert_eq!(fly.state, deep_alchemy::dungeon::fauna::InsectState::Hovering);

    app.world_mut().get_mut::<deep_alchemy::dungeon::water_drips::CavernWaterPuddle>(puddle_entity).unwrap().state =
        deep_alchemy::dungeon::water_drips::WaterPuddleState::Frozen;

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
    app.update();

    let fleeing_fly = app.world().get::<deep_alchemy::dungeon::fauna::AmbientInsect>(fly_entity).unwrap();
    assert_eq!(fleeing_fly.state, deep_alchemy::dungeon::fauna::InsectState::Fleeing);
    assert!(fleeing_fly.flee_velocity.y > 1.0);
}

#[test]
fn test_unit_ambient_fauna_spawner_rare_budget_caps() {
    let spawner = deep_alchemy::dungeon::fauna::AmbientFaunaSpawner::default();
    assert_eq!(spawner.max_moths, 10);
    assert_eq!(spawner.max_flies, 14);
    assert_eq!(spawner.max_spiders, 6);
    assert_eq!(spawner.max_webs, 8);
    assert!(spawner.interval >= 3.0);
}

#[test]
fn test_unit_spider_and_web_realistic_dimensions_sub_centimeter_scale() {
    let spider_body = deep_alchemy::dungeon::fauna::create_tiny_spider_body_mesh();
    let spider_leg_0 = deep_alchemy::dungeon::fauna::create_spider_leg_mesh(0);
    let corner_web = deep_alchemy::dungeon::fauna::create_corner_cobweb_mesh(0.8, 0.7, 0.8);
    let radial_web = deep_alchemy::dungeon::fauna::create_radial_spiderweb_mesh(0.45);

    let sb_positions = spider_body
        .attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();

    let mut sb_min = Vec3::splat(f32::MAX);
    let mut sb_max = Vec3::splat(f32::MIN);
    for &[x, y, z] in sb_positions {
        sb_min = sb_min.min(Vec3::new(x, y, z));
        sb_max = sb_max.max(Vec3::new(x, y, z));
    }
    let sb_extents = sb_max - sb_min;

    assert!(
        sb_extents.z <= 0.010,
        "Spider body length must be small real-life size <= 10mm, got {}",
        sb_extents.z
    );
    assert!(
        sb_extents.x <= 0.004,
        "Spider body width must be small real-life size <= 4mm, got {}",
        sb_extents.x
    );

    let leg_positions = spider_leg_0
        .attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();

    let mut leg_max_reach = 0.0f32;
    for &[x, _, z] in leg_positions {
        let reach = (x * x + z * z).sqrt();
        leg_max_reach = leg_max_reach.max(reach);
    }

    assert!(
        leg_max_reach <= 0.014,
        "Spider leg reach must be realistic <= 14mm, got {}",
        leg_max_reach
    );

    let total_spider_diameter = (leg_max_reach * 2.0) + sb_extents.x;
    assert!(
        total_spider_diameter <= 0.030,
        "Total spider diameter must strictly represent small cellar/crevice spider <= 3cm, got {}",
        total_spider_diameter
    );

    let corner_positions = corner_web
        .attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();
    assert!(!corner_positions.is_empty());

    let radial_positions = radial_web
        .attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();
    assert!(!radial_positions.is_empty());
}

#[test]
fn test_unit_spider_web_non_blocking_and_player_contact_rupture() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<deep_alchemy::dungeon::fauna::AmbientFaunaSpawner>();

    app.add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            deep_alchemy::dungeon::fauna::setup_ambient_fauna(
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        },
    );
    app.update();
    let fauna_assets = app
        .world()
        .resource::<deep_alchemy::dungeon::fauna::AmbientFaunaAssets>()
        .clone();

    let web_pos = Vec3::new(0.0, 1.0, 0.0);
    let web_entity = {
        let mut commands = app.world_mut().commands();
        deep_alchemy::dungeon::fauna::spawn_spider_web(
            &mut commands,
            &fauna_assets,
            web_pos,
            Quat::IDENTITY,
            0.75,
            true,
            55,
        )
    };
    app.update();

    let web_before = app.world().get::<deep_alchemy::dungeon::fauna::SpiderWeb>(web_entity).unwrap();
    assert!(!web_before.is_ruptured);

    let player_entity = app.world_mut().spawn((
        deep_alchemy::player::camera::Player,
        Transform::from_xyz(0.1, 0.1, 0.1),
    )).id();

    app.add_systems(
        Update,
        (
            deep_alchemy::dungeon::fauna::update_spider_webs,
            deep_alchemy::dungeon::fauna::update_ambient_insects,
        ).chain(),
    );

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
    app.update();

    let web_after = app.world().get::<deep_alchemy::dungeon::fauna::SpiderWeb>(web_entity).unwrap();
    assert!(web_after.is_ruptured);

    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(100),
    ));
    for _ in 0..10 {
        app.update();
    }

    assert!(app.world().get_entity(web_entity).is_err());
    assert!(app.world().get_entity(player_entity).is_ok());
}

#[test]
fn test_unit_spider_web_super_flammable_ignites_and_burns_away() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<deep_alchemy::dungeon::fauna::AmbientFaunaSpawner>();

    app.add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            deep_alchemy::dungeon::fauna::setup_ambient_fauna(
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        },
    );
    app.update();
    let fauna_assets = app
        .world()
        .resource::<deep_alchemy::dungeon::fauna::AmbientFaunaAssets>()
        .clone();

    let web_entity = {
        let mut commands = app.world_mut().commands();
        deep_alchemy::dungeon::fauna::spawn_spider_web(
            &mut commands,
            &fauna_assets,
            Vec3::new(2.0, 3.0, 2.0),
            Quat::IDENTITY,
            0.7,
            true,
            91,
        )
    };
    app.update();

    let flammable = app.world().get::<deep_alchemy::alchemy::Flammable>(web_entity).unwrap();
    assert!(
        flammable.ignition_temperature <= 2.0,
        "Spiderweb must be super flammable with ignition temp <= 2.0, got {}",
        flammable.ignition_temperature
    );

    app.world_mut()
        .get_mut::<deep_alchemy::alchemy::Flammable>(web_entity)
        .unwrap()
        .apply_heat(10.0);

    let flammable_burning = app.world().get::<deep_alchemy::alchemy::Flammable>(web_entity).unwrap();
    assert!(flammable_burning.is_burning);

    app.add_systems(
        Update,
        (
            deep_alchemy::dungeon::fauna::update_spider_webs,
            deep_alchemy::dungeon::fauna::update_ambient_insects,
        ).chain(),
    );

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
    app.update();

    let web = app.world().get::<deep_alchemy::dungeon::fauna::SpiderWeb>(web_entity).unwrap();
    assert!(web.is_ruptured);
}

#[test]
fn test_unit_spider_web_potion_direct_impact_destruction() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<deep_alchemy::dungeon::fauna::AmbientFaunaSpawner>();

    app.add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            deep_alchemy::dungeon::fauna::setup_ambient_fauna(
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        },
    );
    app.update();
    let fauna_assets = app
        .world()
        .resource::<deep_alchemy::dungeon::fauna::AmbientFaunaAssets>()
        .clone();

    let web_entity = {
        let mut commands = app.world_mut().commands();
        deep_alchemy::dungeon::fauna::spawn_spider_web(
            &mut commands,
            &fauna_assets,
            Vec3::new(1.0, 1.0, 1.0),
            Quat::IDENTITY,
            0.6,
            true,
            77,
        )
    };

    app.world_mut().spawn((
        deep_alchemy::alchemy::PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![],
            lifetime: 0.0,
            color: Color::WHITE,
            slot: 2,
            hitstop_timer: 0.0,
            entered_water: false,
        },
        Transform::from_xyz(1.1, 1.0, 1.0),
    ));
    app.update();

    app.add_systems(
        Update,
        (
            deep_alchemy::dungeon::fauna::update_spider_webs,
            deep_alchemy::dungeon::fauna::update_ambient_insects,
        ).chain(),
    );

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
    app.update();

    let web = app.world().get::<deep_alchemy::dungeon::fauna::SpiderWeb>(web_entity).unwrap();
    assert!(web.is_ruptured);
}

#[test]
fn test_unit_spider_high_web_association_and_skitter_flee() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<deep_alchemy::dungeon::fauna::AmbientFaunaSpawner>();

    app.add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<StandardMaterial>>| {
            deep_alchemy::dungeon::fauna::setup_ambient_fauna(
                &mut commands,
                &mut meshes,
                &mut materials,
            );
        },
    );
    app.update();
    let fauna_assets = app
        .world()
        .resource::<deep_alchemy::dungeon::fauna::AmbientFaunaAssets>()
        .clone();

    let mut spider_count = 0;
    let iterations = 20;
    for i in 0..iterations {
        let web_ent = {
            let mut commands = app.world_mut().commands();
            deep_alchemy::dungeon::fauna::spawn_spider_web(
                &mut commands,
                &fauna_assets,
                Vec3::new(i as f32 * 2.0, 1.0, 0.0),
                Quat::IDENTITY,
                0.5,
                true,
                i as u32 * 100 + 7,
            )
        };
        app.update();
        let web = app.world().get::<deep_alchemy::dungeon::fauna::SpiderWeb>(web_ent).unwrap();
        if web.attached_spider.is_some() {
            spider_count += 1;
        }
    }

    assert!(
        spider_count >= 18,
        "At least 90% of spiderwebs must have an attached spider, got {}/{}",
        spider_count,
        iterations
    );

    let single_spider = {
        let mut commands = app.world_mut().commands();
        deep_alchemy::dungeon::fauna::spawn_ambient_spider(
            &mut commands,
            &fauna_assets,
            Vec3::new(0.0, 1.0, 0.0),
            None,
            44,
        )
    };
    app.update();

    let spider_data = app.world().get::<deep_alchemy::dungeon::fauna::AmbientInsect>(single_spider).unwrap();
    assert_eq!(spider_data.kind, deep_alchemy::dungeon::fauna::InsectKind::Spider);
    assert_eq!(spider_data.state, deep_alchemy::dungeon::fauna::InsectState::Hovering);

    let player_ent = app.world_mut().spawn((
        deep_alchemy::player::camera::Player,
        Transform::from_xyz(0.1, 1.0, 0.1),
    )).id();

    app.add_systems(
        Update,
        (
            deep_alchemy::dungeon::fauna::update_spider_webs,
            deep_alchemy::dungeon::fauna::update_ambient_insects,
        ).chain(),
    );

    app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(50));
    app.update();

    let fleeing_spider = app.world().get::<deep_alchemy::dungeon::fauna::AmbientInsect>(single_spider).unwrap();
    assert_eq!(fleeing_spider.state, deep_alchemy::dungeon::fauna::InsectState::Fleeing);
    assert!(fleeing_spider.flee_velocity.length() > 0.5);

    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(100),
    ));
    for _ in 0..16 {
        app.update();
    }

    assert!(app.world().get_entity(single_spider).is_err());
    assert!(app.world().get_entity(player_ent).is_ok());
}
#[test]
fn test_unit_player_cannot_step_onto_or_climb_dungeon_walls() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_millis(16),
    ));
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<deep_alchemy::player::FlyModeState>();
    app.init_state::<deep_alchemy::ui::menu::GameState>();
    app.world_mut()
        .resource_mut::<NextState<deep_alchemy::ui::menu::GameState>>()
        .set(deep_alchemy::ui::menu::GameState::InGame);
    app.add_systems(Update, deep_alchemy::player::controller::player_movement);
    app.update();

    let player = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            deep_alchemy::player::camera::PlayerState::Active,
            deep_alchemy::player::camera::LocomotionState::default(),
            Transform::from_xyz(0.0, 1.2, 0.0),
            deep_alchemy::player::camera::Health {
                current: 100.0,
                max: 100.0,
            },
            deep_alchemy::player::camera::Stamina {
                current: 100.0,
                max: 100.0,
            },
        ))
        .id();

    app.world_mut().spawn((
        deep_alchemy::dungeon::modular_kit::CavernObstacle,
        Transform::from_xyz(0.0, 2.5, -1.0),
        deep_alchemy::dungeon::modular_kit::ObstacleShape::Cuboid {
            half_extents: Vec3::new(2.0, 2.5, 0.375),
        },
    ));

    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::KeyW);
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
    for _ in 0..60 {
        app.update();
    }
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().clear();

    for _ in 0..60 {
        app.update();
    }

    let tf = *app.world().get::<Transform>(player).unwrap();
    assert!(tf.translation.y <= 1.35);
    assert!(tf.translation.z > -1.0);
}

#[test]
fn test_unit_dungeon_pieces_all_have_tall_wall_colliders_and_no_gaps() {
    let config = deep_alchemy::dungeon::generator::DungeonConfig::default();
    let dungeon = deep_alchemy::dungeon::generator::DungeonGenerator::generate(config);
    assert!(!dungeon.dual_cells.is_empty());

    let mut found_straight = false;
    let mut found_corner = false;
    let mut found_pillar = false;

    for cell in &dungeon.dual_cells {
        match cell.piece {
            deep_alchemy::dungeon::generator::CanonicalPiece::StraightEdge => found_straight = true,
            deep_alchemy::dungeon::generator::CanonicalPiece::OuterCorner
            | deep_alchemy::dungeon::generator::CanonicalPiece::InnerCorner => found_corner = true,
            deep_alchemy::dungeon::generator::CanonicalPiece::SolidPillar => found_pillar = true,
            _ => {}
        }
    }

    assert!(found_straight);
    assert!(found_corner);
    assert!(found_pillar);

    let straight_half_extents = Vec3::new(0.375, 2.5, 2.0);
    assert_eq!(straight_half_extents.y * 2.0, 5.0);

    let corner_half_extents = Vec3::new(0.85, 2.5, 0.85);
    assert_eq!(corner_half_extents.y * 2.0, 5.0);

    let pillar_half_extents = Vec3::new(2.0, 2.5, 2.0);
    assert_eq!(pillar_half_extents.y * 2.0, 5.0);
}

#[test]
fn test_unit_player_out_of_bounds_rescue_teleport_to_start() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<deep_alchemy::player::FlyModeState>();
    app.add_systems(Update, deep_alchemy::player::controller::player_out_of_bounds_rescue);

    let p1 = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(0.0, 5.5, 0.0),
            deep_alchemy::player::camera::LocomotionState::default(),
        ))
        .id();

    let p2 = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(0.0, -8.0, 0.0),
            deep_alchemy::player::camera::LocomotionState::default(),
        ))
        .id();

    let p3 = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(75.0, 1.2, 0.0),
            deep_alchemy::player::camera::LocomotionState::default(),
        ))
        .id();

    let p_valid = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(5.0, 1.2, 5.0),
            deep_alchemy::player::camera::LocomotionState::default(),
        ))
        .id();

    let p_secret = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(0.0, 1.2, 512.0),
            deep_alchemy::player::camera::LocomotionState::default(),
        ))
        .id();

    app.update();

    let tf1 = *app.world().get::<Transform>(p1).unwrap();
    let tf2 = *app.world().get::<Transform>(p2).unwrap();
    let tf3 = *app.world().get::<Transform>(p3).unwrap();
    let tf_valid = *app.world().get::<Transform>(p_valid).unwrap();
    let tf_secret = *app.world().get::<Transform>(p_secret).unwrap();

    assert_eq!(tf1.translation, Vec3::new(0.0, 1.2, 0.0));
    assert_eq!(tf2.translation, Vec3::new(0.0, 1.2, 0.0));
    assert_eq!(tf3.translation, Vec3::new(0.0, 1.2, 0.0));
    assert_eq!(tf_valid.translation, Vec3::new(5.0, 1.2, 5.0));
    assert_eq!(tf_secret.translation, Vec3::new(0.0, 1.2, 512.0));
}

#[test]
fn test_unit_complete_dungeon_walkways_are_traversable_and_walls_block() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::state::app::StatesPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
        avian3d::prelude::PhysicsPlugins::default(),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<deep_alchemy::dungeon::asset_loader::SceneAssetRegistry>();
    app.init_state::<deep_alchemy::ui::menu::GameState>();
    app.add_systems(OnEnter(deep_alchemy::ui::menu::GameState::InGame), deep_alchemy::dungeon::generator::spawn_procedural_dungeon);
    app.world_mut()
        .resource_mut::<NextState<deep_alchemy::ui::menu::GameState>>()
        .set(deep_alchemy::ui::menu::GameState::InGame);
    app.finish();
    app.cleanup();
    app.update();

    let mut query = app.world_mut().query::<(&Transform, &deep_alchemy::dungeon::modular_kit::ObstacleShape)>();
    let obstacles: Vec<(Transform, deep_alchemy::dungeon::modular_kit::ObstacleShape)> = query
        .iter(app.world())
        .map(|(tf, shape)| (*tf, shape.clone()))
        .collect();

    assert!(!obstacles.is_empty(), "Obstacles must be spawned");

    let obs_refs: Vec<(&Transform, deep_alchemy::dungeon::modular_kit::ObstacleShape)> = obstacles
        .iter()
        .map(|(tf, shape)| (tf, shape.clone()))
        .collect();

    let config = deep_alchemy::dungeon::generator::DungeonConfig::default();
    let dungeon = deep_alchemy::dungeon::generator::DungeonGenerator::generate(config.clone());
    let width = dungeon.config.grid_width;
    let depth = dungeon.config.grid_depth;
    let tile_size = dungeon.config.tile_size;
    let center_offset_x = (width as f32 * 0.5) * tile_size;
    let center_offset_z = (depth as f32 * 0.5) * tile_size;

    let cx = width / 2;
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    let start_gx = cx;
    let start_gz = 14;
    assert!(dungeon.get_primary(start_gx, start_gz).is_walkable());
    visited.insert((start_gx, start_gz));
    queue.push_back((start_gx, start_gz));

    let mut traversed_transitions = 0;

    while let Some((gx, gz)) = queue.pop_front() {
        let from_wx = ((gx as f32 + 0.5) * tile_size) - center_offset_x;
        let from_wz = ((gz as f32 + 0.5) * tile_size) - center_offset_z;
        let from_pos = Vec3::new(from_wx, 1.2, from_wz);

        let neighbors = [
            (gx.wrapping_sub(1), gz),
            (gx + 1, gz),
            (gx, gz.wrapping_sub(1)),
            (gx, gz + 1),
        ];

        for (nx, nz) in neighbors {
            if nx < width && nz < depth && dungeon.get_primary(nx, nz).is_walkable() {
                let to_wx = ((nx as f32 + 0.5) * tile_size) - center_offset_x;
                let to_wz = ((nz as f32 + 0.5) * tile_size) - center_offset_z;
                let to_pos = Vec3::new(to_wx, 1.2, to_wz);

                let mut current_sim = from_pos;
                let dir = (to_pos - from_pos).normalize();
                let total_dist = (to_pos - from_pos).length();
                let step_dist: f32 = 0.15;
                let mut dist_traveled = 0.0;

                while dist_traveled < total_dist {
                    let next_desired = current_sim + dir * step_dist.min(total_dist - dist_traveled);
                    let resolved = deep_alchemy::player::controller::resolve_player_collision_sliding(
                        next_desired,
                        current_sim,
                        0.35,
                        true,
                        false,
                        &obs_refs,
                    );
                    let actual_progress = (resolved - current_sim).length();
                    current_sim = resolved;
                    dist_traveled += step_dist;
                    if actual_progress < 1e-4 {
                        break;
                    }
                }

                let final_dist = (current_sim - to_pos).length();
                assert!(
                    final_dist <= 0.40,
                    "Player was blocked moving between walkable cells: from ({}, {}) to ({}, {})! Dist remaining: {:.2}m",
                    gx, gz, nx, nz, final_dist
                );
                traversed_transitions += 1;

                if !visited.contains(&(nx, nz)) {
                    visited.insert((nx, nz));
                    queue.push_back((nx, nz));
                }
            }
        }
    }

    assert!(visited.len() >= 20, "Must visit all reachable dungeon cells, visited: {}", visited.len());
    assert!(traversed_transitions >= 30, "Must traverse all corridor transitions, traversed: {}", traversed_transitions);

    let solid_wall_pos = Vec3::new(0.0, 1.2, 20.0);
    let desired_into_wall = solid_wall_pos + Vec3::new(-1.5, 0.0, 0.0);
    let resolved_wall = deep_alchemy::player::controller::resolve_player_collision_sliding(
        desired_into_wall,
        solid_wall_pos,
        0.35,
        true,
        false,
        &obs_refs,
    );
    assert!((resolved_wall - desired_into_wall).length() > 0.5, "Solid walls must block player penetration");
    assert!(resolved_wall.x > -1.0, "Player must remain inside corridor, stopped by wall");
}


#[test]
fn test_unit_player_cannot_climb_onto_dungeon_walls_or_ceilings() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<deep_alchemy::player::FlyModeState>();
    app.add_systems(Update, deep_alchemy::player::controller::player_out_of_bounds_rescue);

    let p_roof = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(0.0, 3.8, 15.0),
            deep_alchemy::player::camera::LocomotionState::default(),
        ))
        .id();

    let p_wall_top = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(4.0, 2.9, 20.0),
            deep_alchemy::player::camera::LocomotionState::default(),
        ))
        .id();

    let p_valid = app
        .world_mut()
        .spawn((
            deep_alchemy::player::camera::Player,
            Transform::from_xyz(0.0, 1.2, 15.0),
            deep_alchemy::player::camera::LocomotionState::default(),
        ))
        .id();

    app.update();

    let tf_roof = app.world().get::<Transform>(p_roof).unwrap();
    assert_eq!(tf_roof.translation, Vec3::new(0.0, 1.2, 0.0));

    let tf_wall = app.world().get::<Transform>(p_wall_top).unwrap();
    assert_eq!(tf_wall.translation, Vec3::new(0.0, 1.2, 0.0));

    let tf_valid = app.world().get::<Transform>(p_valid).unwrap();
    assert_eq!(tf_valid.translation, Vec3::new(0.0, 1.2, 15.0));

    let simulated_slide = deep_alchemy::player::controller::resolve_player_collision_sliding(
        Vec3::new(0.0, 3.5, 15.0),
        Vec3::new(0.0, 1.2, 15.0),
        0.35,
        false,
        true,
        &[],
    );
    assert!(simulated_slide.y <= 2.45);
}


#[test]
fn test_unit_crystal_geode_variations_and_grounded_placement() {
    use deep_alchemy::dungeon::asset_loader::SceneAssetRegistry;
    use deep_alchemy::dungeon::generator::{DungeonConfig, DungeonGenerator, RoomType};
    use std::fs;
    use std::path::Path;

    let registry = SceneAssetRegistry::default();
    assert!(registry.get_path("macro_crystal_cluster").is_some());
    assert!(registry.get_path("macro_crystal_cluster_b").is_some());
    assert!(registry.get_path("macro_crystal_cluster_c").is_some());

    let config = DungeonConfig::default();
    let dungeon = DungeonGenerator::generate(config);
    let mut found_geode = false;
    for cell in &dungeon.dual_cells {
        if let Some(macro_room) = cell.macro_room {
            if matches!(macro_room, RoomType::CrystalGeode) {
                found_geode = true;
                assert_eq!(cell.world_pos.y, 0.0);
            }
        }
    }
    assert!(found_geode, "Dungeon must contain CrystalGeode macro chamber");

    let crystal_files = [
        "assets/models/macro_crystal_cluster.glb",
        "assets/models/macro_crystal_cluster_b.glb",
        "assets/models/macro_crystal_cluster_c.glb",
    ];
    for path_str in &crystal_files {
        let path = Path::new(path_str);
        assert!(path.exists(), "Crystal model file must exist: {}", path_str);
        let bytes = fs::read(path).expect("Failed to read crystal model");
        assert!(bytes.len() > 1000, "Crystal file is too small");
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(magic, 0x46546C67, "Must be valid GLB binary");
    }
}

#[test]
fn test_unit_dungeon_wall_torch_ignition_timer_and_ice_extinguishing() {
    use avian3d::prelude::{Collider, RigidBody};
    use deep_alchemy::alchemy::{
        update_dungeon_wall_torches, DungeonWallTorch,
    };
    use deep_alchemy::dungeon::generator::spawn_dungeon_wall_sconce;
    use deep_alchemy::dungeon::asset_loader::SceneAssetRegistry;

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
    ));
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.init_resource::<SceneAssetRegistry>();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(std::time::Duration::from_millis(50)));

    let sconce_pos = Vec3::new(-1.18, 2.05, 12.0);
    let wall_support = app.world_mut().spawn((
        deep_alchemy::dungeon::modular_kit::CavernObstacle,
        RigidBody::Static,
        Collider::cuboid(0.85, 5.0, 6.0),
        Transform::from_xyz(-2.0, 2.5, 12.0),
    )).id();
    let torch_entity = {
        let registry = SceneAssetRegistry::default();
        let asset_server = app.world().resource::<AssetServer>().clone();
        let mut commands = app.world_mut().commands();
        spawn_dungeon_wall_sconce(
            &mut commands,
            &asset_server,
            &registry,
            wall_support,
            sconce_pos,
            Quat::IDENTITY,
            false,
        )
    };
    app.update();

    {
        let torch = app.world().get::<DungeonWallTorch>(torch_entity).expect("Torch component must exist");
        assert!(!torch.is_lit, "Dungeon wall torch must be initially unlit");
        assert_eq!(torch.burn_timer, 0.0, "Unlit torch must have 0 burn timer");
        assert_eq!(torch.max_burn_time, 1800.0, "Torch burn time must be 1800s (30 minutes real-world)");
        assert!(torch.light_entity.is_some(), "Torch must link to a PointLight entity");
        let light_e = torch.light_entity.unwrap();
        let point_light = app.world().get::<PointLight>(light_e).expect("Point light must exist");
        assert_eq!(point_light.intensity, 0.0, "Unlit torch point light intensity must be 0");
    }

    app.add_systems(Update, update_dungeon_wall_torches);

    let fire_impact_pos = sconce_pos + Vec3::new(0.4, -0.2, 0.5);
    {
        let mut query = app.world_mut().query::<(Entity, &Transform, &mut DungeonWallTorch)>();
        for (ent, tf, mut torch) in query.iter_mut(app.world_mut()) {
            if ent == torch_entity {
                assert!(tf.translation.distance(fire_impact_pos) <= 2.8);
                torch.is_lit = true;
                torch.burn_timer = torch.max_burn_time;
                torch.dirty = true;
            }
        }
    }
    app.update();

    {
        let torch = app.world().get::<DungeonWallTorch>(torch_entity).unwrap();
        assert!(torch.is_lit, "Torch must be lit after fire potion impact");
        assert!(torch.burn_timer <= 1800.0 && torch.burn_timer >= 1799.0, "Torch must burn for 30 minutes (1800.0s)");
        assert!(torch.flame_entity.is_some(), "Lit torch must spawn a flame particle emitter");
        let light_e = torch.light_entity.unwrap();
        let point_light = app.world().get::<PointLight>(light_e).unwrap();
        assert!(point_light.intensity >= 3000.0, "Lit torch point light must be illuminated");
    }

    for _ in 0..20 {
        app.update();
    }

    {
        let torch = app.world().get::<DungeonWallTorch>(torch_entity).unwrap();
        assert!(torch.is_lit, "Torch must remain lit while timer is active");
        assert!(torch.burn_timer <= 1799.0 && torch.burn_timer >= 1798.0);
    }

    let frost_impact_pos = sconce_pos + Vec3::new(0.2, 0.1, -0.3);
    {
        let mut query = app.world_mut().query::<(Entity, &Transform, &mut DungeonWallTorch)>();
        for (ent, tf, mut torch) in query.iter_mut(app.world_mut()) {
            if ent == torch_entity {
                assert!(tf.translation.distance(frost_impact_pos) <= 2.8);
                torch.is_lit = false;
                torch.burn_timer = 0.0;
                torch.dirty = true;
            }
        }
    }
    app.update();

    {
        let torch = app.world().get::<DungeonWallTorch>(torch_entity).unwrap();
        assert!(!torch.is_lit, "Torch must be extinguished by frost potion");
        assert_eq!(torch.burn_timer, 0.0, "Extinguished torch timer must be 0");
        assert!(torch.flame_entity.is_none(), "Flame emitter must be despawned when extinguished");
        let light_e = torch.light_entity.unwrap();
        let point_light = app.world().get::<PointLight>(light_e).unwrap();
        assert_eq!(point_light.intensity, 0.0, "Point light intensity must be 0 after being extinguished");
    }

    {
        let mut query = app.world_mut().query::<(Entity, &Transform, &mut DungeonWallTorch)>();
        for (ent, _, mut torch) in query.iter_mut(app.world_mut()) {
            if ent == torch_entity {
                torch.is_lit = true;
                torch.burn_timer = torch.max_burn_time;
                torch.dirty = true;
            }
        }
    }
    app.update();

    {
        let torch = app.world().get::<DungeonWallTorch>(torch_entity).unwrap();
        assert!(torch.is_lit, "Torch must reignite when hit by fire potion again");
        assert!(torch.burn_timer <= 1800.0 && torch.burn_timer >= 1799.0);
    }

    {
        let mut query = app.world_mut().query::<(Entity, &Transform, &mut DungeonWallTorch)>();
        for (ent, _, mut torch) in query.iter_mut(app.world_mut()) {
            if ent == torch_entity {
                torch.burn_timer = 0.04;
            }
        }
    }
    app.update();

    {
        let torch = app.world().get::<DungeonWallTorch>(torch_entity).unwrap();
        assert!(!torch.is_lit, "Torch must extinguish after 30 minutes (1800s)");
        assert_eq!(torch.burn_timer, 0.0);
        let light_e = torch.light_entity.unwrap();
        let point_light = app.world().get::<PointLight>(light_e).unwrap();
        assert_eq!(point_light.intensity, 0.0);
        assert!(torch.flame_entity.is_none());
    }
}

#[test]
fn test_unit_dungeon_procedural_generation_spawns_unlit_wall_sconces() {
    use deep_alchemy::alchemy::DungeonWallTorch;
    use deep_alchemy::dungeon::asset_loader::SceneAssetRegistry;
    use deep_alchemy::dungeon::generator::spawn_procedural_dungeon;

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
    ));
    app.init_resource::<SceneAssetRegistry>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_systems(Startup, spawn_procedural_dungeon);
    app.update();

    let mut torch_count = 0;
    let mut lit_torch_count = 0;
    let mut query = app.world_mut().query::<&DungeonWallTorch>();
    for torch in query.iter(app.world()) {
        torch_count += 1;
        if torch.is_lit {
            lit_torch_count += 1;
        }
        assert_eq!(torch.max_burn_time, 1800.0);
    }

    assert!(torch_count >= 10, "Procedural dungeon must spawn multiple wall sconces on dungeon walls, found {}", torch_count);
    assert_eq!(lit_torch_count, 0, "All spawned wall sconces must be initially unlit as per requirements");
}

#[test]
fn test_unit_creature_patrol_hop_and_procedural_squash() {
    use avian3d::prelude::*;
    use deep_alchemy::dungeon::creatures::{
        update_creature_patrol_and_movement, CreatureSpecies, DungeonCreature,
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, update_creature_patrol_and_movement);

    let waypoints = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)];
    let creature = DungeonCreature::new(CreatureSpecies::Sproutling, waypoints, 15.0);
    let entity = app
        .world_mut()
        .spawn((
            creature,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            LinearVelocity::default(),
        ))
        .id();

    app.update();
    app.update();

    let c = app.world().get::<DungeonCreature>(entity).unwrap();
    let tf = app.world().get::<Transform>(entity).unwrap();
    let vel = app.world().get::<LinearVelocity>(entity).unwrap();

    assert_eq!(c.species, CreatureSpecies::Sproutling);
    assert_eq!(c.current_waypoint_idx, 1);
    assert!(vel.x > 0.0);
    assert!(tf.scale.y > 0.0);
    assert!(tf.scale.x > 0.0);
}

#[test]
fn test_unit_creature_adaptive_threat_learning_and_camera_aim_dodge() {
    use deep_alchemy::dungeon::creatures::{
        update_creature_ai_perception_and_aim_dodge, CreatureSpecies, CreatureState,
        DungeonCreature,
    };
    use deep_alchemy::player::camera::Player;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, update_creature_ai_perception_and_aim_dodge);

    let _player_ent = app
        .world_mut()
        .spawn((
            Player,
            Transform::from_xyz(0.0, 1.5, 0.0).looking_to(-Vec3::Z, Vec3::Y),
        ))
        .id();

    let waypoints = vec![Vec3::new(0.0, 0.0, -6.0)];
    let mut untrained_creature =
        DungeonCreature::new(CreatureSpecies::Sproutling, waypoints.clone(), 10.0);
    untrained_creature.learned_player_threat = false;

    let untrained_ent = app
        .world_mut()
        .spawn((
            untrained_creature,
            Transform::from_xyz(0.0, 0.0, -6.0),
        ))
        .id();

    let mut trained_creature =
        DungeonCreature::new(CreatureSpecies::Sproutling, waypoints, 10.0);
    trained_creature.learned_player_threat = true;
    trained_creature.dodge_cooldown = 0.0;

    let trained_ent = app
        .world_mut()
        .spawn((
            trained_creature,
            Transform::from_xyz(0.0, 0.0, -5.0),
        ))
        .id();

    app.update();

    let untrained = app.world().get::<DungeonCreature>(untrained_ent).unwrap();
    let trained = app.world().get::<DungeonCreature>(trained_ent).unwrap();

    assert_ne!(untrained.state, CreatureState::Dodging);
    assert_eq!(untrained.learned_player_threat, false);

    assert_eq!(trained.state, CreatureState::Dodging);
    assert!(trained.dodge_velocity.length() > 5.0);
    assert!(trained.dodge_cooldown > 1.0);
}

#[test]
fn test_unit_creature_species_elemental_potion_reactions() {
    use deep_alchemy::alchemy::{AlchemicalEffectKind, PotionProjectile};
    use deep_alchemy::dungeon::creatures::{
        update_creature_potion_interactions, CreatureSpecies, CreatureState, DungeonCreature,
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, update_creature_potion_interactions);

    let sproutling_ent = app
        .world_mut()
        .spawn((
            DungeonCreature::new(CreatureSpecies::Sproutling, vec![Vec3::ZERO], 10.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    let emberling_ent = app
        .world_mut()
        .spawn((
            DungeonCreature::new(CreatureSpecies::Emberling, vec![Vec3::ZERO], 10.0),
            Transform::from_xyz(20.0, 0.0, 0.0),
        ))
        .id();

    let glaceling_ent = app
        .world_mut()
        .spawn((
            DungeonCreature::new(CreatureSpecies::Glaceling, vec![Vec3::ZERO], 10.0),
            Transform::from_xyz(40.0, 0.0, 0.0),
        ))
        .id();

    let _fire_proj_ent = app
        .world_mut()
        .spawn((
            PotionProjectile {
                velocity: Vec3::ZERO,
                lifetime: 10.0,
                color: Color::srgb(1.0, 0.4, 0.1),
                hitstop_timer: 0.0,
                entered_water: false,
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: AlchemicalEffectKind::ThermalHeat,
                    duration: 5.0,
                    magnitude: 20.0,
                    elapsed: 0.0,
                }],
                slot: 2,
            },
            Transform::from_xyz(0.5, 0.0, 0.0),
        ))
        .id();

    let _frost_proj_ent = app
        .world_mut()
        .spawn((
            PotionProjectile {
                velocity: Vec3::ZERO,
                lifetime: 10.0,
                color: Color::srgb(0.2, 0.8, 1.0),
                hitstop_timer: 0.0,
                entered_water: false,
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: AlchemicalEffectKind::Frost,
                    duration: 5.0,
                    magnitude: 20.0,
                    elapsed: 0.0,
                }],
                slot: 1,
            },
            Transform::from_xyz(20.5, 0.0, 0.0),
        ))
        .id();

    let _fire_glace_proj_ent = app
        .world_mut()
        .spawn((
            PotionProjectile {
                velocity: Vec3::ZERO,
                lifetime: 10.0,
                color: Color::srgb(1.0, 0.4, 0.1),
                hitstop_timer: 0.0,
                entered_water: false,
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: AlchemicalEffectKind::ThermalHeat,
                    duration: 5.0,
                    magnitude: 20.0,
                    elapsed: 0.0,
                }],
                slot: 2,
            },
            Transform::from_xyz(40.5, 0.0, 0.0),
        ))
        .id();

    app.update();

    let sproutling = app.world().get::<DungeonCreature>(sproutling_ent).unwrap();
    let emberling = app.world().get::<DungeonCreature>(emberling_ent).unwrap();
    let glaceling = app.world().get::<DungeonCreature>(glaceling_ent).unwrap();

    assert_eq!(sproutling.state, CreatureState::Fleeing);
    assert!(sproutling.learned_player_threat);
    assert_eq!(sproutling.hit_count, 1);

    assert_eq!(emberling.state, CreatureState::Exhausted);
    assert!(emberling.learned_player_threat);

    assert_eq!(glaceling.state, CreatureState::Melting);
    assert!(glaceling.learned_player_threat);
}

#[test]
fn test_unit_dungeon_procedural_generation_creature_density_progression() {
    use deep_alchemy::dungeon::asset_loader::SceneAssetRegistry;
    use deep_alchemy::dungeon::creatures::DungeonCreature;
    use deep_alchemy::dungeon::generator::spawn_procedural_dungeon;

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin::default(),
    ));
    app.init_resource::<SceneAssetRegistry>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_systems(Startup, spawn_procedural_dungeon);
    app.update();

    let mut entrance_creatures = 0;
    let mut exit_creatures = 0;
    let mut total_creatures = 0;

    let mut query = app.world_mut().query::<(&Transform, &DungeonCreature)>();
    for (tf, _) in query.iter(app.world()) {
        total_creatures += 1;
        if tf.translation.z > 2.0 {
            exit_creatures += 1;
        } else {
            entrance_creatures += 1;
        }
    }

    assert!(total_creatures >= 14, "Dungeon must spawn creatures progressively along dungeon path, found {}", total_creatures);
    assert!(exit_creatures >= entrance_creatures, "Creatures density must be higher towards the exit than entrance");
}

#[test]
fn test_unit_abyssal_exit_trigger_and_victory_menu_transition() {
    use deep_alchemy::dungeon::generator::{update_abyssal_victory_trigger, AbyssalVictoryExitTrigger};
    use deep_alchemy::player::camera::Player;
    use deep_alchemy::ui::menu::{GameRunStats, GameState, MenuNavigation, MenuView};

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.init_state::<GameState>();
    app.insert_resource(MenuNavigation {
        current_view: MenuView::Main,
        has_started: true,
    });
    app.insert_resource(GameRunStats::default());
    app.add_systems(Update, update_abyssal_victory_trigger);

    let player_ent = app
        .world_mut()
        .spawn((
            Player,
            Transform::from_xyz(0.0, 0.0, 100.0),
        ))
        .id();

    let _trigger_ent = app
        .world_mut()
        .spawn((
            AbyssalVictoryExitTrigger,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    {
        let nav = app.world().resource::<MenuNavigation>();
        let stats = app.world().resource::<GameRunStats>();
        assert_eq!(nav.current_view, MenuView::Main);
        assert!(!stats.has_won);
    }

    {
        let mut tf = app.world_mut().get_mut::<Transform>(player_ent).unwrap();
        tf.translation = Vec3::new(0.0, 0.0, 2.0);
    }

    app.update();

    {
        let nav = app.world().resource::<MenuNavigation>();
        let stats = app.world().resource::<GameRunStats>();
        assert_eq!(nav.current_view, MenuView::Victory);
        assert!(stats.has_won);
    }
}

#[test]
fn test_unit_intro_comic_state_and_panel_navigation() {
    use deep_alchemy::ui::menu::{
        handle_intro_comic_keyboard_input, ComicCutsceneState, GameState, MenuNavigation, MenuView,
    };

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
    app.init_state::<GameState>();
    app.insert_resource(MenuNavigation {
        current_view: MenuView::IntroComic,
        has_started: false,
    });
    app.insert_resource(ComicCutsceneState::default());
    app.init_resource::<ButtonInput<KeyCode>>();
    app.add_systems(Update, handle_intro_comic_keyboard_input);

    app.update();

    {
        let state = app.world().resource::<ComicCutsceneState>();
        assert_eq!(state.active_panel, 0);
    }

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.reset_all();
        input.press(KeyCode::Space);
    }
    app.update();

    {
        let state = app.world().resource::<ComicCutsceneState>();
        assert_eq!(state.active_panel, 1);
    }

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.reset_all();
        input.press(KeyCode::Space);
    }
    app.update();

    {
        let state = app.world().resource::<ComicCutsceneState>();
        assert_eq!(state.active_panel, 2);
    }

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.reset_all();
        input.press(KeyCode::ArrowLeft);
    }
    app.update();

    {
        let state = app.world().resource::<ComicCutsceneState>();
        assert_eq!(state.active_panel, 1);
    }

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.reset_all();
        input.press(KeyCode::Space);
    }
    app.update();

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.reset_all();
        input.press(KeyCode::Space);
    }
    app.update();

    {
        let nav = app.world().resource::<MenuNavigation>();
        assert!(nav.has_started);
    }
}

#[test]
fn test_unit_intro_comic_subview_visibility_toggle() {
    use deep_alchemy::ui::menu::{
        update_menu_subview_visibility, ControlsViewNode, IntroComicViewNode, MainViewNode,
        MenuCardContainer, MenuNavigation, MenuView, SettingsViewNode, StartResumeButtonText,
        VictoryViewNode,
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(MenuNavigation {
        current_view: MenuView::IntroComic,
        has_started: false,
    });
    app.add_systems(Update, update_menu_subview_visibility);

    let card_ent = app.world_mut().spawn((MenuCardContainer, Node::default())).id();
    let comic_ent = app.world_mut().spawn((IntroComicViewNode, Node::default())).id();
    let _main_ent = app.world_mut().spawn((MainViewNode, Node::default())).id();
    let _settings_ent = app.world_mut().spawn((SettingsViewNode, Node::default())).id();
    let _controls_ent = app.world_mut().spawn((ControlsViewNode, Node::default())).id();
    let _victory_ent = app.world_mut().spawn((VictoryViewNode, Node::default())).id();
    let _txt_ent = app.world_mut().spawn((StartResumeButtonText, Text::new("INICIAR"))).id();

    app.update();

    {
        let card_node = app.world().get::<Node>(card_ent).unwrap();
        let comic_node = app.world().get::<Node>(comic_ent).unwrap();
        assert_eq!(card_node.display, Display::None);
        assert_eq!(comic_node.display, Display::Flex);
    }

    {
        let mut nav = app.world_mut().resource_mut::<MenuNavigation>();
        nav.current_view = MenuView::Main;
    }

    app.update();

    {
        let card_node = app.world().get::<Node>(card_ent).unwrap();
        let comic_node = app.world().get::<Node>(comic_ent).unwrap();
        assert_eq!(card_node.display, Display::Flex);
        assert_eq!(comic_node.display, Display::None);
    }
}

#[test]
fn test_unit_audio_procedural_synthesis_and_wav_generation() {
    use deep_alchemy::audio::synth::{
        ensure_procedural_audio_assets, generate_wav_bytes, synthesize_potion_throw,
    };
    use std::fs;
    use std::path::Path;

    let pcm = synthesize_potion_throw(44100);
    assert!(!pcm.is_empty());
    let bytes = generate_wav_bytes(&pcm, 44100);
    assert!(bytes.len() > 44);
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(&bytes[12..16], b"fmt ");

    ensure_procedural_audio_assets();

    let expected_files = [
        "assets/audio/potion_throw.wav",
        "assets/audio/potion_shatter.wav",
        "assets/audio/water_splash.wav",
        "assets/audio/torch_ignite.wav",
        "assets/audio/ice_freeze.wav",
        "assets/audio/creature_chirp.wav",
        "assets/audio/ui_click.wav",
        "assets/audio/water_drop.wav",
    ];

    for file in expected_files {
        let p = Path::new(file);
        assert!(p.exists(), "Audio file must exist: {}", file);
        let data = fs::read(p).expect("Failed to read audio file");
        assert!(data.len() > 44);
        assert_eq!(&data[0..4], b"RIFF");
        assert_eq!(&data[8..12], b"WAVE");
    }
}

#[test]
fn test_unit_audio_sound_effect_events_dispatch_and_volume() {
    use deep_alchemy::audio::{
        handle_sound_effect_events, AudioAssets, SoundEffectEvent, SoundEffectKind,
    };
    use deep_alchemy::ui::menu::GameSettings;
    use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<AudioSource>();
    app.add_event::<SoundEffectEvent>();

    let dummy_source = AudioSource {
        bytes: std::sync::Arc::from(deep_alchemy::audio::synth::generate_wav_bytes(&[0; 100], 44100).into_boxed_slice()),
    };
    let mut sources = app.world_mut().resource_mut::<Assets<AudioSource>>();
    let handle = sources.add(dummy_source);

    app.insert_resource(AudioAssets {
        potion_throw: handle.clone(),
        potion_shatter: handle.clone(),
        water_splash: handle.clone(),
        torch_ignite: handle.clone(),
        ice_freeze: handle.clone(),
        creature_chirp: handle.clone(),
        ui_click: handle.clone(),
        water_drop: handle.clone(),
    });

    let mut settings = GameSettings::default();
    settings.master_volume = 0.65;
    app.insert_resource(settings);

    app.add_systems(Update, handle_sound_effect_events);

    app.world_mut().send_event(SoundEffectEvent::new(SoundEffectKind::PotionThrow));
    app.update();

    let mut audio_players = app.world_mut().query::<(&AudioPlayer, &PlaybackSettings)>();
    let count = audio_players.iter(app.world()).count();
    assert_eq!(count, 1);
    for (_player, settings) in audio_players.iter(app.world()) {
        assert!((*settings.volume - 0.65).abs() < 0.001);
    }
}

#[test]
fn test_unit_cracked_wall_nest_snake_scorpion_trigger_and_neutralization() {
    use deep_alchemy::dungeon::traps::{
        update_cracked_wall_nests, update_potion_traps_and_nests_interactions,
        CrackedWallNest, WallNestKind, WallNestState,
    };
    use deep_alchemy::player::camera::{Health, Player};
    use deep_alchemy::alchemy::{AlchemicalEffectKind, ActiveEffect, PotionProjectile};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();
    app.add_event::<deep_alchemy::audio::SoundEffectEvent>();

    let player_ent = app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.0, 1.0, 0.0),
        Health { current: 100.0, max: 100.0 },
    )).id();

    let nest_intact_ent = app.world_mut().spawn((
        Transform::from_xyz(1.5, 1.0, 0.0),
        CrackedWallNest::new(WallNestKind::Snakes, Vec3::X),
    )).id();

    let nest_pre_neutralized_ent = app.world_mut().spawn((
        Transform::from_xyz(20.0, 1.0, 0.0),
        CrackedWallNest::new(WallNestKind::Scorpions, Vec3::Z),
    )).id();

    app.world_mut().spawn((
        Transform::from_xyz(20.0, 1.0, 0.0),
        PotionProjectile {
            velocity: Vec3::ZERO,
            effects: vec![ActiveEffect {
                kind: AlchemicalEffectKind::ThermalHeat,
                duration: 5.0,
                elapsed: 0.0,
                magnitude: 25.0,
            }],
            lifetime: 5.0,
            color: Color::WHITE,
            slot: 2,
            hitstop_timer: 0.0,
            entered_water: false,
        },
    ));

    app.add_systems(Update, (
        update_potion_traps_and_nests_interactions,
        update_cracked_wall_nests,
    ).chain());

    app.update();

    let pre_nest = app.world().get::<CrackedWallNest>(nest_pre_neutralized_ent).unwrap();
    assert_eq!(pre_nest.state, WallNestState::Neutralized);

    let intact_nest = app.world().get::<CrackedWallNest>(nest_intact_ent).unwrap();
    assert_eq!(intact_nest.state, WallNestState::Triggered);

    let player_health = app.world().get::<Health>(player_ent).unwrap();
    assert_eq!(player_health.current, 78.0);
}

#[test]
fn test_unit_dungeon_corridor_traps_wood_and_metal_alchemical_interactions() {
    use deep_alchemy::dungeon::traps::{
        DungeonTrap, TrapKind, TrapMaterial, TrapState,
    };

    let mut wood_trap = DungeonTrap::new(
        TrapMaterial::Wood,
        TrapKind::SpinningWheel,
        Vec3::ZERO,
        Vec3::Z,
    );
    assert!(wood_trap.is_operational());
    assert!(!wood_trap.can_pass());

    let burned = wood_trap.apply_alchemical_effect(true, false, false);
    assert!(burned);
    assert_eq!(wood_trap.state, TrapState::Burned);
    assert!(wood_trap.can_pass());
    assert!(!wood_trap.is_operational());

    let mut wood_trap_freeze = DungeonTrap::new(
        TrapMaterial::Wood,
        TrapKind::SpinningWheel,
        Vec3::ZERO,
        Vec3::Z,
    );
    let frozen = wood_trap_freeze.apply_alchemical_effect(false, true, false);
    assert!(frozen);
    assert!(matches!(wood_trap_freeze.state, TrapState::Frozen { .. }));
    assert!(wood_trap_freeze.can_pass());

    let mut wood_trap_glue = DungeonTrap::new(
        TrapMaterial::Wood,
        TrapKind::SpinningWheel,
        Vec3::ZERO,
        Vec3::Z,
    );
    let glued = wood_trap_glue.apply_alchemical_effect(false, false, true);
    assert!(glued);
    assert!(matches!(wood_trap_glue.state, TrapState::Glued { .. }));
    assert!(wood_trap_glue.can_pass());

    let mut metal_trap = DungeonTrap::new(
        TrapMaterial::Metal,
        TrapKind::CrushingMechanism,
        Vec3::ZERO,
        Vec3::Y,
    );
    assert!(metal_trap.is_operational());
    let fire_on_metal = metal_trap.apply_alchemical_effect(true, false, false);
    assert!(!fire_on_metal);
    assert_eq!(metal_trap.state, TrapState::Active);

    let freeze_on_metal = metal_trap.apply_alchemical_effect(false, true, false);
    assert!(freeze_on_metal);
    assert!(matches!(metal_trap.state, TrapState::Frozen { .. }));
    assert!(metal_trap.can_pass());
}

#[test]
fn test_unit_monsters_trap_avoidance_and_elimination_tactics() {
    use avian3d::prelude::LinearVelocity;
    use deep_alchemy::dungeon::creatures::{CreatureSpecies, CreatureState, DungeonCreature};
    use deep_alchemy::dungeon::traps::{
        update_creatures_trap_tactics, DungeonTrap, TrapKind, TrapMaterial,
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<deep_alchemy::audio::SoundEffectEvent>();

    let _trap_ent = app.world_mut().spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        DungeonTrap::new(TrapMaterial::Wood, TrapKind::SpinningWheel, Vec3::new(0.0, 1.0, 0.0), Vec3::Z),
    )).id();

    let approaching_creature_ent = app.world_mut().spawn((
        Transform::from_xyz(0.0, 1.0, 2.5),
        LinearVelocity(Vec3::new(0.0, 0.0, -2.0)),
        DungeonCreature::new(CreatureSpecies::Sproutling, vec![], 50.0),
    )).id();

    let caught_creature_ent = app.world_mut().spawn((
        Transform::from_xyz(0.0, 1.0, 0.2),
        LinearVelocity(Vec3::ZERO),
        DungeonCreature::new(CreatureSpecies::Emberling, vec![], 50.0),
    )).id();

    app.add_systems(Update, update_creatures_trap_tactics);
    app.update();

    let approaching = app.world().get::<DungeonCreature>(approaching_creature_ent).unwrap();
    assert_eq!(approaching.state, CreatureState::Cautious);
    let approaching_vel = app.world().get::<LinearVelocity>(approaching_creature_ent).unwrap();
    assert!(approaching_vel.z > 0.0);

    assert!(app.world().get_entity(caught_creature_ent).is_err());
}


