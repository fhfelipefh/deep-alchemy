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
        app.add_systems(Update, (update_active_effects, update_thermal_geysers));
    }
}

fn update_active_effects(
    time: Res<Time>,
    mut receivers: Query<&mut EffectReceiverComponent>,
) {
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

        let entity = world.spawn(EffectReceiverComponent {
            active_effects: vec![
                ActiveEffect {
                    kind: AlchemicalEffectKind::Vitality,
                    duration: 0.1,
                    elapsed: 0.0,
                    magnitude: 20.0,
                },
            ],
        }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_active_effects);
        schedule.run(&mut world);

        let receiver = world.get::<EffectReceiverComponent>(entity).unwrap();
        assert!(!receiver.has_effect(AlchemicalEffectKind::Vitality), "Effect must have expired and been retained out");
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
        let close_entity = app.world_mut().spawn((
            EffectReceiverComponent::default(),
            Transform::from_xyz(2.0, 0.0, 2.0),
        )).id();

        // Spawn far receiver
        let far_entity = app.world_mut().spawn((
            EffectReceiverComponent::default(),
            Transform::from_xyz(20.0, 0.0, 20.0),
        )).id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(16));
        app.insert_resource(time);

        app.update();

        let close_recv = app.world().get::<EffectReceiverComponent>(close_entity).unwrap();
        assert!(close_recv.has_effect(AlchemicalEffectKind::ThermalHeat));
        assert!(close_recv.has_effect(AlchemicalEffectKind::Vaporize));

        let far_recv = app.world().get::<EffectReceiverComponent>(far_entity).unwrap();
        assert!(!far_recv.has_effect(AlchemicalEffectKind::ThermalHeat));
    }
}
