use bevy::{
    prelude::*,
    render::view::RenderLayers,
};
use avian3d::prelude::*;
use super::camera::{HeadHatPart, Health, Player, PlayerCamera, PlayerState};

#[derive(Component)]
pub struct DeathCamera {
    pub target_point: Vec3,
    pub orbit_angle: f32,
    pub distance: f32,
}

pub fn check_player_death(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut PlayerState, &Health, &Transform), With<Player>>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    head_parts: Query<Entity, With<HeadHatPart>>,
) {
    for (player_entity, mut state, health, tf) in player_query.iter_mut() {
        if *state == PlayerState::Active && health.current <= 0.0 {
            *state = PlayerState::Dead;
            info!("Player has died! Activating Ragdoll & 3rd-Person Death Camera");

            // 1. Make Head & Fedora visible to main camera layer
            for head_entity in head_parts.iter() {
                commands.entity(head_entity).insert(RenderLayers::layer(0));
            }

            // 2. Enable Dynamic Ragdoll physics on Player
            commands.entity(player_entity).insert((
                RigidBody::Dynamic,
                LinearVelocity(Vec3::new(0.0, 1.5, -2.0)), // stumble backward impulse
                AngularVelocity(Vec3::new(1.0, 0.2, 0.5)),
            ));

            // 3. Detach Camera and activate 3rd-Person Orbit Death Camera
            if let Ok(cam_entity) = camera_query.get_single() {
                commands.entity(cam_entity).remove_parent();
                commands.entity(cam_entity).insert(DeathCamera {
                    target_point: tf.translation,
                    orbit_angle: 0.0,
                    distance: 3.8,
                });
            }
        }
    }
}

pub fn update_death_camera(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<DeathCamera>)>,
    mut cam_query: Query<(&mut Transform, &mut DeathCamera)>,
) {
    let dt = time.delta_secs();
    let Ok(player_tf) = player_query.get_single() else {
        return;
    };

    for (mut cam_tf, mut death_cam) in cam_query.iter_mut() {
        death_cam.target_point = death_cam.target_point.lerp(player_tf.translation, 0.1);
        death_cam.orbit_angle += 0.35 * dt;

        let height = 2.0;
        let x = death_cam.target_point.x + death_cam.distance * death_cam.orbit_angle.cos();
        let z = death_cam.target_point.z + death_cam.distance * death_cam.orbit_angle.sin();
        let y = death_cam.target_point.y + height;

        cam_tf.translation = Vec3::new(x, y, z);
        cam_tf.look_at(death_cam.target_point + Vec3::Y * 0.5, Vec3::Y);
    }
}
