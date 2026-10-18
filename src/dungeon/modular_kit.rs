use crate::{
    alchemy::{AlchemicalEffectKind, AlchemicalIngredient, ThermalGeyser},
    player::interaction::Interactable,
    render::{
        cel_moss_material::CelShadedMossMaterial,
        particles::{ParticleEmitter, ParticleType},
        water_material::WaterMaterial,
    },
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Component)]
pub struct WaterfallMarker;

#[derive(Component)]
pub struct KeyShadowLight;

#[derive(Component, Clone)]
pub struct WaterfallBaseMesh {
    pub base_positions: Vec<[f32; 3]>,
    pub base_normals: Vec<[f32; 3]>,
}

#[derive(Component)]
pub struct CavernObstacle;

/// Marks imported tall-grass clumps that must remain clear of solid scenery.
/// The marker is assigned from the authored mesh name so old and newly exported
/// grotto kits obey the same placement rule.
#[derive(Component)]
pub struct TallGrassClump;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum ObstacleShape {
    Cylinder { radius: f32, height: f32 },
    Cuboid { half_extents: Vec3 },
}

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_cavern_world,
                crate::dungeon::generator::spawn_procedural_dungeon,
            ),
        )
        .add_systems(
            Update,
            (
                animate_waterfall_mesh,
                mark_imported_tall_grass,
                keep_tall_grass_clear_of_obstacles,
            ),
        );
    }
}

/// Identifies grass clumps from the imported authoring scene. Keeping this at
/// the scene boundary avoids fragile, one-off coordinate edits for individual
/// blades or benches.
pub fn mark_imported_tall_grass(
    mut commands: Commands,
    named_entities: Query<(Entity, &Name), Added<Name>>,
) {
    for (entity, name) in &named_entities {
        if name.as_str().contains("TallGrass") {
            commands.entity(entity).insert(TallGrassClump);
        }
    }
}

/// Repositions a clump by the minimum horizontal distance needed to clear every
/// tall obstacle. This is an invariant, not a bench-specific workaround:
/// any newly registered `CavernObstacle` automatically becomes vegetation-free.
pub fn keep_tall_grass_clear_of_obstacles(
    mut grass_query: Query<(&mut Transform, &GlobalTransform), With<TallGrassClump>>,
    obstacles: Query<
        (&GlobalTransform, &ObstacleShape),
        (With<CavernObstacle>, Without<TallGrassClump>),
    >,
) {
    const GRASS_RADIUS: f32 = 0.34;
    const CLEARANCE: f32 = 0.16;

    for (mut grass_transform, grass_global) in &mut grass_query {
        let grass_pos = grass_global.translation();
        let mut corrected = Vec2::new(grass_pos.x, grass_pos.z);

        // A few passes resolve compound props such as the bench without placing
        // the grass into another leg, rock, or mushroom immediately afterward.
        for _ in 0..6 {
            let mut moved = false;
            for (obstacle_global, shape) in &obstacles {
                let obstacle_transform = obstacle_global.compute_transform();
                let top = match shape {
                    ObstacleShape::Cylinder { height, .. } => {
                        obstacle_transform.translation.y + height * 0.5
                    }
                    ObstacleShape::Cuboid { half_extents } => {
                        obstacle_transform.translation.y + half_extents.y
                    }
                };

                // Low pebbles can coexist with grass; props that rise into the
                // clump reserve their entire horizontal footprint.
                if top <= grass_pos.y + 0.12 {
                    continue;
                }

                let world_delta = Vec3::new(
                    corrected.x - obstacle_transform.translation.x,
                    0.0,
                    corrected.y - obstacle_transform.translation.z,
                );
                let local_delta = obstacle_transform.rotation.inverse() * world_delta;
                let local = Vec2::new(local_delta.x, local_delta.z);

                let push_local = match shape {
                    ObstacleShape::Cylinder { radius, .. } => {
                        let minimum = radius + GRASS_RADIUS + CLEARANCE;
                        let distance = local.length();
                        if distance < minimum {
                            let direction = if distance > 0.0001 {
                                local / distance
                            } else {
                                Vec2::X
                            };
                            Some(direction * (minimum - distance))
                        } else {
                            None
                        }
                    }
                    ObstacleShape::Cuboid { half_extents } => {
                        let half_x = half_extents.x + GRASS_RADIUS + CLEARANCE;
                        let half_z = half_extents.z + GRASS_RADIUS + CLEARANCE;
                        if local.x.abs() < half_x && local.y.abs() < half_z {
                            let to_x = half_x - local.x.abs();
                            let to_z = half_z - local.y.abs();
                            if to_x < to_z {
                                let direction = if local.x >= 0.0 { 1.0 } else { -1.0 };
                                Some(Vec2::new(direction * to_x, 0.0))
                            } else {
                                let direction = if local.y >= 0.0 { 1.0 } else { -1.0 };
                                Some(Vec2::new(0.0, direction * to_z))
                            }
                        } else {
                            None
                        }
                    }
                };

                if let Some(push) = push_local {
                    let world_push = obstacle_transform.rotation * Vec3::new(push.x, 0.0, push.y);
                    corrected += Vec2::new(world_push.x, world_push.z);
                    moved = true;
                }
            }
            if !moved {
                break;
            }
        }

        let correction = corrected - Vec2::new(grass_pos.x, grass_pos.z);
        if correction.length_squared() > 0.000001 {
            // The imported grotto root is untransformed, so its child local
            // space matches world X/Z. This preserves authored blade geometry.
            grass_transform.translation.x += correction.x;
            grass_transform.translation.z += correction.y;
        }
    }
}

