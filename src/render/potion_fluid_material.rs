use crate::alchemy::{selected_potion_effects, PotionCooldowns};
use crate::player::camera::{FirstPersonHeldPotion, HeldPotionLiquid, LocomotionState, Player, PlayerCamera};
use crate::ui::hud::ActiveHotbarSlot;
use bevy::{
    pbr::{Material, MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError},
    },
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
    #[uniform(4)]
    pub bulb_center: Vec4,
}

impl Default for PotionFluidMaterial {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::new(0.12, 0.92, 0.42, 0.85),
            surface_color: LinearRgba::new(0.35, 1.0, 0.65, 1.0),
            slosh_normal: Vec4::new(0.0, 1.0, 0.0, 0.010),
            fluid_params: Vec4::new(0.0, 0.003, 0.0045, 1.0),
            bulb_center: Vec4::new(0.24, -0.20, -0.42, 0.040),
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

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
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
    viewmodel_query: Query<&Transform, With<FirstPersonHeldPotion>>,
    mut potion_query: Query<(
        Option<&GlobalTransform>,
        &mut PotionSloshState,
        &MeshMaterial3d<PotionFluidMaterial>,
    ), With<HeldPotionLiquid>>,
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

    let (player_tf, locomotion) = player_query
        .get_single()
        .map(|(t, l)| (*t, l.clone()))
        .unwrap_or((Transform::IDENTITY, LocomotionState::default()));

    let (player_yaw, _, _) = player_tf.rotation.to_euler(EulerRot::YXZ);
    let (cam_yaw, cam_pitch) = camera_query
        .get_single()
        .map(|ct| {
            let (y, p, _) = ct.rotation.to_euler(EulerRot::YXZ);
            (y, p)
        })
        .unwrap_or((0.0, 0.0));
    let effective_yaw = player_yaw + cam_yaw;

    for (global_tf_opt, mut slosh, mat_handle) in potion_query.iter_mut() {
        if !slosh.initialized {
            slosh.initialized = true;
            slosh.prev_player_pos = player_tf.translation;
            slosh.prev_camera_yaw = effective_yaw;
            slosh.prev_camera_pitch = cam_pitch;
        }

        let raw_yaw_delta = effective_yaw - slosh.prev_camera_yaw;
        let yaw_delta = (raw_yaw_delta + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
        let pitch_delta = cam_pitch - slosh.prev_camera_pitch;
        slosh.prev_camera_yaw = effective_yaw;
        slosh.prev_camera_pitch = cam_pitch;

        let player_pos_delta = player_tf.translation - slosh.prev_player_pos;
        slosh.prev_player_pos = player_tf.translation;

        let cam_right = Quat::from_rotation_y(effective_yaw) * Vec3::X;
        let cam_forward = Quat::from_rotation_y(effective_yaw) * Vec3::NEG_Z;

        let local_vel_x = player_pos_delta.dot(cam_right) / dt;
        let local_vel_z = player_pos_delta.dot(cam_forward) / dt;

        let bob_impact = if locomotion.head_bob_timer > 0.0 {
            (locomotion.head_bob_timer * 2.0).sin() * 0.08 * locomotion.head_bob_intensity
        } else {
            0.0
        };

        let landing_impact = if locomotion.camera_landing_dip > 0.005 {
            (locomotion.camera_landing_dip * 3.5).min(0.4)
        } else {
            0.0
        };

        let inertial_x = -yaw_delta * 14.0 - locomotion.mouse_look_delta.x * 0.025 - local_vel_x * 0.04;
        let inertial_y = pitch_delta * 12.0 - locomotion.mouse_look_delta.y * 0.020 - local_vel_z * 0.04 + bob_impact + landing_impact;

        let k_slosh: f32 = 110.0;
        let c_slosh: f32 = 13.5;
        let spring_force_x = -k_slosh * slosh.current_tilt.x - c_slosh * slosh.tilt_velocity.x;
        let spring_force_y = -k_slosh * slosh.current_tilt.y - c_slosh * slosh.tilt_velocity.y;

        slosh.tilt_velocity.x += (spring_force_x + inertial_x) * dt;
        slosh.tilt_velocity.y += (spring_force_y + inertial_y) * dt;

        let tilt_vel = slosh.tilt_velocity;
        slosh.current_tilt += tilt_vel * dt;
        slosh.current_tilt.x = slosh.current_tilt.x.clamp(-0.55, 0.55);
        slosh.current_tilt.y = slosh.current_tilt.y.clamp(-0.55, 0.55);

        if let Some(mat) = materials.get_mut(mat_handle) {
            mat.base_color = base_linear;
            mat.surface_color = surface_linear;

            let world_up = Vec3::Y;
            let slosh_world_normal = (world_up + cam_right * slosh.current_tilt.x + cam_forward * slosh.current_tilt.y).normalize();
            let fill_level: f32 = 0.010;
            mat.slosh_normal = Vec4::new(slosh_world_normal.x, slosh_world_normal.y, slosh_world_normal.z, fill_level);

            let ripple_amp = (slosh.tilt_velocity.length() * 0.0025 + 0.0012).clamp(0.0008, 0.0045);
            let meniscus_thickness: f32 = 0.0045;
            mat.fluid_params = Vec4::new(t, ripple_amp, meniscus_thickness, 1.0);

            let calculated_center = {
                let cam_rot = camera_query
                    .get_single()
                    .map(|ct| ct.rotation)
                    .unwrap_or(Quat::IDENTITY);
                let cam_pos = camera_query
                    .get_single()
                    .map(|ct| ct.translation)
                    .unwrap_or(Vec3::new(0.0, 0.55, 0.0));
                let sway_pos = viewmodel_query
                    .get_single()
                    .map(|st| st.translation)
                    .unwrap_or(Vec3::new(0.24, -0.20, -0.42));
                player_tf.translation + player_tf.rotation * (cam_pos + cam_rot * sway_pos)
            };

            let center = global_tf_opt
                .map(|gt| gt.translation())
                .filter(|pos| pos.length_squared() > 1e-4)
                .unwrap_or(calculated_center);
            mat.bulb_center = Vec4::new(center.x, center.y, center.z, 0.040);
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
