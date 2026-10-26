use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    render::render_asset::RenderAssetUsages,
};

use crate::{
    alchemy::{AlchemicalEffectKind, Flammable, PotionProjectile},
    dungeon::{
        modular_kit::BioluminescentMushroom,
        water_drips::{CavernWaterPuddle, WaterPuddleState},
    },
    player::camera::Player,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsectKind {
    Moth,
    WaterFly,
    Spider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsectState {
    Hovering,
    Fleeing,
}

#[derive(Component, Debug, Clone)]
pub struct AmbientInsect {
    pub kind: InsectKind,
    pub state: InsectState,
    pub anchor_pos: Vec3,
    pub base_scale: f32,
    pub flutter_phase: f32,
    pub flutter_speed: f32,
    pub wander_offset: Vec3,
    pub wander_target: Vec3,
    pub wander_timer: f32,
    pub flee_timer: f32,
    pub flee_duration: f32,
    pub flee_velocity: Vec3,
    pub puddle_entity: Option<Entity>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct MothWing {
    pub is_left: bool,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SpiderLeg {
    pub leg_index: usize,
    pub is_left: bool,
    pub base_rotation: Quat,
}

#[derive(Component, Debug, Clone)]
pub struct SpiderWeb {
    pub radius: f32,
    pub is_ruptured: bool,
    pub rupture_timer: f32,
    pub rupture_duration: f32,
    pub attached_spider: Option<Entity>,
}

#[derive(Resource, Debug, Clone)]
pub struct AmbientFaunaSpawner {
    pub timer: f32,
    pub interval: f32,
    pub max_moths: usize,
    pub max_flies: usize,
    pub max_spiders: usize,
    pub max_webs: usize,
    pub spawn_seed: u32,
}

impl Default for AmbientFaunaSpawner {
    fn default() -> Self {
        Self {
            timer: 3.5,
            interval: 3.5,
            max_moths: 10,
            max_flies: 14,
            max_spiders: 6,
            max_webs: 8,
            spawn_seed: 1337,
        }
    }
}

#[derive(Resource, Clone)]
pub struct AmbientFaunaAssets {
    pub moth_body_mesh: Handle<Mesh>,
    pub moth_left_wing_mesh: Handle<Mesh>,
    pub moth_right_wing_mesh: Handle<Mesh>,
    pub moth_body_material: Handle<StandardMaterial>,
    pub moth_wing_material: Handle<StandardMaterial>,
    pub fly_mesh: Handle<Mesh>,
    pub fly_material: Handle<StandardMaterial>,
    pub spider_body_mesh: Handle<Mesh>,
    pub spider_leg_meshes: [Handle<Mesh>; 4],
    pub spider_material: Handle<StandardMaterial>,
    pub web_corner_mesh: Handle<Mesh>,
    pub web_radial_mesh: Handle<Mesh>,
    pub web_material: Handle<StandardMaterial>,
}

pub fn create_tiny_moth_body_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let segments = 6;
    let half_len = 0.0045;
    let radius = 0.0011;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    positions.push([0.0, 0.0, -half_len]);
    normals.push([0.0, 0.0, -1.0]);
    uvs.push([0.5, 0.0]);

    positions.push([0.0, 0.0, half_len]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([0.5, 1.0]);

    for slice in 0..2 {
        let z = if slice == 0 { -half_len * 0.4 } else { half_len * 0.4 };
        for i in 0..segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = theta.cos() * radius;
            let y = theta.sin() * radius;
            positions.push([x, y, z]);
            let norm = Vec3::new(x, y, 0.0).normalize_or_zero();
            normals.push([norm.x, norm.y, 0.0]);
            uvs.push([i as f32 / segments as f32, if slice == 0 { 0.3 } else { 0.7 }]);
        }
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        let p0 = 2 + i as u32;
        let p1 = 2 + next as u32;
        indices.push(0);
        indices.push(p0);
        indices.push(p1);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        let p0 = 2 + i as u32;
        let p1 = 2 + next as u32;
        let p2 = 2 + segments as u32 + i as u32;
        let p3 = 2 + segments as u32 + next as u32;
        indices.push(p0);
        indices.push(p2);
        indices.push(p1);
        indices.push(p1);
        indices.push(p2);
        indices.push(p3);
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        let p0 = 2 + segments as u32 + i as u32;
        let p1 = 2 + segments as u32 + next as u32;
        indices.push(1);
        indices.push(p1);
        indices.push(p0);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_tiny_moth_wing_mesh(is_left: bool) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let sign = if is_left { -1.0 } else { 1.0 };
    let positions: Vec<[f32; 3]> = vec![
        [0.0, 0.0, 0.0],
        [sign * 0.0085, 0.0004, -0.0045],
        [sign * 0.0090, 0.0, 0.0005],
        [sign * 0.0048, -0.0002, 0.0040],
        [0.0, 0.0, 0.0022],
    ];
    let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; 5];
    let uvs: Vec<[f32; 2]> = vec![
        [0.0, 0.5],
        [1.0, 0.0],
        [1.0, 0.5],
        [0.5, 1.0],
        [0.0, 1.0],
    ];
    let indices: Vec<u32> = vec![
        0, 1, 2,
        0, 2, 3,
        0, 3, 4,
        0, 2, 1,
        0, 3, 2,
        0, 4, 3,
    ];
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_tiny_fly_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let positions: Vec<[f32; 3]> = vec![
        [0.0, 0.0, -0.0018],
        [0.0, 0.0, 0.0018],
        [-0.0007, 0.0, 0.0],
        [0.0007, 0.0, 0.0],
        [0.0, 0.0006, 0.0],
        [0.0, -0.0006, 0.0],
        [-0.0026, 0.0005, 0.0006],
        [-0.0024, 0.0004, -0.0008],
        [0.0026, 0.0005, 0.0006],
        [0.0024, 0.0004, -0.0008],
    ];
    let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; 10];
    let uvs: Vec<[f32; 2]> = vec![[0.5, 0.5]; 10];
    let indices: Vec<u32> = vec![
        0, 2, 4,
        0, 4, 3,
        0, 3, 5,
        0, 5, 2,
        1, 4, 2,
        1, 3, 4,
        1, 5, 3,
        1, 2, 5,
        2, 6, 7,
        2, 7, 6,
        3, 9, 8,
        3, 8, 9,
    ];
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_tiny_spider_body_mesh() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    positions.push([0.0, 0.0, -0.0028]);
    normals.push([0.0, 0.0, -1.0]);
    uvs.push([0.5, 0.0]);

    positions.push([0.0, 0.0, 0.0050]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([0.5, 1.0]);

    let segments = 6;
    let slices = [
        (-0.0018, 0.0011, 0.0008),
        (-0.0004, 0.0012, 0.0008),
        (0.0005, 0.0006, 0.0005),
        (0.0022, 0.0017, 0.0014),
        (0.0038, 0.0014, 0.0012),
    ];

    for (s_idx, &(z, rx, ry)) in slices.iter().enumerate() {
        for i in 0..segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = theta.cos() * rx;
            let y = theta.sin() * ry;
            positions.push([x, y, z]);
            let norm = Vec3::new(x, y, 0.0).normalize_or_zero();
            normals.push([norm.x, norm.y, 0.0]);
            uvs.push([i as f32 / segments as f32, s_idx as f32 / 4.0]);
        }
    }

    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(0);
        indices.push(2 + i as u32);
        indices.push(2 + next as u32);
    }

    for s in 0..4 {
        let base_a = 2 + (s * segments) as u32;
        let base_b = base_a + segments as u32;
        for i in 0..segments {
            let next = (i + 1) % segments;
            let a0 = base_a + i as u32;
            let a1 = base_a + next as u32;
            let b0 = base_b + i as u32;
            let b1 = base_b + next as u32;
            indices.push(a0);
            indices.push(b0);
            indices.push(a1);
            indices.push(a1);
            indices.push(b0);
            indices.push(b1);
        }
    }

    let last_base = 2 + (4 * segments) as u32;
    for i in 0..segments {
        let next = (i + 1) % segments;
        indices.push(1);
        indices.push(last_base + next as u32);
        indices.push(last_base + i as u32);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_spider_leg_mesh(idx: usize) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let (target_x, target_z) = match idx {
        0 => (0.0090, -0.0055),
        1 => (0.0105, -0.0018),
        2 => (0.0095, 0.0028),
        _ => (0.0115, 0.0065),
    };
    let knee_x = target_x * 0.45;
    let knee_y = 0.0032;
    let knee_z = target_z * 0.45;
    let tip_x = target_x;
    let tip_y = -0.0012;
    let tip_z = target_z;

    let w = 0.00035;
    let positions: Vec<[f32; 3]> = vec![
        [0.0, -w, -w],
        [0.0, w, -w],
        [0.0, w, w],
        [0.0, -w, w],
        [knee_x, knee_y - w, knee_z - w],
        [knee_x, knee_y + w, knee_z - w],
        [knee_x, knee_y + w, knee_z + w],
        [knee_x, knee_y - w, knee_z + w],
        [tip_x, tip_y, tip_z],
    ];
    let normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; 9];
    let uvs: Vec<[f32; 2]> = vec![[0.5, 0.5]; 9];
    let mut indices: Vec<u32> = Vec::new();

    for i in 0..4 {
        let next = (i + 1) % 4;
        indices.push(i as u32);
        indices.push((4 + i) as u32);
        indices.push(next as u32);
        indices.push(next as u32);
        indices.push((4 + i) as u32);
        indices.push((4 + next) as u32);
    }

    for i in 0..4 {
        let next = (i + 1) % 4;
        indices.push((4 + i) as u32);
        indices.push(8);
        indices.push((4 + next) as u32);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_corner_cobweb_mesh(width: f32, height: f32, depth: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.577, 0.577, 0.577]);
    uvs.push([0.5, 0.5]);

    let spokes = [
        Vec3::new(width, 0.0, 0.0),
        Vec3::new(width * 0.82, height * 0.42, 0.0),
        Vec3::new(0.0, height, 0.0),
        Vec3::new(0.0, height * 0.42, depth * 0.82),
        Vec3::new(0.0, 0.0, depth),
        Vec3::new(width * 0.65, -0.04, depth * 0.65),
    ];
    let spoke_count = spokes.len();
    let rings = 4;

    for r in 1..=rings {
        let f = r as f32 / rings as f32;
        for s in 0..spoke_count {
            let p = spokes[s] * f;
            positions.push([p.x, p.y, p.z]);
            normals.push([0.577, 0.577, 0.577]);
            uvs.push([s as f32 / spoke_count as f32, f]);
        }
    }

    for s in 0..spoke_count {
        let next = (s + 1) % spoke_count;
        let p0 = 1 + s as u32;
        let p1 = 1 + next as u32;
        indices.push(0);
        indices.push(p0);
        indices.push(p1);
        indices.push(0);
        indices.push(p1);
        indices.push(p0);
    }

    for r in 0..(rings - 1) {
        let base_a = 1 + (r * spoke_count) as u32;
        let base_b = base_a + spoke_count as u32;
        for s in 0..spoke_count {
            let next = (s + 1) % spoke_count;
            let a0 = base_a + s as u32;
            let a1 = base_a + next as u32;
            let b0 = base_b + s as u32;
            let b1 = base_b + next as u32;
            indices.push(a0);
            indices.push(b0);
            indices.push(a1);
            indices.push(a1);
            indices.push(b0);
            indices.push(b1);
            indices.push(a0);
            indices.push(a1);
            indices.push(b0);
            indices.push(a1);
            indices.push(b1);
            indices.push(b0);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn create_radial_spiderweb_mesh(radius: f32) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    let spoke_count = 8;
    let rings = 4;

    for r in 1..=rings {
        let f = r as f32 / rings as f32;
        let current_r = radius * f;
        for s in 0..spoke_count {
            let theta = (s as f32 / spoke_count as f32) * std::f32::consts::TAU;
            let x = theta.cos() * current_r;
            let z = theta.sin() * current_r;
            positions.push([x, 0.0, z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([(x / (radius * 2.0)) + 0.5, (z / (radius * 2.0)) + 0.5]);
        }
    }

    for s in 0..spoke_count {
        let next = (s + 1) % spoke_count;
        let p0 = 1 + s as u32;
        let p1 = 1 + next as u32;
        indices.push(0);
        indices.push(p0);
        indices.push(p1);
        indices.push(0);
        indices.push(p1);
        indices.push(p0);
    }

    for r in 0..(rings - 1) {
        let base_a = 1 + (r * spoke_count) as u32;
        let base_b = base_a + spoke_count as u32;
        for s in 0..spoke_count {
            let next = (s + 1) % spoke_count;
            let a0 = base_a + s as u32;
            let a1 = base_a + next as u32;
            let b0 = base_b + s as u32;
            let b1 = base_b + next as u32;
            indices.push(a0);
            indices.push(b0);
            indices.push(a1);
            indices.push(a1);
            indices.push(b0);
            indices.push(b1);
            indices.push(a0);
            indices.push(a1);
            indices.push(b0);
            indices.push(a1);
            indices.push(b1);
            indices.push(b0);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

pub fn pseudo_hash_coords(pos: Vec3, seed: u32) -> u32 {
    let x = (pos.x * 100.0).round() as i32;
    let y = (pos.y * 100.0).round() as i32;
    let z = (pos.z * 100.0).round() as i32;
    let mut h = seed.wrapping_add(x as u32).wrapping_mul(0x45d9f3b);
    h = h.wrapping_add(y as u32).wrapping_mul(0x45d9f3b);
    h = h.wrapping_add(z as u32).wrapping_mul(0x45d9f3b);
    h ^= h >> 16;
    h
}

pub fn setup_ambient_fauna(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> AmbientFaunaAssets {
    let moth_body_mesh = meshes.add(create_tiny_moth_body_mesh());
    let moth_left_wing_mesh = meshes.add(create_tiny_moth_wing_mesh(true));
    let moth_right_wing_mesh = meshes.add(create_tiny_moth_wing_mesh(false));
    let moth_body_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.22, 0.20),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        reflectance: 0.2,
        cull_mode: None,
        ..default()
    });
    let moth_wing_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.85, 0.90, 0.88, 0.85),
        emissive: LinearRgba::new(0.04, 0.08, 0.07, 1.0),
        perceptual_roughness: 0.7,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let fly_mesh = meshes.add(create_tiny_fly_mesh());
    let fly_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.18, 0.16, 0.15, 0.85),
        perceptual_roughness: 0.6,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let spider_body_mesh = meshes.add(create_tiny_spider_body_mesh());
    let spider_leg_meshes = [
        meshes.add(create_spider_leg_mesh(0)),
        meshes.add(create_spider_leg_mesh(1)),
        meshes.add(create_spider_leg_mesh(2)),
        meshes.add(create_spider_leg_mesh(3)),
    ];
    let spider_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.10, 0.09),
        perceptual_roughness: 0.75,
        metallic: 0.0,
        reflectance: 0.35,
        cull_mode: None,
        ..default()
    });

    let web_corner_mesh = meshes.add(create_corner_cobweb_mesh(0.8, 0.7, 0.8));
    let web_radial_mesh = meshes.add(create_radial_spiderweb_mesh(0.45));
    let web_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.92, 0.94, 0.96, 0.65),
        perceptual_roughness: 0.8,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        cull_mode: None,
        depth_bias: 150.0,
        ..default()
    });

    let assets = AmbientFaunaAssets {
        moth_body_mesh,
        moth_left_wing_mesh,
        moth_right_wing_mesh,
        moth_body_material: moth_body_mat,
        moth_wing_material: moth_wing_mat,
        fly_mesh,
        fly_material: fly_mat,
        spider_body_mesh,
        spider_leg_meshes,
        spider_material: spider_mat,
        web_corner_mesh,
        web_radial_mesh,
        web_material: web_mat,
    };
    commands.insert_resource(assets.clone());
    commands.init_resource::<AmbientFaunaSpawner>();
    assets
}