pub fn setup_cavern_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    mut moss_materials: ResMut<Assets<CelShadedMossMaterial>>,
) {
    // -------------------------------------------------------------
    // 1. ATMOSPHERE & LIGHTING (RICH NIGHT BLUE - NO PITCH BLACK #000000)
    // -------------------------------------------------------------
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.10, 0.20, 0.32), // Deep fantasy night blue
        brightness: 480.0,
    });

    // Soft skylight beam descending from the upper fissure over the waterfall
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.75, 0.92, 1.0),
            illuminance: 6200.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(1.5, 22.0, -4.5).looking_at(Vec3::new(0.0, 1.2, -8.5), Vec3::Y),
    ));

    // Pathway entrance ambient lantern/crystal light (X = 0.0, Y = 1.8, Z = 0.8) - warms the path and bench!
    commands.spawn((
        PointLight {
            color: Color::srgb(0.96, 0.84, 0.48), // Warm alchemical golden glow
            intensity: 36000.0,
            range: 15.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1.8, 0.8),
    ));

    // Foreground Bench & Grass Accent Light (X = 2.8, Y = 1.3, Z = -0.5)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.20, 0.94, 0.88), // Bioluminescent turquoise rim light
            intensity: 26000.0,
            range: 11.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(2.8, 1.3, -0.5),
    ));

    // Back cliff illuminator directly behind waterfall (X = 0.0, Y = 5.5, Z = -11.2) - shines through the water!
    commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.70, 0.98), // Radiant azure backlight
            intensity: 85000.0,
            range: 28.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 5.5, -11.2),
    ));

    // --- Bioluminescent PointLights (Lateral Margins, Stalactites, Waterfall & Amphitheater) ---
    // Left Mushroom cluster light (X = -3.4, Y = 2.4, Z = -2.8) - Key Shadow Light
    commands.spawn((
        PointLight {
            color: Color::srgb(0.12, 0.96, 0.88), // Electric turquoise #2be8d8
            intensity: 46000.0,
            range: 16.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.4, 2.4, -2.8),
        KeyShadowLight,
    ));

    // Right Mushroom cluster light (X = 4.1, Y = 2.4, Z = -3.3)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.12, 0.96, 0.88),
            intensity: 46000.0,
            range: 16.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.1, 2.4, -3.3),
    ));

    // Waterfall luminous foam and spray bounce light - Key Shadow Light
    commands.spawn((
        PointLight {
            color: Color::srgb(0.35, 0.90, 1.0),
            intensity: 55000.0,
            range: 20.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1.8, -10.2),
        KeyShadowLight,
    ));

    // Upper waterfall cleft / skylight glow
    commands.spawn((
        PointLight {
            color: Color::srgb(0.40, 0.95, 1.0),
            intensity: 65000.0,
            range: 26.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 13.5, -11.2),
    ));

    // Back cliff amphitheater fill light (blue slate - eliminates pitch black void)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.24, 0.38, 0.55),
            intensity: 85000.0,
            range: 35.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 8.0, -11.5),
    ));

    // Overhead ceiling arch rim lights (accentuates sharp hanging stalactites)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.45, 0.65),
            intensity: 42000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.8, 4.2, -4.2),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.45, 0.65),
            intensity: 42000.0,
            range: 18.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(3.8, 4.2, -4.2),
    ));

    // Bioluminescent pool cyan glow from crystalline waters
    commands.spawn((
        PointLight {
            color: Color::srgb(0.18, 0.92, 0.98),
            intensity: 45000.0,
            range: 22.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1.8, -8.5),
    ));

    // High vaulted cavern ceiling fill light (illuminates dome & upper stalactites)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.20, 0.45, 0.65),
            intensity: 75000.0,
            range: 32.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 18.0, -5.5),
    ));

    // Left high vault bioluminescent lichen node glow
    commands.spawn((
        PointLight {
            color: Color::srgb(0.17, 0.95, 0.88),
            intensity: 45000.0,
            range: 22.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-4.5, 17.0, -7.0),
    ));

    // Right high vault bioluminescent lichen node glow
    commands.spawn((
        PointLight {
            color: Color::srgb(0.17, 0.95, 0.88),
            intensity: 45000.0,
            range: 22.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(4.5, 17.0, -7.0),
    ));

    // High ceiling apex cleft skylight glow
    commands.spawn((
        PointLight {
            color: Color::srgb(0.55, 0.95, 1.0),
            intensity: 95000.0,
            range: 36.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 21.0, -9.5),
    ));

    // -------------------------------------------------------------
    // 2. MASTERWORK CAVERN KIT SCENE (.glb)
    // -------------------------------------------------------------
    commands.spawn((
        SceneRoot(asset_server.load("models/cavern_kit.glb#Scene0")),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Hand-built faceted cliff faces sit in front of the broad kit walls. The
    // displaced cells create horizontal strata, recessed ruts, and diagonal
    // fractures that keep the waterfall amphitheater from reading as a flat slab.
    let cliff_material = moss_materials.add(CelShadedMossMaterial {
        base_color: LinearRgba::new(0.13, 0.21, 0.25, 1.0),
        moss_color: LinearRgba::new(0.15, 0.50, 0.27, 1.0),
        light_dir: Vec4::new(0.25, 0.85, -0.45, 0.0),
        moss_params: Vec4::new(0.52, 2.8, 0.18, 3.0),
    });
    for (name, x, seed) in [
        ("Striated_Cliff_Left", -5.25, 0.37),
        ("Striated_Cliff_Right", 5.25, 1.91),
    ] {
        commands.spawn((
            Name::new(name),
            Mesh3d(meshes.add(create_striated_cliff_mesh(4.9, 10.5, 16, 14, seed))),
            MeshMaterial3d(cliff_material.clone()),
            Transform::from_xyz(x, 0.25, -11.32),
        ));

        // The render mesh is deeply displaced, but a slightly inset compound
        // blocker keeps the player from walking through the visible cliff.
        // The pair intentionally leaves the central waterfall corridor open.
        commands.spawn((
            Name::new(format!("Collider_{name}")),
            CavernObstacle,
            ObstacleShape::Cuboid {
                half_extents: Vec3::new(2.45, 5.25, 0.50),
            },
            Transform::from_xyz(x, 5.50, -11.45),
            RigidBody::Static,
            Collider::cuboid(4.9, 10.5, 1.0),
        ));
    }

    // Static physics ground plane for collision
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
    ));

    // Static physical obstacle colliders for rooted mushrooms (Left cluster: Large, Medium, Small)
    commands.spawn((
        Name::new("Collider_Mushroom_Left_Large"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.65,
            height: 2.70,
        },
        Transform::from_xyz(-3.50, 1.30, -3.30),
        RigidBody::Static,
        Collider::cylinder(0.65, 2.70),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Left_Medium"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.48,
            height: 1.90,
        },
        Transform::from_xyz(-2.89, 0.90, -2.52),
        RigidBody::Static,
        Collider::cylinder(0.48, 1.90),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Left_Small"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.38,
            height: 1.25,
        },
        Transform::from_xyz(-4.07, 0.60, -2.20),
        RigidBody::Static,
        Collider::cylinder(0.38, 1.25),
    ));

    // Static physical obstacle colliders for rooted mushrooms (Right cluster: 3 vibrant lathe dome mushrooms)
    commands.spawn((
        Name::new("Collider_Mushroom_Right_1"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.48,
            height: 1.70,
        },
        Transform::from_xyz(4.17, 0.85, -3.53),
        RigidBody::Static,
        Collider::cylinder(0.48, 1.70),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Right_2"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.40,
            height: 1.40,
        },
        Transform::from_xyz(3.58, 0.70, -2.80),
        RigidBody::Static,
        Collider::cylinder(0.40, 1.40),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Right_3"),
        CavernObstacle,
        ObstacleShape::Cylinder {
            radius: 0.32,
            height: 1.05,
        },
        Transform::from_xyz(4.73, 0.50, -3.98),
        RigidBody::Static,
        Collider::cylinder(0.32, 1.05),
    ));

    // Static physical obstacle colliders for organic rock terraces flanking the waterfall
    commands.spawn((
        Name::new("Collider_Rock_Terrace_Left"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(1.75, 1.25, 1.75),
        },
        Transform::from_xyz(-6.8, 1.2, -6.8),
        RigidBody::Static,
        Collider::cuboid(3.5, 2.5, 3.5),
    ));
    commands.spawn((
        Name::new("Collider_Rock_Terrace_Right"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(1.75, 1.25, 1.75),
        },
        Transform::from_xyz(6.8, 1.2, -6.8),
        RigidBody::Static,
        Collider::cuboid(3.5, 2.5, 3.5),
    ));

    // Every authored transition boulder receives a matching simplified collider.
    // These are the rocks that frame the dirt path and bench in cavern_kit.glb;
    // registering them fixes both player clipping and tall-grass placement.
    for (name, x, z, radius, height) in [
        ("Collider_Transition_Rock_Left_01", -2.4, -1.1, 0.58, 0.60),
        ("Collider_Transition_Rock_Left_02", -2.5, -2.3, 0.64, 0.68),
        ("Collider_Transition_Rock_Left_03", -2.7, -3.4, 0.70, 0.76),
        ("Collider_Transition_Rock_Right_01", 2.3, -1.0, 0.56, 0.56),
        ("Collider_Transition_Rock_Right_02", 2.4, -2.2, 0.62, 0.64),
        ("Collider_Transition_Rock_Right_03", 2.6, -3.3, 0.68, 0.72),
        (
            "Collider_Transition_Rock_Fore_Left_A",
            -2.3,
            0.8,
            0.56,
            0.56,
        ),
        (
            "Collider_Transition_Rock_Fore_Left_B",
            -2.4,
            -0.2,
            0.60,
            0.64,
        ),
        (
            "Collider_Transition_Rock_Fore_Right_A",
            2.3,
            0.8,
            0.54,
            0.52,
        ),
        (
            "Collider_Transition_Rock_Fore_Right_B",
            2.4,
            -0.2,
            0.58,
            0.60,
        ),
    ] {
        commands.spawn((
            Name::new(name),
            CavernObstacle,
            ObstacleShape::Cylinder { radius, height },
            Transform::from_xyz(x, height * 0.5, z),
            RigidBody::Static,
            Collider::cylinder(radius, height),
        ));
    }

    // Individual compound blockers for the large rocks that flank the waterfall.
    // The render mesh is irregular, so these overlapping capsules/cylinders keep
    // the whole visible stone bank solid instead of leaving walk-through gaps.
    for (name, x, z, radius, height) in [
        ("Collider_Side_Rock_Left_Front", -5.15, -4.8, 0.95, 1.8),
        ("Collider_Side_Rock_Left_Back", -5.70, -7.2, 1.15, 2.4),
        ("Collider_Side_Rock_Right_Front", 5.15, -4.8, 0.95, 1.8),
        ("Collider_Side_Rock_Right_Back", 5.70, -7.2, 1.15, 2.4),
    ] {
        commands.spawn((
            Name::new(name),
            CavernObstacle,
            ObstacleShape::Cylinder { radius, height },
            Transform::from_xyz(x, height * 0.5, z),
            RigidBody::Static,
            Collider::cylinder(radius, height),
        ));
    }

    // Static physical obstacle colliders for canyon side walls and rear grotto enclosure
    commands.spawn((
        Name::new("Collider_Canyon_Wall_Left"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(1.0, 4.0, 9.0),
        },
        Transform::from_xyz(-8.5, 3.0, -4.5),
        RigidBody::Static,
        Collider::cuboid(2.0, 8.0, 18.0),
    ));
    commands.spawn((
        Name::new("Collider_Canyon_Wall_Right"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(1.0, 4.0, 9.0),
        },
        Transform::from_xyz(8.5, 3.0, -4.5),
        RigidBody::Static,
        Collider::cuboid(2.0, 8.0, 18.0),
    ));
    // Static physical obstacle colliders for rear grotto enclosure flanking portal archway
    commands.spawn((
        Name::new("Collider_Cavern_Wall_Rear_Left"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(3.15, 4.0, 1.0),
        },
        Transform::from_xyz(-5.35, 3.0, 4.0),
        RigidBody::Static,
        Collider::cuboid(6.3, 8.0, 2.0),
    ));
    commands.spawn((
        Name::new("Collider_Cavern_Wall_Rear_Right"),
        CavernObstacle,
        ObstacleShape::Cuboid {
            half_extents: Vec3::new(3.15, 4.0, 1.0),
        },
        Transform::from_xyz(5.35, 3.0, 4.0),
        RigidBody::Static,
        Collider::cuboid(6.3, 8.0, 2.0),
    ));

    // -------------------------------------------------------------
    // 3. CEL-SHADED DYNAMIC WATERFALL CURTAIN & LAKE POOL
    // -------------------------------------------------------------
    // Volumetric curved convex waterfall curtain (flattened half-pipe 48x32 with vertex wave undulation)
    let (waterfall_mesh, waterfall_base) = create_waterfall_mesh(32, 48);
    commands.spawn((
        Mesh3d(meshes.add(waterfall_mesh)),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            base_color: LinearRgba::new(0.12, 0.82, 0.92, 0.35), // Radiant crystalline turquoise
            deep_color: LinearRgba::new(0.03, 0.28, 0.45, 0.48), // Deep azure
            foam_color: LinearRgba::new(0.92, 0.97, 1.0, 0.65),  // Soft aerated froth
            params: Vec4::new(0.0, 1.4, 1.0, 0.45), // z = 1.0 (waterfall mode), w = 0.45 (calibrated bloom)
        })),
        WaterfallMarker,
        waterfall_base,
        bevy::pbr::NotShadowCaster,
        bevy::pbr::NotShadowReceiver,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Wide circular lake water disk (radius = 5.85m to seamlessly enclose waterfall plunge basin & travertine rim)
    commands.spawn((
        Mesh3d(meshes.add(create_water_disk_mesh(5.85, 64))),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            base_color: LinearRgba::new(0.08, 0.72, 0.85, 0.28), // Crystalline clear water
            deep_color: LinearRgba::new(0.02, 0.20, 0.30, 0.45), // Deep turquoise
            foam_color: LinearRgba::new(0.92, 0.98, 1.0, 0.65),  // Soft white foam
            params: Vec4::new(0.0, 1.0, 0.0, 0.45), // z = 0.0 (lake mode), w = 0.45 (calibrated bloom)
        })),
        bevy::pbr::NotShadowCaster,
        bevy::pbr::NotShadowReceiver,
        Transform::from_xyz(0.0, 0.18, -8.5),
    ));

    // -------------------------------------------------------------
    // 4. WATERFALL IMPACT SPRAY & AMBIENT STEAM/FOAM VAPOR
    // -------------------------------------------------------------
    // Wide impact mist / foam emitters where wide waterfall hits lake (spanning X in [-3.6, 3.6])
    for x_off in [-3.6, -2.4, -1.2, 0.0, 1.2, 2.4, 3.6] {
        commands.spawn((
            Transform::from_xyz(x_off, 0.28, -8.5),
            ParticleEmitter {
                spawn_timer: 0.0,
                spawn_rate: 0.035,
                particle_type: ParticleType::WaterSplash,
            },
        ));
        commands.spawn((
            Transform::from_xyz(x_off * 0.8, 0.35, -8.7),
            ParticleEmitter {
                spawn_timer: 0.0,
                spawn_rate: 0.06,
                particle_type: ParticleType::Steam,
            },
        ));
    }

    // Lateral mushroom spore emitters
    commands.spawn((
        Transform::from_xyz(-3.6, 2.2, -3.2),
        ParticleEmitter {
            spawn_timer: 0.0,
            spawn_rate: 0.12,
            particle_type: ParticleType::GlowSpore,
        },
    ));
    commands.spawn((
        Transform::from_xyz(4.2, 2.2, -3.4),
        ParticleEmitter {
            spawn_timer: 0.0,
            spawn_rate: 0.12,
            particle_type: ParticleType::GlowSpore,
        },
    ));

    // -------------------------------------------------------------
    // 5. INTERACTIVE ALCHEMY INGREDIENTS & THERMAL GEYSERS
    // -------------------------------------------------------------
    // Harvestable Cyan Glow Mushroom (Left cluster)
    commands.spawn((
        Interactable {
            label: "Cyan Parasol Mushroom",
        },
        AlchemicalIngredient {
            id: "ing_cyan_spore".to_string(),
            name: "Glow Spore".to_string(),
            primary_effect: AlchemicalEffectKind::Luminescence,
            secondary_effect: AlchemicalEffectKind::Swiftness,
            harvested: false,
        },
        Transform::from_xyz(-3.6, 0.5, -3.2),
        GlobalTransform::default(),
        Collider::cylinder(0.4, 0.8),
    ));

    // Thermal Geyser at travertine terrace
    commands.spawn((
        Transform::from_xyz(-5.8, 0.5, -5.8),
        ThermalGeyser {
            eruption_timer: 0.0,
            eruption_interval: 3.5,
            is_erupting: true,
            heat_radius: 5.0,
        },
        ParticleEmitter {
            spawn_timer: 0.0,
            spawn_rate: 0.08,
            particle_type: ParticleType::Steam,
        },
    ));

    // Rustic mossy bench: placed on clear grass beyond the pool rim and mushroom
    // cluster. Its collision follows the actual construction rather than using
    // one oversized invisible box that intersects surrounding props.
    let bench_position = Vec3::new(2.90, 0.03, -1.05);
    let bench_rotation = Quat::from_rotation_y(-0.45);
    commands.spawn((
        Name::new("Rustic_Mossy_Bench"),
        SceneRoot(asset_server.load("models/alchemy_props.glb#Scene0")),
        Transform::from_translation(bench_position).with_rotation(bench_rotation),
    ));

    let bench_colliders = [
        (
            "Collider_Bench_Seat",
            Vec3::new(0.0, 0.58, 0.0),
            Vec3::new(1.65, 0.18, 0.34),
        ),
        (
            "Collider_Bench_Backrest",
            Vec3::new(0.0, 1.00, 0.28),
            Vec3::new(1.65, 0.38, 0.12),
        ),
        (
            "Collider_Bench_Leg_FL",
            Vec3::new(-0.62, 0.28, 0.20),
            Vec3::new(0.14, 0.56, 0.14),
        ),
        (
            "Collider_Bench_Leg_FR",
            Vec3::new(0.62, 0.28, 0.20),
            Vec3::new(0.14, 0.56, 0.14),
        ),
        (
            "Collider_Bench_Leg_BL",
            Vec3::new(-0.62, 0.28, -0.20),
            Vec3::new(0.14, 0.56, 0.14),
        ),
        (
            "Collider_Bench_Leg_BR",
            Vec3::new(0.62, 0.28, -0.20),
            Vec3::new(0.14, 0.56, 0.14),
        ),
    ];
    for (name, local_offset, dimensions) in bench_colliders {
        commands.spawn((
            Name::new(name),
            CavernObstacle,
            ObstacleShape::Cuboid {
                half_extents: dimensions * 0.5,
            },
            Transform::from_translation(bench_position + bench_rotation * local_offset)
                .with_rotation(bench_rotation),
            RigidBody::Static,
            Collider::cuboid(dimensions.x, dimensions.y, dimensions.z),
        ));
    }

    info!("Masterwork Cel-Shaded Cavern Grotto initialized with Volumetric Waterfall, Organic Mushrooms, and Sobel Outlines!");
}

