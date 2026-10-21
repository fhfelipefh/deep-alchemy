pub mod asset_loader;
pub mod flora;
pub mod generator;
pub mod modular_kit;
pub mod telemetry;
pub mod test_room;

pub use asset_loader::{spawn_scene_asset, spawn_scene_asset_with_collider, SceneAssetRegistry};
pub use generator::{
    CanonicalPiece, DungeonConfig, DungeonGenerator, PrimaryTile, ProceduralDungeon, RoomType,
};
pub use modular_kit::DungeonPlugin;
pub use telemetry::dump_camera_frustum_telemetry;
pub use test_room::SecretTestRoomPlugin;