pub fn spawn_ambient_moth(
    commands: &mut Commands,
    assets: &AmbientFaunaAssets,
    anchor_pos: Vec3,
    seed: u32,
) -> Entity {
    let offset_x = ((seed as f32 * 0.13).sin()) * 0.22;
    let offset_y = (((seed as f32 * 0.27).cos()) * 0.12).abs() + 0.15;
    let offset_z = ((seed as f32 * 0.41).sin()) * 0.22;
    let spawn_pos = anchor_pos + Vec3::new(offset_x, offset_y, offset_z);
    commands
        .spawn((
            Name::new("Ambient_Moth"),
            AmbientInsect {
                kind: InsectKind::Moth,
                state: InsectState::Hovering,
                anchor_pos,
                base_scale: 1.0,
                flutter_phase: (seed % 100) as f32 * 0.0628,
                flutter_speed: 10.5 + (seed % 20) as f32 * 0.25,
                wander_offset: Vec3::new(offset_x, offset_y, offset_z),
                wander_target: Vec3::new(offset_x, offset_y, offset_z),
                wander_timer: 0.8,
                flee_timer: 0.0,
                flee_duration: 1.25,
                flee_velocity: Vec3::ZERO,
                puddle_entity: None,
            },
            Mesh3d(assets.moth_body_mesh.clone()),
            MeshMaterial3d(assets.moth_body_material.clone()),
            Transform::from_translation(spawn_pos),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Moth_Left_Wing"),
                Mesh3d(assets.moth_left_wing_mesh.clone()),
                MeshMaterial3d(assets.moth_wing_material.clone()),
                Transform::from_xyz(-0.0009, 0.0004, -0.0008),
                MothWing { is_left: true },
            ));
            parent.spawn((
                Name::new("Moth_Right_Wing"),
                Mesh3d(assets.moth_right_wing_mesh.clone()),
                MeshMaterial3d(assets.moth_wing_material.clone()),
                Transform::from_xyz(0.0009, 0.0004, -0.0008),
                MothWing { is_left: false },
            ));
        })
        .id()
}

