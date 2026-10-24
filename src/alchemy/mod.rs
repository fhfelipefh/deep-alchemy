use crate::{
    player::camera::{CameraShake, LocomotionState, Player, PlayerCamera, PlayerState},
    ui::{
        hud::ActiveHotbarSlot,
        menu::{GameSettings, GameState},
    },
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::primitives::Aabb;
use bevy::render::render_asset::RenderAssetUsages;
use serde::{Deserialize, Serialize};

pub mod surface_fluid;
pub mod ice_simulation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Component)]
pub enum AlchemicalEffectKind {
    ThermalHeat,
    Vaporize,
    Luminescence,
    Vitality,
    Swiftness,
    Frost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEffect {
    pub kind: AlchemicalEffectKind,
    pub duration: f32,
    pub elapsed: f32,
    pub magnitude: f32,
}

#[derive(Component, Debug, Default)]
pub struct EffectReceiverComponent {
    pub active_effects: Vec<ActiveEffect>,
}

#[allow(dead_code)]
impl EffectReceiverComponent {
    pub fn apply_effect(&mut self, kind: AlchemicalEffectKind, duration: f32, magnitude: f32) {
        if let Some(existing) = self.active_effects.iter_mut().find(|e| e.kind == kind) {
            existing.duration = existing.duration.max(duration);
            existing.magnitude = existing.magnitude.max(magnitude);
            existing.elapsed = 0.0;
        } else {
            self.active_effects.push(ActiveEffect {
                kind,
                duration,
                elapsed: 0.0,
                magnitude,
            });
        }
    }

    pub fn has_effect(&self, kind: AlchemicalEffectKind) -> bool {
        self.active_effects.iter().any(|e| e.kind == kind)
    }
}

#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct AlchemicalIngredient {
    pub id: String,
    pub name: String,
    pub primary_effect: AlchemicalEffectKind,
    pub secondary_effect: AlchemicalEffectKind,
    pub harvested: bool,
}

#[derive(Component, Debug)]
pub struct PotionProjectile {
    pub velocity: Vec3,
    pub effects: Vec<ActiveEffect>,
    pub lifetime: f32,
    pub color: Color,
    pub slot: usize,
    pub hitstop_timer: f32,
    pub entered_water: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PotionSurfaceImpact {
    pub host: Entity,
    pub normal: Vec3,
}

#[derive(Component, Debug)]
pub struct TrajectoryAimIndicator;

#[derive(Component, Debug, Clone)]
pub struct PotionPuddle {
    pub color: Color,
    pub elapsed: f32,
    pub max_lifetime: f32,
    pub radius: f32,
    pub effects: Vec<ActiveEffect>,
    pub is_wall: bool,
}

#[derive(Component, Debug, Clone)]
pub struct PotionContactZone {
    pub puddle_a: Entity,
    pub puddle_b: Entity,
    pub color: Color,
    pub effects: Vec<ActiveEffect>,
    pub radius: f32,
}

#[derive(Component, Debug, Clone)]
pub struct IceFootprint {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Component, Debug)]
pub struct GlassShardParticle {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub ground_y: f32,
    pub settled: bool,
    pub color: Color,
    pub angular_velocity: Vec3,
    pub bounces_remaining: u8,
}

#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct PotionFlask {
    pub id: String,
    pub effects: Vec<ActiveEffect>,
    pub liquid_color: Color,
    pub volume: f32,
}

#[derive(Component, Debug)]
pub struct ThermalGeyser {
    pub eruption_timer: f32,
    pub eruption_interval: f32,
    pub is_erupting: bool,
    pub heat_radius: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Flammable {
    pub burn_duration: f32,
    pub elapsed: f32,
    pub is_burning: bool,
    pub fire_light: Option<Entity>,
    pub fire_emitter: Option<Entity>,
    pub initial_scale: Option<Vec3>,
    pub initial_translation: Option<Vec3>,
    pub center_offset: Option<Vec3>,
    pub temperature: f32,
    pub ignition_temperature: f32,
    pub cooling_rate: f32,
}

impl Flammable {
    pub fn new(burn_duration: f32) -> Self {
        Self {
            burn_duration,
            elapsed: 0.0,
            is_burning: false,
            fire_light: None,
            fire_emitter: None,
            initial_scale: None,
            initial_translation: None,
            center_offset: None,
            temperature: 0.0,
            ignition_temperature: 15.0,
            cooling_rate: 6.0,
        }
    }

    pub fn with_duration(burn_duration: f32) -> Self {
        Self::new(burn_duration)
    }

    pub fn with_center_offset(mut self, offset: Vec3) -> Self {
        if offset != Vec3::ZERO {
            self.center_offset = Some(offset);
        }
        self
    }

    pub fn with_ignition_temperature(mut self, threshold: f32) -> Self {
        self.ignition_temperature = threshold;
        self
    }

    pub fn with_cooling_rate(mut self, rate: f32) -> Self {
        self.cooling_rate = rate;
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    pub fn apply_heat(&mut self, amount: f32) -> bool {
        if self.is_burning {
            return true;
        }
        self.temperature = (self.temperature + amount).max(0.0);
        if self.temperature >= self.ignition_temperature {
            self.ignite();
            true
        } else {
            false
        }
    }

    pub fn cool_down(&mut self, dt: f32) {
        if !self.is_burning && self.temperature > 0.0 {
            self.temperature = (self.temperature - self.cooling_rate * dt).max(0.0);
        }
    }

    pub fn ignite(&mut self) {
        self.temperature = self.temperature.max(self.ignition_temperature);
        self.is_burning = true;
    }

    pub fn extinguish(&mut self, commands: &mut Commands) {
        self.is_burning = false;
        self.elapsed = 0.0;
        self.temperature = 0.0;
        if let Some(light) = self.fire_light.take() {
            commands.entity(light).despawn();
        }
        if let Some(emitter) = self.fire_emitter.take() {
            commands.entity(emitter).despawn();
        }
    }

    pub fn compute_world_center(
        &self,
        transform: &Transform,
        global_transform: Option<&GlobalTransform>,
        aabb: Option<&Aabb>,
    ) -> Vec3 {
        let offset = self
            .center_offset
            .or_else(|| aabb.map(|a| Vec3::from(a.center)))
            .unwrap_or(Vec3::ZERO);
        if let Some(gt) = global_transform {
            gt.transform_point(offset)
        } else {
            transform.transform_point(offset)
        }
    }
}

impl Default for Flammable {
    fn default() -> Self {
        Self::new(3.0)
    }
}

#[derive(Resource, Debug, Clone)]
pub struct PotionCooldowns {
    pub timers: [f32; 6],
    pub max_durations: [f32; 6],
}

impl Default for PotionCooldowns {
    fn default() -> Self {
        Self {
            timers: [0.0; 6],
            max_durations: [0.0; 6],
        }
    }
}

impl PotionCooldowns {
    pub fn is_ready(&self, slot: usize) -> bool {
        let idx = slot.clamp(1, 5);
        self.timers[idx] <= 0.0
    }

    pub fn trigger_cooldown(&mut self, slot: usize) -> f32 {
        let idx = slot.clamp(1, 5);
        let duration = 3.0 + crate::render::particles::fastrand::f32() * 2.0;
        self.timers[idx] = duration;
        self.max_durations[idx] = duration;
        duration
    }
}

pub fn update_potion_cooldowns(time: Res<Time>, mut cooldowns: ResMut<PotionCooldowns>) {
    let dt = time.delta_secs();
    for timer in &mut cooldowns.timers {
        if *timer > 0.0 {
            *timer = (*timer - dt).max(0.0);
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct AlchemyTransientEmitter {
    pub elapsed: f32,
    pub max_lifetime: f32,
}

pub fn update_alchemy_transient_emitters(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AlchemyTransientEmitter)>,
) {
    let dt = time.delta_secs();
    for (entity, mut emitter) in query.iter_mut() {
        emitter.elapsed += dt;
        if emitter.elapsed >= emitter.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}

pub struct AlchemyPlugin;

impl Plugin for AlchemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PotionCooldowns>()
            .init_resource::<surface_fluid::PotionFluidProfiles>()
            .init_resource::<ice_simulation::LakeFreezeSimulation>()
            .add_systems(
                Update,
                (
                    update_potion_cooldowns,
                    update_alchemy_transient_emitters,
                    cleanup_alchemy_memory_limits,
                    update_active_effects,
                    update_thermal_geysers,
                    update_potion_aiming
                        .run_if(resource_exists::<ButtonInput<MouseButton>>)
                        .run_if(resource_exists::<SpatialQueryPipeline>)
                        .run_if(resource_exists::<Assets<StandardMaterial>>),
                    spawn_potion_projectile
                        .run_if(resource_exists::<ButtonInput<MouseButton>>)
                        .run_if(resource_exists::<Assets<StandardMaterial>>),
                    update_potion_projectiles
                        .run_if(resource_exists::<SpatialQueryPipeline>)
                        .run_if(resource_exists::<Assets<StandardMaterial>>),
                    update_potion_puddles.run_if(resource_exists::<Assets<StandardMaterial>>),
                    update_glass_shards.run_if(resource_exists::<Assets<StandardMaterial>>),
                    surface_fluid::update_surface_fluid_patches.run_if(resource_exists::<SpatialQueryPipeline>),
                    surface_fluid::update_surface_fluid_droplet_physics,
                    surface_fluid::capture_surface_fluid_droplets.run_if(resource_exists::<SpatialQueryPipeline>),
                    surface_fluid::merge_surface_fluid_patches,
                    ice_simulation::update_lake_freeze_simulation
                        .run_if(resource_exists::<Assets<StandardMaterial>>),
                    update_ice_footprints,
                    update_flammable_combustion,
                ),
            );
    }
}

pub fn calculate_aim_trajectory_and_impact(
    origin: Vec3,
    initial_velocity: Vec3,
    gravity: f32,
    steps: usize,
    step_time: f32,
    spatial_query: Option<(&SpatialQuery, &SpatialQueryFilter)>,
) -> (Vec<Vec3>, Option<(Vec3, Vec3)>) {
    let mut points = Vec::with_capacity(steps + 1);
    let mut current_pos = origin;
    let mut current_vel = initial_velocity;
    points.push(current_pos);
    let mut hit_info = None;

    for _ in 0..steps {
        let next_vel = current_vel + Vec3::new(0.0, -gravity * step_time, 0.0);
        let displacement = (current_vel + next_vel) * 0.5 * step_time;
        let segment_len = displacement.length();

        if let Some((sq, filter)) = spatial_query {
            if segment_len > 1e-4 {
                let dir = displacement / segment_len;
                if let Some(hit) = sq.cast_ray(
                    current_pos,
                    Dir3::new(dir).unwrap_or(Dir3::NEG_Y),
                    segment_len,
                    true,
                    filter,
                ) {
                    if hit.distance > 0.001 {
                        let impact_pos = current_pos + dir * hit.distance;
                        points.push(impact_pos);
                        hit_info = Some((impact_pos, hit.normal));
                        break;
                    }
                }
            }
        }

        current_pos += displacement;
        current_vel = next_vel;
        points.push(current_pos);
        if current_pos.y < -5.0 {
            break;
        }
    }

    (points, hit_info)
}

pub fn calculate_aim_trajectory(
    origin: Vec3,
    initial_velocity: Vec3,
    gravity: f32,
    steps: usize,
    step_time: f32,
    spatial_query: Option<&SpatialQuery>,
) -> Vec<Vec3> {
    let filter = SpatialQueryFilter::default();
    let sq_tuple = spatial_query.map(|sq| (sq, &filter));
    let (pts, _) = calculate_aim_trajectory_and_impact(
        origin,
        initial_velocity,
        gravity,
        steps,
        step_time,
        sq_tuple,
    );
    pts
}

pub fn apply_throw_inaccuracy(
    base_direction: Vec3,
    is_sprinting: bool,
    time_seed: f32,
) -> Vec3 {
    if !is_sprinting {
        return base_direction.normalize_or_zero();
    }
    let pseudo_rand_x = (time_seed * 127.1 + 311.7).sin() * 43758.547;
    let pseudo_rand_y = (time_seed * 269.5 + 183.3).cos() * 24634.63;
    let fract_x = (pseudo_rand_x.fract() - 0.5) * 2.0;
    let fract_y = (pseudo_rand_y.fract() - 0.5) * 2.0;
    let right = base_direction.cross(Vec3::Y).normalize_or_zero();
    let up = right.cross(base_direction).normalize_or_zero();
    let inaccuracy_amount = 0.20;
    let perturbed = base_direction + right * (fract_x * inaccuracy_amount) + up * (fract_y * inaccuracy_amount);
    perturbed.normalize_or_zero()
}

pub fn selected_potion_effects(slot: usize) -> (Color, Vec<ActiveEffect>) {
    let (color, kind, magnitude) = match slot {
        2 => (
            Color::srgb(0.95, 0.12, 0.16),
            AlchemicalEffectKind::ThermalHeat,
            25.0,
        ),
        3 => (
            Color::srgb(0.95, 0.72, 0.12),
            AlchemicalEffectKind::Vitality,
            30.0,
        ),
        4 => (
            Color::srgb(0.12, 0.42, 0.95),
            AlchemicalEffectKind::Frost,
            20.0,
        ),
        5 => (
            Color::srgb(0.92, 0.95, 1.0),
            AlchemicalEffectKind::Luminescence,
            18.0,
        ),
        _ => (
            Color::srgb(0.12, 0.92, 0.42),
            AlchemicalEffectKind::Swiftness,
            20.0,
        ),
    };
    (
        color,
        vec![ActiveEffect {
            kind,
            duration: 5.0,
            elapsed: 0.0,
            magnitude,
        }],
    )
}

pub fn mix_potion_colors(c1: Color, c2: Color) -> Color {
    let s1 = c1.to_srgba();
    let s2 = c2.to_srgba();
    let r = (s1.red + s2.red) * 0.5;
    let g = (s1.green + s2.green) * 0.5;
    let b = (s1.blue + s2.blue) * 0.5;
    let max_in = (s1.red.max(s1.green).max(s1.blue) + s2.red.max(s2.green).max(s2.blue)) * 0.5;
    let max_mix = r.max(g).max(b).max(0.001);
    let boost = (max_in / max_mix).min(1.4);
    Color::srgb(
        (r * boost).clamp(0.0, 1.0),
        (g * boost).clamp(0.0, 1.0),
        (b * boost).clamp(0.0, 1.0),
    )
}

pub fn merge_potion_effects(base: &mut Vec<ActiveEffect>, incoming: &[ActiveEffect]) {
    for inc in incoming {
        if let Some(existing) = base.iter_mut().find(|e| e.kind == inc.kind) {
            existing.magnitude = (existing.magnitude + inc.magnitude) * 0.85;
            existing.duration = existing.duration.max(inc.duration);
            existing.elapsed = 0.0;
        } else {
            base.push(inc.clone());
        }
    }
    let has_heat = base.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat);
    let has_frost = base.iter().any(|e| e.kind == AlchemicalEffectKind::Frost);
    if has_heat && has_frost && !base.iter().any(|e| e.kind == AlchemicalEffectKind::Vaporize) {
        base.push(ActiveEffect {
            kind: AlchemicalEffectKind::Vaporize,
            duration: 8.0,
            elapsed: 0.0,
            magnitude: 40.0,
        });
    }
}

pub fn update_potion_aiming(
    game_state: Option<Res<State<GameState>>>,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    slot: Option<Res<ActiveHotbarSlot>>,
    cooldowns: Option<Res<PotionCooldowns>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    mut player_query: Query<(Entity, &PlayerState, Option<&mut LocomotionState>), With<Player>>,
    spatial_query: SpatialQuery,
    aim_indicators: Query<Entity, With<TrajectoryAimIndicator>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            for entity in aim_indicators.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    }

    let dt = time.delta_secs();
    let gamepad_aiming = gamepads.iter().any(|g| {
        g.pressed(GamepadButton::LeftTrigger2)
            || g.get(GamepadButton::LeftTrigger2).unwrap_or(0.0) > 0.15
    });
    let is_aiming = (mouse.pressed(MouseButton::Right) || gamepad_aiming)
        && player_query
            .iter()
            .all(|(_, s, _)| *s == PlayerState::Active);

    for (_, _, ref mut loco_opt) in player_query.iter_mut() {
        if let Some(ref mut loco) = loco_opt {
            if is_aiming {
                loco.throw_windup = (loco.throw_windup + 7.0 * dt).min(1.0);
            } else {
                loco.throw_windup = (loco.throw_windup - 10.0 * dt).max(0.0);
            }
        }
    }

    for entity in aim_indicators.iter() {
        commands.entity(entity).despawn();
    }

    if !is_aiming {
        return;
    }

    let Ok(camera_tf) = camera.get_single() else {
        return;
    };

    let windup = player_query
        .iter()
        .find_map(|(_, _, l)| l.as_ref().map(|loco| loco.throw_windup))
        .unwrap_or(0.0);

    let slot_index = slot.as_deref().map(|s| s.0).unwrap_or(1);
    let is_on_cooldown = cooldowns.as_ref().map(|c| !c.is_ready(slot_index)).unwrap_or(false);
    let (mut color, _) = selected_potion_effects(slot_index);
    if is_on_cooldown {
        color = Color::srgba(0.55, 0.55, 0.55, 0.35);
    }
    let forward = *camera_tf.forward();
    let origin = camera_tf.translation() + forward * 0.45 + Vec3::new(0.0, -0.12, 0.0);
    let impulse_dir = (forward + Vec3::new(0.0, 0.22, 0.0)).normalize();
    let initial_velocity = impulse_dir * 16.0;

    let excluded: Vec<Entity> = player_query.iter().map(|(e, _, _)| e).collect();
    let filter = SpatialQueryFilter::default().with_excluded_entities(excluded);

    let (points, hit_info) = calculate_aim_trajectory_and_impact(
        origin,
        initial_velocity,
        9.8,
        28,
        0.045,
        Some((&spatial_query, &filter)),
    );

