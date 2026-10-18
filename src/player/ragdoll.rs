use super::camera::{HeadHatPart, Health, Player, PlayerBodyScene, PlayerCamera, PlayerState};
use avian3d::prelude::*;
use bevy::{prelude::*, render::view::RenderLayers};

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
    mut body_scenes: Query<(Entity, &mut Visibility), With<PlayerBodyScene>>,
) {
    for (player_entity, mut state, health, tf) in player_query.iter_mut() {
        if *state == PlayerState::Active && health.current <= 0.0 {
            *state = PlayerState::Dead;
            info!("Player has died! Activating Ragdoll & 3rd-Person Death Camera");

            for head_entity in head_parts.iter() {
                commands.entity(head_entity).insert(RenderLayers::layer(0));
            }
            for (body_entity, mut vis) in body_scenes.iter_mut() {
                *vis = Visibility::Inherited;
                commands.entity(body_entity).insert(RenderLayers::layer(0));
            }

            commands.entity(player_entity).insert((
                RigidBody::Dynamic,
                LinearVelocity(Vec3::new(0.0, 1.5, -2.0)),
                AngularVelocity(Vec3::new(1.0, 0.2, 0.5)),
            ));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_player_death_system_triggers_ragdoll() {
        let mut app = App::new();
        app.add_systems(Update, check_player_death);

        let head = app
            .world_mut()
            .spawn((HeadHatPart, RenderLayers::layer(1)))
            .id();

        let camera = app
            .world_mut()
            .spawn((PlayerCamera, Transform::IDENTITY))
            .id();

        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                Health {
                    current: 0.0,
                    max: 100.0,
                },
                Transform::from_xyz(0.0, 1.0, 0.0),
            ))
            .id();

        app.update();

        let state = app.world().get::<PlayerState>(player).unwrap();
        assert_eq!(*state, PlayerState::Dead);

        let head_layers = app.world().get::<RenderLayers>(head).unwrap();
        assert!(head_layers.intersects(&RenderLayers::layer(0)));

        assert!(app.world().get::<RigidBody>(player).is_some());
        let vel = app.world().get::<LinearVelocity>(player).unwrap();
        assert_eq!(vel.0, Vec3::new(0.0, 1.5, -2.0));

        assert!(app.world().get::<DeathCamera>(camera).is_some());
    }

    #[test]
    fn test_update_death_camera_system() {
        let mut app = App::new();
        app.add_systems(Update, update_death_camera);

        app.world_mut()
            .spawn((Player, Transform::from_xyz(0.0, 0.0, 0.0)));

        let camera = app
            .world_mut()
            .spawn((
                DeathCamera {
                    target_point: Vec3::ZERO,
                    orbit_angle: 0.0,
                    distance: 4.0,
                },
                Transform::IDENTITY,
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(100));
        app.insert_resource(time);

        app.update();

        let death_cam = app.world().get::<DeathCamera>(camera).unwrap();
        assert!(
            death_cam.orbit_angle > 0.0,
            "Orbit angle should have advanced"
        );
    }
}
