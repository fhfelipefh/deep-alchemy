use crate::{
    dungeon::{asset_loader::{SceneAssetInstance, SceneAssetRegistry}, modular_kit::{CavernObstacle, ObstacleShape}},
    player::camera::Player,
    ui::menu::GameState,
};
use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

pub const SECRET_TEST_ROOM_ORIGIN: Vec3 = Vec3::new(0.0, 0.0, 512.0);

#[derive(Resource, Debug, Default)]
pub struct SecretTestRoomRequest {
    pub requested: bool,
    pub active: bool,
}

#[derive(Component)]
pub struct SecretTestRoom;

pub struct SecretTestRoomPlugin;

impl Plugin for SecretTestRoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SecretTestRoomRequest>()
            .add_systems(OnEnter(GameState::InGame), spawn_secret_test_room)
            .add_systems(Update, adapt_secret_room_environment);
    }
}

fn spawn_secret_test_room(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<SceneAssetRegistry>,
    mut request: ResMut<SecretTestRoomRequest>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    if !request.requested || request.active {
        return;
    }

    request.active = true;
    if let Ok(mut transform) = player.get_single_mut() {
        transform.translation = SECRET_TEST_ROOM_ORIGIN + Vec3::new(0.0, 1.2, 0.0);
        transform.rotation = Quat::IDENTITY;
    }

    commands.spawn((
        SecretTestRoom,
        Name::new("Secret_Test_Room"),
        SceneAssetInstance {
            asset_key: "secret_white_test_room".to_string(),
        },
        SceneRoot(registry.load_scene(&asset_server, "secret_white_test_room")),
        Transform::from_translation(SECRET_TEST_ROOM_ORIGIN),
    ));

    for (name, offset, dimensions, blocks_player) in [
        ("Secret_Test_Floor", Vec3::new(0.0, -0.25, 0.0), Vec3::new(80.0, 0.5, 80.0), false),
        ("Secret_Test_Ceiling", Vec3::new(0.0, 14.25, 0.0), Vec3::new(80.0, 0.5, 80.0), false),
        ("Secret_Test_North", Vec3::new(0.0, 7.0, -40.25), Vec3::new(80.0, 14.0, 0.5), true),
        ("Secret_Test_South", Vec3::new(0.0, 7.0, 40.25), Vec3::new(80.0, 14.0, 0.5), true),
        ("Secret_Test_West", Vec3::new(-40.25, 7.0, 0.0), Vec3::new(0.5, 14.0, 80.0), true),
        ("Secret_Test_East", Vec3::new(40.25, 7.0, 0.0), Vec3::new(0.5, 14.0, 80.0), true),
    ] {
        let mut entity = commands.spawn((
            SecretTestRoom,
            Name::new(name),
            Transform::from_translation(SECRET_TEST_ROOM_ORIGIN + offset),
            RigidBody::Static,
            Collider::cuboid(dimensions.x, dimensions.y, dimensions.z),
        ));
        if blocks_player {
            entity.insert((
                CavernObstacle,
                ObstacleShape::Cuboid {
                    half_extents: dimensions * 0.5,
                },
            ));
        }
    }

    commands.insert_resource(ClearColor(Color::srgb(0.96, 0.96, 0.96)));
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 950.0,
    });

    for light_offset in [
        Vec3::new(0.0, 11.5, 0.0),
        Vec3::new(-20.0, 10.0, -20.0),
        Vec3::new(20.0, 10.0, -20.0),
        Vec3::new(-20.0, 10.0, 20.0),
        Vec3::new(20.0, 10.0, 20.0),
    ] {
        commands.spawn((
            SecretTestRoom,
            PointLight {
                color: Color::WHITE,
                intensity: 1_800_000.0,
                range: 110.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(SECRET_TEST_ROOM_ORIGIN + light_offset),
        ));
    }
}

fn adapt_secret_room_environment(
    request: Res<SecretTestRoomRequest>,
    mut commands: Commands,
    camera_query: Query<Entity, With<crate::player::camera::PlayerCamera>>,
    mut ambient_light: Option<ResMut<AmbientLight>>,
    mut dir_lights: Query<&mut DirectionalLight>,
) {
    if !request.active {
        return;
    }
    for entity in camera_query.iter() {
        commands.entity(entity).remove::<DistanceFog>();
    }
    if let Some(ref mut ambient) = ambient_light {
        ambient.color = Color::WHITE;
        ambient.brightness = 2200.0;
    }
    for mut dir in dir_lights.iter_mut() {
        dir.illuminance = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{asset::AssetPlugin, scene::ScenePlugin, state::app::StatesPlugin};

    #[test]
    fn requested_room_spawns_visual_shell_and_physical_boundaries() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin::default(), StatesPlugin));
        app.init_resource::<SceneAssetRegistry>();
        app.init_resource::<SecretTestRoomRequest>();
        app.init_state::<GameState>();
        app.add_systems(OnEnter(GameState::InGame), spawn_secret_test_room);

        let player = app.world_mut().spawn((Player, Transform::default())).id();
        app.world_mut().resource_mut::<SecretTestRoomRequest>().requested = true;
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::InGame);
        app.update();

        let spawned = app.world_mut().query::<&SecretTestRoom>().iter(app.world()).count();
        assert_eq!(spawned, 12);
        assert_eq!(
            app.world().get::<Transform>(player).unwrap().translation,
            SECRET_TEST_ROOM_ORIGIN + Vec3::new(0.0, 1.2, 0.0)
        );
    }
}
