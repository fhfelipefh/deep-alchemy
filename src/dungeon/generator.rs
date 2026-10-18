use avian3d::prelude::*;
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

/// Logical tile state in the primary dungeon grid.
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

/// Types of macro rooms discovered via pattern matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomType {
    AlchemicalShrine,
    CrystalGeode,
    AncientBridge,
    SporeSanctuary,
}

/// The 6 Canonical Dual Grid Pieces (Stålberg / Dual Grid autotiling architecture).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalPiece {
    Empty,          // 0 solid corners: open walkable space
    OuterCorner,    // 1 solid corner: convex corner
    StraightEdge,   // 2 adjacent solid corners: straight wall edge
    SaddleCrossing, // 2 diagonal solid corners: crossing saddle point
    InnerCorner,    // 3 solid corners: concave corner
    SolidPillar,    // 4 solid corners: solid interior rock/pillar
}

/// A resolved visual dual-grid cell with its canonical piece, orientation, and macro assignment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DualCell {
    pub piece: CanonicalPiece,
    pub rotation_y_deg: f32,
    pub macro_room: Option<RoomType>,
    pub world_pos: Vec3,
}

/// Configuration settings for procedural generation.
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

/// Complete representation of the generated procedural dungeon.
pub struct ProceduralDungeon {
    pub config: DungeonConfig,
    pub primary_grid: Vec<PrimaryTile>,
    pub dual_cells: Vec<DualCell>,
    pub starting_anchor_world: Vec3,
    pub exits: Vec<IVec2>,
}

impl ProceduralDungeon {
    /// Get the tile at logical grid coordinates (gx, gz).
    pub fn get_primary(&self, gx: usize, gz: usize) -> PrimaryTile {
        if gx >= self.config.grid_width || gz >= self.config.grid_depth {
            PrimaryTile::Rock
        } else {
            self.primary_grid[gz * self.config.grid_width + gx]
        }
    }

    /// Convert logical grid coordinate to 3D world space (centered on Starting Grotto at (0, 0, 0)).
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

/// Procedural Dungeon Generator implementing WFC, Dual Grid (Stålberg), and Macro Pattern Matching.
pub struct DungeonGenerator;

impl DungeonGenerator {
    /// Generate a connected procedural dungeon with a guaranteed starting waterfall grotto anchor.
    pub fn generate(config: DungeonConfig) -> ProceduralDungeon {
        let mut rng_state = config.seed;
        let mut next_rand = || -> f32 {
            // Fast LCG deterministic PRNG
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng_state >> 32) as u32 as f32) / (u32::MAX as f32)
        };

        let width = config.grid_width;
        let depth = config.grid_depth;
        let mut grid = vec![PrimaryTile::Rock; width * depth];

        // 1. INVARIANT: The Starting Waterfall Grotto ALWAYS exists at the center (0, 0)
        let cx = width / 2;
        let cz = depth / 2;
        let grotto_half_size = 2; // 5x5 zone centered on (cx, cz)

        for dz in -grotto_half_size..=grotto_half_size {
            for dx in -grotto_half_size..=grotto_half_size {
                let gx = (cx as isize + dx) as usize;
                let gz = (cz as isize + dz) as usize;
                if gx < width && gz < depth {
                    grid[gz * width + gx] = PrimaryTile::StartingGrotto;
                }
            }
        }

        // 2. Generate dynamic branching pathways extending from the grotto exits
        let mut exits = Vec::new();
        let starting_exits = [
            IVec2::new(cx as i32, (cz - grotto_half_size as usize - 1) as i32), // North exit (behind grotto)
            IVec2::new(cx as i32, (cz + grotto_half_size as usize + 1) as i32), // South exit (entry archway)
            IVec2::new((cx - grotto_half_size as usize - 1) as i32, cz as i32), // West exit (flank tunnel)
            IVec2::new((cx + grotto_half_size as usize + 1) as i32, cz as i32), // East exit (flank tunnel)
        ];

        for &exit_pos in &starting_exits {
            if exit_pos.x >= 1 && exit_pos.x < (width as i32 - 1) && exit_pos.y >= 1 && exit_pos.y < (depth as i32 - 1) {
                exits.push(exit_pos);
                grid[exit_pos.y as usize * width + exit_pos.x as usize] = PrimaryTile::Corridor;
            }
        }

        // 3. Constrained Random Walk & Socket Expansion from each exit
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
                // Change direction occasionally with branch probability
                if next_rand() < 0.28 {
                    dir = if dir.x != 0 {
                        if next_rand() < 0.5 { IVec2::new(0, 1) } else { IVec2::new(0, -1) }
                    } else {
                        if next_rand() < 0.5 { IVec2::new(1, 0) } else { IVec2::new(-1, 0) }
                    };
                }