pub fn spawn_ambient_water_fly(
    commands: &mut Commands,
    assets: &AmbientFaunaAssets,
    anchor_pos: Vec3,
    puddle_entity: Option<Entity>,
    seed: u32,
) -> Entity {
    let offset_x = ((seed as f32 * 0.31).sin()) * 0.18;
    let offset_y = 0.08 + (((seed as f32 * 0.57).cos()) * 0.06).abs();
    let offset_z = ((seed as f32 * 0.73).sin()) * 0.18;
    let spawn_pos = anchor_pos + Vec3::new(offset_x, offset_y, offset_z);
    commands
        .spawn((
            Name::new("Ambient_Water_Fly"),
            AmbientInsect {
                kind: InsectKind::WaterFly,
                state: InsectState::Hovering,
                anchor_pos,
                base_scale: 1.0,
                flutter_phase: (seed % 100) as f32 * 0.0628,
                flutter_speed: 48.0 + (seed % 30) as f32 * 0.7,
                wander_offset: Vec3::new(offset_x, offset_y, offset_z),
                wander_target: Vec3::new(offset_x, offset_y, offset_z),
                wander_timer: 0.25,
                flee_timer: 0.0,
                flee_duration: 0.85,
                flee_velocity: Vec3::ZERO,
                puddle_entity,
            },
            Mesh3d(assets.fly_mesh.clone()),
            MeshMaterial3d(assets.fly_material.clone()),
            Transform::from_translation(spawn_pos),
            Visibility::Inherited,
        ))
        .id()
}

