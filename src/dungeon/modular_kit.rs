use bevy::prelude::*;
use avian3d::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::{
    alchemy::{AlchemicalEffectKind, AlchemicalIngredient, ThermalGeyser},
    player::interaction::Interactable,
    render::{
        particles::{ParticleEmitter, ParticleType},
        water_material::WaterMaterial,
    },
};

#[derive(Component)]
pub struct WaterfallMarker;

#[derive(Component, Clone)]
pub struct WaterfallBaseMesh {
    pub base_positions: Vec<[f32; 3]>,
    pub base_normals: Vec<[f32; 3]>,
}

#[derive(Component)]
pub struct CavernObstacle;

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (setup_cavern_world, crate::dungeon::generator::spawn_procedural_dungeon),
        )
        .add_systems(Update, animate_waterfall_mesh);
    }
}

pub fn setup_cavern_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
) {
    // -------------------------------------------------------------
    // 1. ATMOSPHERE & LIGHTING (RICH NIGHT BLUE - NO PITCH BLACK #000000)
    // -------------------------------------------------------------
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.10, 0.18, 0.28), // Deep fantasy night blue
        brightness: 420.0,
    });

    // Soft skylight beam descending from the upper fissure over the waterfall
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.75, 0.92, 1.0),
            illuminance: 6500.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(1.5, 22.0, -4.5).looking_at(Vec3::new(0.0, 1.2, -8.5), Vec3::Y),
    ));

    // --- Bioluminescent PointLights (Lateral Margins, Waterfall & Amphitheater) ---
    // Left Giant Mushroom cap light (Lateral margin: X = -5.4, Y = 2.8, Z = -3.6)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.17, 0.95, 0.88), // Electric turquoise #2be8d8
            intensity: 42000.0,
            range: 14.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-5.4, 2.8, -3.6),
    ));

    // Right Giant Mushroom cap light (Lateral margin: X = 5.4, Y = 2.8, Z = -3.6)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.17, 0.95, 0.88),
            intensity: 42000.0,
            range: 14.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(5.4, 2.8, -3.6),
    ));

    // Waterfall luminous foam and spray bounce light
    commands.spawn((
        PointLight {
            color: Color::srgb(0.35, 0.90, 1.0),
            intensity: 55000.0,
            range: 20.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1.8, -10.2),
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

    // Overhead ceiling arch rim light (illuminates forward stalactites)
    commands.spawn((
        PointLight {
            color: Color::srgb(0.25, 0.35, 0.50),
            intensity: 40000.0,
            range: 22.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 8.5, -2.5),
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
        Transform::from_xyz(-5.0, 1.6, -3.2),
        RigidBody::Static,
        Collider::cylinder(0.52, 3.0),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Left_Medium"),
        CavernObstacle,
        Transform::from_xyz(-4.3, 0.95, -2.4),
        RigidBody::Static,
        Collider::cylinder(0.38, 1.75),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Left_Small"),
        CavernObstacle,
        Transform::from_xyz(-5.5, 0.65, -3.8),
        RigidBody::Static,
        Collider::cylinder(0.28, 1.10),
    ));

    // Static physical obstacle colliders for rooted mushrooms (Right cluster: 3 vibrant dome mushrooms)
    commands.spawn((
        Name::new("Collider_Mushroom_Right_1"),
        CavernObstacle,
        Transform::from_xyz(4.6, 1.4, -3.1),
        RigidBody::Static,
        Collider::cylinder(0.48, 2.6),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Right_2"),
        CavernObstacle,
        Transform::from_xyz(4.0, 0.95, -2.3),
        RigidBody::Static,
        Collider::cylinder(0.38, 1.75),
    ));
    commands.spawn((
        Name::new("Collider_Mushroom_Right_3"),
        CavernObstacle,
        Transform::from_xyz(5.3, 0.65, -3.7),
        RigidBody::Static,
        Collider::cylinder(0.28, 1.10),
    ));

    // Static physical obstacle colliders for organic rock terraces flanking the waterfall
    commands.spawn((
        Name::new("Collider_Rock_Terrace_Left"),
        CavernObstacle,
        Transform::from_xyz(-6.6, 1.2, -7.8),
        RigidBody::Static,
        Collider::cuboid(3.5, 2.5, 3.5),
    ));
    commands.spawn((
        Name::new("Collider_Rock_Terrace_Right"),
        CavernObstacle,
        Transform::from_xyz(6.6, 1.2, -7.8),
        RigidBody::Static,
        Collider::cuboid(3.5, 2.5, 3.5),
    ));

    // -------------------------------------------------------------
    // 3. CEL-SHADED DYNAMIC WATERFALL CURTAIN & LAKE POOL
    // -------------------------------------------------------------
    // Volumetric curved convex waterfall curtain (flattened half-pipe 48x32 with vertex wave undulation)
    let (waterfall_mesh, waterfall_base) = create_waterfall_mesh(32, 48);
    commands.spawn((
        Mesh3d(meshes.add(waterfall_mesh)),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            base_color: LinearRgba::new(0.12, 0.78, 0.90, 0.72), // Radiant translucent cyan
            deep_color: LinearRgba::new(0.03, 0.26, 0.42, 0.85), // Deep azure
            foam_color: LinearRgba::new(0.92, 0.97, 1.0, 0.90), // Soft aerated froth
            params: Vec4::new(0.0, 1.4, 1.0, 1.0), // z = 1.0 (waterfall mode), w = 1.0 (natural bloom)
        })),
        WaterfallMarker,
        waterfall_base,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Wide circular lake water disk (radius = 5.85m to seamlessly enclose waterfall plunge basin & travertine rim)
    commands.spawn((
        Mesh3d(meshes.add(create_water_disk_mesh(5.85, 64))),
        MeshMaterial3d(water_materials.add(WaterMaterial {
            base_color: LinearRgba::new(0.08, 0.68, 0.80, 0.76), // Crystalline clear water
            deep_color: LinearRgba::new(0.02, 0.18, 0.28, 0.88), // Deep turquoise
            foam_color: LinearRgba::new(0.92, 0.98, 1.0, 0.85), // Soft white foam
            params: Vec4::new(0.0, 1.0, 0.0, 0.9), // z = 0.0 (lake mode), w = 0.9 (natural bloom)
        })),
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
        Transform::from_xyz(-5.4, 2.5, -3.6),
        ParticleEmitter {
            spawn_timer: 0.0,
            spawn_rate: 0.12,
            particle_type: ParticleType::GlowSpore,
        },
    ));
    commands.spawn((
        Transform::from_xyz(5.4, 2.5, -3.6),
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
        Interactable { label: "Cyan Parasol Mushroom" },
        AlchemicalIngredient {
            id: "ing_cyan_spore".to_string(),
            name: "Glow Spore".to_string(),
            primary_effect: AlchemicalEffectKind::Luminescence,
            secondary_effect: AlchemicalEffectKind::Swiftness,
            harvested: false,
        },
        Transform::from_xyz(-4.8, 0.5, -3.2),
        GlobalTransform::default(),
        Collider::cylinder(0.4, 0.8),
    ));

    // Thermal Geyser at travertine terrace
    commands.spawn((
        Transform::from_xyz(-4.5, 0.5, -6.5),
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

    // Masterwork Rustic Mossy Wooden Bench (lower right on grass, angled toward lagoon)
    commands.spawn((
        Name::new("Rustic_Mossy_Bench"),
        CavernObstacle,
        SceneRoot(asset_server.load("models/alchemy_props.glb#Scene0")),
        Transform::from_xyz(2.65, 0.04, -2.1).with_rotation(Quat::from_rotation_y(-0.50)),
        RigidBody::Static,
        Collider::cuboid(1.6, 0.9, 0.7),
    ));

    info!("Masterwork Cel-Shaded Cavern Grotto initialized with Volumetric Waterfall, Organic Mushrooms, and Sobel Outlines!");
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
                let wave = (pos[1] * 4.0 - t * 5.0).sin() * 0.05 + (pos[0] * 3.0 + t * 2.5).cos() * 0.025;
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
        app.init_asset::<bevy::scene::Scene>();
        app.add_systems(Startup, setup_cavern_world);
        app.update();

        // Check PointLights spawned (at least 9 including high ceiling vault lights)
        let mut light_query = app.world_mut().query::<&PointLight>();
        let light_count = light_query.iter(app.world()).count();
        assert!(light_count >= 9, "Must spawn at least 9 point lights for bioluminescence, waterfall and high ceiling vault");

        // Check CavernObstacle colliders spawned (6 mushroom stems + 2 rock terraces + 1 rustic mossy bench)
        let mut obstacle_query = app.world_mut().query::<(&CavernObstacle, &Collider, &RigidBody)>();
        let obstacle_count = obstacle_query.iter(app.world()).count();
        assert_eq!(obstacle_count, 9, "Must spawn 9 static colliders for mushroom stems, rock terraces and rustic mossy bench");

        // Check ParticleEmitter spawned
        let mut emitter_query = app.world_mut().query::<&ParticleEmitter>();
        let emitter_count = emitter_query.iter(app.world()).count();
        assert!(emitter_count >= 4, "Must spawn particle emitters for waterfall foam and mushroom spores");

        // Check SceneRoot spawned for cavern kit and alchemy props
        let mut scene_query = app.world_mut().query::<&SceneRoot>();
        let scene_count = scene_query.iter(app.world()).count();
        assert!(scene_count >= 2, "Must spawn cavern_kit and alchemy_props scenes");

        // Check Waterfall spawned with WaterfallMarker
        let mut waterfall_query = app.world_mut().query::<&WaterfallMarker>();
        let waterfall_count = waterfall_query.iter(app.world()).count();
        assert_eq!(waterfall_count, 1, "Must spawn dynamic waterfall entity");
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
