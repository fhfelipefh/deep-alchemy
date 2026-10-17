use bevy::{
    prelude::*,
    render::view::RenderLayers,
    window::{CursorGrabMode, PrimaryWindow},
};
use avian3d::prelude::*;

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

pub fn grab_mouse(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

pub fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Spawn Player Root with Avian3D physics controller
    let player_id = commands
        .spawn((
            Player,
            PlayerState::Active,
            Health { current: 100.0, max: 100.0 },
            Stamina { current: 100.0, max: 100.0 },
            RigidBody::Kinematic,
            Collider::capsule(0.35, 1.1),
            Transform::from_xyz(0.0, 1.2, 0.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Spawn 1st Person Camera at eye height (1.75m)
            parent.spawn((
                PlayerCamera,
                Camera3d::default(),
                RenderLayers::layer(0),
                Transform::from_xyz(0.0, 0.55, 0.0),
                DistanceFog {
                    color: Color::srgba(0.04, 0.08, 0.12, 1.0),
                    falloff: FogFalloff::ExponentialSquared { density: 0.015 },
                    ..default()
                },
            ));

            // Load Player Model (Full Body)
            parent.spawn((
                SceneRoot(asset_server.load("models/player.glb#Scene0")),
                Transform::from_xyz(0.0, -0.9, 0.0),
                RenderLayers::layer(0),
            ));
        })
        .id();

    info!("Player spawned with full-body awareness at {:?}", player_id);
}

// Ensure Head_Hat part gets RenderLayers::layer(1) to avoid 1P clipping
pub fn configure_player_model_layers(
    mut commands: Commands,
    names: Query<(Entity, &Name), Added<Name>>,
) {
    for (entity, name) in names.iter() {
        if name.as_str().contains("Head_Hat") || name.as_str().contains("Hat_") {
            commands.entity(entity).insert((
                HeadHatPart,
                RenderLayers::layer(1), // Invisible to 1P camera (layer 0)
            ));
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

        // Hat must have received HeadHatPart and RenderLayers::layer(1)
        assert!(app.world().get::<HeadHatPart>(hat_entity).is_some());
        let hat_layer = app.world().get::<RenderLayers>(hat_entity).unwrap();
        assert!(!hat_layer.intersects(&RenderLayers::layer(0)));
        assert!(hat_layer.intersects(&RenderLayers::layer(1)));

        // Coat must NOT have HeadHatPart
        assert!(app.world().get::<HeadHatPart>(coat_entity).is_none());
    }

    #[test]
    fn test_grab_mouse_locks_primary_window() {
        let mut app = App::new();
        app.add_systems(Update, grab_mouse);

        let window_entity = app.world_mut().spawn((
            PrimaryWindow,
            Window {
                cursor_options: bevy::window::CursorOptions {
                    grab_mode: CursorGrabMode::None,
                    visible: true,
                    ..default()
                },
                ..default()
            },
        )).id();

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

        let camera_count = app.world_mut().query::<&PlayerCamera>().iter(app.world()).count();
        assert_eq!(camera_count, 1, "Player 1P camera must be spawned");
    }
}
