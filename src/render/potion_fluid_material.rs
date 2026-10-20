use crate::alchemy::{selected_potion_effects, PotionCooldowns};
use crate::player::camera::{HeldPotionLiquid, LocomotionState, Player, PlayerCamera};
use crate::ui::hud::ActiveHotbarSlot;
use bevy::{
    pbr::Material,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PotionFluidMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[uniform(1)]
    pub surface_color: LinearRgba,
    #[uniform(2)]
    pub slosh_normal: Vec4,
    #[uniform(3)]
    pub fluid_params: Vec4,
}

impl Default for PotionFluidMaterial {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::new(0.12, 0.92, 0.42, 0.85),
            surface_color: LinearRgba::new(0.35, 1.0, 0.65, 1.0),
            slosh_normal: Vec4::new(0.0, 1.0, 0.0, 0.08),
            fluid_params: Vec4::new(0.0, 0.015, 0.08, 1.0),
        }
    }
}

impl Material for PotionFluidMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/potion_fluid.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Component, Debug, Clone)]
pub struct PotionSloshState {
    pub current_tilt: Vec2,
    pub tilt_velocity: Vec2,
    pub prev_player_pos: Vec3,
    pub prev_camera_yaw: f32,
    pub prev_camera_pitch: f32,
    pub initialized: bool,
}

impl Default for PotionSloshState {
    fn default() -> Self {
        Self {
            current_tilt: Vec2::ZERO,
            tilt_velocity: Vec2::ZERO,
            prev_player_pos: Vec3::ZERO,
            prev_camera_yaw: 0.0,
            prev_camera_pitch: 0.0,
            initialized: false,
        }
    }
}