/// Creates a deliberately uneven cliff panel with individually faceted cells.
/// The depth displacement combines sedimentary shelves, narrow water-cut ruts,
/// and irregular fractures, so its silhouette and normal buffer both contribute
/// to the existing Sobel outline pass.
pub fn create_striated_cliff_mesh(
    width: f32,
    height: f32,
    columns: usize,
    rows: usize,
    seed: f32,
) -> Mesh {
    let mut positions = Vec::with_capacity(columns * rows * 6);
    let mut normals = Vec::with_capacity(columns * rows * 6);
    let mut indices = Vec::with_capacity(columns * rows * 6);

    let surface_depth = |x: f32, y: f32| {
        let strata = (y * 2.65 + x * 0.42 + seed).sin().abs().powf(14.0) * 0.46;
        let fissure_wave = (x * 3.7 - y * 0.72 + seed * 4.0).sin().abs();
        let fissure = if fissure_wave > 0.945 { -0.42 } else { 0.0 };
        let weathering = (x * 1.45 + seed).sin() * 0.11 + (y * 1.12 - seed * 2.0).cos() * 0.08;
        strata + fissure + weathering
    };

    for row in 0..rows {
        for column in 0..columns {
            let x0 = -width * 0.5 + width * column as f32 / columns as f32;
            let x1 = -width * 0.5 + width * (column + 1) as f32 / columns as f32;
            let y0 = height * row as f32 / rows as f32;
            let y1 = height * (row + 1) as f32 / rows as f32;
            let a = Vec3::new(x0, y0, surface_depth(x0, y0));
            let b = Vec3::new(x1, y0, surface_depth(x1, y0));
            let c = Vec3::new(x1, y1, surface_depth(x1, y1));
            let d = Vec3::new(x0, y1, surface_depth(x0, y1));
            let normal = (b - a).cross(c - a).normalize_or_zero();
            let base = positions.len() as u32;

            for vertex in [a, b, c, d] {
                positions.push(vertex.to_array());
                normals.push(normal.to_array());
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn create_water_disk_mesh(radius: f32, segments: usize) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(segments + 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(segments + 2);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 3);

    // Center vertex
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        positions.push([x, 0.0, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()]);
    }

    for i in 1..=segments as u32 {
        indices.push(0);
        indices.push(i + 1);
        indices.push(i);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_waterfall_mesh(u_segs: usize, v_segs: usize) -> (Mesh, WaterfallBaseMesh) {
    let num_verts = (u_segs + 1) * (v_segs + 1);
    let mut positions = Vec::with_capacity(num_verts);
    let mut normals = Vec::with_capacity(num_verts);
    let mut uvs = Vec::with_capacity(num_verts);
    let mut indices = Vec::with_capacity(u_segs * v_segs * 12);

    let h_top = 17.5;
    let h_bottom = 0.06;
    let z_top = -10.8;
    let z_bottom = -8.5;
    let width_top = 4.2;
    let width_bottom = 6.4;

    for iv in 0..=v_segs {
        let v_frac = iv as f32 / v_segs as f32;
        let y_calc = h_bottom + (1.0 - v_frac) * (h_top - h_bottom);
        // Arcs forward into the lake center at z = -8.5
        let curr_z_center = z_top + v_frac.powf(1.4) * (z_bottom - z_top);
        let curr_width = width_top + v_frac * (width_bottom - width_top);

        for iu in 0..=u_segs {
            let u_frac = iu as f32 / u_segs as f32;
            let u_norm = (u_frac - 0.5) * 2.0; // [-1.0, 1.0]
            let x = u_norm * (curr_width * 0.5);

            // Convex forward curvature: half-pipe bulging toward +Z (viewer)
            let convex_arc = (1.0 - u_norm * u_norm) * 0.52;
            // Fluted vertical cascading ridges
            let fluting = (u_norm * std::f32::consts::PI * 8.0).sin() * 0.08;
            let z = curr_z_center + convex_arc + fluting;

            let bottom_scallop = if iv == v_segs {
                (u_frac * std::f32::consts::PI).sin() * 0.08
            } else {
                0.0
            };
            let y = y_calc - bottom_scallop;

            positions.push([x, y, z]);

            // Smooth outward facing normal
            let nx = -u_norm * 0.45;
            let ny = 0.15;
            let nz = 1.0;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            normals.push([nx / len, ny / len, nz / len]);

            uvs.push([u_frac, v_frac]);
        }
    }

    for iv in 0..v_segs as u32 {
        for iu in 0..u_segs as u32 {
            let row_stride = (u_segs + 1) as u32;
            let v1 = iv * row_stride + iu;
            let v2 = iv * row_stride + iu + 1;
            let v3 = (iv + 1) * row_stride + iu + 1;
            let v4 = (iv + 1) * row_stride + iu;

            // Front face (facing +Z)
            indices.push(v1);
            indices.push(v4);
            indices.push(v2);

            indices.push(v2);
            indices.push(v4);
            indices.push(v3);

            // Back face (double-sided visibility)
            indices.push(v1);
            indices.push(v2);
            indices.push(v4);

            indices.push(v2);
            indices.push(v3);
            indices.push(v4);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    let base_mesh = WaterfallBaseMesh {
        base_positions: positions,
        base_normals: normals,
    };

    (mesh, base_mesh)
}

pub fn animate_waterfall_mesh(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(&Mesh3d, &WaterfallBaseMesh), With<WaterfallMarker>>,
) {
    let t = time.elapsed_secs();
    for (mesh_handle, base) in &query {
        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            let mut new_positions = Vec::with_capacity(base.base_positions.len());
            for (pos, norm) in base.base_positions.iter().zip(base.base_normals.iter()) {
                let wave =
                    (pos[1] * 4.0 - t * 5.0).sin() * 0.05 + (pos[0] * 3.0 + t * 2.5).cos() * 0.025;
                let displaced = [
                    pos[0] + norm[0] * wave,
                    pos[1] + norm[1] * wave * 0.5,
                    pos[2] + norm[2] * wave,
                ];
                new_positions.push(displaced);
            }
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_dungeon_plugin_build() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.init_asset::<WaterMaterial>();
        app.init_asset::<bevy::scene::Scene>();
        app.add_plugins(DungeonPlugin);
        assert!(app.is_plugin_added::<DungeonPlugin>());
    }

    #[test]
    fn test_setup_cavern_world_spawns_environment() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.init_asset::<WaterMaterial>();
        app.init_asset::<CelShadedMossMaterial>();
        app.init_asset::<bevy::scene::Scene>();
        app.add_systems(Startup, setup_cavern_world);
        app.update();

        // Check PointLights spawned (at least 9 including high ceiling vault lights)
        let mut light_query = app.world_mut().query::<&PointLight>();
        let light_count = light_query.iter(app.world()).count();
        assert!(light_count >= 9, "Must spawn at least 9 point lights for bioluminescence, waterfall and high ceiling vault");

        // Check CavernObstacle colliders spawned (6 mushroom stems + 2 rock terraces + 1 rustic mossy bench + canyon walls)
        let mut obstacle_query = app
            .world_mut()
            .query::<(&CavernObstacle, &Collider, &RigidBody)>();
        let obstacle_count = obstacle_query.iter(app.world()).count();
        assert!(obstacle_count >= 9, "Must spawn at least 9 static colliders for mushroom stems, rock terraces and rustic mossy bench");

        // Check ParticleEmitter spawned
        let mut emitter_query = app.world_mut().query::<&ParticleEmitter>();
        let emitter_count = emitter_query.iter(app.world()).count();
        assert!(
            emitter_count >= 4,
            "Must spawn particle emitters for waterfall foam and mushroom spores"
        );

        // Check SceneRoot spawned for cavern kit and alchemy props
        let mut scene_query = app.world_mut().query::<&SceneRoot>();
        let scene_count = scene_query.iter(app.world()).count();
        assert!(
            scene_count >= 2,
            "Must spawn cavern_kit and alchemy_props scenes"
        );

        // Check Waterfall spawned with WaterfallMarker
        let mut waterfall_query = app.world_mut().query::<&WaterfallMarker>();
        let waterfall_count = waterfall_query.iter(app.world()).count();
        assert_eq!(waterfall_count, 1, "Must spawn dynamic waterfall entity");
    }

    #[test]
    fn test_striated_cliff_mesh_has_faceted_geometry() {
        let mesh = create_striated_cliff_mesh(4.9, 10.5, 16, 14, 0.37);
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.indices().is_some());
    }

    #[test]
    fn test_create_water_disk_mesh_attributes_and_topology() {
        let mesh = create_water_disk_mesh(5.0, 32);
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        assert!(mesh.indices().is_some());
    }

    #[test]
    fn test_create_waterfall_mesh_attributes_and_curvature() {
        let (mesh, base) = create_waterfall_mesh(16, 24);
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
        assert!(mesh.indices().is_some());
        assert_eq!(base.base_positions.len(), (16 + 1) * (24 + 1));
        assert_eq!(base.base_normals.len(), (16 + 1) * (24 + 1));
    }
}
