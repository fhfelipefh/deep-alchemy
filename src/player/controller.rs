use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
};
use super::camera::{Player, PlayerCamera, PlayerState, Stamina};

pub fn player_look(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut player_query: Query<(&PlayerState, &mut Transform), (With<Player>, Without<PlayerCamera>)>,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        mouse_delta += event.delta;
    }

    if mouse_delta == Vec2::ZERO {
        return;
    }

    let sensitivity = 0.0022;

    for (state, mut player_tf) in player_query.iter_mut() {
        if *state != PlayerState::Active {
            continue;
        }

        // Yaw rotates the player root body around Y axis
        player_tf.rotate_y(-mouse_delta.x * sensitivity);

        // Pitch rotates the camera entity around local X axis
        for mut cam_tf in camera_query.iter_mut() {
            let (yaw, mut pitch, roll) = cam_tf.rotation.to_euler(EulerRot::YXZ);
            pitch -= mouse_delta.y * sensitivity;
            pitch = pitch.clamp(-1.52, 1.52); // ~87 degrees up and down
            cam_tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        }
    }
}

pub fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&PlayerState, &mut Transform, &mut Stamina), With<Player>>,
) {
    let dt = time.delta_secs();

    for (state, mut tf, mut stamina) in query.iter_mut() {
        if *state != PlayerState::Active {
            continue;
        }

        let mut move_dir = Vec3::ZERO;
        let forward = tf.forward();
        let right = tf.right();

        if keys.pressed(KeyCode::KeyW) {
            move_dir += *forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            move_dir -= *forward;
        }
        if keys.pressed(KeyCode::KeyD) {
            move_dir += *right;
        }
        if keys.pressed(KeyCode::KeyA) {
            move_dir -= *right;
        }

        move_dir.y = 0.0;
        let is_moving = move_dir.length_squared() > 0.0001;

        // Sprinting
        let is_sprinting = keys.pressed(KeyCode::ShiftLeft) && stamina.current > 5.0 && is_moving;
        let speed = if is_sprinting {
            stamina.current = (stamina.current - 15.0 * dt).max(0.0);
            7.5
        } else {
            stamina.current = (stamina.current + 10.0 * dt).min(stamina.max);
            4.2
        };

        if is_moving {
            let vel = move_dir.normalize() * speed * dt;
            tf.translation += vel;
        }
    }
}