    let bead_radius = 0.028 + 0.014 * windup;
    let bead_mesh = meshes.add(Sphere::new(bead_radius).mesh().uv(8, 6));
    let bead_mat = materials.add(StandardMaterial {
        base_color: color.with_alpha(0.85 + 0.12 * windup),
        emissive: color.to_linear() * (2.2 + 2.5 * windup),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let total_pts = points.len().max(2);
    let t = time.elapsed_secs();

    for (i, pt) in points.into_iter().skip(1).enumerate() {
        let bead_flow = ((i as f32 / total_pts as f32) - t * 0.8).rem_euclid(1.0);
        let pulse = (bead_flow * std::f32::consts::PI).sin().max(0.3);
        let scale = 0.72 + 0.38 * pulse;

        commands.spawn((
            TrajectoryAimIndicator,
            Mesh3d(bead_mesh.clone()),
            MeshMaterial3d(bead_mat.clone()),
            Transform::from_translation(pt).with_scale(Vec3::splat(scale)),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));
    }

    if let Some((raw_hit_pos, raw_hit_normal)) = hit_info {
        let (hit_pos, hit_normal) = resolve_surface_impact_normal(raw_hit_pos, raw_hit_normal);
        let decal_mesh = meshes.add(create_flat_circle_mesh(0.35 + 0.08 * windup, 24));
        let decal_mat = materials.add(StandardMaterial {
            base_color: color.with_alpha(0.88),
            emissive: color.to_linear() * (2.0 + 2.0 * windup),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let rotation = Quat::from_rotation_arc(Vec3::Y, hit_normal);
        commands.spawn((
            TrajectoryAimIndicator,
            Mesh3d(decal_mesh),
            MeshMaterial3d(decal_mat),
            Transform::from_translation(hit_pos + hit_normal * 0.015).with_rotation(rotation),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));

        let pulse_radius = 1.15 * (0.96 + 0.08 * (t * 1.8).sin());
        let splash_ring_mesh = meshes.add(crate::dungeon::modular_kit::create_water_ring_mesh(pulse_radius, 0.88, 32));
        let splash_ring_mat = materials.add(StandardMaterial {
            base_color: color.with_alpha(0.60),
            emissive: color.to_linear() * (1.8 + 1.5 * windup),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        commands.spawn((
            TrajectoryAimIndicator,
            Mesh3d(splash_ring_mesh),
            MeshMaterial3d(splash_ring_mat),
            Transform::from_translation(hit_pos + hit_normal * 0.02).with_rotation(rotation),
            bevy::pbr::NotShadowCaster,
            bevy::pbr::NotShadowReceiver,
        ));
    }
}

pub fn spawn_potion_projectile(
    game_state: Option<Res<State<GameState>>>,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    slot: Option<Res<ActiveHotbarSlot>>,
    mut cooldowns_opt: Option<ResMut<PotionCooldowns>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    mut player_query: Query<(&PlayerState, Option<&mut LocomotionState>), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }
    let gamepad_throw = gamepads.iter().any(|g| {
        g.just_pressed(GamepadButton::RightTrigger2)
    });
    if !mouse.just_pressed(MouseButton::Left) && !gamepad_throw {
        return;
    }
    let slot_index = slot.as_deref().map(|s| s.0).unwrap_or(1);
    if let Some(ref cooldowns) = cooldowns_opt {
        if !cooldowns.is_ready(slot_index) {
            return;
        }
    }
    let Ok((player_state, mut locomotion_opt)) = player_query.get_single_mut() else {
        return;
    };
    if *player_state != PlayerState::Active {
        return;
    }
    let Ok(camera) = camera.get_single() else {
        return;
    };

    if let Some(ref mut cooldowns) = cooldowns_opt {
        cooldowns.trigger_cooldown(slot_index);
    }

    if let Some(ref mut loco) = locomotion_opt {
        loco.throw_recoil = 1.0;
        loco.throw_windup = 0.0;
    }

    let is_sprinting = locomotion_opt.as_ref().map(|l| l.is_sprinting).unwrap_or(false);
    let base_direction = *camera.forward();
    let final_direction = apply_throw_inaccuracy(base_direction, is_sprinting, time.elapsed_secs());

    let (color, effects) = selected_potion_effects(slot_index);
    let mut spawn_pos = camera.translation() + base_direction * 0.45 + Vec3::new(0.0, -0.10, 0.0);
    if camera.translation().z > 200.0 {
        let origin = crate::dungeon::test_room::SECRET_TEST_ROOM_ORIGIN;
        spawn_pos.x = spawn_pos.x.clamp(origin.x - 39.8, origin.x + 39.8);
        spawn_pos.y = spawn_pos.y.clamp(origin.y + 0.15, origin.y + 13.85);
        spawn_pos.z = spawn_pos.z.clamp(origin.z - 39.8, origin.z + 39.8);
    }

    let impulse_dir = (final_direction + Vec3::new(0.0, 0.22, 0.0)).normalize();
    let speed = 16.0;
    let linear_vel = impulse_dir * speed;

    let flask_mesh = meshes.add(create_potion_flask_mesh(slot_index));
    let flask_mat = materials.add(StandardMaterial {
        base_color: color.with_alpha(0.85),
        emissive: color.to_linear() * 1.2,
        perceptual_roughness: 0.12,
        metallic: 0.05,
        reflectance: 0.85,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Name::new("Potion_Flask_Projectile"),
        Mesh3d(flask_mesh),
        MeshMaterial3d(flask_mat),
        Transform::from_translation(spawn_pos).with_scale(Vec3::splat(1.0)),
        PotionProjectile {
            velocity: linear_vel,
            effects,
            lifetime: 8.0,
            color,
            slot: slot_index,
            hitstop_timer: 0.0,
            entered_water: false,
        },
    ));
}

fn apply_splash_effects(
    receivers: &mut Query<(Entity, &Transform, &mut EffectReceiverComponent), (Without<PotionProjectile>, Without<PotionPuddle>, Without<Player>)>,
    player_query: &mut Query<(Entity, &Transform, &mut LocomotionState, &mut EffectReceiverComponent), (With<Player>, Without<PotionProjectile>)>,
    impact_point: Vec3,
    effects: &[ActiveEffect],
    excluded_entity: Entity,
    radius: f32,
) {
    for (r_entity, r_tf, mut receiver) in receivers.iter_mut() {
        if r_entity != excluded_entity && r_tf.translation.distance(impact_point) <= radius {
            for effect in effects {
                receiver.apply_effect(effect.kind, effect.duration, effect.magnitude);
            }
        }
    }

    for (_p_entity, p_tf, _p_loco, mut p_recv) in player_query.iter_mut() {
        if p_tf.translation.distance(impact_point) <= radius {
            for effect in effects {
                p_recv.apply_effect(effect.kind, effect.duration, effect.magnitude);
            }
        }
    }
}

fn ignite_nearby_flammables(
    flammables: &mut Query<
        (
            Entity,
            &Transform,
            Option<&GlobalTransform>,
            Option<&Aabb>,
            &mut Flammable,
        ),
        (
            Without<PotionProjectile>,
            Without<PotionPuddle>,
            Without<Player>,
        ),
    >,
    impact_point: Vec3,
    radius: f32,
    base_heat: f32,
) {
    for (_f_entity, f_tf, f_gt, f_aabb, mut flammable) in flammables.iter_mut() {
        let center = flammable.compute_world_center(f_tf, f_gt, f_aabb);
        let dist = center.distance(impact_point);
        if dist <= radius {
            let falloff = if dist <= 0.4 {
                1.0
            } else {
                (1.0 - ((dist - 0.4) / (radius - 0.4)) * 0.35).clamp(0.65, 1.0)
            };
            flammable.apply_heat(base_heat * falloff);
        }
    }
}

fn extinguish_nearby_flammables(
    flammables: &mut Query<
        (
            Entity,
            &Transform,
            Option<&GlobalTransform>,
            Option<&Aabb>,
            &mut Flammable,
        ),
        (
            Without<PotionProjectile>,
            Without<PotionPuddle>,
            Without<Player>,
        ),
    >,
    commands: &mut Commands,
    impact_point: Vec3,
    radius: f32,
) {
    for (_f_entity, f_tf, f_gt, f_aabb, mut flammable) in flammables.iter_mut() {
        let center = flammable.compute_world_center(f_tf, f_gt, f_aabb);
        if center.distance(impact_point) <= radius {
            if flammable.is_burning {
                flammable.extinguish(commands);
                commands.spawn((
                    Transform::from_translation(center + Vec3::new(0.0, 0.35, 0.0)),
                    crate::render::particles::ParticleEmitter {
                        spawn_timer: 0.0,
                        spawn_rate: 0.02,
                        particle_type: crate::render::particles::ParticleType::Steam,
                    },
                    AlchemyTransientEmitter {
                        elapsed: 0.0,
                        max_lifetime: 1.2,
                    },
                ));
            } else if flammable.temperature > 0.0 {
                flammable.temperature = 0.0;
            }
        }
    }
}

fn apply_radial_kinetic_impulse(
    dynamic_props: &mut Query<(Entity, &Transform, &RigidBody, Option<&mut LinearVelocity>, Option<&mut AngularVelocity>), (Without<PotionProjectile>, Without<Player>)>,
    player_query: &mut Query<(Entity, &Transform, &mut LocomotionState, &mut EffectReceiverComponent), (With<Player>, Without<PotionProjectile>)>,
    commands: &mut Commands,
    impact_point: Vec3,
    force: f32,
    radius: f32,
) {
    for (prop_ent, prop_tf, rb, mut lin_vel_opt, mut ang_vel_opt) in dynamic_props.iter_mut() {
        if *rb != RigidBody::Dynamic {
            continue;
        }
        let prop_pos = prop_tf.translation;
        let delta = prop_pos - impact_point;
        let dist = delta.length();
        if dist <= radius && dist > 1e-4 {
            let dir = (delta / dist + Vec3::new(0.0, 0.45, 0.0)).normalize();
            let falloff = force / (dist * dist + 0.5);
            let impulse = dir * falloff;
            if let Some(ref mut lin_vel) = lin_vel_opt {
                lin_vel.0 += impulse;
            } else {
                commands.entity(prop_ent).insert(LinearVelocity(impulse));
            }
            if let Some(ref mut ang) = ang_vel_opt {
                ang.0 += Vec3::new(
                    (crate::render::particles::fastrand::f32() - 0.5) * 8.0,
                    (crate::render::particles::fastrand::f32() - 0.5) * 8.0,
                    (crate::render::particles::fastrand::f32() - 0.5) * 8.0,
                );
            }
        }
    }

    for (_, p_tf, mut loco, _) in player_query.iter_mut() {
        let p_pos = p_tf.translation;
        let delta = p_pos - impact_point;
        let dist = delta.length();
        if dist <= radius * 1.35 && dist > 1e-4 {
            let dir_2d = Vec2::new(delta.x, delta.z).normalize_or_zero();
            let knockback = dir_2d * (force * 0.45 / (dist + 0.3));
            loco.horizontal_velocity += knockback;
        }
    }
}

fn apply_impact_trauma_shake(
    player_query: &Query<(Entity, &Transform, &mut LocomotionState, &mut EffectReceiverComponent), (With<Player>, Without<PotionProjectile>)>,
    shake: &mut Option<ResMut<CameraShake>>,
    impact_point: Vec3,
    max_radius: f32,
    multiplier: f32,
) {
    let Some(ref mut s) = shake else { return; };
    for (_p_ent, p_tf, _p_loco, _p_recv) in player_query.iter() {
        let dist = p_tf.translation.distance(impact_point);
        if dist < max_radius {
            let dir = (p_tf.translation - impact_point).normalize_or_zero();
            let intensity = (((max_radius - dist) / max_radius).clamp(0.20, 0.85)) * multiplier;
            s.add_trauma(intensity, dir);
        }
    }
}

fn spawn_glass_shards_burst(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    impact_point: Vec3,
    impact_normal: Vec3,
    color: Color,
    floor_y: f32,
    count: usize,
    is_wall: bool,
) {
    let tangent = if impact_normal.y.abs() < 0.9 {
        impact_normal.cross(Vec3::Y).normalize()
    } else {
        impact_normal.cross(Vec3::X).normalize()
    };
    let bitangent = impact_normal.cross(tangent).normalize();

    let c = color.to_srgba();
    let stain_base = Color::srgb(
        c.red * 0.78 + 0.18,
        c.green * 0.78 + 0.18,
        c.blue * 0.78 + 0.18,
    );
    let stain_emissive = color.to_linear() * 0.6;

    for i in 0..count {
        let angle_noise = (i as f32 * 4.37).sin() * 0.35;
        let phi = (i as f32) * (std::f32::consts::TAU / (count as f32)) + angle_noise;
        let radial_speed = 0.8 + 1.4 * ((i as f32 * 2.3).sin().abs());
        let normal_speed = 0.4 + 0.7 * ((i as f32 * 3.1).cos().abs());

        let shard_vel = if is_wall {
            let burst = impact_normal * (1.6 + 1.2 * ((i as f32 * 2.5).sin().abs()));
            let lateral = tangent * (phi.cos() * 1.4);
            let down_push = Vec3::NEG_Y * (0.6 + 0.8 * ((i as f32 * 3.1).cos().abs()));
            burst + lateral + down_push
        } else {
            tangent * (phi.cos() * radial_speed)
                + bitangent * (phi.sin() * radial_speed)
                + impact_normal * normal_speed
        };

        let shard_scale = 0.08 + 0.09 * ((i as f32 * 1.7).sin().abs());
        let shard_mesh = meshes.add(create_glass_shard_mesh(shard_scale, i));
        let shard_mat = materials.add(StandardMaterial {
            base_color: stain_base,
            emissive: stain_emissive,
            perceptual_roughness: 0.10,
            metallic: 0.18,
            reflectance: 0.98,
            cull_mode: None,
            double_sided: true,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 1000.0,
            ..default()
        });

        let rot_x = (i as f32 * 1.9).sin() * 0.35;
        let rot_y = phi + (i as f32 * 2.8).cos() * 1.2;
        let rot_z = (i as f32 * 3.4).sin() * 0.35;
        let shard_rot = Quat::from_euler(EulerRot::XYZ, rot_x, rot_y, rot_z);

        let ang_vel = Vec3::new(
            (i as f32 * 3.7).sin() * 8.5,
            (i as f32 * 2.1).cos() * 12.0,
            (i as f32 * 4.9).sin() * 9.5,
        );

        let bounces = if i % 6 == 1 || i % 6 == 3 { 1 } else { 0 };

        commands.spawn((
            GlassShardParticle {
                velocity: shard_vel,
                lifetime: 0.0,
                max_lifetime: 60.0 + 15.0 * ((i as f32 * 1.9).sin().abs()),
                ground_y: floor_y,
                settled: false,
                color,
                angular_velocity: ang_vel,
                bounces_remaining: bounces,
            },
            Mesh3d(shard_mesh),
            MeshMaterial3d(shard_mat),
            Transform::from_translation(impact_point + impact_normal * 0.055)
                .with_rotation(shard_rot),
        ));
    }
}

pub fn resolve_surface_impact_normal(impact_point: Vec3, raw_normal: Vec3) -> (Vec3, Vec3) {
    let x_abs = impact_point.x.abs();
    let is_ramp = x_abs > 2.5 && x_abs < 7.5 && impact_point.z > -16.0 && impact_point.z < 3.8;
    if is_ramp {
        let terr_y = crate::dungeon::modular_kit::get_grotto_terrain_height(impact_point.x, impact_point.z);
        if (impact_point.y - terr_y).abs() <= 0.65 {
            let terr_norm = crate::dungeon::modular_kit::get_grotto_terrain_normal(impact_point.x, impact_point.z);
            let adjusted_pos = Vec3::new(impact_point.x, impact_point.y.max(terr_y), impact_point.z);
            return (adjusted_pos, terr_norm);
        }
    }
    (impact_point, raw_normal.normalize_or_zero())
}

pub fn spawn_massive_steam_cloud(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    center: Vec3,
    intensity: f32,
) {
    let steam_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.88, 0.95, 1.0, 0.32),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let count = (36.0 * intensity).round() as usize;
    for i in 0..count {
        let angle = i as f32 * (std::f32::consts::TAU / count as f32);
        let radius = 0.2 + crate::render::particles::fastrand::f32() * 1.6;
        let speed = 0.6 + crate::render::particles::fastrand::f32() * 1.8;
        let vel = Vec3::new(
            angle.cos() * speed + (crate::render::particles::fastrand::f32() - 0.5) * 0.8,
            1.4 + crate::render::particles::fastrand::f32() * 1.8,
            angle.sin() * speed + (crate::render::particles::fastrand::f32() - 0.5) * 0.8,
        );
        let puff_radius = 0.12 + crate::render::particles::fastrand::f32() * 0.08;
        let sphere_mesh = meshes.add(Sphere::new(puff_radius).mesh().uv(8, 6));

        commands.spawn((
            Mesh3d(sphere_mesh),
            MeshMaterial3d(steam_mat.clone()),
            Transform::from_xyz(
                center.x + angle.cos() * radius * 0.5,
                center.y + 0.10 + crate::render::particles::fastrand::f32() * 0.25,
                center.z + angle.sin() * radius * 0.5,
            ),
            crate::render::particles::Particle {
                lifetime: 0.0,
                max_lifetime: 2.2 + crate::render::particles::fastrand::f32() * 1.4,
                velocity: vel,
                p_type: crate::render::particles::ParticleType::Steam,
            },
        ));
    }

    let emitter_count = 14;
    for i in 0..emitter_count {
        let (r, y_off) = if i == 0 {
            (0.0, 0.10)
        } else if i <= 5 {
            (0.65, 0.12)
        } else {
            (1.35, 0.15)
        };
        let angle = if i == 0 {
            0.0
        } else {
            i as f32 * (std::f32::consts::TAU / (emitter_count - 1) as f32)
        };
        commands.spawn((
            Transform::from_xyz(
                center.x + angle.cos() * r,
                center.y + y_off,
                center.z + angle.sin() * r,
            ),
            crate::render::particles::ParticleEmitter {
                spawn_timer: 0.0,
                spawn_rate: 0.022,
                particle_type: crate::render::particles::ParticleType::Steam,
            },
            AlchemyTransientEmitter {
                elapsed: 0.0,
                max_lifetime: 3.8,
            },
        ));
    }
}

fn spawn_alchemical_puddle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    impact_point: Vec3,
    impact_normal: Vec3,
    color: Color,
    effects: Vec<ActiveEffect>,
    _is_wall: bool,
    is_rtx: bool,
) {
    let (impact_point, impact_normal) = resolve_surface_impact_normal(impact_point, impact_normal);
    let is_frost = effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost);
    let in_water = crate::dungeon::modular_kit::is_in_lake_water(impact_point);

    if is_frost {
        let (mesh_handle, radius, thickness, y_pos, collider_r) = if in_water {
            let r = 1.65;
            let th = 0.22;
            let mesh = meshes.add(create_irregular_ice_block_mesh(r, th, 24));
            (mesh, r, th, crate::alchemy::ice_simulation::ICE_SURFACE_Y + th * 0.5, 1.60)
        } else {
            let r = 1.15;
            let th = 0.08;
            let mesh = meshes.add(create_irregular_ice_block_mesh(r, th, 24));
            (mesh, r, th, impact_point.y + impact_normal.y * 0.04 + th * 0.5, 1.10)
        };

        let ice_mat = materials.add(StandardMaterial {
            base_color: Color::srgba(0.68, 0.88, 0.98, 0.88),
            emissive: LinearRgba::new(0.08, 0.22, 0.40, 0.35),
            perceptual_roughness: 0.15,
            metallic: 0.05,
            reflectance: 0.88,
            cull_mode: None,
            double_sided: true,
            alpha_mode: AlphaMode::Blend,
            depth_bias: 500.0,
            ..default()
        });

        let spawn_pos = Vec3::new(impact_point.x, y_pos, impact_point.z);

        commands.spawn((
            Name::new("Solid_Ice_Platform"),
            crate::alchemy::ice_simulation::IcePlatform,
            PotionPuddle {
                color,
                elapsed: 0.0,
                max_lifetime: 20.0,
                radius,
                effects,
                is_wall: false,
            },
            Mesh3d(mesh_handle),
            MeshMaterial3d(ice_mat),
            Transform::from_translation(spawn_pos),
            RigidBody::Static,
            Collider::cylinder(collider_r, thickness),
            PointLight {
                color: Color::srgb(0.60, 0.85, 1.0),
                intensity: if is_rtx { 2400.0 } else { 0.0 },
                range: 4.2,
                shadows_enabled: false,
                ..default()
            },
        ));
        return;
    }

    let rot = Quat::from_rotation_arc(Vec3::Y, impact_normal);
    let is_fire_puddle = effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat);
    let dx = impact_point.x;
    let dz = impact_point.z - (-8.5);
    let dist_lake = (dx * dx + dz * dz).sqrt();
    let is_near_water = (dist_lake <= 6.8 && impact_point.y <= 0.65) || in_water;
    if is_fire_puddle && is_near_water {
        spawn_massive_steam_cloud(commands, meshes, materials, impact_point, 1.0);
    }
    let mesh = meshes.add(create_conforming_puddle_mesh(1.15, 32, impact_point));
    let puddle_pos = impact_point + impact_normal * 0.045;

    let puddle_mat = materials.add(StandardMaterial {
        base_color: color.with_alpha(0.88),
        emissive: color.to_linear() * 0.32,
        perceptual_roughness: 0.85,
        metallic: 0.0,
        reflectance: 0.12,
        cull_mode: None,
        double_sided: true,
        alpha_mode: AlphaMode::Blend,
        depth_bias: 600.0,
        ..default()
    });

    commands.spawn((
        PotionPuddle {
            color,
            elapsed: 0.0,
            max_lifetime: 75.0,
            radius: 1.15,
            effects,
            is_wall: false,
        },
        Mesh3d(mesh),
        MeshMaterial3d(puddle_mat),
        Transform::from_translation(puddle_pos)
            .with_rotation(rot)
            .with_scale(Vec3::new(0.15, 1.0, 0.15)),
        PointLight {
            color,
            intensity: if is_rtx { 2400.0 } else { 0.0 },
            range: 3.8,
            shadows_enabled: false,
            ..default()
        },
    ));
}

fn process_underwater_freeze_seed(
    simulation: &mut ice_simulation::LakeFreezeSimulation,
    player_query: &mut Query<(Entity, &Transform, &mut LocomotionState, &mut EffectReceiverComponent), (With<Player>, Without<PotionProjectile>)>,
    freeze_center: Vec2,
    intensity: f32,
) {
    simulation.add_freeze_seed(freeze_center, intensity.clamp(0.4, 1.5));
    for (_entity, transform, mut locomotion, mut receiver) in player_query.iter_mut() {
        let position = transform.translation;
        let distance = Vec2::new(position.x, position.z).distance(freeze_center);
        if crate::dungeon::modular_kit::is_in_lake_water(position - Vec3::new(0.0, 1.2, 0.0)) && distance < 1.45 {
            locomotion.is_frozen = true;
            locomotion.freeze_timer = 2.0;
            receiver.apply_effect(AlchemicalEffectKind::Frost, 4.0, 18.0);
        }
    }
}

pub fn update_potion_projectiles(
    time: Res<Time>,
    settings: Option<Res<GameSettings>>,
    fluid_profiles: Option<Res<surface_fluid::PotionFluidProfiles>>,
    spatial_query: SpatialQuery,
    mut projectiles: Query<(Entity, &mut Transform, &mut PotionProjectile, Option<&PotionSurfaceImpact>), Without<Player>>,
    mut player_query: Query<(Entity, &Transform, &mut LocomotionState, &mut EffectReceiverComponent), (With<Player>, Without<PotionProjectile>)>,
    mut receivers: Query<(Entity, &Transform, &mut EffectReceiverComponent), (Without<PotionProjectile>, Without<PotionPuddle>, Without<Player>)>,
    mut flammables: Query<
        (
            Entity,
            &Transform,
            Option<&GlobalTransform>,
            Option<&Aabb>,
            &mut Flammable,
        ),
        (
            Without<PotionProjectile>,
            Without<PotionPuddle>,
            Without<Player>,
        ),
    >,
    mut dynamic_props: Query<(Entity, &Transform, &RigidBody, Option<&mut LinearVelocity>, Option<&mut AngularVelocity>), (Without<PotionProjectile>, Without<Player>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fluid_materials: Option<ResMut<Assets<crate::render::surface_fluid_material::SurfaceFluidMaterial>>>,
    mut freeze_simulation: Option<ResMut<ice_simulation::LakeFreezeSimulation>>,
    mut commands: Commands,
    mut shake: Option<ResMut<CameraShake>>,
) {
    let dt = time.delta_secs();
    let gravity = Vec3::new(0.0, -9.8, 0.0);
    let mut excluded: Vec<Entity> = player_query.iter().map(|(e, _, _, _)| e).collect();

    for (entity, _, _, _) in projectiles.iter() {
        excluded.push(entity);
    }
    let filter = SpatialQueryFilter::default().with_excluded_entities(excluded.clone());

    for (entity, mut transform, mut projectile, impact_surface) in &mut projectiles {
        if projectile.hitstop_timer > 0.0 {
            projectile.hitstop_timer -= dt;
            if projectile.hitstop_timer > 0.0 {
                continue;
            }
            let raw_point = transform.translation;
            let raw_normal = impact_surface
                .map(|impact| impact.normal)
                .unwrap_or_else(|| (transform.rotation * Vec3::Y).normalize());
            let (impact_point, impact_normal) = resolve_surface_impact_normal(raw_point, raw_normal);
            let impact_host = impact_surface.map(|impact| impact.host);

            apply_splash_effects(&mut receivers, &mut player_query, impact_point, &projectile.effects, entity, 2.8);
            apply_radial_kinetic_impulse(&mut dynamic_props, &mut player_query, &mut commands, impact_point, 14.0, 3.8);

            let is_frost = projectile.effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost);
            if is_frost {
                extinguish_nearby_flammables(&mut flammables, &mut commands, impact_point, 2.8);
                if crate::dungeon::modular_kit::is_in_lake_water(impact_point) {
                    if let Some(simulation) = freeze_simulation.as_deref_mut() {
                        let intensity = projectile
                            .effects
                            .iter()
                            .find(|effect| effect.kind == AlchemicalEffectKind::Frost)
                            .map(|effect| effect.magnitude / 10.0)
                            .unwrap_or(1.5);
                        process_underwater_freeze_seed(
                            simulation,
                            &mut player_query,
                            Vec2::new(impact_point.x, impact_point.z),
                            intensity,
                        );
                    }
                }
            }

            let is_fire_impact = projectile.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat) || projectile.slot == 2;
            if is_fire_impact {
                let heat_magnitude = projectile
                    .effects
                    .iter()
                    .find(|e| e.kind == AlchemicalEffectKind::ThermalHeat)
                    .map(|e| e.magnitude)
                    .unwrap_or(25.0);
                ignite_nearby_flammables(&mut flammables, impact_point, 2.8, heat_magnitude);

                let dx = impact_point.x;
                let dz = impact_point.z - (-8.5);
                let dist_lake = (dx * dx + dz * dz).sqrt();
                let is_near_water = (dist_lake <= 6.8 && impact_point.y <= 0.65)
                    || crate::dungeon::modular_kit::is_in_lake_water(impact_point);
                if is_near_water {
                    spawn_massive_steam_cloud(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        impact_point,
                        1.15,
                    );
                }
            }

            let is_wall = impact_normal.y.abs() < 0.65;
            let is_rtx = settings.as_ref().map(|s| s.rtx_enabled).unwrap_or(false);
            let profile = fluid_profiles
                .as_ref()
                .map(|profiles| profiles.for_slot(projectile.slot))
                .unwrap_or_else(|| surface_fluid::PotionFluidProfiles::default().for_slot(projectile.slot));

            if let Some(ref mut surface_materials) = fluid_materials {
                surface_fluid::spawn_surface_fluid_patch(
                    &mut commands,
                    &mut meshes,
                    surface_materials,
                    impact_point,
                    impact_normal,
                    impact_host,
                    profile,
                    projectile.color,
                    projectile.effects.clone(),
                    profile.initial_volume,
                );
            }

            let floor_y = if is_wall {
                let down_ray = spatial_query.cast_ray(
                    impact_point + impact_normal * 0.15,
                    Dir3::NEG_Y,
                    25.0,
                    true,
                    &filter,
                );
                let fy = down_ray.map(|h| (impact_point.y - h.distance) + 0.048).unwrap_or(0.048);
                fy
            } else {
                impact_point.y + 0.052
            };

            if !is_wall {
                spawn_alchemical_puddle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    impact_point,
                    impact_normal,
                    projectile.color,
                    projectile.effects.clone(),
                    false,
                    is_rtx,
                );
            }

            commands.entity(entity).despawn();

            spawn_glass_shards_burst(
                &mut commands,
                &mut meshes,
                &mut materials,
                impact_point,
                impact_normal,
                projectile.color,
                floor_y,
                16,
                is_wall,
            );

            continue;
        }

