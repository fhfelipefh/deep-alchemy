use crate::dungeon::{
    asset_loader::SceneAssetRegistry,
    modular_kit::{CavernObstacle, ObstacleShape},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

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

        for dz in -grotto_half_size..=grotto_half_size {
            for dx in -grotto_half_size..=grotto_half_size {
                let gx = (cx as isize + dx) as usize;
                let gz = (cz as isize + dz) as usize;
                if gx < width && gz < depth {
                    grid[gz * width + gx] = PrimaryTile::StartingGrotto;
                }
            }
        }

        let mut exits = Vec::new();
        let starting_exits = [
            IVec2::new(cx as i32, (cz - grotto_half_size as usize - 1) as i32),
            IVec2::new(cx as i32, (cz + grotto_half_size as usize + 1) as i32),
            IVec2::new((cx - grotto_half_size as usize - 1) as i32, cz as i32),
            IVec2::new((cx + grotto_half_size as usize + 1) as i32, cz as i32),
        ];

        for &exit_pos in &starting_exits {
            if exit_pos.x >= 1
                && exit_pos.x < (width as i32 - 1)
                && exit_pos.y >= 1
                && exit_pos.y < (depth as i32 - 1)
            {
                exits.push(exit_pos);
                grid[exit_pos.y as usize * width + exit_pos.x as usize] = PrimaryTile::Corridor;
            }
        }

        for &start in &exits {
            let mut curr = start;
            let mut dir = match (curr.x - cx as i32, curr.y - cz as i32) {
                (dx, 0) if dx > 0 => IVec2::new(1, 0),
                (dx, 0) if dx < 0 => IVec2::new(-1, 0),
                (0, dy) if dy > 0 => IVec2::new(0, 1),
                _ => IVec2::new(0, -1),
            };

            let steps = config.min_corridor_steps / exits.len();
            for _ in 0..steps {

                if next_rand() < 0.28 {
                    dir = if dir.x != 0 {
                        if next_rand() < 0.5 {
                            IVec2::new(0, 1)
                        } else {
                            IVec2::new(0, -1)
                        }
                    } else {
                        if next_rand() < 0.5 {
                            IVec2::new(1, 0)
                        } else {
                            IVec2::new(-1, 0)
                        }
                    };
                }

                let next_pos = curr + dir;

                if next_pos.x < 1
                    || next_pos.x >= (width as i32 - 1)
                    || next_pos.y < 1
                    || next_pos.y >= (depth as i32 - 1)
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

            Self::carve_macro_chamber(
                &mut grid,
                width,
                depth,
                curr.x as usize,
                curr.y as usize,
                &mut next_rand,
            );
        }

        Self::apply_macro_pattern_matching(&mut grid, width, depth, &mut next_rand);

        let dual_cells = Self::compute_dual_grid(&grid, width, depth, config.tile_size);

        ProceduralDungeon {
            config,
            primary_grid: grid,
            dual_cells,
            starting_anchor_world: Vec3::ZERO,
            exits,
        }
    }

    fn carve_macro_chamber<F: FnMut() -> f32>(
        grid: &mut [PrimaryTile],
        width: usize,
        depth: usize,
        cx: usize,
        cz: usize,
        rng: &mut F,
    ) {
        let room_type = match (rng() * 4.0) as usize {
            0 => RoomType::AlchemicalShrine,
            1 => RoomType::CrystalGeode,
            2 => RoomType::AncientBridge,
            _ => RoomType::SporeSanctuary,
        };

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
        for gz in 1..(depth - 2) {
            for gx in 1..(width - 2) {
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
                    PrimaryTile::RoomCenter(rt) | PrimaryTile::RoomPerimeter(rt) => Some(*rt),
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

            3 => (CanonicalPiece::StraightEdge, 0.0),
            6 => (CanonicalPiece::StraightEdge, 270.0),
            12 => (CanonicalPiece::StraightEdge, 180.0),
            9 => (CanonicalPiece::StraightEdge, 90.0),

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

pub fn spawn_procedural_dungeon(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<SceneAssetRegistry>,
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
            continue;
        }

        let is_exit_corridor =
            cell.world_pos.x.abs() < 1.0 && cell.world_pos.z >= 7.0 && cell.world_pos.z <= 30.0;
        if is_exit_corridor {
            let handle = registry.load_scene(&asset_server, "tile_corridor_straight");
            commands.spawn((
                Name::new("DungeonTile_ExitCorridor"),
                SceneRoot(handle),
                Transform::from_translation(cell.world_pos),
                RigidBody::Static,
                Collider::cuboid(config.tile_size, 0.4, config.tile_size),
            ));
            let wall_offset_left = Vec3::new(-half_tile + 0.35, 1.2, 0.0);
            commands.spawn((
                Name::new("Collider_Dungeon_Wall_Straight_L"),
                CavernObstacle,
                ObstacleShape::Cuboid {
                    half_extents: Vec3::new(0.375, 1.2, config.tile_size * 0.5),
                },
                Transform::from_translation(cell.world_pos + wall_offset_left),
                RigidBody::Static,
                Collider::cuboid(0.75, 2.4, config.tile_size),
            ));
            let wall_offset_right = Vec3::new(half_tile - 0.35, 1.2, 0.0);
            commands.spawn((
                Name::new("Collider_Dungeon_Wall_Straight_R"),
                CavernObstacle,
                ObstacleShape::Cuboid {
                    half_extents: Vec3::new(0.375, 1.2, config.tile_size * 0.5),
                },
                Transform::from_translation(cell.world_pos + wall_offset_right),
                RigidBody::Static,
                Collider::cuboid(0.75, 2.4, config.tile_size),
            ));
            continue;
        }

        let is_exit_corridor_terminus =
            cell.world_pos.x.abs() < 1.0 && (cell.world_pos.z - 32.0).abs() < 1.0;
        if is_exit_corridor_terminus {
            let floor_handle = registry.load_scene(&asset_server, "tile_corridor_straight");
            commands.spawn((
                Name::new("DungeonTile_CorridorTerminus_Floor"),
                SceneRoot(floor_handle),
                Transform::from_translation(cell.world_pos),
                RigidBody::Static,
                Collider::cuboid(config.tile_size, 0.4, config.tile_size),
            ));
            let wall_offset_north = Vec3::new(0.0, 1.2, half_tile - 0.35);
            commands.spawn((
                Name::new("Collider_Dungeon_Wall_Terminus_North"),
                CavernObstacle,
                ObstacleShape::Cuboid {
                    half_extents: Vec3::new(config.tile_size * 0.5, 1.2, 0.375),
                },
                Transform::from_translation(cell.world_pos + wall_offset_north),
                RigidBody::Static,
                Collider::cuboid(config.tile_size, 2.4, 0.75),
            ));
            let wall_offset_west = Vec3::new(-half_tile + 0.35, 1.2, 0.0);
            commands.spawn((
                Name::new("Collider_Dungeon_Wall_Terminus_West"),
                CavernObstacle,
                ObstacleShape::Cuboid {
                    half_extents: Vec3::new(0.375, 1.2, config.tile_size * 0.5),
                },
                Transform::from_translation(cell.world_pos + wall_offset_west),
                RigidBody::Static,
                Collider::cuboid(0.75, 2.4, config.tile_size),
            ));
            let crystal_handle = registry.load_scene(&asset_server, "macro_crystal_cluster");
            commands.spawn((
                Name::new("MacroRoom_CrystalGeode_Terminus"),
                SceneRoot(crystal_handle),
                Transform::from_translation(cell.world_pos + Vec3::new(0.0, 0.0, 1.2)),
                RigidBody::Static,
                Collider::cuboid(1.4, 2.4, 1.2),
                PointLight {
                    color: Color::srgb(0.95, 0.72, 0.35),
                    intensity: 14000.0,
                    range: 20.0,
                    shadows_enabled: false,
                    ..default()
                },
            ));
            continue;
        }

        let is_exit_corridor_flank =
            (cell.world_pos.x - -4.0).abs() < 1.0 && cell.world_pos.z >= 7.0 && cell.world_pos.z <= 30.0;
        if is_exit_corridor_flank {
            let handle = registry.load_scene(&asset_server, "tile_solid_pillar");
            commands.spawn((
                Name::new("DungeonTile_SolidPillar_Flank"),
                SceneRoot(handle),
                Transform::from_translation(cell.world_pos),
                RigidBody::Static,
                Collider::cuboid(config.tile_size, 4.0, config.tile_size),
            ));
            continue;
        }

        if cell.world_pos.x.abs() < 10.0 && cell.world_pos.z > -16.0 && cell.world_pos.z < 6.0 {
            continue;
        }

        let rot = Quat::from_rotation_y(cell.rotation_y_deg.to_radians());

        match cell.piece {
            CanonicalPiece::Empty => {
                let handle = registry.load_scene(&asset_server, "tile_corridor_straight");
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
                            let crystal_handle =
                                registry.load_scene(&asset_server, "macro_crystal_cluster");
                            commands.spawn((
                                Name::new("MacroRoom_CrystalGeode"),
                                SceneRoot(crystal_handle),
                                Transform::from_translation(
                                    cell.world_pos + Vec3::new(0.0, 0.5, 0.0),
                                ),
                                PointLight {
                                    color: Color::srgb(0.17, 0.91, 0.85),
                                    intensity: 22000.0,
                                    range: 16.0,
                                    shadows_enabled: false,
                                    ..default()
                                },
                            ));
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
                        _ => {}
                    }
                }
            }
            CanonicalPiece::StraightEdge => {
                let handle = registry.load_scene(&asset_server, "tile_corridor_straight");
                let is_ns = (cell.rotation_y_deg - 90.0).abs() < 1.0 || (cell.rotation_y_deg - 270.0).abs() < 1.0;
                let mesh_rot = if is_ns {
                    Quat::IDENTITY
                } else {
                    Quat::from_rotation_y(90.0_f32.to_radians())
                };
                commands.spawn((
                    Name::new("DungeonTile_StraightEdge"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos).with_rotation(mesh_rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 0.4, config.tile_size),
                ));

                let wall_offset = rot * Vec3::new(0.0, 1.2, -half_tile + 0.35);
                commands.spawn((
                    Name::new("Collider_Dungeon_Wall_Straight"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(config.tile_size * 0.5, 1.2, 0.375),
                    },
                    Transform::from_translation(cell.world_pos + wall_offset).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 2.4, 0.75),
                ));
            }
            CanonicalPiece::OuterCorner | CanonicalPiece::InnerCorner => {
                let handle = registry.load_scene(&asset_server, "tile_corridor_corner");
                commands.spawn((
                    Name::new("DungeonTile_Corner"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 0.4, config.tile_size),
                ));

                let pillar_offset = rot * Vec3::new(-half_tile + 0.6, 1.4, -half_tile + 0.6);
                commands.spawn((
                    Name::new("Collider_Dungeon_Pillar_Corner"),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(0.7, 1.4, 0.7),
                    },
                    Transform::from_translation(cell.world_pos + pillar_offset),
                    RigidBody::Static,
                    Collider::cuboid(1.4, 2.8, 1.4),
                ));
            }
            CanonicalPiece::SolidPillar => {
                let handle = registry.load_scene(&asset_server, "tile_solid_pillar");
                commands.spawn((
                    Name::new("DungeonTile_SolidPillar"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos),
                    CavernObstacle,
                    ObstacleShape::Cuboid {
                        half_extents: Vec3::new(
                            config.tile_size * 0.5,
                            1.6,
                            config.tile_size * 0.5,
                        ),
                    },
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 3.2, config.tile_size),
                ));
            }
            CanonicalPiece::SaddleCrossing => {
                let handle = registry.load_scene(&asset_server, "tile_corridor_straight");
                commands.spawn((
                    Name::new("DungeonTile_SaddleCrossing"),
                    SceneRoot(handle),
                    Transform::from_translation(cell.world_pos),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 0.4, config.tile_size),
                ));
            }
        }
    }

    info!(
        "Procedural Dungeon dynamically generated: {} dual cells, {} exits linked to Starting Grotto!",
        dungeon.dual_cells.len(),
        dungeon.exits.len()
    );
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
    fn test_dungeon_grotto_exit_dual_cells_walkable() {
        let config = DungeonConfig::default();
        let dungeon = DungeonGenerator::generate(config);

        let exit_cell = dungeon
            .dual_cells
            .iter()
            .find(|c| c.world_pos.x.abs() < 1.0 && (c.world_pos.z - 4.0).abs() < 1.0);
        assert!(
            exit_cell.is_some(),
            "Dungeon must generate a dual-grid cell right at the grotto exit (z = 4.0, x = 0.0)"
        );
        assert_eq!(exit_cell.unwrap().piece, CanonicalPiece::Empty, "Grotto exit cell must be CanonicalPiece::Empty (paved walkable floor) to allow player passage!");
    }
}
