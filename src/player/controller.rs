use super::camera::{
    CameraShake, FirstPersonHeldPotion, HeldPotionLiquid, LocomotionState, Player, PlayerBodyScene,
    PlayerCamera, PlayerState, Stamina,
};
use crate::dungeon::modular_kit::{CavernObstacle, ObstacleShape};
use crate::ui::menu::{GameSettings, GameState};
use bevy::{input::mouse::MouseMotion, prelude::*, render::view::RenderLayers};

pub fn resolve_player_collision_sliding(
    desired_pos: Vec3,
    prev_pos: Vec3,
    player_radius: f32,
    is_grounded: bool,
    obstacles: &[(&Transform, ObstacleShape)],
) -> Vec3 {
    let mut resolved = desired_pos;
    let attempted_move = Vec2::new(desired_pos.x - prev_pos.x, desired_pos.z - prev_pos.z);
    let is_moving = attempted_move.length_squared() > 1e-6;

    const STEP_OFFSET: f32 = 0.40;
    const PLAYER_BASE_HEIGHT: f32 = 1.2;

    let terrain_h = if obstacles.is_empty() {
        0.0
    } else {
        crate::dungeon::modular_kit::get_grotto_terrain_height(resolved.x, resolved.z)
    };
    let player_feet = prev_pos.y - PLAYER_BASE_HEIGHT;
    let step_max_feet = player_feet + STEP_OFFSET;

    let min_wade_floor = terrain_h.max(-1.1);
    let mut highest_support_y = min_wade_floor + PLAYER_BASE_HEIGHT;

    for _ in 0..3 {
        for (tf, shape) in obstacles {
            match shape {
                ObstacleShape::Cylinder { radius, height } => {
                    let obs_y = tf.translation.y;
                    let half_h = height * 0.5;
                    let obs_bottom = obs_y - half_h;
                    let obs_top = obs_y + half_h;

                    if obs_top < player_feet - STEP_OFFSET || obs_bottom > resolved.y + 0.65 {
                        continue;
                    }

                    let center = Vec2::new(tf.translation.x, tf.translation.z);
                    let p = Vec2::new(resolved.x, resolved.z);
                    let diff = p - center;
                    let dist = diff.length();
                    let min_dist = *radius + player_radius;

                    if dist < min_dist {
                        if obs_top <= step_max_feet {
                            let support = obs_top + PLAYER_BASE_HEIGHT;
                            if support > highest_support_y {
                                highest_support_y = support;
                            }
                        } else {
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
                }
                ObstacleShape::Cuboid { half_extents } => {
                    let obs_y = tf.translation.y;
                    let obs_bottom = obs_y - half_extents.y;
                    let obs_top = obs_y + half_extents.y;

                    if obs_top < player_feet - STEP_OFFSET || obs_bottom > resolved.y + 0.65 {
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
                    let to_player = Vec2::new(local_x - clamped_x, local_z - clamped_z);
                    let dist = to_player.length();

                    if inside_box || dist < player_radius {
                        if obs_top <= step_max_feet {
                            let support = obs_top + PLAYER_BASE_HEIGHT;
                            if support > highest_support_y {
                                highest_support_y = support;
                            }
                        } else if inside_box {
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

        let portal_x_limit = 2.2 - player_radius;

        if resolved.z < 3.2 {
            let x_limit = 6.8 - player_radius;
            resolved.x = resolved.x.clamp(-x_limit, x_limit);

            let z_min = -13.2 + player_radius;
            if resolved.z < z_min {
                resolved.z = z_min;
            }

            if resolved.x.abs() > portal_x_limit && resolved.z > 3.2 - player_radius {
                resolved.z = 3.2 - player_radius;
            }
        } else if resolved.z <= 5.0 {
            if resolved.x.abs() > portal_x_limit {
                if prev_pos.z <= 3.2 {
                    resolved.z = 3.2 - player_radius;
                    let x_limit = 6.8 - player_radius;
                    resolved.x = resolved.x.clamp(-x_limit, x_limit);
                } else if prev_pos.z > 5.0 {
                    resolved.z = 5.0 + player_radius;
                } else {
                    resolved.x = resolved.x.clamp(-portal_x_limit, portal_x_limit);
                }
            } else {
                resolved.x = resolved.x.clamp(-portal_x_limit, portal_x_limit);
            }
        } else {
            let dungeon_limit = 46.0 - player_radius;
            resolved.x = resolved.x.clamp(-dungeon_limit, dungeon_limit);
            if resolved.z > dungeon_limit {
                resolved.z = dungeon_limit;
            }
        }
    }

    if is_grounded {
        if highest_support_y > resolved.y {
            resolved.y = highest_support_y;
        } else if resolved.y > highest_support_y && (resolved.y - highest_support_y) <= STEP_OFFSET {
            resolved.y = highest_support_y;
        }
    }

    resolved
}

pub fn player_look(
    game_state: Option<Res<State<GameState>>>,
    settings: Option<Res<GameSettings>>,
    time: Option<Res<Time>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    gamepads: Query<&Gamepad>,
    mut player_query: Query<(&PlayerState, &mut Transform, Option<&mut LocomotionState>), (With<Player>, Without<PlayerCamera>)>,
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

    let dt = time.as_ref().map(|t| t.delta_secs()).unwrap_or(1.0 / 60.0);
    let mut stick_delta = Vec2::ZERO;
    for gamepad in gamepads.iter() {
        let raw_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        let raw_y = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);
        let stick_vec = Vec2::new(raw_x, raw_y);
        let len = stick_vec.length();
        const DEADZONE: f32 = 0.12;
        if len > DEADZONE {
            let norm = stick_vec / len;
            let remapped = (len - DEADZONE) / (1.0 - DEADZONE);
            let curve = remapped * remapped;
            let effective = norm * curve;
            let stick_pixels_per_sec = 850.0;
            stick_delta.x += effective.x * stick_pixels_per_sec * dt;
            stick_delta.y -= effective.y * stick_pixels_per_sec * dt;
        }
    }

    let total_delta = mouse_delta + stick_delta;

    let sensitivity = settings
        .as_ref()
        .map(|s| s.mouse_sensitivity)
        .unwrap_or(0.0022);

    for (state, mut player_tf, mut loco_opt) in player_query.iter_mut() {
        if *state != PlayerState::Active {
            continue;
        }

        if let Some(ref mut loco) = loco_opt {
            loco.mouse_look_delta = total_delta;
        }

        if total_delta == Vec2::ZERO {
            continue;
        }

        player_tf.rotate_y(-total_delta.x * sensitivity);

        if let Some(ref mut loco) = loco_opt {
            loco.target_camera_roll = (-total_delta.x * 0.00045).clamp(-0.026, 0.026);
        }

        for mut cam_tf in camera_query.iter_mut() {
            let (yaw, mut pitch, roll) = cam_tf.rotation.to_euler(EulerRot::YXZ);
            pitch -= total_delta.y * sensitivity;
            pitch = pitch.clamp(-1.52, 1.52);
            cam_tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        }
    }
}

fn detect_player_surfaces<'a>(
    translation: Vec3,
    ice_sheets_iter: impl Iterator<Item = (&'a Transform, &'a crate::alchemy::FrozenIceSheet)>,
    has_lake: bool,
) -> (bool, bool) {
    let p_xz = Vec2::new(translation.x, translation.z);
    let mut on_ice = false;
    for (ice_tf, ice_sheet) in ice_sheets_iter {
        let d = (p_xz - ice_sheet.center).length();
        let current_radius = ice_sheet.radius * ice_tf.scale.x;
        if d <= current_radius + 0.35 {
            let ice_surface_y = ice_sheet.top_y + 1.2;
            if (translation.y - ice_surface_y).abs() <= 0.65 {
                on_ice = true;
                break;
            }
        }
    }
    let in_lake = has_lake && crate::dungeon::modular_kit::is_in_lake_water(translation - Vec3::new(0.0, 0.40, 0.0));
    let is_in_water = in_lake && !on_ice;
    (on_ice, is_in_water)
}

fn compute_player_locomotion_input(
    keys: Option<&ButtonInput<KeyCode>>,
    gamepads: &Query<&Gamepad>,
    forward: Vec3,
    right: Vec3,
    is_frozen: bool,
) -> (Vec3, Vec2, bool, f32) {
    let mut move_dir = Vec3::ZERO;
    let mut strafe_target_roll: f32 = 0.0;
    if !is_frozen {
        if let Some(k) = keys {
            if k.pressed(KeyCode::KeyW) {
                move_dir += forward;
            }
            if k.pressed(KeyCode::KeyS) {
                move_dir -= forward;
            }
            if k.pressed(KeyCode::KeyD) {
                move_dir += right;
                strafe_target_roll += 0.020;
            }
            if k.pressed(KeyCode::KeyA) {
                move_dir -= right;
                strafe_target_roll -= 0.020;
            }
        }

        for gamepad in gamepads.iter() {
            let raw_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
            let raw_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
            let stick = Vec2::new(raw_x, raw_y);
            let len = stick.length();
            const DEADZONE: f32 = 0.15;
            if len > DEADZONE {
                let norm = stick / len;
                let remapped = ((len - DEADZONE) / (1.0 - DEADZONE)).clamp(0.0, 1.0);
                let stick_move = norm * remapped;
                move_dir += forward * stick_move.y;
                move_dir += right * stick_move.x;
                strafe_target_roll += stick_move.x * 0.020;
            }
            if gamepad.pressed(GamepadButton::DPadUp) {
                move_dir += forward;
            }
            if gamepad.pressed(GamepadButton::DPadDown) {
                move_dir -= forward;
            }
        }
    }

    move_dir.y = 0.0;
    let is_moving = move_dir.length_squared() > 0.0001;
    let wish_dir = if is_moving {
        Vec2::new(move_dir.x, move_dir.z).normalize()
    } else {
        Vec2::ZERO
    };

    (move_dir, wish_dir, is_moving, strafe_target_roll)
}

fn try_execute_jump(locomotion: &mut LocomotionState, is_frozen: bool) -> bool {
    if locomotion.jump_buffer_timer > 0.0
        && (locomotion.coyote_timer > 0.0 || locomotion.is_grounded)
        && !locomotion.is_crouching
        && !is_frozen
    {
        locomotion.vertical_velocity = if locomotion.is_in_water { 3.8 } else { 6.8 };
        locomotion.is_grounded = false;
        locomotion.coyote_timer = 0.0;
        locomotion.jump_buffer_timer = 0.0;
        locomotion.squash_scale = Vec3::new(0.96, 1.08, 0.96);
        locomotion.takeoff_timer = 0.06;
        true
    } else {
        false
    }
}

fn calculate_landing_elevation(
    pos: Vec3,
    obstacle_list: &[(&Transform, ObstacleShape)],
    has_lake: bool,
) -> f32 {
    let floor_y = if has_lake {
        crate::dungeon::modular_kit::get_grotto_terrain_height(pos.x, pos.z)
    } else {
        0.0
    };
    let min_wade_floor = floor_y.max(-1.1);
    let mut landing_y = min_wade_floor + 1.2;
    let p_xz = Vec2::new(pos.x, pos.z);
    for (obs_tf, shape) in obstacle_list {
        match shape {
            ObstacleShape::Cylinder { radius, height } => {
                let center = Vec2::new(obs_tf.translation.x, obs_tf.translation.z);
                if (p_xz - center).length() <= *radius + 0.38 {
                    let top = obs_tf.translation.y + height * 0.5 + 1.2;
                    if top <= pos.y + 0.1 && top > landing_y {
                        landing_y = top;
                    }
                }
            }
            ObstacleShape::Cuboid { half_extents } => {
                let (_, rot_y, _) = obs_tf.rotation.to_euler(EulerRot::YXZ);
                let diff_world = p_xz - Vec2::new(obs_tf.translation.x, obs_tf.translation.z);
                let cos_r = (-rot_y).cos();
                let sin_r = (-rot_y).sin();
                let lx = diff_world.x * cos_r - diff_world.y * sin_r;
                let lz = diff_world.x * sin_r + diff_world.y * cos_r;
                if lx.abs() <= half_extents.x + 0.38 && lz.abs() <= half_extents.z + 0.38 {
                    let top = obs_tf.translation.y + half_extents.y + 1.2;
                    if top <= pos.y + 0.1 && top > landing_y {
                        landing_y = top;
                    }
                }
            }
        }
    }
    landing_y
}

fn spawn_water_splash_burst(
    commands: &mut Commands,
    mesh_assets: &mut Assets<Mesh>,
    mat_assets: &mut Assets<StandardMaterial>,
    origin: Vec3,
    count: usize,
    speed_scale: f32,
    radius: f32,
) {
    for _ in 0..count {
        let vel = Vec3::new(
            (crate::render::particles::fastrand::f32() - 0.5) * speed_scale * 1.6,
            0.6 * speed_scale + crate::render::particles::fastrand::f32() * 0.8 * speed_scale,
            (crate::render::particles::fastrand::f32() - 0.5) * speed_scale * 1.6,
        );
        commands.spawn((
            Mesh3d(mesh_assets.add(Sphere::new(radius).mesh().uv(8, 6))),
            MeshMaterial3d(mat_assets.add(StandardMaterial {
                base_color: Color::LinearRgba(LinearRgba::new(0.30, 0.60, 0.82, 0.30)),
                alpha_mode: AlphaMode::Add,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(origin),
            crate::render::particles::Particle {
                lifetime: 0.0,
                max_lifetime: 0.9,
                velocity: vel,
                p_type: crate::render::particles::ParticleType::WaterSplash,
            },
        ));
    }
}

fn spawn_spore_burst(
    commands: &mut Commands,
    mesh_assets: &mut Assets<Mesh>,
    mat_assets: &mut Assets<StandardMaterial>,
    origin: Vec3,
    count: usize,
    base_color: Color,
    radius: f32,
) {
    for _ in 0..count {
        let angle = crate::render::particles::fastrand::f32() * std::f32::consts::TAU;
        let speed = 0.5 + crate::render::particles::fastrand::f32() * 0.6;
        let vel = Vec3::new(
            angle.cos() * speed,
            0.20 + crate::render::particles::fastrand::f32() * 0.30,
            angle.sin() * speed,
        );
        commands.spawn((
            Mesh3d(mesh_assets.add(Sphere::new(radius).mesh().uv(8, 6))),
            MeshMaterial3d(mat_assets.add(StandardMaterial {
                base_color,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(origin),
            crate::render::particles::Particle {
                lifetime: 0.0,
                max_lifetime: 1.5,
                velocity: vel,
                p_type: crate::render::particles::ParticleType::GlowSpore,
            },
        ));
    }
}

fn update_airborne_and_landing(
    tf: &mut Transform,
    locomotion: &mut LocomotionState,
    obstacle_list: &[(&Transform, ObstacleShape)],
    has_lake: bool,
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    shake: &mut Option<ResMut<CameraShake>>,
    is_frozen: bool,
    dt: f32,
) {
    if locomotion.is_grounded {
        return;
    }

    let landing_y = calculate_landing_elevation(tf.translation, obstacle_list, has_lake);
    let gravity = if locomotion.vertical_velocity > 0.0 {
        24.0
    } else {
        36.0
    };
    locomotion.last_fall_velocity = locomotion.vertical_velocity;
    locomotion.vertical_velocity -= gravity * dt;
    tf.translation.y += locomotion.vertical_velocity * dt;

    if tf.translation.y <= landing_y {
        tf.translation.y = landing_y;
        let impact_speed = locomotion.vertical_velocity.abs().max(locomotion.last_fall_velocity.abs());
        locomotion.vertical_velocity = 0.0;
        locomotion.is_grounded = true;
        locomotion.coyote_timer = 0.10;

        let impact_ratio = (impact_speed / 14.0).clamp(0.0, 1.0);
        let squash_factor = 0.06 + 0.10 * impact_ratio;
        locomotion.squash_scale = Vec3::new(
            1.0 + squash_factor * 0.5,
            1.0 - squash_factor,
            1.0 + squash_factor * 0.5,
        );
        locomotion.camera_landing_dip = (impact_speed * 0.016).clamp(0.04, 0.10);

        if impact_speed > 5.0 {
            if let Some(ref mut s) = shake {
                let trauma_val = ((impact_speed - 5.0) / 9.0).clamp(0.18, 0.70);
                s.add_trauma(trauma_val, Vec3::new(0.0, -1.0, 0.0));
            }
        }

        if impact_speed > 4.2 {
            if let (Some(ref mut mesh_assets), Some(ref mut mat_assets)) =
                (meshes.as_mut(), materials.as_mut())
            {
                if locomotion.is_in_water {
                    spawn_water_splash_burst(
                        commands,
                        mesh_assets,
                        mat_assets,
                        Vec3::new(tf.translation.x, 0.22, tf.translation.z),
                        6,
                        1.5,
                        0.08,
                    );
                    crate::dungeon::modular_kit::spawn_water_ripple(
                        commands,
                        mesh_assets,
                        mat_assets,
                        Vec3::new(tf.translation.x, 0.19, tf.translation.z),
                        0.22,
                        1.45,
                        1.5,
                        0.85,
                    );
                } else {
                    spawn_spore_burst(
                        commands,
                        mesh_assets,
                        mat_assets,
                        Vec3::new(tf.translation.x, landing_y - 1.15, tf.translation.z),
                        5,
                        Color::srgba(0.35, 0.95, 0.80, 0.65),
                        0.035,
                    );
                }
            }
        }

        try_execute_jump(locomotion, is_frozen);
    }
}

fn spawn_player_footstep_feedback(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    tf: &Transform,
    locomotion: &mut LocomotionState,
    right: &Dir3,
    is_moving: bool,
    is_sprinting: bool,
    move_dir: Vec3,
    dt: f32,
) {
    if !locomotion.is_grounded || !is_moving || locomotion.is_frozen {
        return;
    }

    let step_dist = locomotion.horizontal_velocity.length() * dt;
    locomotion.distance_since_footstep += step_dist;
    let step_threshold = if is_sprinting { 0.48 } else { 0.62 };
    if locomotion.distance_since_footstep < step_threshold {
        return;
    }

    locomotion.distance_since_footstep = 0.0;
    locomotion.footstep_side = !locomotion.footstep_side;
    let side_offset = if locomotion.footstep_side { 0.18 } else { -0.18 };
    let foot_pos = tf.translation + *right * side_offset - Vec3::new(0.0, 1.15, 0.0);

    let (Some(ref mut mesh_assets), Some(ref mut mat_assets)) =
        (meshes.as_mut(), materials.as_mut())
    else {
        return;
    };

    if locomotion.is_in_water {
        spawn_water_splash_burst(
            commands,
            mesh_assets,
            mat_assets,
            Vec3::new(foot_pos.x, 0.18, foot_pos.z),
            3,
            0.9,
            0.06,
        );
        crate::dungeon::modular_kit::spawn_water_ripple(
            commands,
            mesh_assets,
            mat_assets,
            Vec3::new(foot_pos.x, 0.19, foot_pos.z),
            0.12,
            0.75,
            1.1,
            0.70,
        );
    } else if !locomotion.is_on_ice {
        if is_sprinting {
            spawn_spore_burst(
                commands,
                mesh_assets,
                mat_assets,
                foot_pos + Vec3::new(0.0, 0.08, 0.0),
                2,
                Color::srgba(0.20, 0.95, 0.75, 0.70),
                0.022,
            );
        } else {
            let vel = Vec3::new(
                -move_dir.x * 0.25 + (crate::render::particles::fastrand::f32() - 0.5) * 0.15,
                0.10 + crate::render::particles::fastrand::f32() * 0.12,
                -move_dir.z * 0.25 + (crate::render::particles::fastrand::f32() - 0.5) * 0.15,
            );
            commands.spawn((
                Mesh3d(mesh_assets.add(Sphere::new(0.035).mesh().uv(8, 6))),
                MeshMaterial3d(mat_assets.add(StandardMaterial {
                    base_color: Color::srgba(0.65, 0.60, 0.50, 0.25),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(foot_pos + Vec3::new(0.0, 0.05, 0.0)),
                crate::render::particles::Particle {
                    lifetime: 0.0,
                    max_lifetime: 0.7,
                    velocity: vel,
                    p_type: crate::render::particles::ParticleType::Steam,
                },
            ));
        }
    }
}

fn spawn_player_ice_footprints(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    tf: &Transform,
    locomotion: &mut LocomotionState,
    right: &Dir3,
    is_moving: bool,
    dt: f32,
) {
    if !locomotion.is_on_ice || !is_moving || !locomotion.is_grounded {
        return;
    }

    let step_dist = locomotion.horizontal_velocity.length() * dt;
    locomotion.distance_since_footprint += step_dist;
    if locomotion.distance_since_footprint < 0.52 {
        return;
    }

    locomotion.distance_since_footprint = 0.0;
    locomotion.footstep_side = !locomotion.footstep_side;
    let side_offset = if locomotion.footstep_side { 0.16 } else { -0.16 };
    let foot_pos = tf.translation + *right * side_offset;

    let (Some(ref mut mesh_assets), Some(ref mut mat_assets)) =
        (meshes.as_mut(), materials.as_mut())
    else {
        return;
    };

    let footprint_mesh = mesh_assets.add(crate::alchemy::create_footprint_mesh(0.24, 0.11));
    let footprint_mat = mat_assets.add(StandardMaterial {
        base_color: Color::srgba(0.82, 0.94, 1.0, 0.75),
        emissive: LinearRgba::new(0.12, 0.30, 0.45, 0.4),
        metallic: 0.1,
        perceptual_roughness: 0.4,
        reflectance: 0.8,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        unlit: false,
        ..default()
    });
    let player_yaw = tf.rotation;
    commands.spawn((
        crate::alchemy::IceFootprint {
            lifetime: 0.0,
            max_lifetime: 18.0,
        },
        Transform {
            translation: Vec3::new(foot_pos.x, 0.305, foot_pos.z),
            rotation: player_yaw,
            scale: Vec3::ONE,
        },
        Mesh3d(footprint_mesh),
        MeshMaterial3d(footprint_mat),
    ));
}

pub fn player_movement(
    game_state: Option<Res<State<GameState>>>,
    time: Res<Time>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
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
    ice_sheets_query: Query<
        (&Transform, &crate::alchemy::FrozenIceSheet),
        Without<Player>,
    >,
    lake_query: Query<
        (),
        (
            With<crate::dungeon::modular_kit::LakeWaterPool>,
            Without<Player>,
        ),
    >,
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut shake: Option<ResMut<CameraShake>>,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }

    let dt = time.delta_secs();

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

    let has_lake = !lake_query.is_empty();

    for (state, mut tf, mut stamina, mut locomotion) in query.iter_mut() {
        if *state != PlayerState::Active {
            continue;
        }

        let (on_ice, is_in_water) = detect_player_surfaces(
            tf.translation,
            ice_sheets_query.iter(),
            has_lake,
        );
        locomotion.is_on_ice = on_ice;
        locomotion.is_in_water = is_in_water;

        if locomotion.is_frozen {
            locomotion.freeze_timer = (locomotion.freeze_timer - dt).max(0.0);
            if locomotion.freeze_timer <= 0.0 {
                locomotion.is_frozen = false;
            }
        }

        let is_frozen = locomotion.is_frozen;
        if is_frozen && crate::render::particles::fastrand::f32() < 0.25 {
            commands.spawn((
                Transform::from_xyz(tf.translation.x, tf.translation.y - 0.95, tf.translation.z),
                crate::render::particles::ParticleEmitter {
                    spawn_timer: 0.0,
                    spawn_rate: 0.06,
                    particle_type: crate::render::particles::ParticleType::Steam,
                },
                crate::alchemy::AlchemyTransientEmitter {
                    elapsed: 0.0,
                    max_lifetime: 0.35,
                },
            ));
        }

        let forward = tf.forward();
        let right = tf.right();

        let (move_dir, wish_dir, is_moving, strafe_target_roll) = compute_player_locomotion_input(
            keys.as_deref(),
            &gamepads,
            *forward,
            *right,
            is_frozen,
        );

        if strafe_target_roll.abs() > 1e-4 {
            locomotion.target_camera_roll = (locomotion.target_camera_roll + strafe_target_roll).clamp(-0.028, 0.028);
        }

        locomotion.camera_roll = locomotion.camera_roll.lerp(locomotion.target_camera_roll, (10.0 * dt).min(1.0));
        locomotion.target_camera_roll = locomotion.target_camera_roll.lerp(0.0, (8.0 * dt).min(1.0));
        locomotion.throw_recoil = (locomotion.throw_recoil - 6.0 * dt).max(0.0);

        let gamepad_crouch = gamepads.iter().any(|g| {
            g.pressed(GamepadButton::RightThumb) || g.pressed(GamepadButton::DPadDown)
        });
        locomotion.is_crouching = keys.as_ref().map(|k| k.pressed(KeyCode::ControlLeft)).unwrap_or(false) || gamepad_crouch;

        let gamepad_sprint = gamepads.iter().any(|g| {
            g.pressed(GamepadButton::LeftThumb) || g.pressed(GamepadButton::East)
        });
        let can_sprint = !locomotion.is_crouching && stamina.current > 5.0 && is_moving && !is_frozen;
        let is_sprinting = (keys.as_ref().map(|k| k.pressed(KeyCode::ShiftLeft)).unwrap_or(false) || gamepad_sprint) && can_sprint;
        locomotion.is_sprinting = is_sprinting;

        if is_frozen {
            locomotion.current_speed = 0.0;
        } else if is_sprinting {
            locomotion.current_speed = 7.0;
            stamina.current = (stamina.current - 20.0 * dt).max(0.0);
            locomotion.stamina_regen_delay_timer = 1.0;
        } else if locomotion.is_crouching {
            locomotion.current_speed = 2.2;
            locomotion.stamina_regen_delay_timer =
                (locomotion.stamina_regen_delay_timer - dt).max(0.0);
        } else {
            locomotion.current_speed = 4.2;
            locomotion.stamina_regen_delay_timer =
                (locomotion.stamina_regen_delay_timer - dt).max(0.0);
        }

        if locomotion.is_in_water && !is_frozen {
            locomotion.current_speed *= 0.52;
            stamina.current = (stamina.current - 6.0 * dt).max(0.0);
            if is_moving && crate::render::particles::fastrand::f32() < 0.18 {
                commands.spawn((
                    Transform::from_xyz(tf.translation.x, 0.22, tf.translation.z),
                    crate::render::particles::ParticleEmitter {
                        spawn_timer: 0.0,
                        spawn_rate: 0.07,
                        particle_type: crate::render::particles::ParticleType::WaterSplash,
                    },
                ));
            }
        }

        if !is_sprinting && locomotion.stamina_regen_delay_timer <= 0.0 {
            stamina.current = (stamina.current + 15.0 * dt).min(stamina.max);
        }

        if locomotion.is_grounded {
            locomotion.coyote_timer = 0.10;
        } else {
            locomotion.coyote_timer = (locomotion.coyote_timer - dt).max(0.0);
        }

        let gamepad_jump = gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::South)
        });
        if (keys.as_ref().map(|k| k.just_pressed(KeyCode::Space)).unwrap_or(false) || gamepad_jump) && !locomotion.is_crouching && !is_frozen {
            locomotion.jump_buffer_timer = 0.12;
        } else {
            locomotion.jump_buffer_timer = (locomotion.jump_buffer_timer - dt).max(0.0);
        }

        if locomotion.takeoff_timer > 0.0 {
            locomotion.takeoff_timer = (locomotion.takeoff_timer - dt).max(0.0);
        } else {
            locomotion.squash_scale = locomotion.squash_scale.lerp(Vec3::ONE, (12.0 * dt).min(1.0));
        }

        locomotion.camera_landing_dip = locomotion.camera_landing_dip.lerp(0.0, (14.0 * dt).min(1.0));

        if locomotion.is_grounded {
            if is_frozen {
                locomotion.horizontal_velocity = Vec2::ZERO;
            } else if is_moving {
                locomotion.horizontal_velocity = wish_dir * locomotion.current_speed;
            } else {
                locomotion.horizontal_velocity = Vec2::ZERO;
            }

            try_execute_jump(&mut locomotion, is_frozen);
        } else {
            let drag = (1.0 - 0.05 * dt).clamp(0.0, 1.0);
            locomotion.horizontal_velocity *= drag;

            if is_moving && !is_frozen {
                let current_mag = locomotion.horizontal_velocity.length();
                let max_air_speed = current_mag.max(locomotion.current_speed);
                let air_accel = 12.0;
                locomotion.horizontal_velocity += wish_dir * air_accel * dt;
                let new_mag = locomotion.horizontal_velocity.length();
                if new_mag > max_air_speed && new_mag > 1e-4 {
                    locomotion.horizontal_velocity =
                        locomotion.horizontal_velocity / new_mag * max_air_speed;
                }
            }

            try_execute_jump(&mut locomotion, is_frozen);
        }

        update_airborne_and_landing(
            &mut tf,
            &mut locomotion,
            &obstacle_list,
            has_lake,
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut shake,
            is_frozen,
            dt,
        );

        let h_speed_sq = locomotion.horizontal_velocity.length_squared();
        if h_speed_sq > 0.0001 {
            let prev_pos = tf.translation;
            let vel = Vec3::new(
                locomotion.horizontal_velocity.x * dt,
                0.0,
                locomotion.horizontal_velocity.y * dt,
            );
            let desired_pos = tf.translation + vel;
            let player_radius = 0.38;
            tf.translation = resolve_player_collision_sliding(
                desired_pos,
                tf.translation,
                player_radius,
                locomotion.is_grounded,
                &obstacle_list,
            );

            let actual_dx = tf.translation.x - prev_pos.x;
            let actual_dz = tf.translation.z - prev_pos.z;
            if dt > 1e-5 {
                locomotion.horizontal_velocity = Vec2::new(actual_dx / dt, actual_dz / dt);
            }
        }

        spawn_player_footstep_feedback(
            &mut commands,
            &mut meshes,
            &mut materials,
            &tf,
            &mut locomotion,
            &right,
            is_moving,
            is_sprinting,
            move_dir,
            dt,
        );

        spawn_player_ice_footprints(
            &mut commands,
            &mut meshes,
            &mut materials,
            &tf,
            &mut locomotion,
            &right,
            is_moving,
            dt,
        );

        let floor_h = if has_lake {
            crate::dungeon::modular_kit::get_grotto_terrain_height(tf.translation.x, tf.translation.z)
        } else {
            0.0
        };
        let min_floor = floor_h.max(-1.1);
        let expected_support = if locomotion.is_on_ice {
            0.17 + 0.13 + 1.2
        } else {
            min_floor + 1.2
        };
        if locomotion.is_grounded && tf.translation.y > expected_support + 0.25 {
            locomotion.is_grounded = false;
        }

        let speed_factor = (locomotion.horizontal_velocity.length() / 4.2).clamp(0.5, 1.8);
        let omega = if is_sprinting { 12.5 } else { 8.2 } * speed_factor;
        if locomotion.is_grounded && is_moving {
            locomotion.head_bob_timer += dt * omega;
            locomotion.head_bob_intensity = (locomotion.head_bob_intensity + 8.0 * dt).min(1.0);
        } else {
            locomotion.head_bob_intensity = (locomotion.head_bob_intensity - 10.0 * dt).max(0.0);
            if locomotion.head_bob_intensity <= 0.0 {
                locomotion.head_bob_timer = 0.0;
            }
        }
    }
}

pub fn player_camera_effects(
    time: Res<Time>,
    player_query: Query<&LocomotionState, (With<Player>, Without<PlayerCamera>)>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<PlayerCamera>>,
    mut shake_opt: Option<ResMut<CameraShake>>,
) {
    let dt = time.delta_secs();
    let Ok(locomotion) = player_query.get_single() else {
        return;
    };

    let (shake_offset, shake_roll) = if let Some(ref mut shake) = shake_opt {
        shake.timer += dt;
        let trauma = shake.trauma;
        shake.trauma = (shake.trauma - 1.8 * dt).max(0.0);
        if trauma > 0.001 {
            let p = trauma * trauma;
            let decay = (-3.2 * shake.timer).exp();
            let wave = (24.0 * shake.timer).sin();
            let mag = p * 0.12 * decay * wave;
            let offset = shake.direction * mag;
            let rot_r = (shake.timer * 22.0).sin() * p * 0.050;
            (offset, rot_r)
        } else {
            shake.timer = 0.0;
            shake.direction = Vec3::ZERO;
            (Vec3::ZERO, 0.0)
        }
    } else {
        (Vec3::ZERO, 0.0)
    };

    for (mut cam_tf, mut projection) in camera_query.iter_mut() {
        if let Projection::Perspective(ref mut persp) = *projection {
            let base_fov: f32 = 75.0_f32.to_radians();
            let target_fov = if locomotion.is_sprinting {
                base_fov + 7.0_f32.to_radians()
            } else {
                base_fov
            };
            persp.fov = persp.fov.lerp(target_fov, (10.0 * dt).min(1.0));
        }

        let base_eye_height = if locomotion.is_crouching { 0.25 } else { 0.55 };
        let dip_offset = locomotion.camera_landing_dip;
        let recoil_dip = locomotion.throw_recoil * 0.035;
        let windup_lift = locomotion.throw_windup * 0.020;
        let target_z = shake_offset.z + locomotion.throw_windup * 0.050 - locomotion.throw_recoil * 0.025;

        let bob_intensity = if locomotion.head_bob_intensity > 0.0 {
            locomotion.head_bob_intensity
        } else if locomotion.head_bob_timer > 0.0 {
            1.0
        } else {
            0.0
        };
        let bob_t = locomotion.head_bob_timer;
        let bob_amp_x = if locomotion.is_sprinting { 0.038 } else { 0.022 };
        let bob_amp_y = if locomotion.is_sprinting { 0.055 } else { 0.032 };
        let bob_x = (bob_t).sin() * bob_amp_x * bob_intensity;
        let bob_y = (2.0 * bob_t + std::f32::consts::FRAC_PI_2).sin() * bob_amp_y * bob_intensity;
        let roll_bob = (bob_t).sin() * 0.007 * bob_intensity;

        if bob_intensity > 0.0 {
            cam_tf.translation.y = base_eye_height + bob_y - dip_offset - recoil_dip + windup_lift + shake_offset.y;
            cam_tf.translation.x = bob_x + shake_offset.x;
            cam_tf.translation.z = target_z;
        } else {
            let target_y = base_eye_height - dip_offset - recoil_dip + windup_lift + shake_offset.y;
            cam_tf.translation.y = cam_tf
                .translation
                .y
                .lerp(target_y, (12.0 * dt).min(1.0));
            cam_tf.translation.x = cam_tf.translation.x.lerp(shake_offset.x, (8.0 * dt).min(1.0));
            cam_tf.translation.z = cam_tf.translation.z.lerp(target_z, (12.0 * dt).min(1.0));
        }

        let (yaw, pitch, _) = cam_tf.rotation.to_euler(EulerRot::YXZ);
        let final_roll = locomotion.camera_roll + shake_roll + roll_bob;
        cam_tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, final_roll);
    }
}

pub fn update_player_body_squash(
    locomotion_query: Query<&LocomotionState, With<Player>>,
    mut body_scenes: Query<&mut Transform, With<PlayerBodyScene>>,
) {
    let Ok(locomotion) = locomotion_query.get_single() else {
        return;
    };
    for mut body_tf in body_scenes.iter_mut() {
        body_tf.scale = locomotion.squash_scale;
    }
}

pub fn update_first_person_potion_sway(
    time: Res<Time>,
    player_query: Query<(&PlayerState, &LocomotionState), With<Player>>,
    mut potion_query: Query<(&mut FirstPersonHeldPotion, &mut Transform, &mut Visibility)>,
) {
    let dt = time.delta_secs();
    let Ok((state, locomotion)) = player_query.get_single() else {
        return;
    };

    for (mut sway, mut tf, mut vis) in potion_query.iter_mut() {
        if *state != PlayerState::Active {
            *vis = Visibility::Hidden;
            continue;
        } else if *vis != Visibility::Inherited && *vis != Visibility::Visible {
            *vis = Visibility::Inherited;
        }

        let mouse_delta = locomotion.mouse_look_delta;
        let strafe_vel = locomotion.horizontal_velocity.x;

        let target_offset_x = (-mouse_delta.x * 0.00045 - strafe_vel * 0.005).clamp(-0.035, 0.035);
        let target_offset_y = (mouse_delta.y * 0.00045 - locomotion.vertical_velocity * 0.003).clamp(-0.035, 0.035);
        let target_offset_z = (-locomotion.throw_recoil * 0.06 + locomotion.throw_windup * 0.04).clamp(-0.08, 0.08);
        let target_offset = Vec3::new(target_offset_x, target_offset_y, target_offset_z);

        let k_pos = 145.0;
        let c_pos = 18.5;
        let diff_pos = sway.sway_offset - target_offset;
        let accel_pos = -k_pos * diff_pos - c_pos * sway.sway_velocity;
        sway.sway_velocity += accel_pos * dt;
        let pos_vel = sway.sway_velocity;
        sway.sway_offset += pos_vel * dt;

        let target_pitch = (-mouse_delta.y * 0.0005).clamp(-0.07, 0.07);
        let target_yaw = (mouse_delta.x * 0.0005).clamp(-0.07, 0.07);
        let target_roll = (-mouse_delta.x * 0.0003 - strafe_vel * 0.008).clamp(-0.05, 0.05);
        let target_rot_euler = Vec3::new(target_pitch, target_yaw, target_roll);

        let (cur_yaw, cur_pitch, cur_roll) = sway.sway_rotation.to_euler(EulerRot::YXZ);
        let cur_rot_euler = Vec3::new(cur_pitch, cur_yaw, cur_roll);

        let k_rot = 160.0;
        let c_rot = 19.0;
        let diff_rot = cur_rot_euler - target_rot_euler;
        let accel_rot = -k_rot * diff_rot - c_rot * sway.sway_angular_velocity;
        sway.sway_angular_velocity += accel_rot * dt;
        let ang_vel = sway.sway_angular_velocity;
        let new_rot_euler = cur_rot_euler + ang_vel * dt;
        sway.sway_rotation = Quat::from_euler(EulerRot::YXZ, new_rot_euler.y, new_rot_euler.x, new_rot_euler.z);

        let bob_t = locomotion.head_bob_timer;
        let bob_int = if locomotion.head_bob_intensity > 0.0 {
            locomotion.head_bob_intensity
        } else if locomotion.head_bob_timer > 0.0 {
            1.0
        } else {
            0.0
        };
        let bob_sway = Vec3::new(
            (bob_t).sin() * 0.008 * bob_int,
            (2.0 * bob_t).sin() * 0.012 * bob_int,
            0.0,
        );

        let windup_offset = Vec3::new(
            -0.03 * locomotion.throw_windup,
            0.05 * locomotion.throw_windup,
            0.08 * locomotion.throw_windup,
        );
        let recoil_offset = Vec3::new(
            0.01 * locomotion.throw_recoil,
            -0.04 * locomotion.throw_recoil,
            -0.05 * locomotion.throw_recoil,
        );

        tf.translation = sway.base_position + sway.sway_offset + bob_sway + windup_offset + recoil_offset;
        tf.rotation = sway.sway_rotation;
    }
}

pub fn update_held_potion_visuals(
    mut commands: Commands,
    active_slot_res: Option<Res<crate::ui::hud::ActiveHotbarSlot>>,
    cooldowns_res: Option<Res<crate::alchemy::PotionCooldowns>>,
    mut meshes_res: Option<ResMut<Assets<Mesh>>>,
    mut materials_res: Option<ResMut<Assets<StandardMaterial>>>,
    mut fluid_materials_res: Option<ResMut<Assets<crate::render::potion_fluid_material::PotionFluidMaterial>>>,
    mut potion_query: Query<(Entity, &mut FirstPersonHeldPotion)>,
    liquid_query: Query<&MeshMaterial3d<StandardMaterial>, With<HeldPotionLiquid>>,
    slosh_query: Query<&crate::render::potion_fluid_material::PotionSloshState, With<HeldPotionLiquid>>,
) {
    let (Some(ref mut meshes), Some(ref mut materials)) =
        (meshes_res.as_mut(), materials_res.as_mut())
    else {
        return;
    };

    let active_slot = active_slot_res.map(|s| s.0).unwrap_or(1);
    let (base_color, _) = crate::alchemy::selected_potion_effects(active_slot);
    let is_on_cooldown = cooldowns_res
        .as_ref()
        .map(|c| !c.is_ready(active_slot))
        .unwrap_or(false);
    let liquid_color = if is_on_cooldown {
        Color::srgba(0.45, 0.45, 0.45, 0.45)
    } else {
        base_color
    };

    for (potion_entity, mut potion) in potion_query.iter_mut() {
        if !potion.visuals_spawned || potion.current_slot != active_slot {
            let prev_slosh = slosh_query.iter().next().cloned();
            if potion.visuals_spawned {
                commands.entity(potion_entity).despawn_descendants();
            }

            potion.visuals_spawned = true;
            potion.current_slot = active_slot;

            let glass_mat = materials.add(StandardMaterial {
                base_color: Color::srgba(0.88, 0.94, 1.0, 0.10),
                perceptual_roughness: 0.12,
                metallic: 0.0,
                reflectance: 0.50,
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            let liquid_mat = materials.add(StandardMaterial {
                base_color: liquid_color,
                emissive: liquid_color.to_linear() * 1.2,
                perceptual_roughness: 0.25,
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            let (stopper_color, stopper_roughness, stopper_metallic, stopper_emissive) = match active_slot {
                2 => (Color::srgb(0.85, 0.15, 0.20), 0.25, 0.10, Color::srgb(0.40, 0.06, 0.08)),
                3 => (Color::srgb(0.68, 0.52, 0.22), 0.35, 0.75, Color::BLACK),
                4 => (Color::srgb(0.20, 0.65, 0.95), 0.15, 0.05, Color::srgb(0.06, 0.30, 0.55)),
                5 => (Color::srgb(0.92, 0.95, 1.0), 0.12, 0.05, Color::srgb(0.30, 0.35, 0.42)),
                _ => (Color::srgb(0.55, 0.38, 0.22), 0.85, 0.0, Color::BLACK),
            };

            let stopper_mat = materials.add(StandardMaterial {
                base_color: stopper_color,
                perceptual_roughness: stopper_roughness,
                metallic: stopper_metallic,
                emissive: stopper_emissive.to_linear(),
                ..default()
            });

            let flask_mesh = meshes.add(crate::alchemy::create_held_flask_mesh(active_slot));
            let fluid_mesh = meshes.add(crate::alchemy::create_held_fluid_mesh(active_slot));
            let stopper_mesh = meshes.add(crate::alchemy::create_held_stopper_mesh(active_slot));

            

            commands.entity(potion_entity).with_children(|builder| {
                builder.spawn((
                    Mesh3d(flask_mesh),
                    MeshMaterial3d(glass_mat),
                    Transform::IDENTITY,
                    RenderLayers::layer(2),
                ));

                if let Some(ref mut fluid_materials) = fluid_materials_res {
                    let fluid_mat = fluid_materials.add(crate::render::potion_fluid_material::PotionFluidMaterial {
                        base_color: liquid_color.to_linear(),
                        surface_color: LinearRgba::new(
                            (liquid_color.to_linear().red * 1.25).min(1.0),
                            (liquid_color.to_linear().green * 1.25).min(1.0),
                            (liquid_color.to_linear().blue * 1.25).min(1.0),
                            1.0,
                        ),
                        slosh_normal: Vec4::new(0.0, 1.0, 0.0, 0.008),
                        fluid_params: Vec4::new(0.0, 0.0018, 0.0035, 1.0),
                        bulb_center: Vec4::new(0.0, 0.0, 0.0, 0.042),
                    });
                    builder.spawn((
                        HeldPotionLiquid,
                        Mesh3d(fluid_mesh),
                        MeshMaterial3d(fluid_mat),
                        prev_slosh.unwrap_or_default(),
                        Transform::IDENTITY,
                        RenderLayers::layer(2),
                    ));
                } else {
                    builder.spawn((
                        HeldPotionLiquid,
                        Mesh3d(fluid_mesh),
                        MeshMaterial3d(liquid_mat),
                        Transform::IDENTITY,
                        RenderLayers::layer(2),
                    ));
                }

                builder.spawn((
                    Mesh3d(stopper_mesh),
                    MeshMaterial3d(stopper_mat),
                    Transform::IDENTITY,
                    RenderLayers::layer(2),
                ));

                if let Some(accent_mesh_data) = crate::alchemy::create_held_accent_mesh(active_slot) {
                    let accent_mesh = meshes.add(accent_mesh_data);
                    let (accent_color, accent_roughness, accent_metallic) = match active_slot {
                        1 => (Color::srgb(0.55, 0.40, 0.25), 0.90, 0.0),
                        2 => (Color::srgb(0.85, 0.70, 0.22), 0.30, 0.8),
                        3 => (Color::srgb(0.68, 0.52, 0.22), 0.35, 0.75),
                        _ => (Color::srgb(0.50, 0.50, 0.50), 0.50, 0.5),
                    };
                    let accent_mat = materials.add(StandardMaterial {
                        base_color: accent_color,
                        perceptual_roughness: accent_roughness,
                        metallic: accent_metallic,
                        ..default()
                    });
                    builder.spawn((
                        Mesh3d(accent_mesh),
                        MeshMaterial3d(accent_mat),
                        Transform::IDENTITY,
                        RenderLayers::layer(2),
                    ));
                }

                if active_slot == 3 {
                    let loop_mesh = meshes.add(crate::alchemy::create_ring_mesh(0.0085, 0.0018, 16, 8));
                    let loop_mat = materials.add(StandardMaterial {
                        base_color: Color::srgb(0.68, 0.52, 0.22),
                        perceptual_roughness: 0.35,
                        metallic: 0.75,
                        ..default()
                    });
                    builder.spawn((
                        Mesh3d(loop_mesh),
                        MeshMaterial3d(loop_mat),
                        Transform::from_xyz(0.0, 0.099, 0.0).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                        RenderLayers::layer(2),
                    ));
                }
            });
        } else {
            for mat_handle in liquid_query.iter() {
                if let Some(mat) = materials.get_mut(mat_handle) {
                    mat.base_color = liquid_color;
                    mat.emissive = liquid_color.to_linear() * 1.2;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locomotion_sprint_velocity_and_stamina_drain_gate() {

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

        for _ in 0..40 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_z = app.world().get::<Transform>(player).unwrap().translation.z;

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
        keys.press(KeyCode::KeyD);
        app.world_mut().insert_resource(keys);

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

        app.world_mut().spawn((
            CavernObstacle,
            ObstacleShape::Cuboid {
                half_extents: Vec3::new(0.8, 0.45, 0.35),
            },
            Transform::from_xyz(2.15, 0.5, -1.75),
        ));

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        app.world_mut().insert_resource(keys);

        for _ in 0..40 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_z = app.world().get::<Transform>(player).unwrap().translation.z;

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
        keys.press(KeyCode::KeyA);
        app.world_mut().insert_resource(keys);

        for _ in 0..40 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_x = app.world().get::<Transform>(player).unwrap().translation.x;

        assert!(
            player_x >= -6.46,
            "Player must be blocked by canyon wall: x={:.3} must be >= -6.46",
            player_x
        );
    }

    #[test]
    fn test_collision_blocks_player_at_lateral_rock_terraces() {
        let mut app = App::new();
        app.add_systems(Update, player_movement);
        app.init_resource::<Time>();

        let player = app
            .world_mut()
            .spawn((
                Player,
                PlayerState::Active,
                LocomotionState::default(),
                Transform::from_xyz(-3.8, 1.2, -3.1),
                Stamina {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        app.world_mut().spawn((
            CavernObstacle,
            ObstacleShape::Cuboid {
                half_extents: Vec3::new(1.6, 3.5, 2.1),
            },
            Transform::from_xyz(-6.1, 2.5, -3.1),
        ));

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyA);
        app.world_mut().insert_resource(keys);

        for _ in 0..40 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_x = app.world().get::<Transform>(player).unwrap().translation.x;

        assert!(
            player_x >= -4.13,
            "Player must be blocked before lateral rock terrace: x={:.3} must be >= -4.13",
            player_x
        );
    }

    #[test]
    fn test_player_traverses_portal_archway_into_dungeon() {
        let mut app = App::new();
        app.add_systems(Update, player_movement);
        app.init_resource::<Time>();

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
        keys.press(KeyCode::KeyW);
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
        keys.press(KeyCode::KeyW);
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

    #[test]
    fn test_collision_step_offset_allows_walking_over_small_rock() {
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

        app.world_mut().spawn((
            CavernObstacle,
            ObstacleShape::Cylinder {
                radius: 0.5,
                height: 0.25,
            },
            Transform::from_xyz(0.0, 0.125, -1.0),
        ));

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        app.world_mut().insert_resource(keys);

        for _ in 0..40 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let player_tf = app.world().get::<Transform>(player).unwrap();
        assert!(
            player_tf.translation.z <= -1.0,
            "Player must step over low rock and reach past it: z={:.3} must be <= -1.0",
            player_tf.translation.z
        );
    }

    #[test]
    fn test_sprint_jump_air_momentum_preservation() {
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

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::ShiftLeft);
        keys.press(KeyCode::Space);
        app.world_mut().insert_resource(keys.clone());

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));
        app.update();

        let loco = app.world().get::<LocomotionState>(player).unwrap();
        assert!(
            loco.vertical_velocity > 0.0,
            "Vertical velocity must be positive on takeoff: Vy={:.2}",
            loco.vertical_velocity
        );
        assert!(
            !loco.is_grounded,
            "Player must be airborne on takeoff"
        );
        let h_speed = loco.horizontal_velocity.length();
        assert!(
            h_speed >= 6.8 && h_speed <= 7.1,
            "Horizontal speed on takeoff must equal sprint speed (7.0): speed={:.2}",
            h_speed
        );

        keys.release(KeyCode::KeyW);
        keys.release(KeyCode::ShiftLeft);
        keys.release(KeyCode::Space);
        *app.world_mut().resource_mut::<ButtonInput<KeyCode>>() = keys.clone();

        for _ in 0..5 {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(16));
            app.update();
        }

        let airborne_loco = app.world().get::<LocomotionState>(player).unwrap();
        assert!(
            !airborne_loco.is_grounded,
            "Player must still be airborne"
        );
        let air_speed = airborne_loco.horizontal_velocity.length();
        assert!(
            air_speed >= 6.5,
            "Horizontal air momentum must be conserved even after releasing inputs: air_speed={:.2}",
            air_speed
        );
    }
}
