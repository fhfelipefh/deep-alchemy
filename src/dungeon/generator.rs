use crate::dungeon::{
    asset_loader::SceneAssetRegistry,
    modular_kit::{CavernObstacle, ObstacleShape},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimaryTile {
    Rock,
    Corridor,
    StartingGrotto,
    RoomCenter(RoomType),
    RoomPerimeter(RoomType),
}

impl PrimaryTile {
    pub fn is_walkable(&self) -> bool {
        matches!(
            self,
            PrimaryTile::Corridor
                | PrimaryTile::StartingGrotto
                | PrimaryTile::RoomCenter(_)
                | PrimaryTile::RoomPerimeter(_)
        )
    }

    pub fn is_solid(&self) -> bool {
        !self.is_walkable()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomType {
    AlchemicalShrine,
    CrystalGeode,
    AncientBridge,
    SporeSanctuary,
    AbyssalExit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalPiece {
    Empty,
    OuterCorner,
    StraightEdge,
    SaddleCrossing,
    InnerCorner,
    SolidPillar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DualCell {
    pub piece: CanonicalPiece,
    pub rotation_y_deg: f32,
    pub macro_room: Option<RoomType>,
    pub world_pos: Vec3,
}

#[derive(Debug, Clone)]
pub struct DungeonConfig {
    pub grid_width: usize,
    pub grid_depth: usize,
    pub tile_size: f32,
    pub seed: u64,
    pub min_corridor_steps: usize,
    pub branch_count: usize,
}

impl Default for DungeonConfig {
    fn default() -> Self {
        Self {
            grid_width: 25,
            grid_depth: 25,
            tile_size: 4.0,
            seed: 42,
            min_corridor_steps: 32,
            branch_count: 4,
        }
    }
}

pub struct ProceduralDungeon {
    pub config: DungeonConfig,
    pub primary_grid: Vec<PrimaryTile>,
    pub dual_cells: Vec<DualCell>,
    pub starting_anchor_world: Vec3,
    pub exits: Vec<IVec2>,
}

impl ProceduralDungeon {

    pub fn get_primary(&self, gx: usize, gz: usize) -> PrimaryTile {
        if gx >= self.config.grid_width || gz >= self.config.grid_depth {
            PrimaryTile::Rock
        } else {
            self.primary_grid[gz * self.config.grid_width + gx]
        }
    }

    pub fn grid_to_world(&self, gx: usize, gz: usize) -> Vec3 {
        let center_x = (self.config.grid_width as f32 * 0.5) * self.config.tile_size;
        let center_z = (self.config.grid_depth as f32 * 0.5) * self.config.tile_size;
        Vec3::new(
            (gx as f32 * self.config.tile_size) - center_x,
            0.0,
            (gz as f32 * self.config.tile_size) - center_z,
        )
    }
}

pub struct DungeonGenerator;

impl DungeonGenerator {

    pub fn generate(config: DungeonConfig) -> ProceduralDungeon {
        let mut rng_state = config.seed;
        let mut next_rand = || -> f32 {

            rng_state = rng_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((rng_state >> 32) as u32 as f32) / (u32::MAX as f32)
        };

        let width = config.grid_width;
        let depth = config.grid_depth;
        let mut grid = vec![PrimaryTile::Rock; width * depth];

        let cx = width / 2;
        let cz = depth / 2;
        let grotto_half_size = 2;

        for dz in -grotto_half_size..=0 {
            for dx in -grotto_half_size..=grotto_half_size {
                let gx = (cx as isize + dx) as usize;
                let gz = (cz as isize + dz) as usize;
                if gx < width && gz < depth {
                    grid[gz * width + gx] = PrimaryTile::StartingGrotto;
                }
            }
        }
        grid[(cz + 1) * width + cx] = PrimaryTile::StartingGrotto;

        let mut exits = Vec::new();
        let north_exit = IVec2::new(cx as i32, (cz + 2) as i32);
        exits.push(north_exit);
        grid[north_exit.y as usize * width + north_exit.x as usize] = PrimaryTile::Corridor;

        let junction_z = 19.min(depth - 3);
        for gz in (cz + 2)..=junction_z {
            grid[gz * width + cx] = PrimaryTile::Corridor;
        }

        let north_room_z = 21.min(depth - 4);
        for gz in (junction_z + 1)..=north_room_z {
            grid[gz * width + cx] = PrimaryTile::Corridor;
        }
        Self::carve_macro_chamber(&mut grid, width, depth, cx, north_room_z, RoomType::AncientBridge);

        let exit_chamber_z = depth - 2;
        for gz in (north_room_z + 1)..=exit_chamber_z {
            grid[gz * width + cx] = PrimaryTile::Corridor;
        }
        Self::carve_macro_chamber(&mut grid, width, depth, cx, exit_chamber_z, RoomType::AbyssalExit);

        let east_room_x = (width - 4).min(cx + 6);
        for gx in (cx + 1)..=east_room_x {
            grid[junction_z * width + gx] = PrimaryTile::Corridor;
        }
        Self::carve_macro_chamber(&mut grid, width, depth, east_room_x, junction_z, RoomType::CrystalGeode);

        let west_room_x = 3.max(cx.saturating_sub(6));
        for gx in west_room_x..cx {
            grid[junction_z * width + gx] = PrimaryTile::Corridor;
        }
        Self::carve_macro_chamber(&mut grid, width, depth, west_room_x, junction_z, RoomType::SporeSanctuary);

        let shrine_x = east_room_x;
        let shrine_z = (depth - 3).min(junction_z + 3);
        for gz in (junction_z + 1)..=shrine_z {
            grid[gz * width + shrine_x] = PrimaryTile::Corridor;
        }
        Self::carve_macro_chamber(&mut grid, width, depth, shrine_x, shrine_z, RoomType::AlchemicalShrine);

        let walker_starts = [
            IVec2::new(cx as i32, junction_z as i32),
            IVec2::new(west_room_x as i32, junction_z as i32),
            IVec2::new(east_room_x as i32, junction_z as i32),
            IVec2::new(cx as i32, north_room_z as i32),
            IVec2::new(cx as i32, exit_chamber_z as i32),
            IVec2::new(shrine_x as i32, shrine_z as i32),
        ];

        let steps_per_walker = config.min_corridor_steps / walker_starts.len().max(1);
        for &start in &walker_starts {
            let mut curr = start;
            let mut dir = if next_rand() < 0.5 {
                if next_rand() < 0.5 { IVec2::new(1, 0) } else { IVec2::new(-1, 0) }
            } else {
                if next_rand() < 0.5 { IVec2::new(0, 1) } else { IVec2::new(0, -1) }
            };

            for _ in 0..steps_per_walker {
                if next_rand() < 0.35 {
                    dir = if dir.x != 0 {
                        if next_rand() < 0.5 { IVec2::new(0, 1) } else { IVec2::new(0, -1) }
                    } else {
                        if next_rand() < 0.5 { IVec2::new(1, 0) } else { IVec2::new(-1, 0) }
                    };
                }

                let next_pos = curr + dir;
                if next_pos.x < 2
                    || next_pos.x >= (width as i32 - 2)
                    || next_pos.y < (cz as i32 + 2)
                    || next_pos.y >= (depth as i32 - 2)
                {
                    dir = -dir;
                    continue;
                }

                curr = next_pos;
                let idx = curr.y as usize * width + curr.x as usize;
                if grid[idx] == PrimaryTile::Rock {
                    grid[idx] = PrimaryTile::Corridor;
                }
            }
        }

        Self::apply_macro_pattern_matching(&mut grid, width, depth, &mut next_rand);

        Self::ensure_full_connectivity(&mut grid, width, depth, cx, cz);

        let dual_cells = Self::compute_dual_grid(&grid, width, depth, config.tile_size);

        ProceduralDungeon {
            config,
            primary_grid: grid,
            dual_cells,
            starting_anchor_world: Vec3::ZERO,
            exits,
        }
    }

    fn carve_macro_chamber(
        grid: &mut [PrimaryTile],
        width: usize,
        depth: usize,
        cx: usize,
        cz: usize,
        room_type: RoomType,
    ) {
        for dz in -1..=1 {
            for dx in -1..=1 {
                let gx = (cx as isize + dx) as usize;
                let gz = (cz as isize + dz) as usize;
                if gx >= 1 && gx < width - 1 && gz >= 1 && gz < depth - 1 {
                    let idx = gz * width + gx;
                    if dx == 0 && dz == 0 {
                        grid[idx] = PrimaryTile::RoomCenter(room_type);
                    } else if grid[idx] == PrimaryTile::Rock {
                        grid[idx] = PrimaryTile::RoomPerimeter(room_type);
                    }
                }
            }
        }
    }

    fn apply_macro_pattern_matching<F: FnMut() -> f32>(
        grid: &mut [PrimaryTile],
        width: usize,
        depth: usize,
        rng: &mut F,
    ) {
        let cx = width / 2;
        for gz in 1..(depth - 2) {
            for gx in 1..(width - 2) {
                if gx == cx && gz <= 19 {
                    continue;
                }

                let idx00 = gz * width + gx;
                let idx10 = gz * width + (gx + 1);
                let idx01 = (gz + 1) * width + gx;
                let idx11 = (gz + 1) * width + (gx + 1);

                if grid[idx00] == PrimaryTile::Corridor
                    && grid[idx10] == PrimaryTile::Corridor
                    && grid[idx01] == PrimaryTile::Corridor
                    && grid[idx11] == PrimaryTile::Corridor
                {
                    let room_type = if rng() < 0.5 {
                        RoomType::CrystalGeode
                    } else {
                        RoomType::AlchemicalShrine
                    };

                    grid[idx00] = PrimaryTile::RoomCenter(room_type);
                    grid[idx10] = PrimaryTile::RoomPerimeter(room_type);
                    grid[idx01] = PrimaryTile::RoomPerimeter(room_type);
                    grid[idx11] = PrimaryTile::RoomPerimeter(room_type);
                }
            }
        }
    }

    fn ensure_full_connectivity(
        grid: &mut [PrimaryTile],
        width: usize,
        depth: usize,
        start_x: usize,
        start_z: usize,
    ) {
        loop {
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back((start_x, start_z));
            visited.insert((start_x, start_z));

            while let Some((x, z)) = queue.pop_front() {
                let neighbors = [
                    (x.wrapping_sub(1), z),
                    (x + 1, z),
                    (x, z.wrapping_sub(1)),
                    (x, z + 1),
                ];
                for (nx, nz) in neighbors {
                    if nx < width && nz < depth && !visited.contains(&(nx, nz)) {
                        if grid[nz * width + nx].is_walkable() {
                            visited.insert((nx, nz));
                            queue.push_back((nx, nz));
                        }
                    }
                }
            }

            let mut unvisited_walkables = Vec::new();
            for gz in 0..depth {
                for gx in 0..width {
                    if grid[gz * width + gx].is_walkable() && !visited.contains(&(gx, gz)) {
                        unvisited_walkables.push((gx, gz));
                    }
                }
            }

            if unvisited_walkables.is_empty() {
                break;
            }

            let target = unvisited_walkables[0];
            let mut path_queue = VecDeque::new();
            let mut path_visited = HashSet::new();
            let mut parent: HashMap<(usize, usize), (usize, usize)> = HashMap::new();

            path_queue.push_back(target);
            path_visited.insert(target);
            let mut found_conn = None;

            while let Some((x, z)) = path_queue.pop_front() {
                if visited.contains(&(x, z)) {
                    found_conn = Some((x, z));
                    break;
                }

                let neighbors = [
                    (x.wrapping_sub(1), z),
                    (x + 1, z),
                    (x, z.wrapping_sub(1)),
                    (x, z + 1),
                ];
                for (nx, nz) in neighbors {
                    if nx >= 1 && nx < width - 1 && nz >= 1 && nz < depth - 1 && !path_visited.contains(&(nx, nz)) {
                        path_visited.insert((nx, nz));
                        parent.insert((nx, nz), (x, z));
                        path_queue.push_back((nx, nz));
                    }
                }
            }

            if let Some(mut curr) = found_conn {
                while curr != target {
                    let idx = curr.1 * width + curr.0;
                    if grid[idx] == PrimaryTile::Rock {
                        grid[idx] = PrimaryTile::Corridor;
                    }
                    if let Some(&p) = parent.get(&curr) {
                        curr = p;
                    } else {
                        break;
                    }
                }
                let idx = target.1 * width + target.0;
                if grid[idx] == PrimaryTile::Rock {
                    grid[idx] = PrimaryTile::Corridor;
                }
            } else {
                let idx = target.1 * width + target.0;
                grid[idx] = PrimaryTile::Rock;
            }
        }
    }

    pub fn compute_dual_grid(
        grid: &[PrimaryTile],
        width: usize,
        depth: usize,
        tile_size: f32,
    ) -> Vec<DualCell> {
        let mut dual_cells = Vec::new();
        let center_offset_x = (width as f32 * 0.5) * tile_size;
        let center_offset_z = (depth as f32 * 0.5) * tile_size;

        for gz in 0..(depth - 1) {
            for gx in 0..(width - 1) {

                let c00 = grid[gz * width + gx].is_solid();
                let c10 = grid[gz * width + (gx + 1)].is_solid();
                let c11 = grid[(gz + 1) * width + (gx + 1)].is_solid();
                let c01 = grid[(gz + 1) * width + gx].is_solid();

                let (piece, rot_deg) = Self::resolve_canonical_piece(c00, c10, c11, c01);

                let wx = ((gx as f32 + 0.5) * tile_size) - center_offset_x;
                let wz = ((gz as f32 + 0.5) * tile_size) - center_offset_z;

                let macro_room = [
                    grid[gz * width + gx],
                    grid[gz * width + (gx + 1)],
                    grid[(gz + 1) * width + (gx + 1)],
                    grid[(gz + 1) * width + gx],
                ]
                .iter()
                .find_map(|t| match t {
                    PrimaryTile::RoomCenter(rt) => Some(*rt),
                    _ => None,
                });

                dual_cells.push(DualCell {
                    piece,
                    rotation_y_deg: rot_deg,
                    macro_room,
                    world_pos: Vec3::new(wx, 0.0, wz),
                });
            }
        }

        dual_cells
    }

    pub fn resolve_canonical_piece(
        c00: bool,
        c10: bool,
        c11: bool,
        c01: bool,
    ) -> (CanonicalPiece, f32) {
        let mask = (if c00 { 1 } else { 0 })
            | (if c10 { 2 } else { 0 })
            | (if c11 { 4 } else { 0 })
            | (if c01 { 8 } else { 0 });

        match mask {

            0 => (CanonicalPiece::Empty, 0.0),

            1 => (CanonicalPiece::OuterCorner, 0.0),
            2 => (CanonicalPiece::OuterCorner, 270.0),
            4 => (CanonicalPiece::OuterCorner, 180.0),
            8 => (CanonicalPiece::OuterCorner, 90.0),

            9 => (CanonicalPiece::StraightEdge, 0.0),
            3 => (CanonicalPiece::StraightEdge, 270.0),
            6 => (CanonicalPiece::StraightEdge, 180.0),
            12 => (CanonicalPiece::StraightEdge, 90.0),

            5 => (CanonicalPiece::SaddleCrossing, 0.0),
            10 => (CanonicalPiece::SaddleCrossing, 90.0),

            14 => (CanonicalPiece::InnerCorner, 0.0),
            13 => (CanonicalPiece::InnerCorner, 270.0),
            11 => (CanonicalPiece::InnerCorner, 180.0),
            7 => (CanonicalPiece::InnerCorner, 90.0),

            15 => (CanonicalPiece::SolidPillar, 0.0),

            _ => (CanonicalPiece::Empty, 0.0),
        }
    }

    pub fn verify_reachability(dungeon: &ProceduralDungeon) -> bool {
        let width = dungeon.config.grid_width;
        let depth = dungeon.config.grid_depth;
        let cx = width / 2;
        let cz = depth / 2;

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((cx, cz));
        visited.insert((cx, cz));

        while let Some((x, z)) = queue.pop_front() {
            let neighbors = [
                (x.wrapping_sub(1), z),
                (x + 1, z),
                (x, z.wrapping_sub(1)),
                (x, z + 1),
            ];

            for (nx, nz) in neighbors {
                if nx < width && nz < depth && !visited.contains(&(nx, nz)) {
                    if dungeon.get_primary(nx, nz).is_walkable() {
                        visited.insert((nx, nz));
                        queue.push_back((nx, nz));
                    }
                }
            }
        }

        for gz in 0..depth {
            for gx in 0..width {
                if dungeon.get_primary(gx, gz).is_walkable() && !visited.contains(&(gx, gz)) {
                    return false;
                }
            }
        }

        true
    }
}

pub fn spawn_dungeon_wall_sconce(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &SceneAssetRegistry,
    position: Vec3,
    rotation: Quat,
    is_initially_lit: bool,
) -> Entity {
    let light_offset = rotation * Vec3::new(0.0, 0.28, 0.22);
    let light_entity = commands
        .spawn((
            Name::new("PointLight_Dungeon_Wall_Torch"),
            Transform::from_translation(position + light_offset),
            PointLight {
                color: Color::srgb(1.0, 0.74, 0.38),
                intensity: if is_initially_lit { 3200.0 } else { 0.0 },
                range: 10.0,
                shadows_enabled: false,
                ..default()
            },
        ))
        .id();

    let flame_entity = if is_initially_lit {
        Some(
            commands
                .spawn((
                    Transform::from_translation(position + light_offset),
                    crate::render::particles::ParticleEmitter {
                        spawn_timer: 0.0,
                        spawn_rate: 0.04,
                        particle_type: crate::render::particles::ParticleType::FireFlame,
                    },
                ))
                .id(),
        )
    } else {
        None
    };

    let handle = registry.load_scene(asset_server, "prop_wall_sconce");
    commands
        .spawn((
            Name::new("Prop_Wall_Sconce"),
            SceneRoot(handle),
            Transform::from_translation(position).with_rotation(rotation),
            crate::alchemy::DungeonWallTorch {
                is_lit: is_initially_lit,
                burn_timer: if is_initially_lit { 1800.0 } else { 0.0 },
                max_burn_time: 1800.0,
                initial_light_intensity: 3200.0,
                light_entity: Some(light_entity),
                flame_entity,
                dirty: false,
            },
        ))
        .id()
}

pub fn spawn_procedural_dungeon(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<SceneAssetRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = DungeonConfig::default();
    let dungeon = DungeonGenerator::generate(config.clone());
    let half_tile = config.tile_size * 0.5;

    for cell in &dungeon.dual_cells {
        let is_exit_connector =
            cell.world_pos.x.abs() < 1.0 && (cell.world_pos.z - 4.0).abs() < 1.0;
        if is_exit_connector {
            let handle = registry.load_scene(&asset_server, "tile_grotto_connector");
            commands.spawn((
                Name::new("DungeonTile_GrottoConnector"),
                SceneRoot(handle),
                Transform::from_translation(cell.world_pos),
            ));
            commands.spawn((
                Name::new("Collider_Connector_Wall_Left"),
                CavernObstacle,
                ObstacleShape::Cuboid {
                    half_extents: Vec3::new(0.25, 2.0, 2.0),
                },
                Transform::from_xyz(-2.2, 2.0, 6.0),
                RigidBody::Static,
                Collider::cuboid(0.5, 4.0, 4.0),
            ));
            commands.spawn((
                Name::new("Collider_Connector_Wall_Right"),
                CavernObstacle,
                ObstacleShape::Cuboid {
                    half_extents: Vec3::new(0.25, 2.0, 2.0),
                },
                Transform::from_xyz(2.2, 2.0, 6.0),
                RigidBody::Static,
                Collider::cuboid(0.5, 4.0, 4.0),
            ));
            commands.spawn((
                Name::new("Collider_Connector_Floor"),
                Transform::from_xyz(0.0, -0.2, 6.0),
                RigidBody::Static,
                Collider::cuboid(4.4, 0.4, 4.0),
            ));
            commands.spawn((
                Name::new("Light_Connector_Vault"),
                Transform::from_xyz(0.0, 3.2, 6.0),
                PointLight {
                    color: Color::srgb(1.0, 0.78, 0.42),
                    intensity: 1800.0,
                    range: 8.5,
                    shadows_enabled: false,
                    ..default()
                },
            ));
            continue;
        }

        let is_exit_corridor =
            cell.world_pos.x.abs() < 1.0 && cell.world_pos.z >= 7.0 && cell.world_pos.z <= 26.0;
        if is_exit_corridor {
            let handle = registry.load_scene(&asset_server, "tile_corridor_straight");
            commands.spawn((
                Name::new("DungeonTile_ExitCorridor"),
                SceneRoot(handle),
                Transform::from_translation(cell.world_pos),
                RigidBody::Static,
                Collider::cuboid(config.tile_size, 0.4, config.tile_size),
            ));

            spawn_dungeon_wall_sconce(
                &mut commands,
                &asset_server,
                &registry,
                cell.world_pos + Vec3::new(-1.18, 2.05, 0.0),
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                false,
            );
            spawn_dungeon_wall_sconce(
                &mut commands,
                &asset_server,
                &registry,
                cell.world_pos + Vec3::new(1.18, 2.05, 0.0),
                Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
                false,
            );
            commands.spawn((
                Name::new("Light_Corridor_Vault"),
                Transform::from_translation(cell.world_pos + Vec3::new(0.0, 3.2, 0.0)),
                PointLight {
                    color: Color::srgb(1.0, 0.78, 0.44),
                    intensity: 1800.0,
                    range: 9.0,
                    shadows_enabled: false,
                    ..default()
                },
            ));

            let cx = config.grid_width / 2;
            let center_offset_z = (config.grid_depth as f32 * 0.5) * config.tile_size;
            let gz = (((cell.world_pos.z + center_offset_z) / config.tile_size) - 0.5).round() as usize;

            let west_solid = (cx > 0) && dungeon.get_primary(cx - 1, gz).is_solid();
            let east_solid = (cx + 1 < config.grid_width) && dungeon.get_primary(cx + 1, gz).is_solid();

            if west_solid {
                let wall_offset_left = Vec3::new(-half_tile + 0.425, 2.5, 0.0);
                commands.spawn((
                    Name::new("Collider_Dungeon_Wall_Straight_L"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(0.425, 2.5, config.tile_size * 0.5),
                    },
                    Transform::from_translation(cell.world_pos + wall_offset_left),
                    RigidBody::Static,
                    Collider::cuboid(0.85, 5.0, config.tile_size),
                ));
            }

            if east_solid {
                let wall_offset_right = Vec3::new(half_tile - 0.425, 2.5, 0.0);
                commands.spawn((
                    Name::new("Collider_Dungeon_Wall_Straight_R"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(0.425, 2.5, config.tile_size * 0.5),
                    },
                    Transform::from_translation(cell.world_pos + wall_offset_right),
                    RigidBody::Static,
                    Collider::cuboid(0.85, 5.0, config.tile_size),
                ));
            }
            continue;
        }

        if cell.world_pos.x.abs() < 10.0 && cell.world_pos.z > -16.0 && cell.world_pos.z < 6.0 {
            continue;
        }

        if cell.world_pos.x.abs() < 1.0 && (cell.world_pos.z - 28.0).abs() < 1.0 {
            let handle = registry.load_scene(&asset_server, "tile_corridor_open");
            commands.spawn((
                Name::new("DungeonTile_GrandJunction"),
                SceneRoot(handle),
                Transform::from_translation(cell.world_pos),
                RigidBody::Static,
                Collider::cuboid(config.tile_size, 0.4, config.tile_size),
            ));
            commands.spawn((
                Name::new("Light_Grand_Junction"),
                Transform::from_translation(cell.world_pos + Vec3::new(0.0, 3.3, 0.0)),
                PointLight {
                    color: Color::srgb(1.0, 0.78, 0.42),
                    intensity: 9500.0,
                    range: 24.0,
                    shadows_enabled: false,
                    ..default()
                },
            ));
            let corner_sconces = [
                (Vec3::new(-1.62, 2.05, -1.2), Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                (Vec3::new(1.62, 2.05, -1.2), Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
                (Vec3::new(-1.62, 2.05, 1.2), Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                (Vec3::new(1.62, 2.05, 1.2), Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
            ];
            for (offset, rot) in corner_sconces {
                spawn_dungeon_wall_sconce(
                    &mut commands,
                    &asset_server,
                    &registry,
                    cell.world_pos + offset,
                    rot,
                    false,
                );
            }
            for (cx_sign, cz_sign) in [(-1.0f32, -1.0f32), (1.0f32, -1.0f32), (-1.0f32, 1.0f32), (1.0f32, 1.0f32)] {
                let pillar_pos = cell.world_pos + Vec3::new(cx_sign * (half_tile - 0.425), 2.5, cz_sign * (half_tile - 0.425));
                commands.spawn((
                    Name::new("Collider_Dungeon_GrandJunction_Corner"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(0.425, 2.5, 0.425),
                    },
                    Transform::from_translation(pillar_pos),
                    RigidBody::Static,
                    Collider::cuboid(0.85, 5.0, 0.85),
                ));
            }
            continue;
        }


        let rot = Quat::from_rotation_y(cell.rotation_y_deg.to_radians());

        match cell.piece {
            CanonicalPiece::Empty => {
                let handle = registry.load_scene(&asset_server, "tile_corridor_open");
                commands.spawn((
                    Name::new("DungeonTile_Empty"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 0.4, config.tile_size),
                ));

                if let Some(macro_type) = cell.macro_room {
                    match macro_type {
                        RoomType::CrystalGeode => {
                            let coord_seed = (cell.world_pos.x.abs() as u32).wrapping_mul(73856093)
                                ^ (cell.world_pos.z.abs() as u32).wrapping_mul(19349663);
                            let variant_keys = [
                                "macro_crystal_cluster",
                                "macro_crystal_cluster_b",
                                "macro_crystal_cluster_c",
                            ];
                            let primary_key = variant_keys[(coord_seed % 3) as usize];
                            let primary_scale = 0.88 + ((coord_seed % 38) as f32) * 0.01;
                            let primary_yaw = ((coord_seed % 360) as f32).to_radians();

                            let primary_handle = registry.load_scene(&asset_server, primary_key);
                            commands.spawn((
                                Name::new("MacroRoom_CrystalGeode"),
                                SceneRoot(primary_handle),
                                Transform::from_translation(cell.world_pos)
                                    .with_rotation(Quat::from_rotation_y(primary_yaw))
                                    .with_scale(Vec3::splat(primary_scale)),
                                PointLight {
                                    color: Color::srgb(0.17, 0.91, 0.85),
                                    intensity: 22000.0,
                                    range: 16.0,
                                    shadows_enabled: false,
                                    ..default()
                                },
                            ));

                            let satellite_offsets = [
                                (Vec3::new(1.35, 0.0, 1.25), 1usize, 0.48, 1.25),
                                (Vec3::new(-1.25, 0.0, 1.35), 2usize, 0.42, 2.65),
                                (Vec3::new(1.20, 0.0, -1.30), 0usize, 0.38, 4.10),
                            ];
                            for (offset, v_idx_offset, base_scale, yaw_offset) in satellite_offsets {
                                let sat_key = variant_keys[(coord_seed as usize + v_idx_offset) % 3];
                                let sat_handle = registry.load_scene(&asset_server, sat_key);
                                let sat_scale = base_scale + (((coord_seed >> (v_idx_offset * 3)) % 18) as f32) * 0.01;
                                commands.spawn((
                                    Name::new("MacroRoom_CrystalGeode_Cluster"),
                                    SceneRoot(sat_handle),
                                    Transform::from_translation(cell.world_pos + offset)
                                        .with_rotation(Quat::from_rotation_y(primary_yaw + yaw_offset))
                                        .with_scale(Vec3::splat(sat_scale)),
                                ));
                            }
                        }
                        RoomType::AlchemicalShrine => {
                            let shrine_handle =
                                registry.load_scene(&asset_server, "macro_alchemical_shrine");
                            commands.spawn((
                                Name::new("MacroRoom_AlchemicalShrine"),
                                SceneRoot(shrine_handle),
                                Transform::from_translation(
                                    cell.world_pos + Vec3::new(0.0, 0.5, 0.0),
                                ),
                                RigidBody::Static,
                                Collider::cuboid(1.1, 1.2, 1.1),
                                PointLight {
                                    color: Color::srgb(1.0, 0.72, 0.28),
                                    intensity: 24000.0,
                                    range: 16.0,
                                    shadows_enabled: false,
                                    ..default()
                                },
                            ));
                        }
                        RoomType::AncientBridge => {
                            commands.spawn((
                                Name::new("MacroRoom_AncientBridge_Light"),
                                Transform::from_translation(
                                    cell.world_pos + Vec3::new(0.0, 2.8, 0.0),
                                ),
                                PointLight {
                                    color: Color::srgb(0.25, 0.65, 1.0),
                                    intensity: 22000.0,
                                    range: 16.0,
                                    shadows_enabled: false,
                                    ..default()
                                },
                            ));
                        }
                        RoomType::SporeSanctuary => {
                            let mushroom_handle =
                                registry.load_scene(&asset_server, "prop_mushroom_cyan");
                            commands.spawn((
                                Name::new("MacroRoom_SporeSanctuary_Flora"),
                                SceneRoot(mushroom_handle),
                                Transform::from_translation(
                                    cell.world_pos + Vec3::new(0.0, 0.2, 0.0),
                                ),
                                PointLight {
                                    color: Color::srgb(0.15, 0.95, 0.55),
                                    intensity: 20000.0,
                                    range: 16.0,
                                    shadows_enabled: false,
                                    ..default()
                                },
                            ));
                        }
                        RoomType::AbyssalExit => {
                            commands.spawn((
                                Name::new("MacroRoom_AbyssalExit_Light"),
                                Transform::from_translation(cell.world_pos + Vec3::new(0.0, 3.4, 1.0)),
                                PointLight {
                                    color: Color::srgb(0.20, 0.72, 0.94),
                                    intensity: 26000.0,
                                    range: 18.0,
                                    shadows_enabled: false,
                                    ..default()
                                },
                            ));

                            commands.spawn((
                                Name::new("AbyssalVictoryExitTrigger"),
                                AbyssalVictoryExitTrigger,
                                Transform::from_translation(cell.world_pos + Vec3::new(0.0, 1.0, 1.0)),
                                Collider::cuboid(6.0, 4.0, 6.0),
                                Sensor,
                            ));
                        }
                    }
                } else {
                    commands.spawn((
                        Name::new("Light_Dungeon_Corridor_Open"),
                        Transform::from_translation(cell.world_pos + Vec3::new(0.0, 3.2, 0.0)),
                        PointLight {
                            color: Color::srgb(1.0, 0.76, 0.42),
                            intensity: 2400.0,
                            range: 11.0,
                            shadows_enabled: false,
                            ..default()
                        },
                    ));
                }
            }
            CanonicalPiece::StraightEdge => {
                let handle = registry.load_scene(&asset_server, "tile_straight_edge");
                commands.spawn((
                    Name::new("DungeonTile_StraightEdge"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 0.4, config.tile_size),
                ));

                let wall_offset = rot * Vec3::new(-half_tile + 0.425, 2.5, 0.0);
                commands.spawn((
                    Name::new("Collider_Dungeon_Wall_Straight"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(0.425, 2.5, config.tile_size * 0.5),
                    },
                    Transform::from_translation(cell.world_pos + wall_offset).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(0.85, 5.0, config.tile_size),
                ));

                let torch_offset = rot * Vec3::new(-half_tile + 0.45, 2.05, 0.0);
                let torch_rot = rot * Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
                spawn_dungeon_wall_sconce(
                    &mut commands,
                    &asset_server,
                    &registry,
                    cell.world_pos + torch_offset,
                    torch_rot,
                    false,
                );
            }
            CanonicalPiece::OuterCorner => {
                let handle = registry.load_scene(&asset_server, "tile_outer_corner");
                commands.spawn((
                    Name::new("DungeonTile_OuterCorner"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 0.4, config.tile_size),
                ));

                let pillar_offset = rot * Vec3::new(-half_tile + 0.425, 2.5, half_tile - 0.425);
                commands.spawn((
                    Name::new("Collider_Dungeon_Pillar_Corner"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(0.425, 2.5, 0.425),
                    },
                    Transform::from_translation(cell.world_pos + pillar_offset).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(0.85, 5.0, 0.85),
                ));

                let torch_offset = rot * Vec3::new(-half_tile + 0.45, 2.05, half_tile - 0.45);
                let torch_rot = rot * Quat::from_rotation_y(std::f32::consts::FRAC_PI_4);
                spawn_dungeon_wall_sconce(
                    &mut commands,
                    &asset_server,
                    &registry,
                    cell.world_pos + torch_offset,
                    torch_rot,
                    false,
                );
            }
            CanonicalPiece::InnerCorner => {
                let handle = registry.load_scene(&asset_server, "tile_corridor_corner");
                commands.spawn((
                    Name::new("DungeonTile_InnerCorner"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 0.4, config.tile_size),
                ));

                let wall_right_offset = rot * Vec3::new(half_tile - 0.425, 2.5, 0.0);
                commands.spawn((
                    Name::new("Collider_Dungeon_Wall_InnerCorner_R"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(0.425, 2.5, config.tile_size * 0.5),
                    },
                    Transform::from_translation(cell.world_pos + wall_right_offset).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(0.85, 5.0, config.tile_size),
                ));

                let wall_back_offset = rot * Vec3::new(0.0, 2.5, half_tile - 0.425);
                commands.spawn((
                    Name::new("Collider_Dungeon_Wall_InnerCorner_B"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(config.tile_size * 0.5, 2.5, 0.425),
                    },
                    Transform::from_translation(cell.world_pos + wall_back_offset).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 5.0, 0.85),
                ));

                let corner_rock_offset = rot * Vec3::new(half_tile - 0.5, 2.5, half_tile - 0.5);
                commands.spawn((
                    Name::new("Collider_Dungeon_Corner_Rock"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(0.5, 2.5, 0.5),
                    },
                    Transform::from_translation(cell.world_pos + corner_rock_offset).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(1.0, 5.0, 1.0),
                ));

                let torch_offset = rot * Vec3::new(half_tile - 0.45, 2.05, half_tile - 0.45);
                let torch_rot = rot * Quat::from_rotation_y(-3.0 * std::f32::consts::FRAC_PI_4);
                spawn_dungeon_wall_sconce(
                    &mut commands,
                    &asset_server,
                    &registry,
                    cell.world_pos + torch_offset,
                    torch_rot,
                    false,
                );
            }
            CanonicalPiece::SolidPillar => {
                let handle = registry.load_scene(&asset_server, "tile_solid_pillar");
                commands.spawn((
                    Name::new("DungeonTile_SolidPillar"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 0.4, config.tile_size),
                ));

                commands.spawn((
                    Name::new("DungeonTile_BedrockObstacle"),
                    Transform::from_translation(cell.world_pos + Vec3::new(0.0, 2.5, 0.0)),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(
                            config.tile_size * 0.5,
                            2.5,
                            config.tile_size * 0.5,
                        ),
                    },
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 5.0, config.tile_size),
                ));
            }
            CanonicalPiece::SaddleCrossing => {
                let handle = registry.load_scene(&asset_server, "tile_corridor_open");
                commands.spawn((
                    Name::new("DungeonTile_SaddleCrossing"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 0.4, config.tile_size),
                ));

                for (sx, sz) in [(-1.0f32, -1.0f32), (1.0f32, 1.0f32)] {
                    let pillar_offset = rot * Vec3::new(
                        sx * (half_tile - 0.425),
                        2.5,
                        sz * (half_tile - 0.425),
                    );
                    commands.spawn((
                        Name::new("Collider_Dungeon_Pillar_Saddle"),
                        CavernObstacle,
                        ObstacleShape::Cuboid {
                            half_extents: Vec3::new(0.425, 2.5, 0.425),
                        },
                        Transform::from_translation(cell.world_pos + pillar_offset).with_rotation(rot),
                        RigidBody::Static,
                        Collider::cuboid(0.85, 5.0, 0.85),
                    ));
                }
            }
        }
    }

    spawn_dungeon_creatures_along_path(&mut commands, &asset_server, &registry, &dungeon);
    crate::dungeon::traps::spawn_dungeon_puzzles_and_traps(&mut commands, &mut meshes, &mut materials, &dungeon);

    info!(
        "Procedural Dungeon dynamically generated: {} dual cells, {} exits linked to Starting Grotto!",
        dungeon.dual_cells.len(),
        dungeon.exits.len()
    );
}

#[derive(Component)]
pub struct AbyssalVictoryExitTrigger;

pub fn update_abyssal_victory_trigger(
    player_query: Query<&Transform, With<crate::player::camera::Player>>,
    trigger_query: Query<&Transform, With<AbyssalVictoryExitTrigger>>,
    mut stats: ResMut<crate::ui::menu::GameRunStats>,
    mut nav: ResMut<crate::ui::menu::MenuNavigation>,
    mut next_state: ResMut<NextState<crate::ui::menu::GameState>>,
) {
    if stats.has_won {
        return;
    }
    let Ok(player_tf) = player_query.get_single() else {
        return;
    };
    for trigger_tf in trigger_query.iter() {
        let dist = (player_tf.translation - trigger_tf.translation).length();
        if dist < 6.5 {
            stats.has_won = true;
            nav.current_view = crate::ui::menu::MenuView::Victory;
            next_state.set(crate::ui::menu::GameState::MainMenu);
            break;
        }
    }
}

pub fn spawn_dungeon_creatures_along_path(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &SceneAssetRegistry,
    dungeon: &ProceduralDungeon,
) {
    use crate::dungeon::creatures::{spawn_dungeon_creature_instance, CreatureSpecies};

    let width = dungeon.config.grid_width;
    let depth = dungeon.config.grid_depth;
    let cz = depth / 2;
    let exit_z = depth - 2;

    let mut south_cells = Vec::new();
    let mut mid_cells = Vec::new();
    let mut north_cells = Vec::new();

    for gz in (cz + 2)..depth {
        for gx in 2..(width - 2) {
            let primary = dungeon.get_primary(gx, gz);
            if primary.is_walkable() {
                let world_pos = dungeon.grid_to_world(gx, gz);
                if gz <= cz + 3 {
                    south_cells.push(world_pos);
                } else if gz <= 18 {
                    mid_cells.push(world_pos);
                } else {
                    north_cells.push(world_pos);
                }
            }
        }
    }

    if !south_cells.is_empty() {
        for i in 0..2.min(south_cells.len()) {
            let pos = south_cells[i * (south_cells.len() / 2)];
            let waypoints = vec![pos + Vec3::new(1.5, 0.0, 0.0), pos + Vec3::new(-1.5, 0.0, 0.0)];
            spawn_dungeon_creature_instance(
                commands,
                asset_server,
                registry,
                CreatureSpecies::Sproutling,
                pos + Vec3::new(0.0, 0.2, 0.0),
                waypoints,
                (exit_z as f32 - (cz + 2) as f32) * dungeon.config.tile_size,
            );
        }
    }

    if !mid_cells.is_empty() {
        let step = mid_cells.len() / 4;
        for i in 0..4.min(mid_cells.len()) {
            let pos = mid_cells[i * step];
            let species = if i % 2 == 0 {
                CreatureSpecies::Sproutling
            } else {
                CreatureSpecies::Emberling
            };
            let waypoints = vec![
                pos + Vec3::new(2.0, 0.0, 1.0),
                pos + Vec3::new(-2.0, 0.0, -1.0),
            ];
            spawn_dungeon_creature_instance(
                commands,
                asset_server,
                registry,
                species,
                pos + Vec3::new(0.0, 0.2, 0.0),
                waypoints,
                (exit_z as f32 - 17.0) * dungeon.config.tile_size,
            );
        }
    }

    if !north_cells.is_empty() {
        let count = 8.min(north_cells.len());
        let step = north_cells.len() / count.max(1);
        for i in 0..count {
            let pos = north_cells[i * step];
            let species = match i % 3 {
                0 => CreatureSpecies::Glaceling,
                1 => CreatureSpecies::Emberling,
                _ => CreatureSpecies::Sproutling,
            };
            let waypoints = vec![
                pos + Vec3::new(1.8, 0.0, 1.8),
                pos + Vec3::new(-1.8, 0.0, -1.8),
            ];
            spawn_dungeon_creature_instance(
                commands,
                asset_server,
                registry,
                species,
                pos + Vec3::new(0.0, 0.2, 0.0),
                waypoints,
                (exit_z as f32 - 21.0) * dungeon.config.tile_size,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_grotto_always_anchored_at_center() {
        let config = DungeonConfig::default();
        let dungeon = DungeonGenerator::generate(config.clone());
        let cx = config.grid_width / 2;
        let cz = config.grid_depth / 2;

        assert_eq!(dungeon.get_primary(cx, cz), PrimaryTile::StartingGrotto);
        assert_eq!(dungeon.get_primary(cx - 1, cz), PrimaryTile::StartingGrotto);
        assert_eq!(dungeon.get_primary(cx + 1, cz), PrimaryTile::StartingGrotto);
        assert_eq!(dungeon.get_primary(cx, cz - 1), PrimaryTile::StartingGrotto);
        assert_eq!(dungeon.get_primary(cx, cz + 1), PrimaryTile::StartingGrotto);
    }

    #[test]
    fn test_canonical_pieces_mapping_all_16_combinations() {

        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(false, false, false, false).0,
            CanonicalPiece::Empty
        );

        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(true, false, false, false).0,
            CanonicalPiece::OuterCorner
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(false, true, false, false).0,
            CanonicalPiece::OuterCorner
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(false, false, true, false).0,
            CanonicalPiece::OuterCorner
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(false, false, false, true).0,
            CanonicalPiece::OuterCorner
        );

        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(true, true, false, false).0,
            CanonicalPiece::StraightEdge
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(false, true, true, false).0,
            CanonicalPiece::StraightEdge
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(false, false, true, true).0,
            CanonicalPiece::StraightEdge
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(true, false, false, true).0,
            CanonicalPiece::StraightEdge
        );

        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(true, false, true, false).0,
            CanonicalPiece::SaddleCrossing
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(false, true, false, true).0,
            CanonicalPiece::SaddleCrossing
        );

        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(false, true, true, true).0,
            CanonicalPiece::InnerCorner
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(true, false, true, true).0,
            CanonicalPiece::InnerCorner
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(true, true, false, true).0,
            CanonicalPiece::InnerCorner
        );
        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(true, true, true, false).0,
            CanonicalPiece::InnerCorner
        );

        assert_eq!(
            DungeonGenerator::resolve_canonical_piece(true, true, true, true).0,
            CanonicalPiece::SolidPillar
        );
    }

    #[test]
    fn test_dungeon_connectivity_and_reachability_gate() {
        let config = DungeonConfig::default();
        let dungeon = DungeonGenerator::generate(config);
        assert!(
            DungeonGenerator::verify_reachability(&dungeon),
            "All generated corridors and rooms MUST be reachable from Starting Grotto!"
        );
    }

    #[test]
    fn test_multi_seed_reachability_guarantee() {
        for seed in 1..=100 {
            let config = DungeonConfig {
                seed,
                ..default()
            };
            let dungeon = DungeonGenerator::generate(config);
            assert!(
                DungeonGenerator::verify_reachability(&dungeon),
                "Dungeon with seed {} failed reachability: unreachable rooms or corridors detected!",
                seed
            );
        }
    }

    #[test]
    fn test_no_blocking_colliders_in_entrance_corridor() {
        let config = DungeonConfig::default();
        let dungeon = DungeonGenerator::generate(config.clone());
        let cx = config.grid_width / 2;

        for gz in 14..=19 {
            let tile = dungeon.get_primary(cx, gz);
            assert!(
                tile.is_walkable(),
                "Entrance walkway at (cx={}, gz={}) must be walkable! Found: {:?}",
                cx,
                gz,
                tile
            );
        }
    }

    #[test]
    fn test_all_macro_room_types_generated() {
        let config = DungeonConfig::default();
        let dungeon = DungeonGenerator::generate(config);

        let mut has_crystal_geode = false;
        let mut has_alchemical_shrine = false;
        let mut has_ancient_bridge = false;
        let mut has_spore_sanctuary = false;
        let mut has_abyssal_exit = false;

        for &tile in &dungeon.primary_grid {
            match tile {
                PrimaryTile::RoomCenter(RoomType::CrystalGeode) => has_crystal_geode = true,
                PrimaryTile::RoomCenter(RoomType::AlchemicalShrine) => has_alchemical_shrine = true,
                PrimaryTile::RoomCenter(RoomType::AncientBridge) => has_ancient_bridge = true,
                PrimaryTile::RoomCenter(RoomType::SporeSanctuary) => has_spore_sanctuary = true,
                PrimaryTile::RoomCenter(RoomType::AbyssalExit) => has_abyssal_exit = true,
                _ => {}
            }
        }

        assert!(has_crystal_geode, "CrystalGeode chamber must be generated!");
        assert!(has_alchemical_shrine, "AlchemicalShrine chamber must be generated!");
        assert!(has_ancient_bridge, "AncientBridge chamber must be generated!");
        assert!(has_spore_sanctuary, "SporeSanctuary chamber must be generated!");
        assert!(has_abyssal_exit, "AbyssalExit chamber must be generated!");
    }

    #[test]
    fn test_grand_junction_crossroads_branches() {
        let config = DungeonConfig::default();
        let dungeon = DungeonGenerator::generate(config.clone());
        let cx = config.grid_width / 2;
        let junction_z = 19;

        assert!(dungeon.get_primary(cx, junction_z).is_walkable());
        assert!(dungeon.get_primary(cx, junction_z + 1).is_walkable());
        assert!(dungeon.get_primary(cx + 1, junction_z).is_walkable());
        assert!(dungeon.get_primary(cx - 1, junction_z).is_walkable());
    }

    #[test]
    fn test_deterministic_seed_generation() {
        let config1 = DungeonConfig {
            seed: 12345,
            ..default()
        };
        let config2 = DungeonConfig {
            seed: 12345,
            ..default()
        };
        let config3 = DungeonConfig {
            seed: 99999,
            ..default()
        };

        let d1 = DungeonGenerator::generate(config1);
        let d2 = DungeonGenerator::generate(config2);
        let d3 = DungeonGenerator::generate(config3);

        assert_eq!(d1.primary_grid, d2.primary_grid);
        assert_ne!(d1.primary_grid, d3.primary_grid);
    }

    #[test]
    fn test_modular_kit_canonical_pieces_generated() {
        let config = DungeonConfig::default();
        let dungeon = DungeonGenerator::generate(config);
        assert!(!dungeon.dual_cells.is_empty());
        let has_outer_corner = dungeon.dual_cells.iter().any(|c| c.piece == CanonicalPiece::OuterCorner);
        let has_straight = dungeon.dual_cells.iter().any(|c| c.piece == CanonicalPiece::StraightEdge);
        let has_empty = dungeon.dual_cells.iter().any(|c| c.piece == CanonicalPiece::Empty);
        let has_solid = dungeon.dual_cells.iter().any(|c| c.piece == CanonicalPiece::SolidPillar);
        assert!(has_outer_corner);
        assert!(has_straight);
        assert!(has_empty);
        assert!(has_solid);
    }
}
