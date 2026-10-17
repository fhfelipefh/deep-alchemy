use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
};
use super::camera::{Player, PlayerCamera, PlayerState, Stamina};
use crate::ui::menu::{GameSettings, GameState};

pub fn player_look(
    game_state: Option<Res<State<GameState>>>,
    settings: Option<Res<GameSettings>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut player_query: Query<(&PlayerState, &mut Transform), (With<Player>, Without<PlayerCamera>)>,
    mut camera_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }

    let mut mouse_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        mouse_delta += event.delta;
    }

    if mouse_delta == Vec2::ZERO {
        return;
    }

    let sensitivity = settings.as_ref().map(|s| s.mouse_sensitivity).unwrap_or(0.0022);

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
    game_state: Option<Res<State<GameState>>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&PlayerState, &mut Transform, &mut Stamina), With<Player>>,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_movement_forward_and_stamina() {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(100));
        world.insert_resource(time);

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        world.insert_resource(keys);

        let player = world.spawn((
            Player,
            PlayerState::Active,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Stamina { current: 100.0, max: 100.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(player_movement);
        schedule.run(&mut world);

        let tf = world.get::<Transform>(player).unwrap();
        // Since forward in Bevy is -Z, walking forward should decrease Z
        assert!(tf.translation.z < 0.0, "Player must have moved forward");

        // Now test sprinting with Shift
        world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::ShiftLeft);
        world.resource_mut::<Time>().advance_by(std::time::Duration::from_secs(1));
        schedule.run(&mut world);

        let stamina = world.get::<Stamina>(player).unwrap();
        assert!(stamina.current < 100.0, "Sprinting must consume stamina");
    }

    #[test]
    fn test_player_look_rotates_player_and_camera() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<MouseMotion>();
        app.add_systems(Update, player_look);

        let player = app.world_mut().spawn((
            Player,
            PlayerState::Active,
            Transform::IDENTITY,
        )).id();

        let camera = app.world_mut().spawn((
            PlayerCamera,
            Transform::IDENTITY,
        )).id();

        // Send mouse motion event
        app.world_mut().send_event(MouseMotion {
            delta: Vec2::new(100.0, 50.0),
        });

        app.update();

        let player_tf = app.world().get::<Transform>(player).unwrap();
        let cam_tf = app.world().get::<Transform>(camera).unwrap();

        assert_ne!(player_tf.rotation, Quat::IDENTITY, "Player yaw must have changed");
        assert_ne!(cam_tf.rotation, Quat::IDENTITY, "Camera pitch must have changed");
    }
}