pub fn spawn_ambient_spider(
    commands: &mut Commands,
    assets: &AmbientFaunaAssets,
    pos: Vec3,
    web_entity: Option<Entity>,
    seed: u32,
) -> Entity {
    commands
        .spawn((
            Name::new("Ambient_Spider"),
            AmbientInsect {
                kind: InsectKind::Spider,
                state: InsectState::Hovering,
                anchor_pos: pos,
                base_scale: 1.0,
                flutter_phase: (seed % 100) as f32 * 0.0628,
                flutter_speed: 1.0,
                wander_offset: Vec3::ZERO,
                wander_target: Vec3::ZERO,
                wander_timer: 2.2 + ((seed % 10) as f32 * 0.2),
                flee_timer: 0.0,
                flee_duration: 1.1,
                flee_velocity: Vec3::ZERO,
                puddle_entity: web_entity,
            },
            Mesh3d(assets.spider_body_mesh.clone()),
            MeshMaterial3d(assets.spider_material.clone()),
            Transform::from_translation(pos),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            for idx in 0..4 {
                parent.spawn((
                    Name::new("Spider_Leg_Left"),
                    Mesh3d(assets.spider_leg_meshes[idx].clone()),
                    MeshMaterial3d(assets.spider_material.clone()),
                    Transform::from_xyz(-0.0012, 0.0, 0.0015 - (idx as f32 * 0.0012)),
                    SpiderLeg {
                        leg_index: idx,
                        is_left: true,
                        base_rotation: Quat::IDENTITY,
                    },
                ));
                parent.spawn((
                    Name::new("Spider_Leg_Right"),
                    Mesh3d(assets.spider_leg_meshes[idx].clone()),
                    MeshMaterial3d(assets.spider_material.clone()),
                    Transform::from_xyz(0.0012, 0.0, 0.0015 - (idx as f32 * 0.0012))
                        .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                    SpiderLeg {
                        leg_index: idx,
                        is_left: false,
                        base_rotation: Quat::from_rotation_y(std::f32::consts::PI),
                    },
                ));
            }
        })
        .id()
}

