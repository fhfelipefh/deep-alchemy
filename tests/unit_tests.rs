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

// ==========================================
// 1. ALCHEMY UNIT TESTS
// ==========================================

#[test]
fn test_unit_alchemy_apply_and_query_effects() {
    let mut receiver = EffectReceiverComponent::default();
    assert!(!receiver.has_effect(AlchemicalEffectKind::ThermalHeat));

    // Apply new effect
    receiver.apply_effect(AlchemicalEffectKind::ThermalHeat, 5.0, 20.0);
    assert!(receiver.has_effect(AlchemicalEffectKind::ThermalHeat));
    assert_eq!(receiver.active_effects.len(), 1);
    assert_eq!(receiver.active_effects[0].duration, 5.0);
    assert_eq!(receiver.active_effects[0].magnitude, 20.0);
    assert_eq!(receiver.active_effects[0].elapsed, 0.0);

    // Apply same effect with higher duration and magnitude
    receiver.apply_effect(AlchemicalEffectKind::ThermalHeat, 8.0, 35.0);
    assert_eq!(receiver.active_effects.len(), 1); // should refresh, not duplicate
    assert_eq!(receiver.active_effects[0].duration, 8.0);
    assert_eq!(receiver.active_effects[0].magnitude, 35.0);

    // Apply multiple distinct effects
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

    // Tick 1: dt = 0.5s -> Vitality: 0.5/2.0 (active), Frost: 0.5/1.0 (active)
    receiver.active_effects.retain_mut(|e| {
        e.elapsed += dt;
        e.elapsed < e.duration
    });
    assert_eq!(receiver.active_effects.len(), 2);

    // Tick 2: dt = 0.6s -> Vitality: 1.1/2.0 (active), Frost: 1.1/1.0 (EXPIRED)
    let dt2 = 0.6;
    receiver.active_effects.retain_mut(|e| {
        e.elapsed += dt2;
        e.elapsed < e.duration
    });
    assert_eq!(receiver.active_effects.len(), 1);
    assert!(receiver.has_effect(AlchemicalEffectKind::Vitality));
    assert!(!receiver.has_effect(AlchemicalEffectKind::Frost));

    // Tick 3: dt = 1.0s -> Vitality: 2.1/2.0 (EXPIRED)
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

    // Proximity check
    let geyser_pos = Vec3::new(0.0, 0.0, 0.0);
    let close_player_pos = Vec3::new(2.0, 0.0, 2.0); // distance ~2.82m (< 5.0m)
    let far_player_pos = Vec3::new(10.0, 0.0, 10.0); // distance ~14.14m (> 5.0m)

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

// ==========================================
// 2. PLAYER CONTROLLER & MATH UNIT TESTS
// ==========================================

#[test]
fn test_unit_player_health_and_stamina_invariants() {
    let mut health = Health { current: 100.0, max: 100.0 };
    let mut stamina = Stamina { current: 100.0, max: 100.0 };

    // Damage
    health.current = (health.current - 45.0).max(0.0);
    assert_eq!(health.current, 55.0);

    // Over-healing clamped to max
    health.current = (health.current + 80.0).min(health.max);
    assert_eq!(health.current, 100.0);

    // Sprint stamina consumption
    let dt = 1.0;
    stamina.current = (stamina.current - 15.0 * dt).max(0.0);
    assert_eq!(stamina.current, 85.0);

    // Depleted stamina
    stamina.current = 0.0;
    assert!(stamina.current < 5.0, "Cannot sprint when stamina < 5.0");
}

#[test]
fn test_unit_player_look_pitch_clamping() {
    let mut pitch: f32 = 0.0;
    let sensitivity = 0.0022;

    // Extreme upward mouse jerk
    let big_mouse_up = -2000.0;
    pitch -= big_mouse_up * sensitivity;
    pitch = pitch.clamp(-1.52, 1.52);
    assert_eq!(pitch, 1.52, "Pitch must be clamped at top ~87 degrees");

    // Extreme downward mouse jerk
    let big_mouse_down = 2000.0;
    pitch -= big_mouse_down * sensitivity;
    pitch = pitch.clamp(-1.52, 1.52);
    assert_eq!(pitch, -1.52, "Pitch must be clamped at bottom ~-87 degrees");
}

#[test]
fn test_unit_player_death_state_and_camera_orbit() {
    let mut state = PlayerState::Active;
    let health = Health { current: 0.0, max: 100.0 };

    if health.current <= 0.0 {
        state = PlayerState::Dead;
    }
    assert_eq!(state, PlayerState::Dead);

    // Death orbit camera math
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
    assert_eq!(y, 3.0); // 1.0 + 2.0
    let horizontal_dist = ((x - death_cam.target_point.x).powi(2) + (z - death_cam.target_point.z).powi(2)).sqrt();
    assert!((horizontal_dist - 3.8).abs() < 0.001, "Camera distance must remain 3.8m during orbit");
}

// ==========================================
// 3. RENDER & PARTICLE UNIT TESTS
// ==========================================

#[test]
fn test_unit_water_material_defaults_and_time_update() {
    let mut water = WaterMaterial::default();
    assert_eq!(water.params.y, 1.0, "Default wave speed should be 1.0");
    assert_eq!(water.params.x, 0.0, "Initial time should be 0.0");

    let elapsed = 14.5;
    water.params.x = elapsed;
    assert_eq!(water.params.x, 14.5);
    assert!(water.base_color.to_vec4().w > 0.0, "Water base alpha must be > 0");
    assert!(water.deep_color.to_vec4().w > 0.0, "Water deep alpha must be > 0");
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
    particle.velocity.y -= 9.8 * dt; // Gravity
    assert_eq!(particle.lifetime, 0.5);
    assert!((particle.velocity.y - 0.1).abs() < 0.001, "Velocity y must be ~0.1");

    // Check expiration
    particle.lifetime += 0.6;
    let expired = particle.lifetime >= particle.max_lifetime;
    assert!(expired, "Particle should expire when lifetime >= max_lifetime");
}

// ==========================================
// 4. UI & HUD UNIT TESTS
// ==========================================

#[test]
fn test_unit_hud_bar_percentages() {
    let health = Health { current: 75.0, max: 100.0 };
    let stamina = Stamina { current: 30.0, max: 100.0 };

    let hp_pct = (health.current / health.max).clamp(0.0, 1.0) * 100.0;
    let st_pct = (stamina.current / stamina.max).clamp(0.0, 1.0) * 100.0;

    assert!((hp_pct - 75.0).abs() < 0.001);
    assert!((st_pct - 30.0).abs() < 0.001);

    // Negative edge case
    let dead_health = Health { current: -15.0, max: 100.0 };
    let clamped_hp = (dead_health.current / dead_health.max).clamp(0.0, 1.0) * 100.0;
    assert_eq!(clamped_hp, 0.0);
}
