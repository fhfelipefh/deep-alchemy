use bevy::prelude::*;
use avian3d::prelude::*;
use crate::{
    alchemy::{AlchemicalEffectKind, AlchemicalIngredient, ThermalGeyser},
    player::interaction::Interactable,
    render::{
        particles::{ParticleEmitter, ParticleType},
        water_material::WaterMaterial,
    },
};

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cavern_world);
    }
}

pub fn setup_cavern_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
) {
    // 1. Atmosphere & Lighting (Deep Cel-Shaded Contrast)
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.08, 0.12, 0.22),
        brightness: 120.0,
    });

    // Sun shaft / Cave skylight
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.75, 0.85, 1.0),
            illuminance: 3200.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 25.0, 15.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    // Warm amber glow from thermal vents
    commands.spawn((
        PointLight {
            color: Color::srgb(1.0, 0.55, 0.15),
            intensity: 8000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-6.0, 2.5, 2.0),
    ));

    // Cyan bioluminescent light near player spawn
    commands.spawn((
        PointLight {
            color: Color::srgb(0.1, 0.9, 1.0),
            intensity: 6500.0,
            range: 15.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(3.0, 3.0, -3.0),
    ));

    // 2. Cavern Kit Architecture (.glb)
    commands.spawn((
        SceneRoot(asset_server.load("models/cavern_kit.glb#Scene0")),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Cavern Floor with Physics Collider
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.14, 0.15, 0.18),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    // 3. THE GRAND WATERFALL BEHIND PLAYER (Spawn is at 0, 1.2, 0 looking forward +Z)
    // Waterfall is at Z = -11.0 falling from Y = 16.0 down to Y = 0.5
    let waterfall_height = 16.0;
    let waterfall_width = 8.0;

    // A. Vertical Waterfall Curtain (Rushing cascading foam)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(waterfall_width, waterfall_height))),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            base_color: LinearRgba::new(0.4, 0.85, 1.0, 0.92),
            deep_color: LinearRgba::new(0.1, 0.5, 0.8, 0.95),
            foam_color: LinearRgba::new(1.0, 1.0, 1.0, 1.0),
            params: Vec4::new(0.0, 3.5, 0.1, 0.9), // Higher speed for waterfall rush!
        })),
        Transform::from_xyz(0.0, 8.0, -11.2)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ));

    // B. Waterfall Splash / Mist Particle Emitters at base (Z = -11.0, Y = 0.5)
    for x_off in [-2.5, 0.0, 2.5] {
        commands.spawn((
            Transform::from_xyz(x_off, 0.6, -11.0),
            ParticleEmitter {
                spawn_timer: 0.0,
                spawn_rate: 0.08,
                particle_type: ParticleType::WaterSplash,
            },
        ));
    }

    // C. Stream River Lake flowing from the waterfall past player into the cavern
    // The river stretches from Z = -12.0 to Z = 25.0
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(14.0, 45.0))),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            base_color: LinearRgba::new(0.05, 0.72, 0.82, 0.85),
            deep_color: LinearRgba::new(0.01, 0.25, 0.40, 0.95),
            foam_color: LinearRgba::new(0.95, 0.98, 1.0, 0.98),
            params: Vec4::new(0.0, 1.2, 0.2, 0.8),
        })),
        Transform::from_xyz(0.0, 0.22, 5.0),
    ));

    // 4. Thermal Rim Pools & Geysers (At X = -6.0, Z = 0.0 to 6.0)
    commands.spawn((
        Transform::from_xyz(-6.0, 0.6, 2.0),
        ThermalGeyser {
            eruption_timer: 0.0,
            eruption_interval: 3.0,
            is_erupting: true,
            heat_radius: 5.5,
        },
        ParticleEmitter {
            spawn_timer: 0.0,
            spawn_rate: 0.07,
            particle_type: ParticleType::Steam,
        },
    ));

    // Second thermal vent
    commands.spawn((
        Transform::from_xyz(-7.5, 0.6, 5.5),
        ThermalGeyser {
            eruption_timer: 1.5,
            eruption_interval: 4.0,
            is_erupting: false,
            heat_radius: 4.5,
        },
        ParticleEmitter {
            spawn_timer: 0.0,
            spawn_rate: 0.12,
            particle_type: ParticleType::Steam,
        },
    ));

    // 5. Interactive Alchemy Campsite (Right of player: X = 2.5, Z = 2.0)
    commands.spawn((
        SceneRoot(asset_server.load("models/alchemy_props.glb#Scene0")),
        Transform::from_xyz(2.0, 0.0, 2.0),
    ));

    // Spore emitter around glowing mushroom
    commands.spawn((
        Transform::from_xyz(4.2, 0.8, 2.0),
        ParticleEmitter {
            spawn_timer: 0.0,
            spawn_rate: 0.18,
            particle_type: ParticleType::GlowSpore,
        },
    ));

    // Harvestable Cyan Glow Mushroom
    commands.spawn((
        Interactable { label: "Cyan Glow Mushroom" },
        AlchemicalIngredient {
            id: "ing_cyan_spore".to_string(),
            name: "Glow Spore".to_string(),
            primary_effect: AlchemicalEffectKind::Luminescence,
            secondary_effect: AlchemicalEffectKind::Swiftness,
            harvested: false,
        },
        Transform::from_xyz(4.2, 0.5, 2.0),
        GlobalTransform::default(),
        Collider::cylinder(0.4, 0.8),
    ));

    // Collectible Potion Flasks
    commands.spawn((
        Interactable { label: "Potion of Swiftness" },
        Transform::from_xyz(2.0, 0.4, 2.0),
        GlobalTransform::default(),
        RigidBody::Dynamic,
        Collider::cylinder(0.15, 0.35),
    ));

    commands.spawn((
        Interactable { label: "Potion of Vitality" },
        Transform::from_xyz(2.5, 0.4, 2.0),
        GlobalTransform::default(),
        RigidBody::Dynamic,
        Collider::cylinder(0.15, 0.35),
    ));

    info!("Deep Cavern Grotto initialized with Grand Waterfall, Thermal Geysers, and River Stream!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_plugin_build() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(DungeonPlugin);
        assert!(app.is_plugin_added::<DungeonPlugin>());
    }

    #[test]
    fn test_setup_cavern_world_spawns_environment() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<bevy::scene::Scene>();
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.init_asset::<WaterMaterial>();
        app.add_systems(Startup, setup_cavern_world);
        app.update();

        // Verify grand waterfall and river elements were spawned
        assert!(app.world().resource::<AmbientLight>().brightness > 0.0);
        let water_plane_count = app.world_mut().query::<&MeshMaterial3d<WaterMaterial>>().iter(app.world()).count();
        assert!(water_plane_count >= 2, "Grand waterfall curtain and river stream must be spawned");

        let geyser_count = app.world_mut().query::<&ThermalGeyser>().iter(app.world()).count();
        assert_eq!(geyser_count, 2, "Thermal geysers must be spawned");
    }
}