pub fn spawn_spider_web(
    commands: &mut Commands,
    assets: &AmbientFaunaAssets,
    pos: Vec3,
    rotation: Quat,
    radius: f32,
    is_corner: bool,
    seed: u32,
) -> Entity {
    let roll_spider = (seed % 100) < 94;
    let mesh_handle = if is_corner {
        assets.web_corner_mesh.clone()
    } else {
        assets.web_radial_mesh.clone()
    };

    let mut spider_ent_opt = None;
    if roll_spider {
        let spider_pos = pos + rotation * Vec3::new(0.02, 0.01, 0.02);
        let spider_ent = spawn_ambient_spider(commands, assets, spider_pos, None, seed.wrapping_add(17));
        spider_ent_opt = Some(spider_ent);
    }

    let web_entity = commands
        .spawn((
            Name::new("Spider_Web"),
            SpiderWeb {
                radius,
                is_ruptured: false,
                rupture_timer: 0.0,
                rupture_duration: 0.45,
                attached_spider: spider_ent_opt,
            },
            Flammable::new(0.5)
                .with_ignition_temperature(1.5)
                .with_cooling_rate(2.0),
            Mesh3d(mesh_handle),
            MeshMaterial3d(assets.web_material.clone()),
            Transform::from_translation(pos).with_rotation(rotation),
            Visibility::Inherited,
        ))
        .id();

    if let Some(spider_ent) = spider_ent_opt {
        commands.entity(spider_ent).insert(AmbientInsect {
            kind: InsectKind::Spider,
            state: InsectState::Hovering,
            anchor_pos: pos + rotation * Vec3::new(0.02, 0.01, 0.02),
            base_scale: 1.0,
            flutter_phase: (seed % 100) as f32 * 0.0628,
            flutter_speed: 1.0,
            wander_offset: Vec3::ZERO,
            wander_target: Vec3::ZERO,
            wander_timer: 2.2 + ((seed % 10) as f32 * 0.2),
            flee_timer: 0.0,
            flee_duration: 1.1,
            flee_velocity: Vec3::ZERO,
            puddle_entity: Some(web_entity),
        });
    }

    web_entity
}

pub fn spawn_initial_ambient_fauna(
    commands: &mut Commands,
    assets: &AmbientFaunaAssets,
    mushroom_positions: &[Vec3],
    puddle_positions: &[(Vec3, Option<Entity>)],
    web_positions: &[(Vec3, Quat, f32, bool)],
) {
    for (idx, &mush_pos) in mushroom_positions.iter().enumerate() {
        spawn_ambient_moth(commands, assets, mush_pos, (idx as u32).wrapping_mul(73));
        if idx % 2 == 0 {
            spawn_ambient_moth(
                commands,
                assets,
                mush_pos + Vec3::new(0.08, 0.1, -0.06),
                (idx as u32).wrapping_mul(73).wrapping_add(19),
            );
        }
    }

    for (idx, &(puddle_pos, puddle_entity)) in puddle_positions.iter().enumerate() {
        let count = 2 + (idx % 2);
        for c in 0..count {
            let seed = (idx as u32).wrapping_mul(101).wrapping_add(c as u32 * 37);
            spawn_ambient_water_fly(commands, assets, puddle_pos, puddle_entity, seed);
        }
    }

    for (idx, &(pos, rot, rad, is_corner)) in web_positions.iter().enumerate() {
        let seed = (idx as u32).wrapping_mul(139).wrapping_add(53);
        spawn_spider_web(commands, assets, pos, rot, rad, is_corner, seed);
    }
}

pub fn update_spider_webs(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<SpiderWeb>, Without<PotionProjectile>)>,
    mut flammables_query: Query<&mut Flammable>,
    mut webs_query: Query<(Entity, &mut Transform, &mut SpiderWeb), (Without<Player>, Without<PotionProjectile>)>,
    projectiles_query: Query<(&Transform, &PotionProjectile), (Without<Player>, Without<SpiderWeb>)>,
    mut insects_query: Query<&mut AmbientInsect>,
) {
    let dt = time.delta_secs();
    let player_pos_opt = player_query.iter().next().map(|t| t.translation);

    for (entity, mut transform, mut web) in &mut webs_query {
        if let Ok(flammable) = flammables_query.get_mut(entity) {
            if flammable.is_burning && !web.is_ruptured {
                web.is_ruptured = true;
                web.rupture_timer = 0.0;
                if let Some(spider_ent) = web.attached_spider {
                    if let Ok(mut spider) = insects_query.get_mut(spider_ent) {
                        if spider.state == InsectState::Hovering {
                            spider.state = InsectState::Fleeing;
                            spider.flee_velocity = Vec3::new(1.0, 1.4, 0.5);
                            spider.flee_duration = 0.9;
                            spider.flee_timer = 0.0;
                        }
                    }
                }
            }
        }

        if !web.is_ruptured {
            if let Some(player_pos) = player_pos_opt {
                let player_torso = player_pos + Vec3::new(0.0, 0.9, 0.0);
                let dist_sq = transform.translation.distance_squared(player_torso);
                let trigger_dist = web.radius + 0.35;
                if dist_sq < trigger_dist * trigger_dist {
                    web.is_ruptured = true;
                    web.rupture_timer = 0.0;
                    if let Some(spider_ent) = web.attached_spider {
                        if let Ok(mut spider) = insects_query.get_mut(spider_ent) {
                            if spider.state == InsectState::Hovering {
                                spider.state = InsectState::Fleeing;
                                let mut away = (spider.anchor_pos - player_pos).normalize_or_zero();
                                if away.length_squared() < 0.001 {
                                    away = Vec3::Y;
                                }
                                spider.flee_velocity = (away + Vec3::Y * 0.5).normalize() * 1.8;
                                spider.flee_duration = 1.0;
                                spider.flee_timer = 0.0;
                            }
                        }
                    }
                }
            }

            for (proj_tf, proj) in &projectiles_query {
                let dist_sq = transform.translation.distance_squared(proj_tf.translation);
                let hit_dist = web.radius + 0.45;
                if dist_sq < hit_dist * hit_dist {
                    let is_fire = proj.effects.iter().any(|e| e.kind == AlchemicalEffectKind::ThermalHeat) || proj.slot == 2;
                    if is_fire {
                        if let Ok(mut flammable) = flammables_query.get_mut(entity) {
                            flammable.ignite();
                        }
                    }
                    web.is_ruptured = true;
                    web.rupture_timer = 0.0;
                    if let Some(spider_ent) = web.attached_spider {
                        if let Ok(mut spider) = insects_query.get_mut(spider_ent) {
                            if spider.state == InsectState::Hovering {
                                spider.state = InsectState::Fleeing;
                                spider.flee_velocity = Vec3::new(0.5, 1.5, 0.5);
                                spider.flee_duration = 1.0;
                                spider.flee_timer = 0.0;
                            }
                        }
                    }
                }
            }
        }

        if web.is_ruptured {
            web.rupture_timer += dt;
            let progress = (web.rupture_timer / web.rupture_duration).clamp(0.0, 1.0);
            if progress >= 1.0 {
                commands.entity(entity).despawn_recursive();
                continue;
            }
            let scale_factor = (1.0 - progress).powf(1.6).max(0.001);
            transform.scale = Vec3::new(scale_factor, scale_factor * 0.7, scale_factor);
        }
    }
}