                let next_pos = curr + dir;
                // Boundary check (keep at least 1 tile padding border)
                if next_pos.x < 1 || next_pos.x >= (width as i32 - 1) || next_pos.y < 1 || next_pos.y >= (depth as i32 - 1) {
                    dir = -dir;
                    continue;
                }

                curr = next_pos;
                let idx = curr.y as usize * width + curr.x as usize;
                if grid[idx] == PrimaryTile::Rock {
                    grid[idx] = PrimaryTile::Corridor;
                }
            }

            // Spawn a macro chamber at the terminal end of each main corridor
            Self::carve_macro_chamber(&mut grid, width, depth, curr.x as usize, curr.y as usize, &mut next_rand);
        }

        // 4. Pattern Matching Multi-Tile Pass: detect 2x2 or 3x3 open zones and promote to Macro Rooms
        Self::apply_macro_pattern_matching(&mut grid, width, depth, &mut next_rand);

        // 5. Dual Grid Transformation: compute the 6 Canonical Pieces
        let dual_cells = Self::compute_dual_grid(&grid, width, depth, config.tile_size);

        ProceduralDungeon {
            config,
            primary_grid: grid,
            dual_cells,
            starting_anchor_world: Vec3::ZERO,
            exits,
        }
    }

    /// Carve a 3x3 open chamber at (cx, cz).
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

    /// Pattern Matching Multi-Tile: Identify 2x2 open clusters and tag them as Macro Rooms
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

                // If all 4 are generic corridors, promote into a thematic macro chamber
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

    /// Compute the shifted Dual Grid (half-tile offset) and map to 6 Canonical Pieces.
    pub fn compute_dual_grid(
        grid: &[PrimaryTile],
        width: usize,
        depth: usize,
        tile_size: f32,
    ) -> Vec<DualCell> {
        let mut dual_cells = Vec::new();
        let center_offset_x = (width as f32 * 0.5) * tile_size;
        let center_offset_z = (depth as f32 * 0.5) * tile_size;

        // Dual grid has (width - 1) x (depth - 1) cells
        for gz in 0..(depth - 1) {
            for gx in 0..(width - 1) {
                // The 4 corners of the dual quad are the centers of primary tiles
                // c00 = Top-Left, c10 = Top-Right, c11 = Bottom-Right, c01 = Bottom-Left
                let c00 = grid[gz * width + gx].is_solid();
                let c10 = grid[gz * width + (gx + 1)].is_solid();
                let c11 = grid[(gz + 1) * width + (gx + 1)].is_solid();
                let c01 = grid[(gz + 1) * width + gx].is_solid();

                let (piece, rot_deg) = Self::resolve_canonical_piece(c00, c10, c11, c01);

                // Position is shifted by +0.5 tile relative to primary grid
                let wx = ((gx as f32 + 0.5) * tile_size) - center_offset_x;
                let wz = ((gz as f32 + 0.5) * tile_size) - center_offset_z;

                // Check if any corner belongs to a macro room
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

    /// Resolve 4 corner states into one of the 6 Canonical Pieces with correct rotation.
    pub fn resolve_canonical_piece(c00: bool, c10: bool, c11: bool, c01: bool) -> (CanonicalPiece, f32) {
        let mask = (if c00 { 1 } else { 0 })
            | (if c10 { 2 } else { 0 })
            | (if c11 { 4 } else { 0 })
            | (if c01 { 8 } else { 0 });

        match mask {
            // 0 solid corners: Empty (walkable open cavern)
            0 => (CanonicalPiece::Empty, 0.0),

            // 1 solid corner: Outer Convex Corner (smooth rounded corner)
            1 => (CanonicalPiece::OuterCorner, 0.0),    // Top-Left
            2 => (CanonicalPiece::OuterCorner, 90.0),   // Top-Right
            4 => (CanonicalPiece::OuterCorner, 180.0),  // Bottom-Right
            8 => (CanonicalPiece::OuterCorner, 270.0),  // Bottom-Left

            // 2 adjacent solid corners: Straight Wall Edge
            3 => (CanonicalPiece::StraightEdge, 0.0),   // Top edge (c00, c10)
            6 => (CanonicalPiece::StraightEdge, 90.0),  // Right edge (c10, c11)
            12 => (CanonicalPiece::StraightEdge, 180.0), // Bottom edge (c11, c01)
            9 => (CanonicalPiece::StraightEdge, 270.0), // Left edge (c01, c00)

            // 2 diagonal solid corners: Saddle Point (Crossing)
            5 => (CanonicalPiece::SaddleCrossing, 0.0),  // (c00, c11)
            10 => (CanonicalPiece::SaddleCrossing, 90.0), // (c10, c01)

            // 3 solid corners: Inner Concave Corner
            14 => (CanonicalPiece::InnerCorner, 0.0),   // c00 is open
            13 => (CanonicalPiece::InnerCorner, 90.0),  // c10 is open
            11 => (CanonicalPiece::InnerCorner, 180.0), // c11 is open
            7 => (CanonicalPiece::InnerCorner, 270.0),  // c01 is open

            // 4 solid corners: Solid Rock / Interior Pillar
            15 => (CanonicalPiece::SolidPillar, 0.0),

            _ => (CanonicalPiece::Empty, 0.0),
        }
    }

    /// Check connectivity: verify that all walkable tiles can reach the starting grotto via BFS.
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

        // Check if all walkable tiles are visited
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

/// Spawns the physical 3D procedural dungeon meshes and Avian3D colliders in Bevy.
pub fn spawn_procedural_dungeon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let config = DungeonConfig::default();
    let dungeon = DungeonGenerator::generate(config.clone());

    // Rich Cel-Shaded Materials for procedural dungeon elements
    let mat_floor_path = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.36, 0.25), // Rich warm earthen flagstone
        perceptual_roughness: 0.82,
        ..default()
    });

    let mat_floor_border = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.28, 0.32), // Dark river slate border
        perceptual_roughness: 0.85,
        ..default()
    });

    let mat_wall_moss = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.52, 0.30), // Subterranean velvet moss footing
        perceptual_roughness: 0.75,
        ..default()
    });

    let mat_wall_stone = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.42, 0.50), // Chiseled cave slate with high specular relief
        perceptual_roughness: 0.78,
        ..default()
    });

    let mat_fungi_glow = materials.add(StandardMaterial {
        base_color: Color::LinearRgba(LinearRgba::new(0.25, 0.95, 0.92, 1.0)), // #2be8d8
        emissive: LinearRgba::new(3.8, 7.8, 7.5, 1.0),
        perceptual_roughness: 0.20,
        ..default()
    });

    let mat_geode_glow = materials.add(StandardMaterial {
        base_color: Color::LinearRgba(LinearRgba::new(0.20, 0.95, 1.0, 1.0)),
        emissive: LinearRgba::new(4.5, 8.5, 9.2, 1.0),
        perceptual_roughness: 0.15,
        ..default()
    });

    let mat_shrine_gold = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.75, 0.35),
        emissive: LinearRgba::new(4.2, 3.2, 0.8, 1.0),
        perceptual_roughness: 0.30,
        ..default()
    });

    let half_tile = config.tile_size * 0.5;

    // Instantiate physical dual-grid pieces with Avian3D physics colliders & lighting
    for cell in &dungeon.dual_cells {
        // Skip spawning dungeon kit pieces inside the handcrafted Starting Grotto cavern amphitheater
        // (Cavern grotto encompasses X in [-28, 28], Z in [-28, 8])
        if cell.world_pos.x.abs() < 28.0 && cell.world_pos.z > -28.0 && cell.world_pos.z < 8.0 {
            continue;
        }

        let rot = Quat::from_rotation_y(cell.rotation_y_deg.to_radians());

        match cell.piece {
            CanonicalPiece::Empty => {
                // Dual-tone Walkable Paved Floor (dark slate border + central warm flagstone)
                commands.spawn((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(config.tile_size, config.tile_size))),
                    MeshMaterial3d(mat_floor_border.clone()),
                    Transform::from_translation(cell.world_pos).with_rotation(rot),
                    RigidBody::Static,
                    Collider::half_space(Vec3::Y),
                ));
                commands.spawn((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(config.tile_size * 0.78, config.tile_size * 0.78))),
                    MeshMaterial3d(mat_floor_path.clone()),
                    Transform::from_translation(cell.world_pos + Vec3::new(0.0, 0.008, 0.0)).with_rotation(rot),
                ));

                // If this is a special macro chamber, spawn thematic centerpieces
                if let Some(macro_type) = cell.macro_room {
                    match macro_type {
                        RoomType::CrystalGeode => {
                            // Towering glowing crystal formation with cyan bioluminescent light
                            commands.spawn((
                                Mesh3d(meshes.add(Cone::new(0.48, 3.4).mesh())),
                                MeshMaterial3d(mat_geode_glow.clone()),
                                Transform::from_translation(cell.world_pos + Vec3::new(0.0, 1.7, 0.0)),
                                PointLight {
                                    color: Color::srgb(0.17, 0.91, 0.85), // #2be8d8
                                    intensity: 22000.0,
                                    range: 16.0,
                                    shadows_enabled: false,
                                    ..default()
                                },
                            ));
                        }
                        RoomType::AlchemicalShrine => {
                            // Ancient rune pedestal with warm brazier light (faceted rock pedestal)
                            commands.spawn((
                                Mesh3d(meshes.add(Cuboid::new(1.1, 1.2, 1.1).mesh())),
                                MeshMaterial3d(mat_shrine_gold.clone()),
                                Transform::from_translation(cell.world_pos + Vec3::new(0.0, 0.6, 0.0)),
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
                // Dual-tone Paved Corridor Floor
                commands.spawn((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(config.tile_size, config.tile_size))),
                    MeshMaterial3d(mat_floor_path.clone()),
                    Transform::from_translation(cell.world_pos),
                    RigidBody::Static,
                    Collider::half_space(Vec3::Y),
                ));

                // 1. Lower Mossy Rock Base (protruding footing with physics collider)
                let base_offset = rot * Vec3::new(0.0, 0.6, -half_tile + 0.45);
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(config.tile_size, 1.2, 0.9).mesh())),
                    MeshMaterial3d(mat_wall_moss.clone()),
                    Transform::from_translation(cell.world_pos + base_offset).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 1.2, 0.9),
                ));

                // 2. Upper Chiseled Rock Wall (faceted stone with physics collider)
                let wall_offset = rot * Vec3::new(0.0, 2.4, -half_tile + 0.35);
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(config.tile_size, 2.4, 0.75).mesh())),
                    MeshMaterial3d(mat_wall_stone.clone()),
                    Transform::from_translation(cell.world_pos + wall_offset).with_rotation(rot),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 2.4, 0.75),
                ));

                // 3. Vaulted Overhead Cavern Rib (creates underground ceiling arch feel)
                let rib_offset = rot * Vec3::new(0.0, 3.8, -half_tile + 0.75);
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(config.tile_size, 0.5, 0.9).mesh())),
                    MeshMaterial3d(mat_wall_stone.clone()),
                    Transform::from_translation(cell.world_pos + rib_offset).with_rotation(rot),
                ));

                // 4. Bioluminescent Wall Lichen / Mushroom Cluster & PointLight (#2be8d8)
                let light_offset = rot * Vec3::new(0.0, 1.6, -half_tile + 0.95);
                commands.spawn((
                    Mesh3d(meshes.add(Cone::new(0.20, 0.28).mesh())),
                    MeshMaterial3d(mat_fungi_glow.clone()),
                    Transform::from_translation(cell.world_pos + light_offset).with_rotation(rot),
                    PointLight {
                        color: Color::srgb(0.17, 0.91, 0.85), // #2be8d8 soft cyan/turquoise
                        intensity: 14000.0,
                        range: 12.0,
                        shadows_enabled: false,
                        ..default()
                    },
                ));
            }
            CanonicalPiece::OuterCorner | CanonicalPiece::InnerCorner => {
                // Paved Turn Floor
                commands.spawn((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(config.tile_size, config.tile_size))),
                    MeshMaterial3d(mat_floor_path.clone()),
                    Transform::from_translation(cell.world_pos),
                    RigidBody::Static,
                    Collider::half_space(Vec3::Y),
                ));

                // 1. Lower Mossy Rock Pillar Base (faceted rock block)
                let base_offset = rot * Vec3::new(-half_tile + 0.6, 0.6, -half_tile + 0.6);
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.8, 1.2, 1.8).mesh())),
                    MeshMaterial3d(mat_wall_moss.clone()),
                    Transform::from_translation(cell.world_pos + base_offset).with_rotation(rot * Quat::from_rotation_y(0.38)),
                    RigidBody::Static,
                    Collider::cuboid(1.8, 1.2, 1.8),
                ));

                // 2. Upper Chiseled Stone Pillar Body (faceted rock block)
                let pillar_offset = rot * Vec3::new(-half_tile + 0.6, 2.6, -half_tile + 0.6);
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.4, 2.8, 1.4).mesh())),
                    MeshMaterial3d(mat_wall_stone.clone()),
                    Transform::from_translation(cell.world_pos + pillar_offset).with_rotation(rot * Quat::from_rotation_y(-0.25)),
                    RigidBody::Static,
                    Collider::cuboid(1.4, 2.8, 1.4),
                ));

                // 3. Embedded Glowing Quartz Node & PointLight on Corner
                let crystal_offset = rot * Vec3::new(-half_tile + 1.1, 1.3, -half_tile + 1.1);
                commands.spawn((
                    Mesh3d(meshes.add(Cone::new(0.22, 0.75).mesh())),
                    MeshMaterial3d(mat_geode_glow.clone()),
                    Transform::from_translation(cell.world_pos + crystal_offset),
                    PointLight {
                        color: Color::srgb(0.17, 0.91, 0.85), // #2be8d8
                        intensity: 15000.0,
                        range: 12.0,
                        shadows_enabled: false,
                        ..default()
                    },
                ));
            }
            CanonicalPiece::SolidPillar => {
                // Multi-tiered Solid Cavern Rock Formations
                let base_pos = cell.world_pos + Vec3::new(0.0, 0.8, 0.0);
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(config.tile_size, 1.6, config.tile_size).mesh())),
                    MeshMaterial3d(mat_wall_moss.clone()),
                    Transform::from_translation(base_pos),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size, 1.6, config.tile_size),
                ));

                let upper_pos = cell.world_pos + Vec3::new(0.0, 3.2, 0.0);
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(config.tile_size * 0.92, 3.2, config.tile_size * 0.92).mesh())),
                    MeshMaterial3d(mat_wall_stone.clone()),
                    Transform::from_translation(upper_pos),
                    RigidBody::Static,
                    Collider::cuboid(config.tile_size * 0.92, 3.2, config.tile_size * 0.92),
                ));
            }
            CanonicalPiece::SaddleCrossing => {
                // Crossing floor
                commands.spawn((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(config.tile_size, config.tile_size))),
                    MeshMaterial3d(mat_floor_path.clone()),
                    Transform::from_translation(cell.world_pos),
                    RigidBody::Static,
                    Collider::half_space(Vec3::Y),
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
        // 0 solid -> Empty
        assert_eq!(DungeonGenerator::resolve_canonical_piece(false, false, false, false).0, CanonicalPiece::Empty);

        // 1 solid -> OuterCorner
        assert_eq!(DungeonGenerator::resolve_canonical_piece(true, false, false, false).0, CanonicalPiece::OuterCorner);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(false, true, false, false).0, CanonicalPiece::OuterCorner);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(false, false, true, false).0, CanonicalPiece::OuterCorner);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(false, false, false, true).0, CanonicalPiece::OuterCorner);

        // 2 adjacent -> StraightEdge
        assert_eq!(DungeonGenerator::resolve_canonical_piece(true, true, false, false).0, CanonicalPiece::StraightEdge);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(false, true, true, false).0, CanonicalPiece::StraightEdge);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(false, false, true, true).0, CanonicalPiece::StraightEdge);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(true, false, false, true).0, CanonicalPiece::StraightEdge);

        // 2 diagonal -> SaddleCrossing
        assert_eq!(DungeonGenerator::resolve_canonical_piece(true, false, true, false).0, CanonicalPiece::SaddleCrossing);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(false, true, false, true).0, CanonicalPiece::SaddleCrossing);

        // 3 solid -> InnerCorner
        assert_eq!(DungeonGenerator::resolve_canonical_piece(false, true, true, true).0, CanonicalPiece::InnerCorner);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(true, false, true, true).0, CanonicalPiece::InnerCorner);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(true, true, false, true).0, CanonicalPiece::InnerCorner);
        assert_eq!(DungeonGenerator::resolve_canonical_piece(true, true, true, false).0, CanonicalPiece::InnerCorner);

        // 4 solid -> SolidPillar
        assert_eq!(DungeonGenerator::resolve_canonical_piece(true, true, true, true).0, CanonicalPiece::SolidPillar);
    }

    #[test]
    fn test_dungeon_connectivity_and_reachability_gate() {
        let config = DungeonConfig::default();
        let dungeon = DungeonGenerator::generate(config);
        assert!(DungeonGenerator::verify_reachability(&dungeon), "All generated corridors and rooms MUST be reachable from Starting Grotto!");
    }

    #[test]
    fn test_deterministic_seed_generation() {
        let config1 = DungeonConfig { seed: 12345, ..default() };
        let config2 = DungeonConfig { seed: 12345, ..default() };
        let config3 = DungeonConfig { seed: 99999, ..default() };

        let d1 = DungeonGenerator::generate(config1);
        let d2 = DungeonGenerator::generate(config2);
        let d3 = DungeonGenerator::generate(config3);

        assert_eq!(d1.primary_grid, d2.primary_grid);
        assert_ne!(d1.primary_grid, d3.primary_grid);
    }
}
