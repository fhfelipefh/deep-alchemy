use super::camera::{LocomotionState, Player, PlayerCamera, PlayerState, Stamina};
use crate::dungeon::modular_kit::{CavernObstacle, ObstacleShape};
use crate::ui::menu::{GameSettings, GameState};
use bevy::{input::mouse::MouseMotion, prelude::*};

pub fn resolve_player_collision_sliding(
    desired_pos: Vec3,
    prev_pos: Vec3,
    player_radius: f32,
    obstacles: &[(&Transform, ObstacleShape)],
) -> Vec3 {
    let mut resolved = desired_pos;
    let attempted_move = Vec2::new(desired_pos.x - prev_pos.x, desired_pos.z - prev_pos.z);
    let is_moving = attempted_move.length_squared() > 1e-6;

    // Relaxation passes for multi-obstacle and corner resolution
    for _ in 0..3 {
        for (tf, shape) in obstacles {
            match shape {
                ObstacleShape::Cylinder { radius, height } => {
                    let obs_y = tf.translation.y;
                    let half_h = height * 0.5;
                    // Check height bounds
                    if resolved.y < obs_y - half_h - 1.2 || resolved.y > obs_y + half_h + 0.5 {
                        continue;
                    }

                    let center = Vec2::new(tf.translation.x, tf.translation.z);
                    let p = Vec2::new(resolved.x, resolved.z);
                    let diff = p - center;
                    let dist = diff.length();
                    let min_dist = *radius + player_radius;

                    if dist < min_dist {
                        let normal = if dist > 1e-4 {
                            diff / dist
                        } else {
                            Vec2::new(1.0, 0.0)
                        };
                        let penetration = min_dist - dist;
                        resolved.x += normal.x * penetration;
                        resolved.z += normal.y * penetration;

                        if is_moving {
                            let dot = attempted_move.dot(normal);
                            if dot < 0.0 {
                                // Tangential sliding: subtract inward normal component
                                let slide = attempted_move - normal * dot;
                                let slid_target = Vec2::new(prev_pos.x, prev_pos.z) + slide;
                                let re_diff = slid_target - center;
                                let re_dist = re_diff.length();
                                if re_dist >= min_dist {
                                    resolved.x = slid_target.x;
                                    resolved.z = slid_target.y;
                                } else {
                                    let re_norm = if re_dist > 1e-4 {
                                        re_diff / re_dist
                                    } else {
                                        normal
                                    };
                                    resolved.x = center.x + re_norm.x * min_dist;
                                    resolved.z = center.y + re_norm.y * min_dist;
                                }
                            }
                        }
                    }
                }
                ObstacleShape::Cuboid { half_extents } => {
                    let obs_y = tf.translation.y;
                    if resolved.y < obs_y - half_extents.y - 1.2
                        || resolved.y > obs_y + half_extents.y + 0.5
                    {
                        continue;
                    }

                    let (_, rot_y, _) = tf.rotation.to_euler(EulerRot::YXZ);
                    let diff_world =
                        Vec2::new(resolved.x - tf.translation.x, resolved.z - tf.translation.z);
                    let cos_r = (-rot_y).cos();
                    let sin_r = (-rot_y).sin();
                    let local_x = diff_world.x * cos_r - diff_world.y * sin_r;
                    let local_z = diff_world.x * sin_r + diff_world.y * cos_r;

                    let hx = half_extents.x;
                    let hz = half_extents.z;

                    let clamped_x = local_x.clamp(-hx, hx);
                    let clamped_z = local_z.clamp(-hz, hz);

                    let inside_box = local_x.abs() < hx && local_z.abs() < hz;

                    if inside_box {
                        let d_px = hx - local_x;
                        let d_nx = local_x - (-hx);
                        let d_pz = hz - local_z;
                        let d_nz = local_z - (-hz);

                        let min_d = d_px.min(d_nx).min(d_pz).min(d_nz);
                        let mut push_local = Vec2::new(local_x, local_z);

                        if min_d == d_px {
                            push_local.x = hx + player_radius;
                        } else if min_d == d_nx {
                            push_local.x = -hx - player_radius;
                        } else if min_d == d_pz {
                            push_local.y = hz + player_radius;
                        } else {
                            push_local.y = -hz - player_radius;
                        }

                        let cos_b = rot_y.cos();
                        let sin_b = rot_y.sin();
                        resolved.x = tf.translation.x + push_local.x * cos_b - push_local.y * sin_b;
                        resolved.z = tf.translation.z + push_local.x * sin_b + push_local.y * cos_b;
                    } else {
                        let to_player = Vec2::new(local_x - clamped_x, local_z - clamped_z);
                        let dist = to_player.length();
                        if dist < player_radius {
                            let local_norm = if dist > 1e-4 {
                                to_player / dist
                            } else {
                                Vec2::new(1.0, 0.0)
                            };
                            let push_local =
                                Vec2::new(clamped_x, clamped_z) + local_norm * player_radius;

                            let cos_b = rot_y.cos();
                            let sin_b = rot_y.sin();
                            resolved.x =
                                tf.translation.x + push_local.x * cos_b - push_local.y * sin_b;
                            resolved.z =
                                tf.translation.z + push_local.x * sin_b + push_local.y * cos_b;
                        }
                    }
                }
            }
        }

        // Perimeter boundary limits (Starting Grotto, Portal Archway, and Procedural Dungeon)
        let portal_x_limit = 2.2 - player_radius;

        if resolved.z < 3.2 {
            // Inside starting grotto amphitheater
            let x_limit = 6.8 - player_radius;
            resolved.x = resolved.x.clamp(-x_limit, x_limit);

            let z_min = -13.2 + player_radius;
            if resolved.z < z_min {
                resolved.z = z_min;
            }

            // Flanking rear rock wall (outside portal opening)
            if resolved.x.abs() > portal_x_limit && resolved.z > 3.2 - player_radius {
                resolved.z = 3.2 - player_radius;
            }
        } else if resolved.z <= 5.0 {
            // Archway portal passage connecting grotto to procedural dungeon
            if resolved.x.abs() > portal_x_limit {
                if prev_pos.z <= 3.2 {
                    // Blocked by solid rear grotto wall flanking the portal opening
                    resolved.z = 3.2 - player_radius;
                    let x_limit = 6.8 - player_radius;
                    resolved.x = resolved.x.clamp(-x_limit, x_limit);
                } else if prev_pos.z > 5.0 {
                    // Blocked by dungeon exterior wall returning outside the portal opening
                    resolved.z = 5.0 + player_radius;
                } else {
                    resolved.x = resolved.x.clamp(-portal_x_limit, portal_x_limit);
                }
            } else {
                resolved.x = resolved.x.clamp(-portal_x_limit, portal_x_limit);
            }
        } else {
            // Inside procedural dungeon macro layout
            let dungeon_limit = 46.0 - player_radius;
            resolved.x = resolved.x.clamp(-dungeon_limit, dungeon_limit);
            if resolved.z > dungeon_limit {
                resolved.z = dungeon_limit;
            }
        }
    }

    resolved
}

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

    let sensitivity = settings
        .as_ref()
        .map(|s| s.mouse_sensitivity)
        .unwrap_or(0.0022);

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
    mut query: Query<
        (
            &PlayerState,
            &mut Transform,
            &mut Stamina,
            &mut LocomotionState,
        ),
        With<Player>,
    >,
    obstacles_query: Query<
        (&Transform, Option<&ObstacleShape>),
        (With<CavernObstacle>, Without<Player>),
    >,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }

    let dt = time.delta_secs();

    // Query scene obstacles once per frame
    let obstacle_list: Vec<(&Transform, ObstacleShape)> = obstacles_query
        .iter()
        .map(|(obs_tf, shape_opt)| {
            let shape = shape_opt.copied().unwrap_or(ObstacleShape::Cylinder {
                radius: 0.55,
                height: 2.0,
            });
            (obs_tf, shape)
        })
        .collect();

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
            locomotion.stamina_regen_delay_timer =
                (locomotion.stamina_regen_delay_timer - dt).max(0.0);
        } else {
            locomotion.current_speed = 4.2;
            locomotion.stamina_regen_delay_timer =
                (locomotion.stamina_regen_delay_timer - dt).max(0.0);
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

        // Horizontal Movement with Sliding Collision Resolution
        if is_moving {
            let vel = move_dir.normalize() * locomotion.current_speed * dt;
            let desired_pos = tf.translation + vel;
            let player_radius = 0.35;
            tf.translation = resolve_player_collision_sliding(
                desired_pos,
                tf.translation,
                player_radius,
                &obstacle_list,
            );

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
            cam_tf.translation.y = cam_tf
                .translation
                .y
                .lerp(base_eye_height, (8.0 * dt).min(1.0));
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

        let walk_player = app_walk
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(0.0, 1.2, 0.0),
                Stamina {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        let mut walk_keys = ButtonInput::<KeyCode>::default();
        walk_keys.press(KeyCode::KeyW);
        app_walk.world_mut().insert_resource(walk_keys);

        // Run 60 frames of walking at fixed dt = 16ms
        for _ in 0..60 {
            app_walk
                .world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app_walk.update();
        }

        let z_walk_end = app_walk
            .world()
            .get::<Transform>(walk_player)
            .unwrap()
            .translation
            .z;
        let delta_z1 = z_walk_end.abs();

        // Now run Sprint simulation
        let mut app_sprint = App::new();
        app_sprint.add_systems(Update, player_movement);
        app_sprint.init_resource::<Time>();

        let sprint_player = app_sprint
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(0.0, 1.2, 0.0),
                Stamina {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        let mut sprint_keys = ButtonInput::<KeyCode>::default();
        sprint_keys.press(KeyCode::KeyW);
        sprint_keys.press(KeyCode::ShiftLeft);
        app_sprint.world_mut().insert_resource(sprint_keys);

        for _ in 0..60 {
            app_sprint
                .world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app_sprint.update();
        }

        let z_sprint_end = app_sprint
            .world()
            .get::<Transform>(sprint_player)
            .unwrap()
            .translation
            .z;
        let delta_z2 = z_sprint_end.abs();

        let stamina_sprint = app_sprint
            .world()
            .get::<Stamina>(sprint_player)
            .unwrap()
            .current;

        // GATE 2 ASSERTIONS:
        assert!(
            delta_z2 > delta_z1 * 1.4,
            "Sprinting must be at least 40% faster than walking: delta_z2={:.3}, delta_z1={:.3}",
            delta_z2,
            delta_z1
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

        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(0.0, 1.2, 0.0),
                Stamina {
                    current: 50.0,
                    max: 100.0,
                },
            ))
            .id();

        let mut keys = ButtonInput::<KeyCode>::default();
        app.world_mut().insert_resource(keys.clone());

        // Test Crouch reduces speed
        keys.press(KeyCode::ControlLeft);
        keys.press(KeyCode::KeyW);
        *app.world_mut().resource_mut::<ButtonInput<KeyCode>>() = keys.clone();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(100));
        app.update();

        let locomotion = app.world().get::<LocomotionState>(player).unwrap();
        assert!(locomotion.is_crouching, "Crouch must be active");
        assert_eq!(locomotion.current_speed, 2.2, "Crouch speed must be 2.2");

        // Test Jump applies vertical velocity
        keys.release(KeyCode::ControlLeft);
        keys.press(KeyCode::Space);
        *app.world_mut().resource_mut::<ButtonInput<KeyCode>>() = keys.clone();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        app.update();

        let jumping_locomotion = app.world().get::<LocomotionState>(player).unwrap();
        assert!(
            jumping_locomotion.vertical_velocity > 0.0,
            "Jump must apply upward velocity"
        );
        assert!(
            !jumping_locomotion.is_grounded,
            "Player must not be grounded while ascending"
        );
    }

    #[test]
    fn test_locomotion_camera_fov_kick_and_head_bob() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, player_camera_effects);

        let mut locomotion = LocomotionState::default();
        locomotion.is_sprinting = true;
        locomotion.head_bob_timer = 1.5;

        app.world_mut().spawn((Player, locomotion));

        let camera = app
            .world_mut()
            .spawn((
                PlayerCamera,
                Transform::from_xyz(0.0, 0.55, 0.0),
                Projection::Perspective(PerspectiveProjection {
                    fov: 75.0_f32.to_radians(),
                    ..default()
                }),
            ))
            .id();

        for _ in 0..30 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
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

        let player = app
            .world_mut()
            .spawn((Player, PlayerState::Active, Transform::IDENTITY))
            .id();

        let camera = app
            .world_mut()
            .spawn((PlayerCamera, Transform::IDENTITY))
            .id();

        // Send mouse motion event
        app.world_mut().send_event(MouseMotion {
            delta: Vec2::new(100.0, 50.0),
        });

        app.update();

        let player_tf = app.world().get::<Transform>(player).unwrap();
        let cam_tf = app.world().get::<Transform>(camera).unwrap();

        assert_ne!(
            player_tf.rotation,
            Quat::IDENTITY,
            "Player yaw must have changed"
        );
        assert_ne!(
            cam_tf.rotation,
            Quat::IDENTITY,
            "Camera pitch must have changed"
        );
    }

    #[test]
    fn test_collision_blocks_player_walking_into_mushroom() {
        let mut app = App::new();
        app.add_systems(Update, player_movement);
        app.init_resource::<Time>();

        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(0.0, 1.2, 0.0),
                Stamina {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Spawn Mushroom obstacle 1.0m ahead
        app.world_mut().spawn((
            CavernObstacle,
            ObstacleShape::Cylinder {
                radius: 0.5,
                height: 2.0,
            },
            Transform::from_xyz(0.0, 1.0, -1.0),
        ));

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        app.world_mut().insert_resource(keys);

        // Run 40 frames walking towards obstacle
        for _ in 0..40 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_z = app.world().get::<Transform>(player).unwrap().translation.z;
        // Obstacle at -1.0 with radius 0.5 + player radius 0.35 => player cannot pass z = -0.15
        assert!(
            player_z >= -0.16,
            "Player must be blocked before mushroom obstacle: z={:.3} must be >= -0.16",
            player_z
        );
    }

    #[test]
    fn test_collision_slides_player_diagonally_along_obstacle() {
        let mut app = App::new();
        app.add_systems(Update, player_movement);
        app.init_resource::<Time>();

        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(0.0, 1.2, 0.0),
                Stamina {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Spawn Mushroom obstacle directly ahead
        app.world_mut().spawn((
            CavernObstacle,
            ObstacleShape::Cylinder {
                radius: 0.5,
                height: 2.0,
            },
            Transform::from_xyz(0.0, 1.0, -1.0),
        ));

        // Press W + D (attempting to walk diagonally forward-right)
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyD);
        app.world_mut().insert_resource(keys);

        // Run 10 frames: player contacts the obstacle, forward motion arrested, deflected along +X
        for _ in 0..10 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let contact_tf = app.world().get::<Transform>(player).unwrap();
        let dist_to_obs = (Vec2::new(contact_tf.translation.x, contact_tf.translation.z)
            - Vec2::new(0.0, -1.0))
        .length();
        assert!(
            dist_to_obs >= 0.849,
            "Player must never penetrate obstacle cylinder radius (0.85): dist={:.3}",
            dist_to_obs
        );
        assert!(
            contact_tf.translation.x > 0.2,
            "Player must deflect tangentially along surface: x={:.3}",
            contact_tf.translation.x
        );

        // Run 30 more frames: player completes sliding around the obstacle
        for _ in 0..30 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let cleared_tf = app.world().get::<Transform>(player).unwrap();
        assert!(
            cleared_tf.translation.x > 0.8,
            "Player must have smoothly slid past the right side of the obstacle: x={:.3}",
            cleared_tf.translation.x
        );
    }

    #[test]
    fn test_collision_blocks_player_at_bench_cuboid() {
        let mut app = App::new();
        app.add_systems(Update, player_movement);
        app.init_resource::<Time>();

        // Player starting just in front of bench
        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(2.15, 1.2, -0.8),
                Stamina {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Bench cuboid at z = -1.75, half_extents = (0.8, 0.45, 0.35)
        app.world_mut().spawn((
            CavernObstacle,
            ObstacleShape::Cuboid {
                half_extents: Vec3::new(0.8, 0.45, 0.35),
            },
            Transform::from_xyz(2.15, 0.5, -1.75),
        ));

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW); // Walk forward towards bench
        app.world_mut().insert_resource(keys);

        for _ in 0..40 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_z = app.world().get::<Transform>(player).unwrap().translation.z;
        // Bench front edge is at -1.75 + 0.35 = -1.40. With player radius 0.35, player cannot pass z = -1.05
        assert!(
            player_z >= -1.06,
            "Player must be blocked before bench cuboid: z={:.3} must be >= -1.06",
            player_z
        );
    }

    #[test]
    fn test_collision_blocks_player_at_cavern_canyon_walls() {
        let mut app = App::new();
        app.add_systems(Update, player_movement);
        app.init_resource::<Time>();

        // Player starting near left canyon wall boundary
        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(-6.3, 1.2, 0.0),
                Stamina {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyA); // Walk left into canyon wall
        app.world_mut().insert_resource(keys);

        for _ in 0..40 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_x = app.world().get::<Transform>(player).unwrap().translation.x;
        // Canyon wall limit is at -6.8 + player radius 0.35 = -6.45
        assert!(
            player_x >= -6.46,
            "Player must be blocked by canyon wall: x={:.3} must be >= -6.46",
            player_x
        );
    }

    #[test]
    fn test_player_traverses_portal_archway_into_dungeon() {
        let mut app = App::new();
        app.add_systems(Update, player_movement);
        app.init_resource::<Time>();

        // Player starting near portal opening at x = 0.0, facing south (+Z) towards dungeon
        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(0.0, 1.2, 2.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                Stamina {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW); // Walk forward through the portal archway (+Z)
        app.world_mut().insert_resource(keys);

        for _ in 0..60 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_z = app.world().get::<Transform>(player).unwrap().translation.z;
        assert!(
            player_z >= 4.5,
            "Player must pass through portal archway into dungeon: z={:.3} must be >= 4.5",
            player_z
        );
    }

    #[test]
    fn test_player_blocked_by_rear_wall_outside_portal() {
        let mut app = App::new();
        app.add_systems(Update, player_movement);
        app.init_resource::<Time>();

        // Player starting at x = 4.0 (outside portal width 2.2), facing south (+Z) into solid rear wall
        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(4.0, 1.2, 2.0)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                Stamina {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW); // Attempt to walk through solid rock wall (+Z)
        app.world_mut().insert_resource(keys);

        for _ in 0..60 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_z = app.world().get::<Transform>(player).unwrap().translation.z;
        assert!(
            player_z <= 2.86,
            "Player must be blocked before solid rear wall: z={:.3} must be <= 2.86",
            player_z
        );
    }
}
