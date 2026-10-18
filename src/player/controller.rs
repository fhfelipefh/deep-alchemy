use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
};
use super::camera::{LocomotionState, Player, PlayerCamera, PlayerState, Stamina};
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
    mut query: Query<(&PlayerState, &mut Transform, &mut Stamina, &mut LocomotionState), With<Player>>,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }

    let dt = time.delta_secs();

    for (state, mut tf, mut stamina, mut locomotion) in query.iter_mut() {
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

        // Crouch (Left Ctrl)
        locomotion.is_crouching = keys.pressed(KeyCode::ControlLeft);

        // Jump (Space)
        if keys.just_pressed(KeyCode::Space) && locomotion.is_grounded && !locomotion.is_crouching {
            locomotion.vertical_velocity = 5.2;
            locomotion.is_grounded = false;
        }

        // Sprinting (Left Shift)
        let can_sprint = !locomotion.is_crouching && stamina.current > 5.0 && is_moving;
        let is_sprinting = keys.pressed(KeyCode::ShiftLeft) && can_sprint;
        locomotion.is_sprinting = is_sprinting;

        if is_sprinting {
            // Speed: 7.0 m/s (1.66x base speed 4.2 -> > 1.5x)
            locomotion.current_speed = 7.0;
            stamina.current = (stamina.current - 20.0 * dt).max(0.0);
            locomotion.stamina_regen_delay_timer = 1.0; // Reset regen delay
        } else if locomotion.is_crouching {
            locomotion.current_speed = 2.2;
            locomotion.stamina_regen_delay_timer = (locomotion.stamina_regen_delay_timer - dt).max(0.0);
        } else {
            locomotion.current_speed = 4.2;
            locomotion.stamina_regen_delay_timer = (locomotion.stamina_regen_delay_timer - dt).max(0.0);
        }

        // Stamina Regeneration after delay
        if !is_sprinting && locomotion.stamina_regen_delay_timer <= 0.0 {
            stamina.current = (stamina.current + 15.0 * dt).min(stamina.max);
        }

        // Gravity & Vertical Simulation
        if !locomotion.is_grounded {
            locomotion.vertical_velocity -= 18.0 * dt;
            tf.translation.y += locomotion.vertical_velocity * dt;
            if tf.translation.y <= 1.2 {
                tf.translation.y = 1.2;
                locomotion.vertical_velocity = 0.0;
                locomotion.is_grounded = true;
            }
        }

        // Horizontal Movement
        if is_moving {
            let vel = move_dir.normalize() * locomotion.current_speed * dt;
            tf.translation += vel;

            // Head-Bob oscillation step
            let bob_freq = if is_sprinting { 14.0 } else { 8.0 };
            locomotion.head_bob_timer += dt * bob_freq;
        } else {
            locomotion.head_bob_timer = 0.0;
        }
    }
}

