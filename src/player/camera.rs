use avian3d::prelude::*;
use bevy::{
    prelude::*,
    render::view::RenderLayers,
    window::{CursorGrabMode, PrimaryWindow},
};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct HeadHatPart;

#[derive(Component, Debug, PartialEq, Eq)]
pub enum PlayerState {
    Active,
    Dead,
}

#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct CameraShake {
    pub trauma: f32,
    pub direction: Vec3,
    pub timer: f32,
}

impl Default for CameraShake {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            direction: Vec3::ZERO,
            timer: 0.0,
        }
    }
}

impl CameraShake {
    pub fn add_trauma(&mut self, amount: f32, direction: Vec3) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
        if direction.length_squared() > 1e-4 {
            let dir_norm = direction.normalize();
            self.direction = if self.direction.length_squared() > 1e-4 {
                (self.direction + dir_norm).normalize()
            } else {
                dir_norm
            };
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct LocomotionState {
    pub is_sprinting: bool,
    pub is_crouching: bool,
    pub is_grounded: bool,
    pub vertical_velocity: f32,
    pub stamina_regen_delay_timer: f32,
    pub head_bob_timer: f32,
    pub current_speed: f32,
    pub horizontal_velocity: Vec2,
    pub is_in_water: bool,
    pub is_frozen: bool,
    pub freeze_timer: f32,
    pub is_on_ice: bool,
    pub footstep_side: bool,
    pub distance_since_footprint: f32,
    pub distance_since_footstep: f32,
    pub coyote_timer: f32,
    pub jump_buffer_timer: f32,
    pub camera_landing_dip: f32,
    pub squash_scale: Vec3,
    pub takeoff_timer: f32,
    pub last_fall_velocity: f32,
    pub camera_roll: f32,
    pub target_camera_roll: f32,
    pub throw_windup: f32,
    pub throw_recoil: f32,
}

impl Default for LocomotionState {
    fn default() -> Self {
        Self {
            is_sprinting: false,
            is_crouching: false,
            is_grounded: true,
            vertical_velocity: 0.0,
            stamina_regen_delay_timer: 0.0,
            head_bob_timer: 0.0,
            current_speed: 4.2,
            horizontal_velocity: Vec2::ZERO,
            is_in_water: false,
            is_frozen: false,
            freeze_timer: 0.0,
            is_on_ice: false,
            footstep_side: false,
            distance_since_footprint: 0.0,
            distance_since_footstep: 0.0,
            coyote_timer: 0.0,
            jump_buffer_timer: 0.0,
            camera_landing_dip: 0.0,
            squash_scale: Vec3::ONE,
            takeoff_timer: 0.0,
            last_fall_velocity: 0.0,
            camera_roll: 0.0,
            target_camera_roll: 0.0,
            throw_windup: 0.0,
            throw_recoil: 0.0,
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct BoneJoint {
    pub name: String,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct RagdollLimb {
    pub limb_name: String,
}

pub fn grab_mouse(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

pub fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {

    let player_id = commands
        .spawn((
            Player,
            PlayerState::Active,
            LocomotionState::default(),
            crate::alchemy::EffectReceiverComponent::default(),
            Health {
                current: 100.0,
                max: 100.0,
            },
            Stamina {
                current: 100.0,
                max: 100.0,
            },
            RigidBody::Kinematic,
            Collider::capsule(0.35, 1.1),
            Transform::from_xyz(0.0, 1.2, 0.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerCamera,
                Camera3d::default(),

                Msaa::Off,
                bevy::core_pipeline::prepass::DepthPrepass,
                bevy::core_pipeline::prepass::NormalPrepass,
                crate::render::outline::OutlineSettings::default(),
                bevy::core_pipeline::bloom::Bloom::NATURAL,
                RenderLayers::layer(0),
                Transform::from_xyz(0.0, 0.55, 0.0),
                DistanceFog {
                    color: Color::srgba(0.05, 0.09, 0.16, 1.0),
                    falloff: FogFalloff::ExponentialSquared { density: 0.008 },
                    ..default()
                },
            ));

            parent.spawn((
                PlayerBodyScene,
                SceneRoot(asset_server.load("models/player.glb#Scene0")),
                Transform::from_xyz(0.0, -0.9, 0.0),
                Visibility::Hidden,
                RenderLayers::layer(1),
            ));

            let joints = [
                ("Spine", Vec3::new(0.0, 0.0, 0.0)),
                ("Chest", Vec3::new(0.0, 0.35, 0.0)),
                ("Head", Vec3::new(0.0, 0.65, 0.0)),
                ("LeftArm", Vec3::new(-0.35, 0.35, 0.0)),
                ("RightArm", Vec3::new(0.35, 0.35, 0.0)),
                ("LeftLeg", Vec3::new(-0.18, -0.45, 0.0)),
                ("RightLeg", Vec3::new(0.18, -0.45, 0.0)),
                ("Backpack", Vec3::new(0.0, 0.3, -0.22)),
                ("AlchemyPouch", Vec3::new(0.22, 0.0, 0.15)),
            ];

            for (joint_name, local_pos) in joints {
                let mut joint_cmd = parent.spawn((
                    BoneJoint {
                        name: joint_name.to_string(),
                    },
                    RagdollLimb {
                        limb_name: joint_name.to_string(),
                    },
                    Transform::from_translation(local_pos),
                    GlobalTransform::default(),
                ));

                if joint_name == "Head" {

                    joint_cmd.insert((HeadHatPart, RenderLayers::layer(1)));
                }
            }
        })
        .id();

    info!("Player spawned with full-body awareness at {:?}", player_id);
}

#[derive(Component)]
pub struct PlayerBodyScene;

pub fn configure_player_model_layers(
    mut commands: Commands,
    player_query: Query<&PlayerState, With<Player>>,
    mut body_query: Query<(Entity, &mut Visibility), With<PlayerBodyScene>>,
    children_query: Query<&Children>,
    names: Query<(Entity, &Name), Added<Name>>,
) {
    if let Ok(state) = player_query.get_single() {
        let target_vis = match state {
            PlayerState::Active => Visibility::Hidden,
            PlayerState::Dead => Visibility::Inherited,
        };

        for (body_entity, mut vis) in body_query.iter_mut() {
            if *vis != target_vis {
                *vis = target_vis;
            }
            apply_layer_to_descendants(body_entity, RenderLayers::layer(1), &mut commands, &children_query);
        }
    }

    for (entity, name) in names.iter() {
        let n = name.as_str();
        if n.contains("Head_Hat") || n.contains("Hat_") {
            commands.entity(entity).insert((
                HeadHatPart,
                RenderLayers::layer(1),
            ));
        } else if n.contains("Arm")
            || n.contains("Hand")
            || n.contains("Leg")
            || n.contains("Boot")
            || n.contains("Foot")
            || n.contains("Torso")
            || n.contains("Vest")
            || n.contains("CoatTail")
            || n.contains("Thigh")
            || n.contains("Shin")
            || n.contains("Backpack")
            || n.contains("Belt")
            || n.contains("Pouch")
            || n.contains("Strap")
            || n.contains("Player_")
        {
            commands.entity(entity).insert((
                RenderLayers::layer(1),
            ));
        }
    }
}

fn apply_layer_to_descendants(
    entity: Entity,
    layer: RenderLayers,
    commands: &mut Commands,
    children_query: &Query<&Children>,
) {
    commands.entity(entity).insert(layer.clone());
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            apply_layer_to_descendants(*child, layer.clone(), commands, children_query);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_player_model_layers_assigns_layer_1_to_hat() {
        let mut app = App::new();
        app.add_systems(Update, configure_player_model_layers);

        let hat_entity = app.world_mut().spawn(Name::new("Head_Hat")).id();
        let coat_entity = app.world_mut().spawn(Name::new("Player_Coat")).id();

        app.update();

        assert!(app.world().get::<HeadHatPart>(hat_entity).is_some());
        let hat_layer = app.world().get::<RenderLayers>(hat_entity).unwrap();
        assert!(!hat_layer.intersects(&RenderLayers::layer(0)));
        assert!(hat_layer.intersects(&RenderLayers::layer(1)));

        assert!(app.world().get::<HeadHatPart>(coat_entity).is_none());
    }

    #[test]
    fn test_grab_mouse_locks_primary_window() {
        let mut app = App::new();
        app.add_systems(Update, grab_mouse);

        let window_entity = app
            .world_mut()
            .spawn((
                PrimaryWindow,
                Window {
                    cursor_options: bevy::window::CursorOptions {
                        grab_mode: CursorGrabMode::None,
                        visible: true,
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        app.update();

        let window = app.world().get::<Window>(window_entity).unwrap();
        assert_eq!(window.cursor_options.grab_mode, CursorGrabMode::Locked);
        assert!(!window.cursor_options.visible);
    }

    #[test]
    fn test_setup_player_spawns_player_hierarchy() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<bevy::scene::Scene>();
        app.add_systems(Startup, setup_player);
        app.update();

        let player_count = app.world_mut().query::<&Player>().iter(app.world()).count();
        assert_eq!(player_count, 1, "Player root entity must be spawned");

        let camera_count = app
            .world_mut()
            .query::<&PlayerCamera>()
            .iter(app.world())
            .count();
        assert_eq!(camera_count, 1, "Player 1P camera must be spawned");

        let bone_count = app
            .world_mut()
            .query::<&BoneJoint>()
            .iter(app.world())
            .count();
        assert!(
            bone_count >= 8,
            "Player hierarchy must include skeleton bone joints for full body awareness"
        );

        let has_locomotion = app
            .world_mut()
            .query::<(&Player, &LocomotionState)>()
            .iter(app.world())
            .count()
            == 1;
        assert!(has_locomotion, "Player must possess LocomotionState");
    }
}