pub fn update_potion_sloshing(
    time: Res<Time>,
    player_query: Query<(&Transform, &LocomotionState), With<Player>>,
    camera_query: Query<&Transform, With<PlayerCamera>>,
    mut potion_query: Query<(&mut PotionSloshState, &MeshMaterial3d<PotionFluidMaterial>), With<HeldPotionLiquid>>,
    mut materials: ResMut<Assets<PotionFluidMaterial>>,
    active_slot_res: Option<Res<ActiveHotbarSlot>>,
    cooldowns_res: Option<Res<PotionCooldowns>>,
) {
    let dt = time.delta_secs().clamp(0.001, 0.05);
    let t = time.elapsed_secs();
    let active_slot = active_slot_res.map(|s| s.0).unwrap_or(1);
    let (base_col, _) = selected_potion_effects(active_slot);
    let is_on_cooldown = cooldowns_res
        .as_ref()
        .map(|c| !c.is_ready(active_slot))
        .unwrap_or(false);

    let base_linear = if is_on_cooldown {
        LinearRgba::new(0.35, 0.35, 0.35, 0.45)
    } else {
        base_col.to_linear()
    };

    let surface_linear = if is_on_cooldown {
        LinearRgba::new(0.55, 0.55, 0.55, 0.65)
    } else {
        LinearRgba::new(
            (base_linear.red * 1.35).min(1.0),
            (base_linear.green * 1.35).min(1.0),
            (base_linear.blue * 1.35).min(1.0),
            1.0,
        )
    };

    let (player_tf, mouse_look_extra) = player_query
        .get_single()
        .map(|(t, l)| (*t, l.mouse_look_delta))
        .unwrap_or((Transform::IDENTITY, Vec2::ZERO));

    let (cam_yaw, cam_pitch) = camera_query
        .get_single()
        .map(|ct| {
            let (y, p, _) = ct.rotation.to_euler(EulerRot::YXZ);
            (y, p)
        })
        .unwrap_or((0.0, 0.0));

    for (mut slosh, mat_handle) in potion_query.iter_mut() {
        if !slosh.initialized {
            slosh.initialized = true;
            slosh.prev_player_pos = player_tf.translation;
            slosh.prev_camera_yaw = cam_yaw;
            slosh.prev_camera_pitch = cam_pitch;
        }

        let yaw_delta = cam_yaw - slosh.prev_camera_yaw;
        let pitch_delta = cam_pitch - slosh.prev_camera_pitch;
        slosh.prev_camera_yaw = cam_yaw;
        slosh.prev_camera_pitch = cam_pitch;

        let player_pos_delta = player_tf.translation - slosh.prev_player_pos;
        slosh.prev_player_pos = player_tf.translation;

        let forward = player_tf.forward();
        let right = player_tf.right();
        let local_accel_x = player_pos_delta.dot(*right) / (dt * dt + 1e-5);
        let local_accel_z = player_pos_delta.dot(*forward) / (dt * dt + 1e-5);

        let inertial_x = -yaw_delta * 14.0 - mouse_look_extra.x * 0.025 - local_accel_x.clamp(-35.0, 35.0) * 0.012;
        let inertial_y = pitch_delta * 12.0 - mouse_look_extra.y * 0.020 - local_accel_z.clamp(-35.0, 35.0) * 0.012;

        let k_slosh: f32 = 110.0;
        let c_slosh: f32 = 13.5;
        let spring_force_x = -k_slosh * slosh.current_tilt.x - c_slosh * slosh.tilt_velocity.x;
        let spring_force_y = -k_slosh * slosh.current_tilt.y - c_slosh * slosh.tilt_velocity.y;

        slosh.tilt_velocity.x += (spring_force_x + inertial_x) * dt;
        slosh.tilt_velocity.y += (spring_force_y + inertial_y) * dt;

        let tilt_vel = slosh.tilt_velocity;
        slosh.current_tilt += tilt_vel * dt;
        slosh.current_tilt.x = slosh.current_tilt.x.clamp(-0.75, 0.75);
        slosh.current_tilt.y = slosh.current_tilt.y.clamp(-0.75, 0.75);

        if let Some(mat) = materials.get_mut(mat_handle) {
            mat.base_color = base_linear;
            mat.surface_color = surface_linear;

            let normal = Vec3::new(-slosh.current_tilt.x, 1.0, -slosh.current_tilt.y).normalize();
            let fill_level: f32 = 0.06;
            mat.slosh_normal = Vec4::new(normal.x, normal.y, normal.z, fill_level);

            let ripple_amp = (slosh.tilt_velocity.length() * 0.025 + 0.010).clamp(0.006, 0.038);
            let meniscus_thickness: f32 = 0.09;
            mat.fluid_params = Vec4::new(t, ripple_amp, meniscus_thickness, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_potion_fluid_material_default_and_shader() {
        let mat = PotionFluidMaterial::default();
        assert_eq!(mat.alpha_mode(), AlphaMode::Blend);
        match PotionFluidMaterial::fragment_shader() {
            ShaderRef::Path(path) => {
                assert_eq!(
                    path.path(),
                    std::path::Path::new("shaders/potion_fluid.wgsl")
                );
            }
            _ => panic!("Expected ShaderRef::Path for potion_fluid.wgsl"),
        }
    }

    #[test]
    fn test_potion_slosh_spring_damper_convergence() {
        let mut slosh = PotionSloshState {
            current_tilt: Vec2::new(0.5, -0.3),
            tilt_velocity: Vec2::ZERO,
            initialized: true,
            ..default()
        };

        let dt = 0.016;
        let k_slosh: f32 = 110.0;
        let c_slosh: f32 = 13.5;

        for _ in 0..120 {
            let spring_force_x = -k_slosh * slosh.current_tilt.x - c_slosh * slosh.tilt_velocity.x;
            let spring_force_y = -k_slosh * slosh.current_tilt.y - c_slosh * slosh.tilt_velocity.y;

            slosh.tilt_velocity.x += spring_force_x * dt;
            slosh.tilt_velocity.y += spring_force_y * dt;

            slosh.current_tilt += slosh.tilt_velocity * dt;
        }

        assert!(slosh.current_tilt.length() < 0.01);
        assert!(slosh.tilt_velocity.length() < 0.01);
    }
}