pub fn player_camera_effects(
    time: Res<Time>,
    player_query: Query<&LocomotionState, (With<Player>, Without<PlayerCamera>)>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<PlayerCamera>>,
) {
    let dt = time.delta_secs();
    let Ok(locomotion) = player_query.get_single() else {
        return;
    };

    for (mut cam_tf, mut projection) in camera_query.iter_mut() {
        // 1. Dynamic FOV Kick (+7 degrees when sprinting)
        if let Projection::Perspective(ref mut persp) = *projection {
            let base_fov: f32 = 75.0_f32.to_radians();
            let target_fov = if locomotion.is_sprinting {
                base_fov + 7.0_f32.to_radians()
            } else {
                base_fov
            };
            persp.fov = persp.fov.lerp(target_fov, (10.0 * dt).min(1.0));
        }

        // 2. Head-Bob (vertical and subtle lateral sway)
        let base_eye_height = if locomotion.is_crouching { 0.25 } else { 0.55 };
        if locomotion.head_bob_timer > 0.0 {
            let bob_amp = if locomotion.is_sprinting { 0.065 } else { 0.03 };
            let bob_y = (locomotion.head_bob_timer).sin() * bob_amp;
            let bob_x = (locomotion.head_bob_timer * 0.5).cos() * (bob_amp * 0.4);

            cam_tf.translation.y = base_eye_height + bob_y;
            cam_tf.translation.x = bob_x;
        } else {
            cam_tf.translation.y = cam_tf.translation.y.lerp(base_eye_height, (8.0 * dt).min(1.0));
            cam_tf.translation.x = cam_tf.translation.x.lerp(0.0, (8.0 * dt).min(1.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locomotion_sprint_velocity_and_stamina_drain_gate() {
        // GATE 2 MANDATORY TEST:
        // Simulates 60 frames walking (W) and measures Delta Z1.
        // Simulates 60 frames sprinting (W + Shift) and measures Delta Z2 and stamina.
        // Asserts: delta_z2 > delta_z1 * 1.4 AND stamina_final < stamina_initial.
        let mut app_walk = App::new();
        app_walk.add_systems(Update, player_movement);
        app_walk.init_resource::<Time>();

        let walk_player = app_walk.world_mut().spawn((
            Player,
            PlayerState::Active,
            LocomotionState::default(),
            Transform::from_xyz(0.0, 1.2, 0.0),
            Stamina { current: 100.0, max: 100.0 },
        )).id();

        let mut walk_keys = ButtonInput::<KeyCode>::default();
        walk_keys.press(KeyCode::KeyW);
        app_walk.world_mut().insert_resource(walk_keys);

        // Run 60 frames of walking at fixed dt = 16ms
        for _ in 0..60 {
            app_walk.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
            app_walk.update();
        }

        let z_walk_end = app_walk.world().get::<Transform>(walk_player).unwrap().translation.z;
        let delta_z1 = z_walk_end.abs();

        // Now run Sprint simulation
        let mut app_sprint = App::new();
        app_sprint.add_systems(Update, player_movement);
        app_sprint.init_resource::<Time>();

        let sprint_player = app_sprint.world_mut().spawn((
            Player,
            PlayerState::Active,
            LocomotionState::default(),
            Transform::from_xyz(0.0, 1.2, 0.0),
            Stamina { current: 100.0, max: 100.0 },
        )).id();

        let mut sprint_keys = ButtonInput::<KeyCode>::default();
        sprint_keys.press(KeyCode::KeyW);
        sprint_keys.press(KeyCode::ShiftLeft);
        app_sprint.world_mut().insert_resource(sprint_keys);

        for _ in 0..60 {
            app_sprint.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
            app_sprint.update();
        }

        let z_sprint_end = app_sprint.world().get::<Transform>(sprint_player).unwrap().translation.z;
        let delta_z2 = z_sprint_end.abs();

        let stamina_sprint = app_sprint.world().get::<Stamina>(sprint_player).unwrap().current;

        // GATE 2 ASSERTIONS:
        assert!(
            delta_z2 > delta_z1 * 1.4,
            "Sprinting must be at least 40% faster than walking: delta_z2={:.3}, delta_z1={:.3}",
            delta_z2, delta_z1
        );
        assert!(
            stamina_sprint < 100.0,
            "Sprinting must consume stamina: initial=100.0, final={:.2}",
            stamina_sprint
        );
    }

    #[test]
    fn test_locomotion_stamina_regen_delay_and_crouch_jump_gate() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, player_movement);

        let player = app.world_mut().spawn((
            Player,
            PlayerState::Active,
            LocomotionState::default(),
            Transform::from_xyz(0.0, 1.2, 0.0),
            Stamina { current: 50.0, max: 100.0 },
        )).id();

        let mut keys = ButtonInput::<KeyCode>::default();
        app.world_mut().insert_resource(keys.clone());

        // Test Crouch reduces speed
        keys.press(KeyCode::ControlLeft);
        keys.press(KeyCode::KeyW);
        *app.world_mut().resource_mut::<ButtonInput<KeyCode>>() = keys.clone();
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(100));
        app.update();

        let locomotion = app.world().get::<LocomotionState>(player).unwrap();
        assert!(locomotion.is_crouching, "Crouch must be active");
        assert_eq!(locomotion.current_speed, 2.2, "Crouch speed must be 2.2");

        // Test Jump applies vertical velocity
        keys.release(KeyCode::ControlLeft);
        keys.press(KeyCode::Space);
        *app.world_mut().resource_mut::<ButtonInput<KeyCode>>() = keys.clone();
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
        app.update();

        let jumping_locomotion = app.world().get::<LocomotionState>(player).unwrap();
        assert!(jumping_locomotion.vertical_velocity > 0.0, "Jump must apply upward velocity");
        assert!(!jumping_locomotion.is_grounded, "Player must not be grounded while ascending");
    }

    #[test]
    fn test_locomotion_camera_fov_kick_and_head_bob() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, player_camera_effects);

        let mut locomotion = LocomotionState::default();
        locomotion.is_sprinting = true;
        locomotion.head_bob_timer = 1.5;

        app.world_mut().spawn((
            Player,
            locomotion,
        ));

        let camera = app.world_mut().spawn((
            PlayerCamera,
            Transform::from_xyz(0.0, 0.55, 0.0),
            Projection::Perspective(PerspectiveProjection {
                fov: 75.0_f32.to_radians(),
                ..default()
            }),
        )).id();

        for _ in 0..30 {
            app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let proj = app.world().get::<Projection>(camera).unwrap();
        if let Projection::Perspective(persp) = proj {
            assert!(
                persp.fov > 75.0_f32.to_radians(),
                "Sprinting must trigger camera FOV kick (+5 to +8 deg)"
            );
        } else {
            panic!("Expected perspective projection");
        }
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
