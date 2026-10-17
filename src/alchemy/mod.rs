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