pub fn update_ambient_insects(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    puddles_query: Query<&CavernWaterPuddle>,
    webs_query: Query<&SpiderWeb>,
    mut insects_query: Query<(
        Entity,
        &mut Transform,
        &mut AmbientInsect,
        Option<&Children>,
    ), Without<Player>>,
    mut decor_query: Query<(Option<&MothWing>, Option<&SpiderLeg>, &mut Transform), (Without<AmbientInsect>, Without<Player>)>,
) {
    let dt = time.delta_secs();
    let player_pos_opt = player_query.iter().next().map(|t| t.translation);

    for (entity, mut transform, mut insect, children_opt) in &mut insects_query {
        if let Some(puddle_ent) = insect.puddle_entity {
            if insect.kind == InsectKind::WaterFly {
                if let Ok(puddle) = puddles_query.get(puddle_ent) {
                    if puddle.state != WaterPuddleState::Liquid && insect.state == InsectState::Hovering {
                        insect.state = InsectState::Fleeing;
                        let seed = (transform.translation.x * 100.0).abs() as u32;
                        let hx = ((seed as f32 * 0.43).sin()) * 2.2;
                        let hz = ((seed as f32 * 0.87).cos()) * 2.2;
                        insect.flee_velocity = Vec3::new(hx, 3.5, hz);
                        insect.flee_duration = 0.85;
                        insect.flee_timer = 0.0;
                    }
                }
            } else if insect.kind == InsectKind::Spider {
                if let Ok(web) = webs_query.get(puddle_ent) {
                    if web.is_ruptured && insect.state == InsectState::Hovering {
                        insect.state = InsectState::Fleeing;
                        let seed = (transform.translation.x * 100.0).abs() as u32;
                        let hx = ((seed as f32 * 0.37).sin()) * 1.6;
                        let hz = ((seed as f32 * 0.79).cos()) * 1.6;
                        insect.flee_velocity = Vec3::new(hx, 1.2, hz);
                        insect.flee_duration = 1.0;
                        insect.flee_timer = 0.0;
                    }
                }
            }
        }

        if insect.state == InsectState::Hovering {
            if let Some(player_pos) = player_pos_opt {
                let to_insect = transform.translation - player_pos;
                let dist_sq = to_insect.length_squared();
                let threshold = match insect.kind {
                    InsectKind::Moth => 2.8,
                    InsectKind::WaterFly => 2.8,
                    InsectKind::Spider => 2.2,
                };
                if dist_sq < threshold * threshold {
                    insect.state = InsectState::Fleeing;
                    let mut flee_dir = to_insect.normalize_or_zero();
                    if flee_dir.length_squared() < 0.001 {
                        flee_dir = Vec3::Y;
                    }
                    let (speed, duration, upward) = match insect.kind {
                        InsectKind::Moth => (2.5, 1.25, 0.65),
                        InsectKind::WaterFly => (4.0, 0.85, 0.45),
                        InsectKind::Spider => (1.8, 1.10, 0.30),
                    };
                    flee_dir.y += upward;
                    let jitter_lat = ((transform.translation.x * 19.3 + transform.translation.z * 37.7).sin()) * 0.35;
                    flee_dir.x += jitter_lat;
                    flee_dir = flee_dir.normalize();
                    insect.flee_velocity = flee_dir * speed;
                    insect.flee_duration = duration;
                    insect.flee_timer = 0.0;
                }
            }
        }

        if insect.state == InsectState::Fleeing {
            insect.flee_timer += dt;
            let progress = (insect.flee_timer / insect.flee_duration).clamp(0.0, 1.0);
            if progress >= 1.0 {
                commands.entity(entity).despawn_recursive();
                continue;
            }
            transform.translation += insect.flee_velocity * dt;
            let scale_factor = (1.0 - progress).powf(1.4).max(0.0001);
            transform.scale = Vec3::splat(insect.base_scale * scale_factor);
            insect.flutter_phase += insect.flutter_speed * 1.8 * dt;

            if let Some(children) = children_opt {
                for child in children {
                    if let Ok((moth_opt, spider_opt, mut child_tf)) = decor_query.get_mut(*child) {
                        if let Some(wing) = moth_opt {
                            let angle = (insect.flutter_phase.sin() * 0.7) - 0.2;
                            let z_rot = if wing.is_left { angle } else { -angle };
                            child_tf.rotation = Quat::from_rotation_z(z_rot);
                        } else if let Some(leg) = spider_opt {
                            let gait = (insect.flutter_phase * 16.0).sin();
                            let is_tripod_a = (leg.leg_index % 2 == 0 && leg.is_left) || (leg.leg_index % 2 == 1 && !leg.is_left);
                            let swing = if is_tripod_a { gait * 0.45 } else { -gait * 0.45 };
                            child_tf.rotation = leg.base_rotation * Quat::from_rotation_y(swing);
                        }
                    }
                }
            }
            continue;
        }

        insect.flutter_phase += insect.flutter_speed * dt;
        insect.wander_timer -= dt;
        if insect.wander_timer <= 0.0 {
            let seed = (insect.flutter_phase * 10.0) as u32;
            match insect.kind {
                InsectKind::Moth => {
                    let rx = ((seed as f32 * 0.37).sin()) * 0.28;
                    let ry = (((seed as f32 * 0.61).cos()) * 0.15) + 0.10;
                    let rz = ((seed as f32 * 0.89).sin()) * 0.28;
                    insect.wander_target = Vec3::new(rx, ry, rz);
                    insect.wander_timer = 0.8 + ((seed % 10) as f32 * 0.05);
                }
                InsectKind::WaterFly => {
                    let rx = ((seed as f32 * 0.43).sin()) * 0.16;
                    let ry = (((seed as f32 * 0.71).cos()) * 0.05) + 0.08;
                    let rz = ((seed as f32 * 0.97).sin()) * 0.16;
                    insect.wander_target = Vec3::new(rx, ry, rz);
                    insect.wander_timer = 0.2 + ((seed % 8) as f32 * 0.02);
                }
                InsectKind::Spider => {
                    let rx = ((seed as f32 * 0.53).sin()) * 0.08;
                    let ry = (((seed as f32 * 0.83).cos()) * 0.04) * 0.5;
                    let rz = ((seed as f32 * 0.29).sin()) * 0.08;
                    insect.wander_target = Vec3::new(rx, ry, rz);
                    insect.wander_timer = 1.8 + ((seed % 12) as f32 * 0.15);
                }
            }
        }

        let lerp_speed = match insect.kind {
            InsectKind::Moth => 2.2,
            InsectKind::WaterFly => 6.0,
            InsectKind::Spider => 1.8,
        };
        insect.wander_offset = insect.wander_offset.lerp(insect.wander_target, (lerp_speed * dt).min(1.0));

        match insect.kind {
            InsectKind::Moth => {
                let bob_x = (insect.flutter_phase * 0.35).sin() * 0.035;
                let bob_y = (insect.flutter_phase * 0.70).cos() * 0.025;
                let bob_z = (insect.flutter_phase * 0.45).sin() * 0.035;
                transform.translation = insect.anchor_pos + insect.wander_offset + Vec3::new(bob_x, bob_y, bob_z);
                transform.rotation = Quat::from_rotation_y(insect.flutter_phase * 0.38) * Quat::from_rotation_x(bob_y * 1.8);
                transform.scale = Vec3::splat(insect.base_scale);

                if let Some(children) = children_opt {
                    for child in children {
                        if let Ok((Some(wing), _, mut wing_tf)) = decor_query.get_mut(*child) {
                            let angle = (insect.flutter_phase.sin() * 0.55) - 0.15;
                            let z_rot = if wing.is_left { angle } else { -angle };
                            wing_tf.rotation = Quat::from_rotation_z(z_rot);
                        }
                    }
                }
            }
            InsectKind::WaterFly => {
                let jitter_x = (insect.flutter_phase * 2.1).sin() * 0.003;
                let jitter_y = (insect.flutter_phase * 2.7).cos() * 0.0025;
                let jitter_z = (insect.flutter_phase * 2.4).sin() * 0.003;
                transform.translation = insect.anchor_pos + insect.wander_offset + Vec3::new(jitter_x, jitter_y, jitter_z);
                transform.rotation = Quat::from_rotation_y(insect.flutter_phase * 1.6);
                transform.scale = Vec3::splat(insect.base_scale);
            }
            InsectKind::Spider => {
                transform.translation = insect.anchor_pos + insect.wander_offset;
                transform.scale = Vec3::splat(insect.base_scale);

                let is_moving = insect.wander_timer < 0.8;
                if let Some(children) = children_opt {
                    for child in children {
                        if let Ok((_, Some(leg), mut leg_tf)) = decor_query.get_mut(*child) {
                            if is_moving {
                                let gait = (insect.flutter_phase * 8.0).sin();
                                let is_tripod_a = (leg.leg_index % 2 == 0 && leg.is_left) || (leg.leg_index % 2 == 1 && !leg.is_left);
                                let swing = if is_tripod_a { gait * 0.25 } else { -gait * 0.25 };
                                leg_tf.rotation = leg.base_rotation * Quat::from_rotation_y(swing);
                            } else {
                                let twitch = (insect.flutter_phase * 0.5 + leg.leg_index as f32).sin() * 0.03;
                                leg_tf.rotation = leg.base_rotation * Quat::from_rotation_y(twitch);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn update_ambient_fauna_spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<AmbientFaunaSpawner>,
    assets_opt: Option<Res<AmbientFaunaAssets>>,
    player_query: Query<&Transform, With<Player>>,
    insects_query: Query<(&AmbientInsect, &Transform)>,
    mushrooms_query: Query<(&GlobalTransform, &BioluminescentMushroom)>,
    lights_query: Query<(&GlobalTransform, &PointLight), Without<BioluminescentMushroom>>,
    puddles_query: Query<(Entity, &GlobalTransform, &CavernWaterPuddle)>,
    webs_query: Query<(Entity, &GlobalTransform, &SpiderWeb)>,
) {
    spawner.timer += time.delta_secs();
    if spawner.timer < spawner.interval {
        return;
    }
    spawner.timer = 0.0;
    spawner.spawn_seed = spawner.spawn_seed.wrapping_add(1);

    let Some(assets) = assets_opt else {
        return;
    };

    let mut current_moths = 0;
    let mut current_flies = 0;
    let mut current_spiders = 0;
    for (insect, _) in &insects_query {
        match insect.kind {
            InsectKind::Moth => current_moths += 1,
            InsectKind::WaterFly => current_flies += 1,
            InsectKind::Spider => current_spiders += 1,
        }
    }
    let current_webs = webs_query.iter().count();

    let player_pos = player_query.iter().next().map(|t| t.translation).unwrap_or(Vec3::ZERO);

    if current_moths < spawner.max_moths {
        for (gt, _) in &mushrooms_query {
            if current_moths >= spawner.max_moths {
                break;
            }
            let pos = gt.translation();
            let dist = pos.distance(player_pos);
            if dist >= 3.2 && dist <= 24.0 {
                let hash = pseudo_hash_coords(pos, spawner.spawn_seed);
                if hash % 100 < 32 {
                    let occupied = insects_query.iter().any(|(i, t)| {
                        i.kind == InsectKind::Moth && t.translation.distance_squared(pos) < 1.0
                    });
                    if !occupied {
                        spawn_ambient_moth(&mut commands, &assets, pos + Vec3::new(0.0, 0.15, 0.0), hash);
                        current_moths += 1;
                        if (hash >> 8) % 2 == 0 && current_moths < spawner.max_moths {
                            spawn_ambient_moth(&mut commands, &assets, pos + Vec3::new(0.12, 0.22, -0.08), hash.wrapping_add(99));
                            current_moths += 1;
                        }
                    }
                }
            }
        }

        if current_moths < spawner.max_moths {
            for (gt, _) in &lights_query {
                if current_moths >= spawner.max_moths {
                    break;
                }
                let pos = gt.translation();
                let dist = pos.distance(player_pos);
                if dist >= 3.2 && dist <= 24.0 {
                    let hash = pseudo_hash_coords(pos, spawner.spawn_seed.wrapping_mul(3));
                    if hash % 100 < 25 {
                        let occupied = insects_query.iter().any(|(i, t)| {
                            i.kind == InsectKind::Moth && t.translation.distance_squared(pos) < 1.0
                        });
                        if !occupied {
                            spawn_ambient_moth(&mut commands, &assets, pos + Vec3::new(0.0, -0.2, 0.0), hash);
                            current_moths += 1;
                        }
                    }
                }
            }
        }
    }

    if current_flies < spawner.max_flies {
        for (puddle_entity, gt, puddle) in &puddles_query {
            if current_flies >= spawner.max_flies {
                break;
            }
            if puddle.state != WaterPuddleState::Liquid {
                continue;
            }
            let pos = gt.translation();
            let dist = pos.distance(player_pos);
            if dist >= 3.2 && dist <= 24.0 {
                let hash = pseudo_hash_coords(pos, spawner.spawn_seed.wrapping_mul(13));
                if hash % 100 < 35 {
                    let occupied = insects_query.iter().any(|(i, t)| {
                        i.kind == InsectKind::WaterFly && t.translation.distance_squared(pos) < 0.8
                    });
                    if !occupied {
                        let count = 2 + (hash % 2) as usize;
                        for c in 0..count {
                            if current_flies >= spawner.max_flies {
                                break;
                            }
                            let sub_seed = hash.wrapping_add((c as u32).wrapping_mul(47));
                            spawn_ambient_water_fly(&mut commands, &assets, pos, Some(puddle_entity), sub_seed);
                            current_flies += 1;
                        }
                    }
                }
            }
        }
    }

    if current_webs < spawner.max_webs {
        let candidate_corners = [
            (Vec3::new(1.85, 4.10, 5.50), Quat::from_rotation_y(0.0), 0.75, true),
            (Vec3::new(-1.85, 0.35, 8.20), Quat::from_rotation_y(std::f32::consts::PI), 0.65, true),
            (Vec3::new(-1.85, 4.15, 12.00), Quat::from_rotation_y(std::f32::consts::PI), 0.70, true),
            (Vec3::new(1.85, 0.30, 15.50), Quat::from_rotation_y(0.0), 0.65, true),
            (Vec3::new(1.85, 4.15, 20.00), Quat::from_rotation_y(0.0), 0.70, true),
            (Vec3::new(-1.85, 0.35, 24.00), Quat::from_rotation_y(std::f32::consts::PI), 0.65, true),
        ];

        for (pos, rot, rad, is_corner) in candidate_corners {
            if current_webs >= spawner.max_webs {
                break;
            }
            let dist = pos.distance(player_pos);
            if dist >= 4.0 && dist <= 24.0 {
                let occupied = webs_query.iter().any(|(_, gt, _)| gt.translation().distance_squared(pos) < 2.0);
                if !occupied {
                    let hash = pseudo_hash_coords(pos, spawner.spawn_seed.wrapping_mul(29));
                    if hash % 100 < 30 {
                        spawn_spider_web(&mut commands, &assets, pos, rot, rad, is_corner, hash);
                    }
                }
            }
        }
    }

    if current_spiders < spawner.max_spiders {
        for (web_ent, gt, web) in &webs_query {
            if current_spiders >= spawner.max_spiders {
                break;
            }
            if web.attached_spider.is_none() && !web.is_ruptured {
                let pos = gt.translation();
                let hash = pseudo_hash_coords(pos, spawner.spawn_seed.wrapping_mul(31));
                let spider_pos = pos + Vec3::new(0.02, 0.01, 0.02);
                let spider_ent = spawn_ambient_spider(&mut commands, &assets, spider_pos, Some(web_ent), hash);
                commands.entity(web_ent).insert(SpiderWeb {
                    radius: web.radius,
                    is_ruptured: web.is_ruptured,
                    rupture_timer: web.rupture_timer,
                    rupture_duration: web.rupture_duration,
                    attached_spider: Some(spider_ent),
                });
                current_spiders += 1;
            }
        }
    }
}
