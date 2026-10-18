use crate::{
    player::camera::{PlayerCamera, PlayerState},
    ui::{hud::ActiveHotbarSlot, menu::GameState},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
}

#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct PotionFlask {
    pub id: String,
    pub effects: Vec<ActiveEffect>,
    pub liquid_color: Color,
    pub volume: f32, // 1.0 = full, 0.0 = empty
}

#[derive(Component, Debug)]
pub struct ThermalGeyser {
    pub eruption_timer: f32,
    pub eruption_interval: f32,
    pub is_erupting: bool,
    pub heat_radius: f32,
}

pub struct AlchemyPlugin;

impl Plugin for AlchemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_active_effects,
                update_thermal_geysers,
                spawn_potion_projectile.run_if(resource_exists::<ButtonInput<MouseButton>>),
                update_potion_projectiles.run_if(resource_exists::<SpatialQueryPipeline>),
            ),
        );
    }
}

fn selected_potion_effects(slot: usize) -> (Color, Vec<ActiveEffect>) {
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

fn spawn_potion_projectile(
    game_state: Option<Res<State<GameState>>>,
    mouse: Res<ButtonInput<MouseButton>>,
    slot: Option<Res<ActiveHotbarSlot>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    player: Query<&PlayerState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }
    if !mouse.just_pressed(MouseButton::Left) || player.iter().any(|s| *s != PlayerState::Active) {
        return;
    }
    let Ok(camera) = camera.get_single() else {
        return;
    };
    let slot_index = slot.as_deref().map(|s| s.0).unwrap_or(1);
    let (color, effects) = selected_potion_effects(slot_index);
    let direction = *camera.forward();
    let origin = camera.translation() + direction * 0.55;
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.10).mesh().uv(12, 8))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            emissive: color.to_linear(),
            perceptual_roughness: 0.18,
            ..default()
        })),
        Transform::from_translation(origin),
        PotionProjectile {
            velocity: direction * 15.0,
            effects,
            lifetime: 4.0,
        },
    ));
}

fn update_potion_projectiles(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut projectiles: Query<(Entity, &mut Transform, &mut PotionProjectile)>,
    mut receivers: Query<&mut EffectReceiverComponent>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut projectile) in &mut projectiles {
        let distance = projectile.velocity.length() * dt;
        let direction = projectile.velocity.normalize_or_zero();
        let hit = spatial_query.cast_ray(
            transform.translation,
            Dir3::new(direction).unwrap_or(Dir3::NEG_Z),
            distance,
            true,
            &SpatialQueryFilter::default(),
        );
        if let Some(hit) = hit {
            if let Ok(mut receiver) = receivers.get_mut(hit.entity) {
                for effect in &projectile.effects {
                    receiver.apply_effect(effect.kind, effect.duration, effect.magnitude);
                }
            }
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += projectile.velocity * dt;
        projectile.lifetime -= dt;
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn update_active_effects(time: Res<Time>, mut receivers: Query<&mut EffectReceiverComponent>) {
    let dt = time.delta_secs();
    for mut receiver in receivers.iter_mut() {
        receiver.active_effects.retain_mut(|effect| {
            effect.elapsed += dt;
            effect.elapsed < effect.duration
        });
    }
}

fn update_thermal_geysers(
    time: Res<Time>,
    mut geysers: Query<(&mut ThermalGeyser, &Transform)>,
    mut receivers: Query<(&Transform, &mut EffectReceiverComponent)>,
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

        // Refresh with longer duration
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

        // Spawn geyser that is erupting
        app.world_mut().spawn((
            ThermalGeyser {
                eruption_timer: 0.0,
                eruption_interval: 5.0,
                is_erupting: true,
                heat_radius: 6.0,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Spawn close receiver
        let close_entity = app
            .world_mut()
            .spawn((
                EffectReceiverComponent::default(),
                Transform::from_xyz(2.0, 0.0, 2.0),
            ))
            .id();

        // Spawn far receiver
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
}
