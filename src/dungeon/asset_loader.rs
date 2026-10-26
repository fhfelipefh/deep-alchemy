use avian3d::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Debug, Clone)]
pub struct SceneAssetRegistry {
    pub assets: HashMap<String, String>,
}

impl Default for SceneAssetRegistry {
    fn default() -> Self {
        let mut assets = HashMap::new();
        assets.insert("env_cave_wall".to_string(), "models/env_cave_wall.glb#Scene0".to_string());
        assets.insert("prop_mushroom_cyan".to_string(), "models/prop_mushroom_cyan.glb#Scene0".to_string());
        assets.insert("prop_wooden_bench".to_string(), "models/prop_wooden_bench.glb#Scene0".to_string());
        assets.insert("prop_faceted_rock".to_string(), "models/prop_faceted_rock.glb#Scene0".to_string());
        assets.insert("tile_corridor_straight".to_string(), "models/tile_corridor_straight.glb#Scene0".to_string());
        assets.insert("tile_corridor_corner".to_string(), "models/tile_corridor_corner.glb#Scene0".to_string());
        assets.insert("tile_outer_corner".to_string(), "models/tile_outer_corner.glb#Scene0".to_string());
        assets.insert("tile_straight_edge".to_string(), "models/tile_straight_edge.glb#Scene0".to_string());
        assets.insert("tile_corridor_open".to_string(), "models/tile_corridor_open.glb#Scene0".to_string());
        assets.insert("tile_solid_pillar".to_string(), "models/tile_solid_pillar.glb#Scene0".to_string());
        assets.insert("macro_alchemical_shrine".to_string(), "models/macro_alchemical_shrine.glb#Scene0".to_string());
        assets.insert("macro_crystal_cluster".to_string(), "models/macro_crystal_cluster.glb#Scene0".to_string());
        assets.insert("macro_crystal_cluster_b".to_string(), "models/macro_crystal_cluster_b.glb#Scene0".to_string());
        assets.insert("macro_crystal_cluster_c".to_string(), "models/macro_crystal_cluster_c.glb#Scene0".to_string());
        assets.insert("cavern_grotto".to_string(), "models/cavern_kit.glb#Scene0".to_string());
        assets.insert("alchemy_props".to_string(), "models/alchemy_props.glb#Scene0".to_string());
        assets.insert("faceted_rocks_kit".to_string(), "models/faceted_rocks_kit.glb#Scene0".to_string());
        assets.insert("dungeon_kit".to_string(), "models/dungeon_kit.glb#Scene0".to_string());
        assets.insert("secret_white_test_room".to_string(), "models/secret_white_test_room.glb#Scene0".to_string());
        assets.insert("tile_grotto_connector".to_string(), "models/tile_grotto_connector.glb#Scene0".to_string());
        assets.insert("prop_wall_sconce".to_string(), "models/prop_wall_sconce.glb#Scene0".to_string());
        assets.insert("creature_sproutling".to_string(), "models/creature_sproutling.glb#Scene0".to_string());
        assets.insert("creature_emberling".to_string(), "models/creature_emberling.glb#Scene0".to_string());
        assets.insert("creature_glaceling".to_string(), "models/creature_glaceling.glb#Scene0".to_string());
        assets.insert("prop_forest_tree".to_string(), "models/prop_forest_tree.glb#Scene0".to_string());
        Self { assets }
    }
}

impl SceneAssetRegistry {
    pub fn get_path(&self, key: &str) -> Option<&str> {
        self.assets.get(key).map(|s| s.as_str())
    }

    pub fn register(&mut self, key: impl Into<String>, path: impl Into<String>) {
        self.assets.insert(key.into(), path.into());
    }

    pub fn load_scene(&self, asset_server: &AssetServer, key: &str) -> Handle<Scene> {
        let path = self.get_path(key).unwrap_or("models/cavern_kit.glb#Scene0");
        asset_server.load(path)
    }
}

#[derive(Component, Debug, Clone)]
pub struct SceneAssetInstance {
    pub asset_key: String,
}

pub fn spawn_scene_asset(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &SceneAssetRegistry,
    key: &str,
    transform: Transform,
) -> Entity {
    let handle = registry.load_scene(asset_server, key);
    commands.spawn((
        Name::new(key.to_string()),
        SceneAssetInstance {
            asset_key: key.to_string(),
        },
        SceneRoot(handle),
        transform,
    )).id()
}

pub fn spawn_scene_asset_with_collider(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &SceneAssetRegistry,
    key: &str,
    transform: Transform,
    collider: Collider,
) -> Entity {
    let handle = registry.load_scene(asset_server, key);
    commands.spawn((
        Name::new(key.to_string()),
        SceneAssetInstance {
            asset_key: key.to_string(),
        },
        SceneRoot(handle),
        transform,
        RigidBody::Static,
        collider,
    )).id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_asset_registry_dictionary_mapping() {
        let registry = SceneAssetRegistry::default();
        assert!(registry.get_path("env_cave_wall").is_some());
        assert!(registry.get_path("prop_mushroom_cyan").is_some());
        assert!(registry.get_path("prop_wooden_bench").is_some());
        assert!(registry.get_path("prop_faceted_rock").is_some());
        assert!(registry.get_path("tile_corridor_straight").is_some());
        assert!(registry.get_path("tile_corridor_corner").is_some());
        assert!(registry.get_path("tile_outer_corner").is_some());
        assert!(registry.get_path("tile_straight_edge").is_some());
        assert!(registry.get_path("tile_corridor_open").is_some());
        assert!(registry.get_path("tile_solid_pillar").is_some());
        assert!(registry.get_path("macro_alchemical_shrine").is_some());
        assert!(registry.get_path("macro_crystal_cluster").is_some());
        assert!(registry.get_path("macro_crystal_cluster_b").is_some());
        assert!(registry.get_path("macro_crystal_cluster_c").is_some());
        assert!(registry.get_path("cavern_grotto").is_some());
        assert!(registry.get_path("secret_white_test_room").is_some());
        assert!(registry.get_path("prop_wall_sconce").is_some());
        assert!(registry.get_path("creature_sproutling").is_some());
        assert!(registry.get_path("creature_emberling").is_some());
        assert!(registry.get_path("creature_glaceling").is_some());
        assert!(registry.get_path("prop_forest_tree").is_some());
    }

    #[test]
    fn test_scene_asset_registry_custom_registration() {
        let mut registry = SceneAssetRegistry::default();
        registry.register("custom_prop", "models/custom.glb#Scene0");
        assert_eq!(registry.get_path("custom_prop"), Some("models/custom.glb#Scene0"));
    }
}