        let current_pos = transform.translation;
        let is_frost = projectile.effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost);
        let in_water = crate::dungeon::modular_kit::is_in_lake_water(current_pos);

        if in_water {
            if current_pos.y <= 0.22 && !projectile.entered_water {
                projectile.entered_water = true;
                crate::dungeon::modular_kit::spawn_water_ripple(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    Vec3::new(current_pos.x, 0.20, current_pos.z),
                    0.25,
                    2.20,
                    1.6,
                    0.85,
                );
                for _ in 0..8 {
                    let angle = crate::render::particles::fastrand::f32() * std::f32::consts::TAU;
                    let speed = 0.8 + crate::render::particles::fastrand::f32() * 1.4;
                    let vel = Vec3::new(
                        angle.cos() * speed,
                        1.2 + crate::render::particles::fastrand::f32() * 1.5,
                        angle.sin() * speed,
                    );
                    commands.spawn((
                        Mesh3d(meshes.add(surface_fluid::create_teardrop_droplet_mesh(0.045, 8, 6))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::LinearRgba(LinearRgba::new(0.35, 0.70, 0.95, 0.45)),
                            alpha_mode: AlphaMode::Add,
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_translation(Vec3::new(current_pos.x, 0.22, current_pos.z)),
                        crate::render::particles::Particle {
                            lifetime: 0.0,
                            max_lifetime: 0.8,
                            velocity: vel,
                            p_type: crate::render::particles::ParticleType::WaterSplash,
                        },
                    ));
                }

                let is_fire_water = projectile.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat) || projectile.slot == 2;
                if is_fire_water {
                    spawn_massive_steam_cloud(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        Vec3::new(current_pos.x, 0.06, current_pos.z),
                        1.25,
                    );
                }
            }

            projectile.velocity.x *= (1.0 - 5.5 * dt).max(0.0);
            projectile.velocity.z *= (1.0 - 5.5 * dt).max(0.0);
            projectile.velocity.y = (projectile.velocity.y - 3.8 * dt).clamp(-1.85, 0.20);

            let floor_y = crate::dungeon::modular_kit::get_grotto_terrain_height(current_pos.x, current_pos.z);
            if current_pos.y <= floor_y + 0.08 {
                let impact_point = Vec3::new(current_pos.x, floor_y + 0.04, current_pos.z);

                apply_splash_effects(&mut receivers, &mut player_query, impact_point, &projectile.effects, entity, 2.8);
                apply_impact_trauma_shake(&player_query, &mut shake, impact_point, 6.0, 0.8);
                apply_radial_kinetic_impulse(&mut dynamic_props, &mut player_query, &mut commands, impact_point, 10.0, 3.2);

                spawn_glass_shards_burst(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    impact_point,
                    Vec3::Y,
                    projectile.color,
                    floor_y + 0.02,
                    8,
                    false,
                );

                if is_frost {
                    extinguish_nearby_flammables(&mut flammables, &mut commands, impact_point, 2.8);
                    if let Some(simulation) = freeze_simulation.as_deref_mut() {
                        let intensity = projectile
                            .effects
                            .iter()
                            .find(|effect| effect.kind == AlchemicalEffectKind::Frost)
                            .map(|effect| effect.magnitude / 10.0)
                            .unwrap_or(1.0);
                        process_underwater_freeze_seed(
                            simulation,
                            &mut player_query,
                            Vec2::new(current_pos.x, current_pos.z),
                            intensity,
                        );
                    }
                    commands.entity(entity).despawn();
                    continue;
                }

                let is_fire_impact = projectile.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat) || projectile.slot == 2;
                if is_fire_impact {
                    let heat_magnitude = projectile
                        .effects
                        .iter()
                        .find(|e| e.kind == AlchemicalEffectKind::ThermalHeat)
                        .map(|e| e.magnitude)
                        .unwrap_or(25.0);
                    ignite_nearby_flammables(&mut flammables, impact_point, 2.8, heat_magnitude);

                    let dx = impact_point.x;
                    let dz = impact_point.z - (-8.5);
                    let dist_lake = (dx * dx + dz * dz).sqrt();
                    let is_near_water = (dist_lake <= 6.8 && impact_point.y <= 0.65)
                        || crate::dungeon::modular_kit::is_in_lake_water(impact_point);
                    if is_near_water {
                        spawn_massive_steam_cloud(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            impact_point,
                            1.15,
                        );
                    }
                }

                let is_rtx = settings.as_ref().map(|s| s.rtx_enabled).unwrap_or(false);
                spawn_alchemical_puddle(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    impact_point,
                    Vec3::Y,
                    projectile.color,
                    projectile.effects.clone(),
                    false,
                    is_rtx,
                );

                commands.spawn((
                    Transform::from_xyz(impact_point.x, impact_point.y + 0.1, impact_point.z),
                    crate::render::particles::ParticleEmitter {
                        spawn_timer: 0.0,
                        spawn_rate: 0.08,
                        particle_type: crate::render::particles::ParticleType::WaterSplash,
                    },
                    AlchemyTransientEmitter {
                        elapsed: 0.0,
                        max_lifetime: 1.5,
                    },
                ));

                commands.entity(entity).despawn();
                continue;
            }
        } else {
            projectile.velocity += gravity * dt;
        }

        let step = projectile.velocity * dt;
        let distance = step.length();
        let direction = projectile.velocity.normalize_or_zero();

        let hit = if distance > 1e-4 {
            spatial_query.cast_ray(
                transform.translation,
                Dir3::new(direction).unwrap_or(Dir3::NEG_Y),
                distance,
                true,
                &filter,
            )
        } else {
            None
        };

        if let Some(hit) = hit {
            if hit.distance > 0.001 && hit.entity != entity && !excluded.contains(&hit.entity) {
                let raw_impact_point = transform.translation + direction * hit.distance;
                let (impact_point, impact_normal) = resolve_surface_impact_normal(raw_impact_point, hit.normal);

                projectile.hitstop_timer = 0.045;
                transform.translation = impact_point;
                transform.rotation = Quat::from_rotation_arc(Vec3::Y, impact_normal);
                projectile.velocity = Vec3::ZERO;
                commands.entity(entity).insert(PotionSurfaceImpact {
                    host: hit.entity,
                    normal: impact_normal,
                });

                let flash_mesh = meshes.add(Sphere::new(0.20).mesh().uv(8, 6));
                let flash_mat = materials.add(StandardMaterial {
                    base_color: projectile.color.with_alpha(0.95),
                    emissive: projectile.color.to_linear() * 6.5,
                    unlit: true,
                    alpha_mode: AlphaMode::Add,
                    ..default()
                });
                commands.spawn((
                    Mesh3d(flash_mesh),
                    MeshMaterial3d(flash_mat),
                    Transform::from_translation(impact_point + impact_normal * 0.04),
                    AlchemyTransientEmitter {
                        elapsed: 0.0,
                        max_lifetime: 0.045,
                    },
                ));

                apply_impact_trauma_shake(&player_query, &mut shake, impact_point, 5.0, 1.0);

                continue;
            }
        }

        transform.translation += step;

        let in_grotto = transform.translation.x.abs() > 2.5 && transform.translation.x.abs() < 7.5 && transform.translation.z > -16.0 && transform.translation.z < 3.8;
        if in_grotto {
            let terr_y = crate::dungeon::modular_kit::get_grotto_terrain_height(transform.translation.x, transform.translation.z);
            if transform.translation.y <= terr_y + 0.05 {
                let terr_norm = crate::dungeon::modular_kit::get_grotto_terrain_normal(transform.translation.x, transform.translation.z);
                let impact_point = Vec3::new(transform.translation.x, terr_y + 0.02, transform.translation.z);
                let impact_normal = terr_norm;
                projectile.hitstop_timer = 0.045;
                transform.translation = impact_point;
                transform.rotation = Quat::from_rotation_arc(Vec3::Y, impact_normal);
                projectile.velocity = Vec3::ZERO;
                commands.entity(entity).insert(PotionSurfaceImpact {
                    host: entity,
                    normal: impact_normal,
                });
                let flash_mesh = meshes.add(Sphere::new(0.20).mesh().uv(8, 6));
                let flash_mat = materials.add(StandardMaterial {
                    base_color: projectile.color.with_alpha(0.95),
                    emissive: projectile.color.to_linear() * 6.5,
                    unlit: true,
                    alpha_mode: AlphaMode::Add,
                    ..default()
                });
                commands.spawn((
                    Mesh3d(flash_mesh),
                    MeshMaterial3d(flash_mat),
                    Transform::from_translation(impact_point + impact_normal * 0.04),
                    AlchemyTransientEmitter {
                        elapsed: 0.0,
                        max_lifetime: 0.045,
                    },
                ));
                apply_impact_trauma_shake(&player_query, &mut shake, impact_point, 5.0, 1.0);
                continue;
            }
        }

        if transform.translation.z > 200.0 {
            let origin = crate::dungeon::test_room::SECRET_TEST_ROOM_ORIGIN;
            let min_x = origin.x - 40.0;
            let max_x = origin.x + 40.0;
            let min_y = origin.y;
            let max_y = origin.y + 14.0;
            let min_z = origin.z - 40.0;
            let max_z = origin.z + 40.0;

            let mut wall_impact: Option<(Vec3, Vec3)> = None;
            if transform.translation.x <= min_x {
                wall_impact = Some((Vec3::new(min_x + 0.02, transform.translation.y.clamp(0.05, 13.95), transform.translation.z.clamp(min_z + 0.05, max_z - 0.05)), Vec3::X));
            } else if transform.translation.x >= max_x {
                wall_impact = Some((Vec3::new(max_x - 0.02, transform.translation.y.clamp(0.05, 13.95), transform.translation.z.clamp(min_z + 0.05, max_z - 0.05)), Vec3::NEG_X));
            } else if transform.translation.z <= min_z {
                wall_impact = Some((Vec3::new(transform.translation.x.clamp(min_x + 0.05, max_x - 0.05), transform.translation.y.clamp(0.05, 13.95), min_z + 0.02), Vec3::Z));
            } else if transform.translation.z >= max_z {
                wall_impact = Some((Vec3::new(transform.translation.x.clamp(min_x + 0.05, max_x - 0.05), transform.translation.y.clamp(0.05, 13.95), max_z - 0.02), Vec3::NEG_Z));
            } else if transform.translation.y <= min_y {
                wall_impact = Some((Vec3::new(transform.translation.x.clamp(min_x + 0.05, max_x - 0.05), min_y + 0.02, transform.translation.z.clamp(min_z + 0.05, max_z - 0.05)), Vec3::Y));
            } else if transform.translation.y >= max_y {
                wall_impact = Some((Vec3::new(transform.translation.x.clamp(min_x + 0.05, max_x - 0.05), max_y - 0.02, transform.translation.z.clamp(min_z + 0.05, max_z - 0.05)), Vec3::NEG_Y));
            }

            if let Some((impact_point, impact_normal)) = wall_impact {
                projectile.hitstop_timer = 0.045;
                transform.translation = impact_point;
                transform.rotation = Quat::from_rotation_arc(Vec3::Y, impact_normal);
                projectile.velocity = Vec3::ZERO;
                commands.entity(entity).insert(PotionSurfaceImpact {
                    host: entity,
                    normal: impact_normal,
                });
                let flash_mesh = meshes.add(Sphere::new(0.20).mesh().uv(8, 6));
                let flash_mat = materials.add(StandardMaterial {
                    base_color: projectile.color.with_alpha(0.95),
                    emissive: projectile.color.to_linear() * 6.5,
                    unlit: true,
                    alpha_mode: AlphaMode::Add,
                    ..default()
                });
                commands.spawn((
                    Mesh3d(flash_mesh),
                    MeshMaterial3d(flash_mat),
                    Transform::from_translation(impact_point + impact_normal * 0.04),
                    AlchemyTransientEmitter {
                        elapsed: 0.0,
                        max_lifetime: 0.045,
                    },
                ));
                apply_impact_trauma_shake(&player_query, &mut shake, impact_point, 5.0, 1.0);
                continue;
            }
        }
        transform.rotate_local_x(7.5 * dt);
        transform.rotate_local_y(4.2 * dt);
        projectile.lifetime -= dt;
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_potion_puddles(
    time: Res<Time>,
    settings: Option<Res<GameSettings>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut puddles: Query<(
        Entity,
        &mut PotionPuddle,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
        Option<&mut PointLight>,
    )>,
    mut contact_zones: Query<(
        Entity,
        &PotionContactZone,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    ), Without<PotionPuddle>>,
    mut receivers: Query<(&Transform, &mut EffectReceiverComponent), (Without<PotionPuddle>, Without<PotionContactZone>)>,
    mut flammables: Query<
        (
            &Transform,
            Option<&GlobalTransform>,
            Option<&Aabb>,
            &mut Flammable,
        ),
        (Without<PotionPuddle>, Without<PotionContactZone>),
    >,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let is_rtx = settings.as_ref().map(|s| s.rtx_enabled).unwrap_or(false);

    let mut active_puddles: Vec<(Entity, Vec3, f32, Color, Vec<ActiveEffect>, f32)> = Vec::new();

    for (entity, mut puddle, mut transform, mat_handle, mut point_light) in puddles.iter_mut() {
        puddle.elapsed += dt;
        if puddle.elapsed >= puddle.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        let is_frost_puddle = puddle.effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost);
        let fade_start = if is_frost_puddle {
            (puddle.max_lifetime - 5.0).max(1.0)
        } else {
            50.0
        };
        let fade_duration = (puddle.max_lifetime - fade_start).max(0.1);
        let (alpha, fade_factor) = if puddle.elapsed <= fade_start {
            (0.88, 1.0)
        } else {
            let progress = ((puddle.elapsed - fade_start) / fade_duration).clamp(0.0, 1.0);
            (0.88 * (1.0 - progress), 1.0 - progress)
        };

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            if is_frost_puddle {
                mat.base_color = Color::srgba(0.68, 0.88, 0.98, alpha);
                mat.emissive = LinearRgba::new(0.08, 0.22, 0.40, 0.35 * fade_factor);
                mat.perceptual_roughness = 0.15;
                mat.metallic = 0.05;
                mat.reflectance = 0.88;
            } else {
                mat.base_color = puddle.color.with_alpha(alpha);
                mat.emissive = puddle.color.to_linear() * (0.32 * fade_factor);
                mat.perceptual_roughness = 0.85;
                mat.metallic = 0.0;
                mat.reflectance = 0.12;
            }
        }

        if let Some(ref mut light) = point_light {
            light.intensity = if is_rtx { 2400.0 * fade_factor } else { 0.0 };
            light.color = if is_frost_puddle { Color::srgb(0.60, 0.85, 1.0) } else { puddle.color };
            light.range = 3.8;
            light.shadows_enabled = false;
        }

        let spread_duration = 0.38;
        let scale_spread = if is_frost_puddle {
            if puddle.elapsed > fade_start {
                let progress = (puddle.elapsed - fade_start) / fade_duration;
                1.0 - progress * 0.12
            } else {
                1.0
            }
        } else if puddle.elapsed < spread_duration {
            let t = (puddle.elapsed / spread_duration).clamp(0.0, 1.0);
            (0.15 + 0.85 * (1.0 - (1.0 - t).powi(3))) * puddle.radius
        } else if puddle.elapsed > fade_start {
            let progress = (puddle.elapsed - fade_start) / fade_duration;
            (1.0 - progress * 0.12) * puddle.radius
        } else {
            puddle.radius
        };
        transform.scale = if is_frost_puddle {
            Vec3::splat(scale_spread)
        } else {
            Vec3::new(scale_spread, 1.0, scale_spread)
        };

        active_puddles.push((
            entity,
            transform.translation,
            puddle.radius,
            puddle.color,
            puddle.effects.clone(),
            puddle.elapsed,
        ));

        for (r_tf, mut receiver) in receivers.iter_mut() {
            if r_tf.translation.distance(transform.translation) <= puddle.radius * 1.15 {
                for effect in &puddle.effects {
                    receiver.apply_effect(effect.kind, effect.duration, effect.magnitude);
                }
            }
        }

        let has_thermal = puddle.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat);
        if has_thermal {
            let heat_power = puddle
                .effects
                .iter()
                .filter(|e| e.kind == AlchemicalEffectKind::ThermalHeat)
                .map(|e| e.magnitude)
                .fold(25.0, f32::max);
            for (f_tf, f_gt, f_aabb, mut flammable) in flammables.iter_mut() {
                let center = flammable.compute_world_center(f_tf, f_gt, f_aabb);
                let dist = center.distance(transform.translation);
                if dist <= puddle.radius * 1.25 {
                    let proximity = (1.0 - (dist / (puddle.radius * 1.25)).clamp(0.0, 1.0)).max(0.35);
                    flammable.apply_heat(heat_power * 1.8 * proximity * dt);
                }
            }
        }
        let dx = transform.translation.x;
        let dz = transform.translation.z - (-8.5);
        let dist_lake = (dx * dx + dz * dz).sqrt();
        let near_water = dist_lake <= 6.8 || crate::dungeon::modular_kit::is_in_lake_water(transform.translation);
        if has_thermal && puddle.elapsed <= dt && near_water {
            for i in 0..12 {
                let angle = i as f32 * (std::f32::consts::TAU / 12.0);
                let r = 0.4 + (i as f32 % 3.0) * 0.45;
                commands.spawn((
                    Transform::from_xyz(
                        transform.translation.x + angle.cos() * r,
                        transform.translation.y + 0.15,
                        transform.translation.z + angle.sin() * r,
                    ),
                    crate::render::particles::ParticleEmitter {
                        spawn_timer: 0.0,
                        spawn_rate: 0.022,
                        particle_type: crate::render::particles::ParticleType::Steam,
                    },
                    AlchemyTransientEmitter {
                        elapsed: 0.0,
                        max_lifetime: 3.5,
                    },
                ));
            }
        }
    }

    let mut existing_contacts: Vec<(Entity, Entity, Entity)> = Vec::new();
    for (cz_ent, cz, mut cz_tf, cz_mat) in contact_zones.iter_mut() {
        let p_a = active_puddles.iter().find(|(e, _, _, _, _, _)| *e == cz.puddle_a);
        let p_b = active_puddles.iter().find(|(e, _, _, _, _, _)| *e == cz.puddle_b);
        if let (Some(a), Some(b)) = (p_a, p_b) {
            let dist = a.1.distance(b.1);
            if dist < a.2 + b.2 {
                let overlap = (a.2 + b.2 - dist).max(0.0);
                let contact_r = (overlap * 0.5).clamp(0.18, a.2.min(b.2));
                let mid = (a.1 + b.1) * 0.5;
                cz_tf.translation = mid + Vec3::Y * 0.003;
                cz_tf.scale = Vec3::new(contact_r, 1.0, contact_r);

                let oldest_elapsed = a.5.max(b.5);
                let fade_factor = if oldest_elapsed <= 50.0 {
                    1.0
                } else {
                    (1.0 - (oldest_elapsed - 50.0) / 25.0).clamp(0.0, 1.0)
                };

                if let Some(mat) = materials.get_mut(&cz_mat.0) {
                    mat.base_color = cz.color.with_alpha(0.88 * fade_factor);
                    mat.emissive = cz.color.to_linear() * (0.35 * fade_factor);
                    mat.perceptual_roughness = 0.85;
                    mat.metallic = 0.0;
                    mat.reflectance = 0.12;
                }

                for (r_tf, mut receiver) in receivers.iter_mut() {
                    if r_tf.translation.distance(mid) <= contact_r * 1.25 {
                        for effect in &cz.effects {
                            receiver.apply_effect(effect.kind, effect.duration, effect.magnitude);
                        }
                    }
                }

                if cz.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat) {
                    let heat_power = cz
                        .effects
                        .iter()
                        .filter(|e| e.kind == AlchemicalEffectKind::ThermalHeat)
                        .map(|e| e.magnitude)
                        .fold(25.0, f32::max);
                    for (f_tf, f_gt, f_aabb, mut flammable) in flammables.iter_mut() {
                        let center = flammable.compute_world_center(f_tf, f_gt, f_aabb);
                        let dist = center.distance(mid);
                        if dist <= contact_r * 1.25 {
                            let proximity = (1.0 - (dist / (contact_r * 1.25)).clamp(0.0, 1.0)).max(0.35);
                            flammable.apply_heat(heat_power * 1.8 * proximity * dt);
                        }
                    }
                }

                existing_contacts.push((cz_ent, cz.puddle_a, cz.puddle_b));
                continue;
            }
        }
        commands.entity(cz_ent).despawn();
    }

    for i in 0..active_puddles.len() {
        for j in (i + 1)..active_puddles.len() {
            let a = &active_puddles[i];
            let b = &active_puddles[j];
            let dist = a.1.distance(b.1);
            if dist < a.2 + b.2 {
                let already_exists = existing_contacts.iter().any(|(_, ea, eb)| {
                    (*ea == a.0 && *eb == b.0) || (*ea == b.0 && *eb == a.0)
                });
                if !already_exists {
                    let overlap = (a.2 + b.2 - dist).max(0.0);
                    let contact_r = (overlap * 0.5).clamp(0.18, a.2.min(b.2));
                    let mid = (a.1 + b.1) * 0.5;

                    let mixed_color = mix_potion_colors(a.3, b.3);
                    let mut mixed_effects = a.4.clone();
                    merge_potion_effects(&mut mixed_effects, &b.4);

                    let mat = materials.add(StandardMaterial {
                        base_color: mixed_color.with_alpha(0.88),
                        emissive: mixed_color.to_linear() * 0.35,
                        perceptual_roughness: 0.85,
                        metallic: 0.0,
                        reflectance: 0.12,
                        cull_mode: None,
                        double_sided: true,
                        alpha_mode: AlphaMode::Blend,
                        depth_bias: 750.0,
                        ..default()
                    });

                    let mesh = if let Some(ref mut meshes_res) = meshes {
                        meshes_res.add(create_irregular_puddle_mesh(1.0, 24))
                    } else {
                        Handle::default()
                    };

                    commands.spawn((
                        PotionContactZone {
                            puddle_a: a.0,
                            puddle_b: b.0,
                            color: mixed_color,
                            effects: mixed_effects.clone(),
                            radius: contact_r,
                        },
                        Mesh3d(mesh),
                        MeshMaterial3d(mat),
                        Transform::from_translation(mid + Vec3::Y * 0.003)
                            .with_scale(Vec3::new(contact_r, 1.0, contact_r)),
                    ));

                    let has_thermal = mixed_effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat);
                    let has_frost = mixed_effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost);
                    let has_luminescence = mixed_effects.iter().any(|e| e.kind == AlchemicalEffectKind::Luminescence);
                    let has_reactive = mixed_effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat || e.kind == AlchemicalEffectKind::Vitality);

                    if has_thermal && has_frost {
                        commands.spawn((
                            Transform::from_translation(mid + Vec3::new(0.0, 0.18, 0.0)),
                            crate::render::particles::ParticleEmitter {
                                spawn_timer: 0.0,
                                spawn_rate: 0.04,
                                particle_type: crate::render::particles::ParticleType::Steam,
                            },
                            AlchemyTransientEmitter {
                                elapsed: 0.0,
                                max_lifetime: 2.5,
                            },
                        ));
                    }

                    if has_luminescence && has_reactive {
                        commands.spawn((
                            Transform::from_translation(mid + Vec3::new(0.0, 0.22, 0.0)),
                            crate::render::particles::ParticleEmitter {
                                spawn_timer: 0.0,
                                spawn_rate: 0.035,
                                particle_type: crate::render::particles::ParticleType::GlowSpore,
                            },
                            AlchemyTransientEmitter {
                                elapsed: 0.0,
                                max_lifetime: 1.8,
                            },
                            PointLight {
                                color: Color::srgb(0.98, 0.92, 0.40),
                                intensity: 4500.0,
                                range: 5.5,
                                shadows_enabled: false,
                                ..default()
                            },
                        ));
                    }

                    for (r_tf, mut receiver) in receivers.iter_mut() {
                        if r_tf.translation.distance(mid) <= contact_r * 1.25 {
                            for effect in &mixed_effects {
                                receiver.apply_effect(effect.kind, effect.duration, effect.magnitude);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn update_glass_shards(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shards: Query<(Entity, &mut GlassShardParticle, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    let gravity = Vec3::new(0.0, -26.0, 0.0);
    for (entity, mut shard, mut transform, mat_handle) in shards.iter_mut() {
        shard.lifetime += dt;
        if shard.lifetime >= shard.max_lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        if !shard.settled {
            shard.velocity += gravity * dt;
            transform.translation += shard.velocity * dt;

            let rot_delta = Quat::from_euler(
                EulerRot::XYZ,
                shard.angular_velocity.x * dt,
                shard.angular_velocity.y * dt,
                shard.angular_velocity.z * dt,
            );
            transform.rotation = transform.rotation * rot_delta;

            if transform.translation.y <= shard.ground_y && shard.velocity.y < 0.0 {
                if shard.bounces_remaining > 0 {
                    shard.bounces_remaining -= 1;
                    shard.velocity.y = -shard.velocity.y * 0.35;
                    shard.velocity.x *= 0.65;
                    shard.velocity.z *= 0.65;
                    shard.angular_velocity *= 0.55;
                    transform.translation.y = shard.ground_y + 0.01;
                } else {
                    transform.translation.y = shard.ground_y;
                    shard.velocity.y = 0.0;
                    shard.velocity.x *= 0.20;
                    shard.velocity.z *= 0.20;
                    shard.settled = true;
                    let flat_rot = Quat::from_rotation_y(transform.rotation.to_euler(EulerRot::YXZ).0);
                    transform.rotation = flat_rot;
                }
            }
        } else {
            let friction = (1.0 - 12.0 * dt).max(0.0);
            shard.velocity.x *= friction;
            shard.velocity.z *= friction;
            transform.translation.x += shard.velocity.x * dt;
            transform.translation.z += shard.velocity.z * dt;
            transform.translation.y = shard.ground_y;
        }

        if transform.translation.z > 200.0 {
            let origin = crate::dungeon::test_room::SECRET_TEST_ROOM_ORIGIN;
            transform.translation.x = transform.translation.x.clamp(origin.x - 39.9, origin.x + 39.9);
            transform.translation.z = transform.translation.z.clamp(origin.z - 39.9, origin.z + 39.9);
            if shard.settled {
                transform.translation.y = origin.y;
            }
        }

        let fade_time = 3.0;
        if shard.max_lifetime - shard.lifetime < fade_time {
            let remaining = (shard.max_lifetime - shard.lifetime) / fade_time;
            if let Some(mat) = materials.get_mut(&mat_handle.0) {
                let c = shard.color.to_srgba();
                let stain_base = Color::srgba(
                    c.red * 0.78 + 0.18,
                    c.green * 0.78 + 0.18,
                    c.blue * 0.78 + 0.18,
                    0.95 * remaining.clamp(0.0, 1.0),
                );
                mat.base_color = stain_base;
                mat.emissive = shard.color.to_linear() * (1.6 * remaining.clamp(0.0, 1.0));
            }
        }
    }
}

pub fn update_ice_footprints(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut IceFootprint)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut footprint) in query.iter_mut() {
        footprint.lifetime += dt;
        if footprint.lifetime >= footprint.max_lifetime {
            commands.entity(entity).despawn();
        } else {
            let progress = footprint.lifetime / footprint.max_lifetime;
            let scale_factor = (1.0 - progress * 0.35).max(0.1);
            tf.scale = Vec3::new(scale_factor, scale_factor, scale_factor);
        }
    }
}

pub fn update_flammable_combustion(
    time: Res<Time>,
    mut commands: Commands,
    mut flammables: Query<(
        Entity,
        &mut Transform,
        Option<&GlobalTransform>,
        Option<&Aabb>,
        &mut Flammable,
        Option<&Name>,
    )>,
    bench_obstacles: Query<Entity, With<crate::dungeon::modular_kit::BenchObstacle>>,
    mut point_lights: Query<&mut PointLight>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let burning_sources: Vec<Vec3> = flammables
        .iter()
        .filter(|(_, _, _, _, f, _)| f.is_burning && f.elapsed >= 0.25)
        .map(|(_, tf, gt, aabb, f, _)| f.compute_world_center(tf, gt, aabb))
        .collect();

    for (entity, mut transform, gt, aabb, mut flammable, name) in flammables.iter_mut() {
        let center = flammable.compute_world_center(&transform, gt, aabb);
        if !flammable.is_burning {
            let mut heat_absorbed = 0.0;
            for source_pos in &burning_sources {
                let dist = center.distance(*source_pos);
                if dist <= 1.25 {
                    let proximity = (1.0 - dist / 1.25).clamp(0.2, 1.0);
                    heat_absorbed += 30.0 * proximity * dt;
                }
            }
            if heat_absorbed > 0.0 {
                flammable.apply_heat(heat_absorbed);
            } else {
                flammable.cool_down(dt);
            }
        }

        if !flammable.is_burning {
            continue;
        }

        if flammable.initial_scale.is_none() {
            flammable.initial_scale = Some(transform.scale);
        }
        let base_scale = flammable.initial_scale.unwrap_or(transform.scale);

        if flammable.initial_translation.is_none() {
            flammable.initial_translation = Some(transform.translation);
        }
        let base_trans = flammable.initial_translation.unwrap_or(transform.translation);

        let local_offset = flammable
            .center_offset
            .filter(|&c| c != Vec3::ZERO)
            .or_else(|| aabb.map(|a| Vec3::from(a.center)))
            .unwrap_or(Vec3::ZERO);

        if flammable.fire_light.is_none() {
            let mut fire_l_ent = None;
            let mut fire_e_ent = None;
            commands.entity(entity).insert((
                Visibility::default(),
                InheritedVisibility::default(),
            )).with_children(|parent| {
                let l = parent
                    .spawn((
                        PointLight {
                            color: Color::srgb(1.0, 0.42, 0.08),
                            intensity: 3200.0,
                            range: 4.8,
                            shadows_enabled: false,
                            ..default()
                        },
                        Transform::from_xyz(local_offset.x, local_offset.y + 0.35, local_offset.z),
                    ))
                    .id();
                fire_l_ent = Some(l);

                let e = parent
                    .spawn((
                        crate::render::particles::ParticleEmitter {
                            spawn_timer: 0.0,
                            spawn_rate: 0.045,
                            particle_type: crate::render::particles::ParticleType::FireFlame,
                        },
                        Transform::from_xyz(local_offset.x, local_offset.y + 0.25, local_offset.z),
                    ))
                    .id();
                fire_e_ent = Some(e);
            });
            flammable.fire_light = fire_l_ent;
            flammable.fire_emitter = fire_e_ent;
        }

        if let Some(light_ent) = flammable.fire_light {
            if let Ok(mut light) = point_lights.get_mut(light_ent) {
                let flicker = ((time.elapsed_secs() * 18.0).sin() * 550.0)
                    + ((time.elapsed_secs() * 33.0).cos() * 320.0);
                light.intensity = (3200.0 + flicker).max(1200.0);
            }
        }

        flammable.elapsed += dt;
        let progress = (flammable.elapsed / flammable.burn_duration).clamp(0.0, 1.0);

        if progress < 0.60 {
            let shudder = ((flammable.elapsed * 32.0).sin()) * 0.035 * (progress * 1.5);
            transform.scale = base_scale * (1.0 + shudder);
        } else {
            let collapse_t = (progress - 0.60) / 0.40;
            let scale_y = (1.0 - collapse_t * 0.94).max(0.03);
            let scale_xz = (1.0 - collapse_t * 0.85).max(0.06);
            let current_scale = base_scale * Vec3::new(scale_xz, scale_y, scale_xz);
            transform.scale = current_scale;
            if local_offset != Vec3::ZERO {
                let rot_scale = transform.rotation * (current_scale * local_offset);
                let base_rot_scale = transform.rotation * (base_scale * local_offset);
                transform.translation = base_trans + base_rot_scale - rot_scale - Vec3::new(0.0, collapse_t * 0.05, 0.0);
            } else {
                transform.translation.y = base_trans.y - (collapse_t * 0.05);
            }
        }

        if flammable.elapsed >= flammable.burn_duration {
            flammable.is_burning = false;
            flammable.elapsed = -999.0;

            let ash_pos = center + Vec3::new(0.0, 0.20, 0.0);
            commands.spawn((
                Transform::from_translation(ash_pos),
                crate::render::particles::ParticleEmitter {
                    spawn_timer: 0.0,
                    spawn_rate: 0.05,
                    particle_type: crate::render::particles::ParticleType::Steam,
                },
                AlchemyTransientEmitter {
                    elapsed: 0.0,
                    max_lifetime: 1.8,
                },
            ));

            if let Some(n) = name {
                if n.as_str().contains("Bench") {
                    for obstacle_ent in &bench_obstacles {
                        commands.queue(move |world: &mut World| {
                            if let Ok(obs_mut) = world.get_entity_mut(obstacle_ent) {
                                obs_mut.despawn();
                            }
                        });
                    }
                }
            }

            commands.queue(move |world: &mut World| {
                if let Ok(ent_cmd) = world.get_entity_mut(entity) {
                    ent_cmd.despawn_recursive();
                }
            });
        }
    }
}

pub fn cleanup_alchemy_memory_limits(
    mut commands: Commands,
    query_puddles: Query<(Entity, &PotionPuddle)>,
    query_shards: Query<(Entity, &GlassShardParticle)>,
    query_footprints: Query<(Entity, &IceFootprint)>,
    query_projectiles: Query<(Entity, &PotionProjectile)>,
    query_emitters: Query<(Entity, &AlchemyTransientEmitter)>,
) {
    const MAX_PUDDLES: usize = 36;
    let puddle_count = query_puddles.iter().count();
    if puddle_count > MAX_PUDDLES {
        let mut puddles: Vec<(Entity, f32)> = query_puddles.iter().map(|(e, p)| (e, p.elapsed)).collect();
        puddles.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (e, _) in puddles.into_iter().take(puddle_count - MAX_PUDDLES) {
            commands.queue(move |world: &mut World| {
                if let Ok(ent) = world.get_entity_mut(e) {
                    ent.despawn();
                }
            });
        }
    }

    const MAX_SHARDS: usize = 128;
    let shard_count = query_shards.iter().count();
    if shard_count > MAX_SHARDS {
        let mut shards: Vec<(Entity, f32)> = query_shards.iter().map(|(e, s)| (e, s.lifetime)).collect();
        shards.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (e, _) in shards.into_iter().take(shard_count - MAX_SHARDS) {
            commands.queue(move |world: &mut World| {
                if let Ok(ent) = world.get_entity_mut(e) {
                    ent.despawn();
                }
            });
        }
    }

    const MAX_FOOTPRINTS: usize = 48;
    let footprint_count = query_footprints.iter().count();
    if footprint_count > MAX_FOOTPRINTS {
        let mut prints: Vec<(Entity, f32)> = query_footprints.iter().map(|(e, f)| (e, f.lifetime)).collect();
        prints.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (e, _) in prints.into_iter().take(footprint_count - MAX_FOOTPRINTS) {
            commands.queue(move |world: &mut World| {
                if let Ok(ent) = world.get_entity_mut(e) {
                    ent.despawn();
                }
            });
        }
    }

    const MAX_PROJECTILES: usize = 24;
    let proj_count = query_projectiles.iter().count();
    if proj_count > MAX_PROJECTILES {
        let mut projs: Vec<(Entity, f32)> = query_projectiles.iter().map(|(e, p)| (e, p.lifetime)).collect();
        projs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (e, _) in projs.into_iter().take(proj_count - MAX_PROJECTILES) {
            commands.queue(move |world: &mut World| {
                if let Ok(ent) = world.get_entity_mut(e) {
                    ent.despawn();
                }
            });
        }
    }

    const MAX_TRANSIENT_EMITTERS: usize = 32;
    let emitter_count = query_emitters.iter().count();
    if emitter_count > MAX_TRANSIENT_EMITTERS {
        let mut emitters: Vec<(Entity, f32)> = query_emitters.iter().map(|(e, em)| (e, em.elapsed)).collect();
        emitters.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (e, _) in emitters.into_iter().take(emitter_count - MAX_TRANSIENT_EMITTERS) {
            commands.queue(move |world: &mut World| {
                if let Ok(ent) = world.get_entity_mut(e) {
                    ent.despawn();
                }
            });
        }
    }
}

pub fn create_ice_disk_mesh(radius: f32, segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(segments * 3 + 2);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(segments * 3 + 2);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 12);

    let half_h = 0.08;
    positions.push([0.0, half_h, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    colors.push([0.65, 0.88, 0.98, 0.82]);

    let step = std::f32::consts::TAU / segments as f32;
    for i in 0..segments {
        let angle = i as f32 * step;
        let facet_var = ((i as f32 * 3.7).sin() * 0.012) + ((i as f32 * 7.1).cos() * 0.008);
        let px = angle.cos() * radius;
        let pz = angle.sin() * radius;
        positions.push([px, half_h + facet_var, pz]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5]);
        colors.push([0.90, 0.96, 1.0, 0.92]);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.extend_from_slice(&[0, (i + 1) as u32, (next + 1) as u32]);
    }

    let rim_start = positions.len() as u32;
    for i in 0..segments {
        let angle = i as f32 * step;
        let px = angle.cos() * radius;
        let pz = angle.sin() * radius;
        let nx = angle.cos();
        let nz = angle.sin();

        positions.push([px, half_h, pz]);
        normals.push([nx, 0.0, nz]);
        uvs.push([i as f32 / segments as f32, 0.0]);
        colors.push([0.92, 0.98, 1.0, 0.95]);

        positions.push([px, -half_h, pz]);
        normals.push([nx, 0.0, nz]);
        uvs.push([i as f32 / segments as f32, 1.0]);
        colors.push([0.50, 0.75, 0.92, 0.80]);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        let top_cur = rim_start + (i * 2) as u32;
        let bot_cur = top_cur + 1;
        let top_next = rim_start + (next * 2) as u32;
        let bot_next = top_next + 1;

        indices.extend_from_slice(&[top_cur, bot_cur, top_next]);
        indices.extend_from_slice(&[top_next, bot_cur, bot_next]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_irregular_ice_block_mesh(radius: f32, height: f32, segments: usize) -> Mesh {
    let seg_count = segments.max(12);
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let half_h = height * 0.5;
    let step = std::f32::consts::TAU / seg_count as f32;

    let mut profile: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(seg_count);
    for i in 0..seg_count {
        let angle = i as f32 * step;
        let r_pert = 1.0
            + 0.14 * (angle * 3.0 + 0.8).sin() * 0.6
            + 0.10 * (angle * 5.0 - 1.2).cos() * 0.4
            + 0.06 * (angle * 7.0 + 2.1).sin() * 0.3;
        let r = radius * r_pert;
        let top_var = ((i as f32 * 3.7).sin() * 0.035) + ((i as f32 * 7.1).cos() * 0.025);
        let top_y = half_h + top_var;
        let bot_y = -half_h;
        profile.push((angle, r, top_y, bot_y));
    }

    let top_center_idx = positions.len() as u32;
    positions.push([0.0, half_h + 0.04, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    colors.push([0.72, 0.92, 1.0, 0.92]);

    let top_ring_start = positions.len() as u32;
    for &(angle, r, top_y, _) in &profile {
        let px = angle.cos() * r;
        let pz = angle.sin() * r;
        positions.push([px, top_y, pz]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5]);
        colors.push([0.88, 0.96, 1.0, 0.95]);
    }

    for i in 0..seg_count {
        let next = (i + 1) % seg_count;
        indices.push(top_center_idx);
        indices.push(top_ring_start + i as u32);
        indices.push(top_ring_start + next as u32);
    }

    let bot_center_idx = positions.len() as u32;
    positions.push([0.0, -half_h, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    colors.push([0.35, 0.60, 0.85, 0.85]);

    let bot_ring_start = positions.len() as u32;
    for &(angle, r, _, bot_y) in &profile {
        let px = angle.cos() * r;
        let pz = angle.sin() * r;
        positions.push([px, bot_y, pz]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push([0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5]);
        colors.push([0.40, 0.65, 0.88, 0.85]);
    }

    for i in 0..seg_count {
        let next = (i + 1) % seg_count;
        indices.push(bot_center_idx);
        indices.push(bot_ring_start + next as u32);
        indices.push(bot_ring_start + i as u32);
    }

    for i in 0..seg_count {
        let next = (i + 1) % seg_count;
        let (angle_a, r_a, top_ya, bot_ya) = profile[i];
        let (angle_b, r_b, top_yb, bot_yb) = profile[next];

        let p_top_a = Vec3::new(angle_a.cos() * r_a, top_ya, angle_a.sin() * r_a);
        let p_bot_a = Vec3::new(angle_a.cos() * r_a, bot_ya, angle_a.sin() * r_a);
        let p_top_b = Vec3::new(angle_b.cos() * r_b, top_yb, angle_b.sin() * r_b);
        let p_bot_b = Vec3::new(angle_b.cos() * r_b, bot_yb, angle_b.sin() * r_b);

        let v_side = p_top_b - p_top_a;
        let v_down = p_bot_a - p_top_a;
        let face_norm = v_side.cross(v_down).normalize_or_zero();
        let norm_arr = [face_norm.x, face_norm.y, face_norm.z];

        let u_a = i as f32 / seg_count as f32;
        let u_b = (i + 1) as f32 / seg_count as f32;

        let wall_start = positions.len() as u32;

        positions.push([p_top_a.x, p_top_a.y, p_top_a.z]);
        normals.push(norm_arr);
        uvs.push([u_a, 0.0]);
        colors.push([0.85, 0.95, 1.0, 0.92]);

        positions.push([p_bot_a.x, p_bot_a.y, p_bot_a.z]);
        normals.push(norm_arr);
        uvs.push([u_a, 1.0]);
        colors.push([0.32, 0.58, 0.82, 0.88]);

        positions.push([p_top_b.x, p_top_b.y, p_top_b.z]);
        normals.push(norm_arr);
        uvs.push([u_b, 0.0]);
        colors.push([0.85, 0.95, 1.0, 0.92]);

        positions.push([p_bot_b.x, p_bot_b.y, p_bot_b.z]);
        normals.push(norm_arr);
        uvs.push([u_b, 1.0]);
        colors.push([0.32, 0.58, 0.82, 0.88]);

        indices.push(wall_start);
        indices.push(wall_start + 2);
        indices.push(wall_start + 1);

        indices.push(wall_start + 2);
        indices.push(wall_start + 3);
        indices.push(wall_start + 1);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_footprint_mesh(length: f32, width: f32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(16);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(16);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(16);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(16);
    let mut indices: Vec<u32> = Vec::with_capacity(36);

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);
    colors.push([0.72, 0.88, 0.98, 0.75]);

    let segments = 12;
    let step = std::f32::consts::TAU / segments as f32;
    for i in 0..segments {
        let angle = i as f32 * step;
        let half_w = width * 0.5;
        let half_l = length * 0.5;
        let px = angle.cos() * half_w;
        let pz = angle.sin() * half_l;

        positions.push([px, 0.0, pz]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5]);
        colors.push([0.95, 0.98, 1.0, 0.90]);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.extend_from_slice(&[0, (i + 1) as u32, (next + 1) as u32]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_lathe_mesh(profile: &[(f32, f32)], segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let n_pts = profile.len();
    if n_pts < 2 || segments < 3 {
        return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    }

    let mut profile_normals = Vec::with_capacity(n_pts);
    for j in 0..n_pts {
        let (dr, dy) = if j == 0 {
            (profile[1].0 - profile[0].0, profile[1].1 - profile[0].1)
        } else if j == n_pts - 1 {
            (profile[j].0 - profile[j - 1].0, profile[j].1 - profile[j - 1].1)
        } else {
            (profile[j + 1].0 - profile[j - 1].0, profile[j + 1].1 - profile[j - 1].1)
        };
        let len = (dr * dr + dy * dy).sqrt().max(1e-6);
        profile_normals.push((dy / len, -dr / len));
    }

    for j in 0..n_pts {
        let (r, y) = profile[j];
        let (nr_2d, ny_2d) = profile_normals[j];
        let v = j as f32 / (n_pts - 1) as f32;

        for i in 0..segments {
            let theta = i as f32 / segments as f32 * std::f32::consts::TAU;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let u = i as f32 / segments as f32;

            positions.push([r * cos_t, y, r * sin_t]);
            normals.push([nr_2d * cos_t, ny_2d, nr_2d * sin_t]);
            uvs.push([u, v]);
        }
    }

    for j in 0..(n_pts - 1) {
        let r0 = (j * segments) as u32;
        let r1 = ((j + 1) * segments) as u32;
        for i in 0..segments {
            let next_i = ((i + 1) % segments) as u32;
            let i = i as u32;
            let a = r0 + i;
            let b = r0 + next_i;
            let c = r1 + next_i;
            let d = r1 + i;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_ring_mesh(radius: f32, tube_radius: f32, radial_segments: usize, tubular_segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for j in 0..tubular_segments {
        let v = j as f32 / tubular_segments as f32 * std::f32::consts::TAU;
        let cos_v = v.cos();
        let sin_v = v.sin();

        for i in 0..radial_segments {
            let u = i as f32 / radial_segments as f32 * std::f32::consts::TAU;
            let cos_u = u.cos();
            let sin_u = u.sin();

            let x = (radius + tube_radius * cos_v) * cos_u;
            let y = tube_radius * sin_v;
            let z = (radius + tube_radius * cos_v) * sin_u;

            let nx = cos_v * cos_u;
            let ny = sin_v;
            let nz = cos_v * sin_u;

            positions.push([x, y, z]);
            normals.push([nx, ny, nz]);
            uvs.push([i as f32 / radial_segments as f32, j as f32 / tubular_segments as f32]);
        }
    }

    for j in 0..tubular_segments {
        let next_j = (j + 1) % tubular_segments;
        let r0 = (j * radial_segments) as u32;
        let r1 = (next_j * radial_segments) as u32;

        for i in 0..radial_segments {
            let next_i = ((i + 1) % radial_segments) as u32;
            let i = i as u32;
            let a = r0 + i;
            let b = r0 + next_i;
            let c = r1 + next_i;
            let d = r1 + i;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_held_flask_mesh(slot: usize) -> Mesh {
    match slot {
        2 => {
            let profile = [
                (0.006, -0.048),
                (0.046, -0.048),
                (0.052, -0.042),
                (0.042, -0.015),
                (0.031, 0.015),
                (0.019, 0.046),
                (0.019, 0.082),
                (0.024, 0.085),
                (0.020, 0.089),
            ];
            create_lathe_mesh(&profile, 20)
        }
        3 => {
            let profile = [
                (0.006, -0.050),
                (0.032, -0.050),
                (0.048, -0.036),
                (0.048, 0.024),
                (0.025, 0.050),
                (0.022, 0.052),
                (0.022, 0.072),
                (0.026, 0.075),
                (0.022, 0.077),
            ];
            create_lathe_mesh(&profile, 8)
        }
        4 => {
            let profile = [
                (0.006, -0.052),
                (0.026, -0.048),
                (0.045, -0.024),
                (0.046, -0.004),
                (0.037, 0.022),
                (0.024, 0.046),
                (0.016, 0.072),
                (0.016, 0.096),
                (0.020, 0.099),
                (0.017, 0.103),
            ];
            create_lathe_mesh(&profile, 20)
        }
        5 => {
            let profile = [
                (0.005, -0.060),
                (0.022, -0.038),
                (0.044, -0.012),
                (0.044, 0.032),
                (0.024, 0.062),
                (0.018, 0.082),
                (0.022, 0.085),
                (0.019, 0.089),
            ];
            create_lathe_mesh(&profile, 6)
        }
        _ => {
            let profile = [
                (0.005, -0.046),
                (0.024, -0.043),
                (0.040, -0.028),
                (0.048, -0.002),
                (0.045, 0.024),
                (0.030, 0.044),
                (0.017, 0.054),
                (0.017, 0.082),
                (0.021, 0.085),
                (0.018, 0.089),
            ];
            create_lathe_mesh(&profile, 20)
        }
    }
}

pub fn create_held_fluid_mesh(slot: usize) -> Mesh {
    match slot {
        2 => {
            let profile = [
                (0.005, -0.042),
                (0.040, -0.042),
                (0.045, -0.037),
                (0.035, -0.012),
                (0.025, 0.014),
                (0.015, 0.038),
            ];
            create_lathe_mesh(&profile, 20)
        }
        3 => {
            let profile = [
                (0.005, -0.045),
                (0.026, -0.045),
                (0.042, -0.032),
                (0.042, 0.018),
                (0.021, 0.036),
            ];
            create_lathe_mesh(&profile, 8)
        }
        4 => {
            let profile = [
                (0.005, -0.046),
                (0.021, -0.042),
                (0.039, -0.020),
                (0.040, -0.002),
                (0.031, 0.018),
                (0.019, 0.038),
                (0.012, 0.052),
            ];
            create_lathe_mesh(&profile, 20)
        }
        5 => {
            let profile = [
                (0.004, -0.052),
                (0.018, -0.034),
                (0.038, -0.010),
                (0.038, 0.025),
                (0.019, 0.048),
            ];
            create_lathe_mesh(&profile, 6)
        }
        _ => {
            let profile = [
                (0.004, -0.040),
                (0.020, -0.037),
                (0.034, -0.024),
                (0.041, -0.002),
                (0.038, 0.020),
                (0.024, 0.036),
                (0.013, 0.046),
            ];
            create_lathe_mesh(&profile, 20)
        }
    }
}

pub fn create_held_stopper_mesh(slot: usize) -> Mesh {
    match slot {
        2 => {
            let profile = [
                (0.016, 0.080),
                (0.018, 0.088),
                (0.025, 0.102),
                (0.018, 0.114),
                (0.004, 0.118),
            ];
            create_lathe_mesh(&profile, 8)
        }
        3 => {
            let profile = [
                (0.019, 0.074),
                (0.028, 0.077),
                (0.029, 0.088),
                (0.022, 0.092),
                (0.005, 0.093),
            ];
            create_lathe_mesh(&profile, 8)
        }
        4 => {
            let profile = [
                (0.013, 0.094),
                (0.018, 0.108),
                (0.004, 0.126),
            ];
            create_lathe_mesh(&profile, 6)
        }
        5 => {
            let profile = [
                (0.015, 0.082),
                (0.016, 0.094),
                (0.015, 0.106),
                (0.004, 0.124),
            ];
            create_lathe_mesh(&profile, 6)
        }
        _ => {
            let profile = [
                (0.013, 0.076),
                (0.0155, 0.086),
                (0.0185, 0.102),
                (0.0170, 0.108),
                (0.004, 0.110),
            ];
            create_lathe_mesh(&profile, 16)
        }
    }
}

pub fn create_held_accent_mesh(slot: usize) -> Option<Mesh> {
    match slot {
        1 => {
            let profile = [
                (0.0172, 0.059),
                (0.0195, 0.061),
                (0.0192, 0.064),
                (0.0198, 0.067),
                (0.0192, 0.070),
                (0.0172, 0.072),
            ];
            Some(create_lathe_mesh(&profile, 18))
        }
        2 => {
            let profile = [
                (0.0192, 0.065),
                (0.0220, 0.067),
                (0.0220, 0.075),
                (0.0192, 0.077),
            ];
            Some(create_lathe_mesh(&profile, 18))
        }
        3 => {
            let profile = [
                (0.0225, 0.052),
                (0.0255, 0.054),
                (0.0255, 0.068),
                (0.0225, 0.070),
            ];
            Some(create_lathe_mesh(&profile, 8))
        }
        _ => None,
    }
}

pub fn create_potion_flask_mesh(slot: usize) -> Mesh {
    match slot {
        2 => {
            let profile = [
                (0.008, -0.12),
                (0.12, -0.12),
                (0.14, -0.10),
                (0.11, -0.03),
                (0.07, 0.04),
                (0.04, 0.12),
                (0.055, 0.14),
                (0.035, 0.15),
                (0.015, 0.18),
            ];
            create_lathe_mesh(&profile, 18)
        }
        3 => {
            let profile = [
                (0.01, -0.12),
                (0.08, -0.12),
                (0.12, -0.09),
                (0.12, 0.04),
                (0.065, 0.10),
                (0.055, 0.13),
                (0.07, 0.14),
                (0.07, 0.17),
                (0.015, 0.18),
            ];
            create_lathe_mesh(&profile, 8)
        }
        4 => {
            let profile = [
                (0.01, -0.13),
                (0.06, -0.12),
                (0.11, -0.06),
                (0.11, -0.01),
                (0.08, 0.05),
                (0.04, 0.12),
                (0.055, 0.14),
                (0.035, 0.15),
                (0.015, 0.18),
            ];
            create_lathe_mesh(&profile, 18)
        }
        5 => {
            let profile = [
                (0.01, -0.15),
                (0.05, -0.09),
                (0.11, -0.03),
                (0.11, 0.05),
                (0.06, 0.10),
                (0.04, 0.13),
                (0.055, 0.14),
                (0.035, 0.15),
                (0.015, 0.18),
            ];
            create_lathe_mesh(&profile, 6)
        }
        _ => {
            let profile = [
                (0.01, -0.12),
                (0.07, -0.11),
                (0.12, -0.06),
                (0.13, 0.0),
                (0.11, 0.06),
                (0.045, 0.11),
                (0.06, 0.14),
                (0.038, 0.15),
                (0.015, 0.18),
            ];
            create_lathe_mesh(&profile, 18)
        }
    }
}

pub fn create_flat_circle_mesh(radius: f32, segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 3);

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);

    for i in 0..segments {
        let theta = i as f32 / segments as f32 * std::f32::consts::TAU;
        let x = radius * theta.cos();
        let z = radius * theta.sin();
        positions.push([x, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
    }

    for i in 0..segments {
        let curr = (i + 1) as u32;
        let next = ((i + 1) % segments + 1) as u32;
        indices.extend_from_slice(&[0, curr, next]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_conforming_puddle_mesh(
    base_radius: f32,
    segments: usize,
    center_world: Vec3,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 1);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 1);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(segments * 3 + 1);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 18);

    let in_water = crate::dungeon::modular_kit::is_in_lake_water(center_world);
    let center_terr = if in_water {
        0.18
    } else {
        crate::dungeon::modular_kit::get_grotto_terrain_height(center_world.x, center_world.z)
    };
    positions.push([0.0, 0.028, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let perturbation = 1.0
            + 0.06 * (theta * 3.0).sin()
            + 0.04 * (theta * 5.0 + 0.8).cos()
            + 0.02 * (theta * 7.0).sin();
        let r_mid = base_radius * perturbation * 0.62;
        let x = r_mid * theta.cos();
        let z = r_mid * theta.sin();
        let dy = if in_water {
            0.0
        } else {
            let terr_h = crate::dungeon::modular_kit::get_grotto_terrain_height(center_world.x + x, center_world.z + z);
            (terr_h - center_terr).clamp(-0.06, 0.06)
        };
        positions.push([x, 0.022 + dy * 0.75, z]);
        let norm = Vec3::new(theta.cos() * 0.12, 0.98, theta.sin() * 0.12).normalize();
        normals.push([norm.x, norm.y, norm.z]);
        uvs.push([0.5 + (x / (base_radius * 2.4)), 0.5 + (z / (base_radius * 2.4))]);
    }

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let perturbation = 1.0
            + 0.06 * (theta * 3.0).sin()
            + 0.04 * (theta * 5.0 + 0.8).cos()
            + 0.02 * (theta * 7.0).sin();
        let r_rim = base_radius * perturbation;
        let x = r_rim * theta.cos();
        let z = r_rim * theta.sin();
        let dy = if in_water {
            0.0
        } else {
            let terr_h = crate::dungeon::modular_kit::get_grotto_terrain_height(center_world.x + x, center_world.z + z);
            (terr_h - center_terr).clamp(-0.06, 0.06)
        };
        positions.push([x, 0.012 + dy, z]);
        let norm = Vec3::new(theta.cos() * 0.25, 0.95, theta.sin() * 0.25).normalize();
        normals.push([norm.x, norm.y, norm.z]);
        uvs.push([0.5 + (x / (base_radius * 2.4)), 0.5 + (z / (base_radius * 2.4))]);
    }

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let perturbation = 1.0
            + 0.06 * (theta * 3.0).sin()
            + 0.04 * (theta * 5.0 + 0.8).cos()
            + 0.02 * (theta * 7.0).sin();
        let r_skirt = base_radius * perturbation * 1.02;
        let x = r_skirt * theta.cos();
        let z = r_skirt * theta.sin();
        let dy = if in_water {
            0.0
        } else {
            let terr_h = crate::dungeon::modular_kit::get_grotto_terrain_height(center_world.x + x, center_world.z + z);
            (terr_h - center_terr).clamp(-0.06, 0.06)
        };
        positions.push([x, -0.065 + dy, z]);
        normals.push([theta.cos(), 0.0, theta.sin()]);
        uvs.push([0.5 + (x / (base_radius * 2.4)), 0.5 + (z / (base_radius * 2.4))]);
    }

    let mid_start = 1 as u32;
    let rim_start = (1 + segments) as u32;
    let skirt_start = (1 + segments * 2) as u32;
    let seg_u32 = segments as u32;

    for i in 0..seg_u32 {
        let next_i = (i + 1) % seg_u32;

        let curr_mid = mid_start + i;
        let next_mid = mid_start + next_i;
        indices.extend_from_slice(&[0, next_mid, curr_mid]);

        let curr_rim = rim_start + i;
        let next_rim = rim_start + next_i;
        indices.extend_from_slice(&[curr_mid, next_mid, next_rim]);
        indices.extend_from_slice(&[curr_mid, next_rim, curr_rim]);

        let curr_skirt = skirt_start + i;
        let next_skirt = skirt_start + next_i;
        indices.extend_from_slice(&[curr_rim, next_rim, next_skirt]);
        indices.extend_from_slice(&[curr_rim, next_skirt, curr_skirt]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_irregular_puddle_mesh(base_radius: f32, segments: usize) -> Mesh {
    create_conforming_puddle_mesh(base_radius, segments, Vec3::ZERO)
}

pub fn create_wall_splatter_mesh(base_radius: f32, segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 1);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments * 3 + 1);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(segments * 3 + 1);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 18);

    positions.push([0.0, 0.022, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let base_pert = 1.0 + 0.28 * (theta * 3.0).sin() + 0.18 * (theta * 5.0 + 0.5).cos();
        let pert = if sin_t < 0.0 {
            let drip = (-sin_t).powf(1.6) * (0.80 + 0.70 * ((theta * 8.0).sin().max(0.0)).powi(3));
            base_pert + drip
        } else {
            base_pert * 0.72
        };

        let r_mid = base_radius * pert * 0.60;
        let x = r_mid * cos_t;
        let z = r_mid * sin_t;
        let wrap = -0.050 * (x / base_radius).powi(2);
        let y = 0.016 + wrap;
        positions.push([x, y, z]);
        let norm = Vec3::new(cos_t * 0.15, 0.98, sin_t * 0.15).normalize();
        normals.push([norm.x, norm.y, norm.z]);
        uvs.push([0.5 + (x / (base_radius * 2.5)), 0.5 + (z / (base_radius * 2.5))]);
    }

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let base_pert = 1.0 + 0.28 * (theta * 3.0).sin() + 0.18 * (theta * 5.0 + 0.5).cos();
        let pert = if sin_t < 0.0 {
            let drip = (-sin_t).powf(1.6) * (0.80 + 0.70 * ((theta * 8.0).sin().max(0.0)).powi(3));
            base_pert + drip
        } else {
            base_pert * 0.72
        };

        let r_rim = base_radius * pert;
        let x = r_rim * cos_t;
        let z = r_rim * sin_t;
        let wrap = -0.050 * (x / base_radius).powi(2);
        let y = 0.008 + wrap;
        positions.push([x, y, z]);
        let norm = Vec3::new(cos_t * 0.28, 0.94, sin_t * 0.28).normalize();
        normals.push([norm.x, norm.y, norm.z]);
        uvs.push([0.5 + (x / (base_radius * 2.5)), 0.5 + (z / (base_radius * 2.5))]);
    }

    for i in 0..segments {
        let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let sin_t = theta.sin();
        let cos_t = theta.cos();
        let base_pert = 1.0 + 0.28 * (theta * 3.0).sin() + 0.18 * (theta * 5.0 + 0.5).cos();
        let pert = if sin_t < 0.0 {
            let drip = (-sin_t).powf(1.6) * (0.80 + 0.70 * ((theta * 8.0).sin().max(0.0)).powi(3));
            base_pert + drip
        } else {
            base_pert * 0.72
        };

        let r_skirt = base_radius * pert * 1.03;
        let x = r_skirt * cos_t;
        let z = r_skirt * sin_t;
        let wrap = -0.050 * (x / base_radius).powi(2);
        let y = -0.055 + wrap;
        positions.push([x, y, z]);
        normals.push([cos_t, 0.0, sin_t]);
        uvs.push([0.5 + (x / (base_radius * 2.5)), 0.5 + (z / (base_radius * 2.5))]);
    }

    let mid_start = 1 as u32;
    let rim_start = (1 + segments) as u32;
    let skirt_start = (1 + segments * 2) as u32;
    let seg_u32 = segments as u32;

    for i in 0..seg_u32 {
        let next_i = (i + 1) % seg_u32;

        let curr_mid = mid_start + i;
        let next_mid = mid_start + next_i;
        indices.extend_from_slice(&[0, next_mid, curr_mid]);

        let curr_rim = rim_start + i;
        let next_rim = rim_start + next_i;
        indices.extend_from_slice(&[curr_mid, next_mid, next_rim]);
        indices.extend_from_slice(&[curr_mid, next_rim, curr_rim]);

        let curr_skirt = skirt_start + i;
        let next_skirt = skirt_start + next_i;
        indices.extend_from_slice(&[curr_rim, next_rim, next_skirt]);
        indices.extend_from_slice(&[curr_rim, next_skirt, curr_skirt]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

pub fn create_glass_shard_mesh(scale: f32, shard_type: usize) -> Mesh {
    match shard_type % 3 {
        0 => {
            let positions: Vec<[f32; 3]> = vec![
                [0.0, scale * 0.28, -scale * 0.85],
                [scale * 0.22, 0.0, -scale * 0.15],
                [scale * 0.08, 0.0, scale * 0.95],
                [-scale * 0.16, 0.0, scale * 0.35],
                [-scale * 0.24, 0.0, -scale * 0.40],
                [0.0, scale * 0.08, 0.0],
            ];
            let normals: Vec<[f32; 3]> = vec![
                [0.0, 0.95, -0.30],
                [0.85, 0.40, -0.20],
                [0.25, 0.80, 0.50],
                [-0.75, 0.50, 0.40],
                [-0.85, 0.40, -0.30],
                [0.0, 1.0, 0.0],
            ];
            let indices: Vec<u32> = vec![
                0, 1, 5,
                1, 2, 5,
                2, 3, 5,
                3, 4, 5,
                4, 0, 5,
                0, 5, 1,
                1, 5, 2,
                2, 5, 3,
                3, 5, 4,
                4, 5, 0,
            ];
            Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_indices(Indices::U32(indices))
        }
        1 => {
            let positions: Vec<[f32; 3]> = vec![
                [-scale * 0.65, 0.0, -scale * 0.55],
                [scale * 0.70, 0.0, -scale * 0.35],
                [scale * 0.20, 0.0, scale * 0.75],
                [-scale * 0.40, 0.0, scale * 0.35],
                [0.0, scale * 0.18, 0.0],
            ];
            let normals: Vec<[f32; 3]> = vec![
                [-0.65, 0.55, -0.50],
                [0.70, 0.55, -0.35],
                [0.20, 0.80, 0.55],
                [-0.50, 0.70, 0.35],
                [0.0, 1.0, 0.0],
            ];
            let indices: Vec<u32> = vec![
                4, 0, 1,
                4, 1, 2,
                4, 2, 3,
                4, 3, 0,
                0, 2, 1,
                0, 3, 2,
            ];
            Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_indices(Indices::U32(indices))
        }
        _ => {
            let positions: Vec<[f32; 3]> = vec![
                [-scale * 0.45, scale * 0.22, -scale * 0.35],
                [scale * 0.45, scale * 0.22, -scale * 0.35],
                [scale * 0.35, scale * 0.22, scale * 0.45],
                [-scale * 0.35, scale * 0.22, scale * 0.45],
                [-scale * 0.55, 0.0, -scale * 0.45],
                [scale * 0.55, 0.0, -scale * 0.45],
                [scale * 0.45, 0.0, scale * 0.55],
                [-scale * 0.45, 0.0, scale * 0.55],
            ];
            let normals: Vec<[f32; 3]> = vec![
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [-0.6, -0.5, -0.6],
                [0.6, -0.5, -0.6],
                [0.6, -0.5, 0.6],
                [-0.6, -0.5, 0.6],
            ];
            let indices: Vec<u32> = vec![
                0, 1, 2, 0, 2, 3,
                4, 6, 5, 4, 7, 6,
                0, 5, 1, 0, 4, 5,
                1, 6, 2, 1, 5, 6,
                2, 7, 3, 2, 6, 7,
                3, 4, 0, 3, 7, 4,
            ];
            Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                .with_inserted_indices(Indices::U32(indices))
        }
    }
}

pub fn update_active_effects(time: Res<Time>, mut receivers: Query<&mut EffectReceiverComponent>) {
    let dt = time.delta_secs();
    for mut receiver in receivers.iter_mut() {
        receiver.active_effects.retain_mut(|effect| {
            effect.elapsed += dt;
            effect.elapsed < effect.duration
        });
    }
}

pub fn update_thermal_geysers(
    time: Res<Time>,
    mut geysers: Query<(&mut ThermalGeyser, &Transform)>,
    mut receivers: Query<(&Transform, &mut EffectReceiverComponent), Without<Player>>,
    mut player_query: Query<(&Transform, &mut EffectReceiverComponent), With<Player>>,
    mut shake: Option<ResMut<CameraShake>>,
) {
    let dt = time.delta_secs();
    for (mut geyser, g_tf) in geysers.iter_mut() {
        geyser.eruption_timer += dt;
        if geyser.eruption_timer >= geyser.eruption_interval {
            geyser.eruption_timer = 0.0;
            geyser.is_erupting = !geyser.is_erupting;
        }

        if geyser.is_erupting {
            let g_pos = g_tf.translation;
            let radius_sq = geyser.heat_radius * geyser.heat_radius;
            for (r_tf, mut receiver) in receivers.iter_mut() {
                if g_pos.distance_squared(r_tf.translation) <= radius_sq {
                    receiver.apply_effect(AlchemicalEffectKind::ThermalHeat, 1.5, 25.0);
                    receiver.apply_effect(AlchemicalEffectKind::Vaporize, 1.0, 10.0);
                }
            }

            for (p_tf, mut p_recv) in player_query.iter_mut() {
                let dist = g_pos.distance(p_tf.translation);
                if dist <= geyser.heat_radius {
                    p_recv.apply_effect(AlchemicalEffectKind::ThermalHeat, 1.5, 25.0);
                    p_recv.apply_effect(AlchemicalEffectKind::Vaporize, 1.0, 10.0);
                }
                if dist <= 3.8 {
                    if let Some(ref mut s) = shake {
                        let intensity = ((3.8 - dist) / 3.8).clamp(0.15, 0.65);
                        s.add_trauma(intensity * 1.8 * dt, Vec3::new(0.0, 1.0, 0.0));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alchemy_plugin_build() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AlchemyPlugin);
        assert!(app.is_plugin_added::<AlchemyPlugin>());
    }

    #[test]
    fn test_effect_receiver_apply_and_query() {
        let mut receiver = EffectReceiverComponent::default();
        assert!(!receiver.has_effect(AlchemicalEffectKind::Frost));

        receiver.apply_effect(AlchemicalEffectKind::Frost, 3.0, 15.0);
        assert!(receiver.has_effect(AlchemicalEffectKind::Frost));
        assert_eq!(receiver.active_effects.len(), 1);

        receiver.apply_effect(AlchemicalEffectKind::Frost, 5.0, 25.0);
        assert_eq!(receiver.active_effects.len(), 1);
        assert_eq!(receiver.active_effects[0].duration, 5.0);
        assert_eq!(receiver.active_effects[0].magnitude, 25.0);
    }

    #[test]
    fn test_system_update_active_effects() {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(200));
        world.insert_resource(time);

        let entity = world
            .spawn(EffectReceiverComponent {
                active_effects: vec![ActiveEffect {
                    kind: AlchemicalEffectKind::Vitality,
                    duration: 0.1,
                    elapsed: 0.0,
                    magnitude: 20.0,
                }],
            })
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_active_effects);
        schedule.run(&mut world);

        let receiver = world.get::<EffectReceiverComponent>(entity).unwrap();
        assert!(
            !receiver.has_effect(AlchemicalEffectKind::Vitality),
            "Effect must have expired and been retained out"
        );
    }

    #[test]
    fn test_system_update_thermal_geysers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AlchemyPlugin);

        app.world_mut().spawn((
            ThermalGeyser {
                eruption_timer: 0.0,
                eruption_interval: 5.0,
                is_erupting: true,
                heat_radius: 6.0,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        let close_entity = app
            .world_mut()
            .spawn((
                EffectReceiverComponent::default(),
                Transform::from_xyz(2.0, 0.0, 2.0),
            ))
            .id();

        let far_entity = app
            .world_mut()
            .spawn((
                EffectReceiverComponent::default(),
                Transform::from_xyz(20.0, 0.0, 20.0),
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(16));
        app.insert_resource(time);

        app.update();

        let close_recv = app
            .world()
            .get::<EffectReceiverComponent>(close_entity)
            .unwrap();
        assert!(close_recv.has_effect(AlchemicalEffectKind::ThermalHeat));
        assert!(close_recv.has_effect(AlchemicalEffectKind::Vaporize));

        let far_recv = app
            .world()
            .get::<EffectReceiverComponent>(far_entity)
            .unwrap();
        assert!(!far_recv.has_effect(AlchemicalEffectKind::ThermalHeat));
    }

    #[test]
    fn test_potion_selected_effects_all_slots_unique_and_colored() {
        let (c1, e1) = selected_potion_effects(1);
        let (c2, e2) = selected_potion_effects(2);
        let (c3, e3) = selected_potion_effects(3);
        let (c4, e4) = selected_potion_effects(4);
        let (c5, e5) = selected_potion_effects(5);

        assert_eq!(e1[0].kind, AlchemicalEffectKind::Swiftness);
        assert_eq!(e2[0].kind, AlchemicalEffectKind::ThermalHeat);
        assert_eq!(e3[0].kind, AlchemicalEffectKind::Vitality);
        assert_eq!(e4[0].kind, AlchemicalEffectKind::Frost);
        assert_eq!(e5[0].kind, AlchemicalEffectKind::Luminescence);

        assert_ne!(c1, c2);
        assert_ne!(c2, c3);
        assert_ne!(c3, c4);
        assert_ne!(c4, c5);
    }

    #[test]
    fn test_aim_trajectory_calculation_and_impact_ballistic_curve() {
        let origin = Vec3::new(0.0, 1.5, 0.0);
        let vel = Vec3::new(0.0, 5.0, -10.0);
        let (pts, hit) = calculate_aim_trajectory_and_impact(origin, vel, 9.8, 25, 0.045, None);
        assert!(pts.len() >= 20);
        assert_eq!(pts[0], origin);
        assert!(pts[pts.len() - 1].z < origin.z);
        assert!(hit.is_none());

        let mut max_y = origin.y;
        for p in &pts {
            if p.y > max_y {
                max_y = p.y;
            }
        }
        assert!(max_y > origin.y, "Parabolic arc must reach an apex higher than origin");
        assert!(pts[pts.len() - 1].y < max_y, "Arc must descend after apex");
    }

    #[test]
    fn test_aim_trajectory_calculation_generates_arc() {
        let origin = Vec3::new(0.0, 1.5, 0.0);
        let vel = Vec3::new(0.0, 5.0, -10.0);
        let pts = calculate_aim_trajectory(origin, vel, 9.8, 20, 0.05, None);
        assert!(pts.len() >= 15);
        assert_eq!(pts[0], origin);
        assert!(pts[pts.len() - 1].z < origin.z);
    }

    #[test]
    fn test_potion_aiming_spawns_indicators_and_cleans_up() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.add_plugins(avian3d::prelude::PhysicsPlugins::default());
        app.init_resource::<Assets<StandardMaterial>>();
        app.init_resource::<Assets<Mesh>>();

        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Right);
        app.insert_resource(mouse);

        app.world_mut().spawn((
            Player,
            PlayerState::Active,
        ));

        app.world_mut().spawn((
            PlayerCamera,
            Transform::from_xyz(0.0, 1.75, 0.0),
            GlobalTransform::from_translation(Vec3::new(0.0, 1.75, 0.0)),
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_aiming);
        schedule.run(app.world_mut());

        let mut indicator_query = app.world_mut().query::<&TrajectoryAimIndicator>();
        let count = indicator_query.iter(app.world()).count();
        assert!(count >= 15, "Aiming must spawn trajectory arc indicators");

        let mouse_released = ButtonInput::<MouseButton>::default();
        app.insert_resource(mouse_released);
        schedule.run(app.world_mut());

        let mut indicator_query_after = app.world_mut().query::<&TrajectoryAimIndicator>();
        let count_after = indicator_query_after.iter(app.world()).count();
        assert_eq!(count_after, 0, "Releasing aim must despawn all indicators");
    }

    #[test]
    fn test_potion_projectile_flight_lifecycle_no_premature_despawn() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.add_plugins(avian3d::prelude::PhysicsPlugins::default());
        app.init_resource::<Assets<StandardMaterial>>();
        app.init_resource::<Assets<Mesh>>();

        let initial_pos = Vec3::new(0.0, 5.0, 0.0);
        let initial_vel = Vec3::new(0.0, 2.0, -12.0);

        let proj_entity = app
            .world_mut()
            .spawn((
                Transform::from_translation(initial_pos),
                PotionProjectile {
                    velocity: initial_vel,
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::Swiftness,
                        duration: 5.0,
                        elapsed: 0.0,
                        magnitude: 20.0,
                    }],
                    lifetime: 8.0,
                    color: Color::srgb(0.12, 0.92, 0.42),
                    slot: 1,
                    hitstop_timer: 0.0,
                    entered_water: false,
                },
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_projectiles);

        for _ in 0..10 {
            schedule.run(app.world_mut());
        }

        let proj_opt = app.world().get::<PotionProjectile>(proj_entity);
        assert!(
            proj_opt.is_some(),
            "Projectile must remain active during in-flight propagation across multiple frames"
        );

        let tf = app.world().get::<Transform>(proj_entity).unwrap();
        assert!(
            tf.translation.z < -4.0,
            "Projectile must travel forward along Z axis"
        );
        let proj = proj_opt.unwrap();
        assert!(
            proj.lifetime < 7.6,
            "Projectile lifetime must decrease during flight"
        );
        assert!(
            proj.velocity.y < initial_vel.y,
            "Vertical velocity must decrease due to gravity"
        );
    }

    #[test]
    fn test_potion_projectile_impact_and_splash_lifecycle() {
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
        app.init_resource::<Assets<crate::render::surface_fluid_material::SurfaceFluidMaterial>>();
        app.init_resource::<surface_fluid::PotionFluidProfiles>();
        app.add_systems(Update, update_potion_projectiles);

        let target_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, -3.0),
                GlobalTransform::from_translation(Vec3::new(0.0, 0.0, -3.0)),
                RigidBody::Static,
                Collider::cuboid(4.0, 4.0, 0.5),
                EffectReceiverComponent::default(),
            ))
            .id();

        app.update();

        let proj_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, -1.0),
                PotionProjectile {
                    velocity: Vec3::new(0.0, 0.0, -50.0),
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::ThermalHeat,
                        duration: 6.0,
                        elapsed: 0.0,
                        magnitude: 35.0,
                    }],
                    lifetime: 8.0,
                    color: Color::srgb(0.95, 0.12, 0.16),
                    slot: 2,
                    hitstop_timer: 0.0,
                    entered_water: false,
                },
            ))
            .id();

        app.update();
        if app.world().get_entity(proj_entity).is_ok() {
            app.update();
        }

        assert!(
            app.world().get_entity(proj_entity).is_err(),
            "Projectile must despawn upon impacting the target obstacle"
        );

        let target_recv = app.world().get::<EffectReceiverComponent>(target_entity).unwrap();
        assert!(
            target_recv.has_effect(AlchemicalEffectKind::ThermalHeat),
            "Receiver at impact location must receive the potion alchemical effect"
        );

        let mut puddle_query = app.world_mut().query::<&PotionPuddle>();
        let puddle_count = puddle_query.iter(app.world()).count();
        assert_eq!(puddle_count, 0, "Wall impacts must not create a disconnected alchemical puddle");

        let mut film_query = app.world_mut().query::<&surface_fluid::SurfaceFluidPatch>();
        assert_eq!(film_query.iter(app.world()).count(), 1, "Wall impacts must leave one attached surface film");

        let mut shards_query = app.world_mut().query::<&GlassShardParticle>();
        let shards_count = shards_query.iter(app.world()).count();
        assert_eq!(shards_count, 16, "Exactly 16 glass shard particles must be spawned on shatter");
    }

    #[test]
    fn test_sprint_throw_inaccuracy_application() {
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let still_dir = apply_throw_inaccuracy(forward, false, 1.23);
        assert_eq!(still_dir, forward);

        let running_dir = apply_throw_inaccuracy(forward, true, 4.56);
        assert_ne!(running_dir, forward);
        let dot = running_dir.dot(forward);
        assert!(dot >= 0.75, "Direction must be within 20% variance window");
    }

    #[test]
    fn test_potion_puddle_lifetime_greater_than_60s_and_gradual_fade() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();

        let color = Color::srgb(0.95, 0.12, 0.16);
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let mat_handle = materials.add(StandardMaterial {
            base_color: color.with_alpha(0.88),
            ..default()
        });

        let puddle_entity = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color,
                    elapsed: 0.0,
                    max_lifetime: 75.0,
                    radius: 1.0,
                    effects: Vec::new(),
                    is_wall: false,
                },
                MeshMaterial3d(mat_handle.clone()),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        assert!(app.world().get::<PotionPuddle>(puddle_entity).unwrap().max_lifetime > 60.0);

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs(30));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_puddles);
        schedule.run(app.world_mut());

        let mat_30s = app.world().resource::<Assets<StandardMaterial>>().get(&mat_handle).unwrap();
        assert_eq!(mat_30s.base_color.to_srgba().alpha, 0.88);

        let mut time2 = Time::<()>::default();
        time2.advance_by(std::time::Duration::from_secs(32));
        app.insert_resource(time2);
        schedule.run(app.world_mut());

        let mat_62s = app.world().resource::<Assets<StandardMaterial>>().get(&mat_handle).unwrap();
        assert!(mat_62s.base_color.to_srgba().alpha < 0.88);
        assert!(mat_62s.base_color.to_srgba().alpha > 0.0);

        let mut time3 = Time::<()>::default();
        time3.advance_by(std::time::Duration::from_secs(20));
        app.insert_resource(time3);
        schedule.run(app.world_mut());

        assert!(app.world().get_entity(puddle_entity).is_err());
    }

    #[test]
    fn test_create_potion_flask_mesh_has_all_slots() {
        for slot in 1..=6 {
            let mesh = create_potion_flask_mesh(slot);
            assert!(mesh.count_vertices() > 50);
        }
    }

    #[test]
    fn test_create_flat_circle_and_irregular_puddle_mesh() {
        let circle = create_flat_circle_mesh(0.38, 24);
        assert!(circle.count_vertices() >= 25);

        let puddle = create_irregular_puddle_mesh(0.72, 24);
        assert!(puddle.count_vertices() >= 25);
    }

    #[test]
    fn test_update_glass_shards_bounce_and_fade() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();

        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let mat_handle = materials.add(StandardMaterial::default());

        let shard_entity = app
            .world_mut()
            .spawn((
                GlassShardParticle {
                    velocity: Vec3::new(0.3, -8.0, 0.0),
                    lifetime: 0.0,
                    max_lifetime: 65.0,
                    ground_y: 0.0,
                    settled: false,
                    color: Color::srgb(0.85, 0.95, 1.0),
                    angular_velocity: Vec3::ZERO,
                    bounces_remaining: 0,
                },
                MeshMaterial3d(mat_handle.clone()),
                Transform::from_xyz(0.0, -0.01, 0.0),
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_glass_shards);
        schedule.run(app.world_mut());

        let shard = app.world().get::<GlassShardParticle>(shard_entity).unwrap();
        assert_eq!(shard.velocity.y, 0.0);
        assert!(shard.settled);

        let mut time2 = Time::<()>::default();
        time2.advance_by(std::time::Duration::from_secs(30));
        app.insert_resource(time2);
        schedule.run(app.world_mut());
        assert!(app.world().get_entity(shard_entity).is_ok());
    }

    #[test]
    fn test_potion_shards_persist_on_ground_longer_than_50s_and_puddle_spreads() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();

        let color = Color::srgb(0.2, 0.8, 0.9);
        let puddle_mat = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());

        let puddle_entity = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color,
                    elapsed: 0.0,
                    max_lifetime: 75.0,
                    radius: 1.0,
                    effects: Vec::new(),
                    is_wall: false,
                },
                MeshMaterial3d(puddle_mat),
                Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(0.15, 1.0, 0.15)),
            ))
            .id();

        let mut shard_materials = Vec::new();
        {
            let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
            for _ in 0..16 {
                shard_materials.push(materials.add(StandardMaterial::default()));
            }
        }

        let mut shard_entities = Vec::new();
        for (i, shard_mat) in shard_materials.into_iter().enumerate() {
            let entity = app
                .world_mut()
                .spawn((
                    GlassShardParticle {
                        velocity: Vec3::ZERO,
                        lifetime: 0.0,
                        max_lifetime: 60.0 + (i as f32),
                        ground_y: 0.0,
                        settled: true,
                        color: Color::srgb(0.85, 0.95, 1.0),
                        angular_velocity: Vec3::ZERO,
                        bounces_remaining: 0,
                    },
                    MeshMaterial3d(shard_mat),
                    Transform::from_xyz(0.1 * i as f32, 0.0, 0.1 * i as f32),
                ))
                .id();
            shard_entities.push(entity);
        }

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(400));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems((update_potion_puddles, update_glass_shards));
        schedule.run(app.world_mut());

        let tf = app.world().get::<Transform>(puddle_entity).unwrap();
        assert_eq!(tf.scale.x, 1.0);
        assert_eq!(tf.scale.z, 1.0);

        let mut time2 = Time::<()>::default();
        time2.advance_by(std::time::Duration::from_secs(50));
        app.insert_resource(time2);
        schedule.run(app.world_mut());

        for shard_id in shard_entities {
            assert!(app.world().get_entity(shard_id).is_ok());
            let shard = app.world().get::<GlassShardParticle>(shard_id).unwrap();
            assert!(shard.settled);
            assert_eq!(shard.color, Color::srgb(0.85, 0.95, 1.0));
        }
    }

    #[test]
    fn test_volumetric_puddle_mesh_skirt_and_normals() {
        let puddle = create_irregular_puddle_mesh(1.10, 32);
        assert!(puddle.count_vertices() >= 97);
        let positions = puddle.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        if let bevy::render::mesh::VertexAttributeValues::Float32x3(pts) = positions {
            assert!(pts[0][1] > 0.02);
            let has_skirt = pts.iter().any(|p| p[1] < -0.05);
            assert!(has_skirt);
        }
    }

    #[test]
    fn test_glass_shard_archetypes() {
        for shard_type in 0..3 {
            let shard = create_glass_shard_mesh(0.12, shard_type);
            assert!(shard.count_vertices() >= 5);
        }
    }

    #[test]
    fn test_mix_potion_colors_and_effects_reactions() {
        let red = Color::srgb(0.95, 0.12, 0.16);
        let blue = Color::srgb(0.12, 0.42, 0.95);
        let purple = mix_potion_colors(red, blue);
        let s = purple.to_srgba();
        assert!(s.red > 0.4 && s.blue > 0.4);

        let mut effects = vec![ActiveEffect {
            kind: AlchemicalEffectKind::ThermalHeat,
            duration: 6.0,
            elapsed: 0.0,
            magnitude: 30.0,
        }];
        let incoming = vec![ActiveEffect {
            kind: AlchemicalEffectKind::Frost,
            duration: 5.0,
            elapsed: 0.0,
            magnitude: 25.0,
        }];
        merge_potion_effects(&mut effects, &incoming);
        assert!(effects.iter().any(|e| e.kind == AlchemicalEffectKind::Vaporize));
        assert!(effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat));
        assert!(effects.iter().any(|e| e.kind == AlchemicalEffectKind::Frost));
    }

    #[test]
    fn test_create_wall_splatter_mesh_downward_drips_and_wrap() {
        let wall_mesh = create_wall_splatter_mesh(1.10, 32);
        assert!(wall_mesh.count_vertices() >= 97);
        let positions = wall_mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        if let bevy::render::mesh::VertexAttributeValues::Float32x3(pts) = positions {
            let has_downward_drip = pts.iter().any(|p| p[2] < -1.1);
            assert!(has_downward_drip);
            let has_conformal_wrap = pts.iter().any(|p| p[1] < 0.0);
            assert!(has_conformal_wrap);
        }
    }

    #[test]
    fn test_touching_puddles_mix_only_at_contact_without_increasing_size() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();
        app.init_resource::<Assets<Mesh>>();

        let red = Color::srgb(0.95, 0.12, 0.16);
        let blue = Color::srgb(0.12, 0.42, 0.95);

        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let mat1 = materials.add(StandardMaterial::default());
        let mat2 = materials.add(StandardMaterial::default());

        let p1 = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color: red,
                    elapsed: 5.0,
                    max_lifetime: 75.0,
                    radius: 1.15,
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::ThermalHeat,
                        duration: 6.0,
                        elapsed: 0.0,
                        magnitude: 25.0,
                    }],
                    is_wall: false,
                },
                MeshMaterial3d(mat1),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        let p2 = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color: blue,
                    elapsed: 1.0,
                    max_lifetime: 75.0,
                    radius: 1.15,
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::Frost,
                        duration: 5.0,
                        elapsed: 0.0,
                        magnitude: 20.0,
                    }],
                    is_wall: false,
                },
                MeshMaterial3d(mat2),
                Transform::from_xyz(0.5, 0.0, 0.0),
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_puddles);
        schedule.run(app.world_mut());

        assert!(app.world().get_entity(p1).is_ok());
        assert!(app.world().get_entity(p2).is_ok());

        let puddle1 = app.world().get::<PotionPuddle>(p1).unwrap();
        assert_eq!(puddle1.radius, 1.15);
        let puddle2 = app.world().get::<PotionPuddle>(p2).unwrap();
        assert_eq!(puddle2.radius, 1.15);

        let mut contact_query = app.world_mut().query::<&PotionContactZone>();
        let contacts: Vec<&PotionContactZone> = contact_query.iter(app.world()).collect();
        assert_eq!(contacts.len(), 1);
        assert!(contacts[0].effects.iter().any(|e| e.kind == AlchemicalEffectKind::Vaporize));
        assert!(contacts[0].radius <= 1.15);
    }

    #[test]
    fn test_non_touching_puddles_do_not_mix() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();
        app.init_resource::<Assets<Mesh>>();

        let red = Color::srgb(0.95, 0.12, 0.16);
        let blue = Color::srgb(0.12, 0.42, 0.95);

        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let mat1 = materials.add(StandardMaterial::default());
        let mat2 = materials.add(StandardMaterial::default());

        let p1 = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color: red,
                    elapsed: 5.0,
                    max_lifetime: 75.0,
                    radius: 1.15,
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::ThermalHeat,
                        duration: 6.0,
                        elapsed: 0.0,
                        magnitude: 25.0,
                    }],
                    is_wall: false,
                },
                MeshMaterial3d(mat1),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        let p2 = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color: blue,
                    elapsed: 1.0,
                    max_lifetime: 75.0,
                    radius: 1.15,
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::Frost,
                        duration: 5.0,
                        elapsed: 0.0,
                        magnitude: 20.0,
                    }],
                    is_wall: false,
                },
                MeshMaterial3d(mat2),
                Transform::from_xyz(4.0, 0.0, 0.0),
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_puddles);
        schedule.run(app.world_mut());

        assert!(app.world().get_entity(p1).is_ok());
        assert!(app.world().get_entity(p2).is_ok());

        let mut contact_query = app.world_mut().query::<&PotionContactZone>();
        let contacts: Vec<&PotionContactZone> = contact_query.iter(app.world()).collect();
        assert_eq!(contacts.len(), 0);
    }

    #[test]
    fn test_puddle_subtle_bioluminescence_and_non_mirrored_properties() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();

        let color = Color::srgb(0.2, 0.7, 0.9);
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let mat_handle = materials.add(StandardMaterial {
            base_color: color.with_alpha(0.88),
            emissive: color.to_linear() * 0.32,
            perceptual_roughness: 0.85,
            metallic: 0.0,
            reflectance: 0.12,
            ..default()
        });

        app.world_mut().spawn((
            PotionPuddle {
                color,
                elapsed: 2.0,
                max_lifetime: 75.0,
                radius: 1.15,
                effects: Vec::new(),
                is_wall: false,
            },
            MeshMaterial3d(mat_handle.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_puddles);
        schedule.run(app.world_mut());

        let mat = app.world().resource::<Assets<StandardMaterial>>().get(&mat_handle).unwrap();
        assert!(mat.perceptual_roughness >= 0.80);
        assert_eq!(mat.metallic, 0.0);
        assert!(mat.reflectance <= 0.20);
        let emissive_linear = color.to_linear() * 0.35;
        assert!(mat.emissive.red <= emissive_linear.red);
    }

    #[test]
    fn test_puddle_rtx_light_subtle_and_less_than_half_of_mushrooms() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_resource::<Assets<StandardMaterial>>();

        let color = Color::srgb(0.12, 0.85, 0.35);
        let mut materials = app.world_mut().resource_mut::<Assets<StandardMaterial>>();
        let mat_handle = materials.add(StandardMaterial::default());

        let puddle_entity = app
            .world_mut()
            .spawn((
                PotionPuddle {
                    color,
                    elapsed: 2.0,
                    max_lifetime: 75.0,
                    radius: 1.15,
                    effects: Vec::new(),
                    is_wall: false,
                },
                MeshMaterial3d(mat_handle),
                Transform::from_xyz(0.0, 0.0, 0.0),
                PointLight {
                    color,
                    intensity: 0.0,
                    range: 3.8,
                    shadows_enabled: false,
                    ..default()
                },
            ))
            .id();

        let mut settings = GameSettings::default();
        settings.rtx_enabled = true;
        app.insert_resource(settings);

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        app.insert_resource(time);

        let mut schedule = Schedule::default();
        schedule.add_systems(update_potion_puddles);
        schedule.run(app.world_mut());

        let light_on = app.world().get::<PointLight>(puddle_entity).unwrap();
        assert_eq!(light_on.color, color);
        assert_eq!(light_on.intensity, 2400.0);
        assert!(light_on.intensity < 13000.0);

        app.world_mut().resource_mut::<GameSettings>().rtx_enabled = false;
        schedule.run(app.world_mut());

        let light_off = app.world().get::<PointLight>(puddle_entity).unwrap();
        assert_eq!(light_off.intensity, 0.0);
    }

    #[test]
    fn test_potion_wall_impact_starts_a_connected_surface_film() {
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
        app.init_resource::<Assets<crate::render::surface_fluid_material::SurfaceFluidMaterial>>();
        app.init_resource::<surface_fluid::PotionFluidProfiles>();
        app.add_systems(Update, update_potion_projectiles);

        let _wall_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 3.0, -4.0),
                GlobalTransform::from_translation(Vec3::new(0.0, 3.0, -4.0)),
                RigidBody::Static,
                Collider::cuboid(10.0, 10.0, 0.5),
            ))
            .id();

        let _floor_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, -0.5, -4.0),
                GlobalTransform::from_translation(Vec3::new(0.0, -0.5, -4.0)),
                RigidBody::Static,
                Collider::cuboid(20.0, 1.0, 20.0),
            ))
            .id();

        app.update();

        let proj_entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 4.5, -3.2),
                GlobalTransform::from_translation(Vec3::new(0.0, 4.5, -3.2)),
                PotionProjectile {
                    velocity: Vec3::new(0.0, 0.0, -30.0),
                    effects: vec![ActiveEffect {
                        kind: AlchemicalEffectKind::Swiftness,
                        duration: 5.0,
                        elapsed: 0.0,
                        magnitude: 20.0,
                    }],
                    lifetime: 5.0,
                    color: Color::srgb(0.2, 0.9, 0.5),
                    slot: 1,
                    hitstop_timer: 0.0,
                    entered_water: false,
                },
            ))
            .id();

        app.update();
        if app.world().get_entity(proj_entity).is_ok() {
            app.update();
        }

        assert!(app.world().get_entity(proj_entity).is_err());

        let mut film_query = app.world_mut().query::<&surface_fluid::SurfaceFluidPatch>();
        let films: Vec<&surface_fluid::SurfaceFluidPatch> = film_query.iter(app.world()).collect();
        assert_eq!(films.len(), 1);
        assert!(films[0].normal.y.abs() < 0.65);
        assert!(films[0].volume > 0.0);

        let mut puddle_query = app.world_mut().query::<&PotionPuddle>();
        assert_eq!(puddle_query.iter(app.world()).count(), 0, "Wall impacts must not create a disconnected ground puddle");

    }

    #[test]
    fn test_secret_test_room_potion_boundary_containment() {
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
        app.init_resource::<Assets<crate::render::surface_fluid_material::SurfaceFluidMaterial>>();
        app.init_resource::<surface_fluid::PotionFluidProfiles>();
        app.add_systems(Update, update_potion_projectiles);
        app.update();

        let origin = crate::dungeon::test_room::SECRET_TEST_ROOM_ORIGIN;
        let proj_entity = app
            .world_mut()
            .spawn((
                Transform::from_translation(origin + Vec3::new(39.0, 5.0, 0.0)),
                GlobalTransform::from_translation(origin + Vec3::new(39.0, 5.0, 0.0)),
                PotionProjectile {
                    velocity: Vec3::new(40.0, 0.0, 0.0),
                    effects: vec![],
                    lifetime: 5.0,
                    color: Color::srgb(1.0, 0.0, 0.0),
                    slot: 1,
                    hitstop_timer: 0.0,
                    entered_water: false,
                },
            ))
            .id();

        app.update();
        let proj = app.world().get::<PotionProjectile>(proj_entity).unwrap();
        assert!(proj.hitstop_timer > 0.0);
        let impact_opt = app.world().get::<PotionSurfaceImpact>(proj_entity);
        assert!(impact_opt.is_some());
        let tf = app.world().get::<Transform>(proj_entity).unwrap();
        assert!(tf.translation.x <= origin.x + 40.0);
    }
}


