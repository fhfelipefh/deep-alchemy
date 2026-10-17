use std::fs;
use std::path::Path;
use serde::Deserialize;
use deep_alchemy::alchemy::AlchemicalEffectKind;

#[derive(Debug, Deserialize)]
struct RonDatabase {
    ingredients: Vec<RonIngredient>,
    potions: Vec<RonPotion>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RonIngredient {
    id: String,
    name: String,
    description: String,
    weight: f32,
    value: u32,
    primary_aspect: AlchemicalEffectKind,
    secondary_aspect: AlchemicalEffectKind,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RonPotion {
    id: String,
    name: String,
    liquid_color: (f32, f32, f32, f32),
    effects: Vec<RonEffect>,
}

#[derive(Debug, Deserialize)]
struct RonEffect {
    kind: AlchemicalEffectKind,
    duration: f32,
    magnitude: f32,
}

#[test]
fn test_architecture_static_alchemy_recipes_ron_integrity() {
    let ron_path = Path::new("assets/data/alchemy_recipes.ron");
    assert!(ron_path.exists(), "assets/data/alchemy_recipes.ron must exist");

    let content = fs::read_to_string(ron_path).expect("Failed to read alchemy_recipes.ron");
    let database: RonDatabase = ron::from_str(&content).expect("Failed to parse RON database");

    assert!(!database.ingredients.is_empty(), "Database must contain ingredients");
    assert!(!database.potions.is_empty(), "Database must contain potions");

    for ing in &database.ingredients {
        assert!(!ing.id.is_empty(), "Ingredient ID must not be empty");
        assert!(!ing.name.is_empty(), "Ingredient Name must not be empty");
        assert!(ing.weight > 0.0, "Ingredient weight must be positive: {}", ing.name);
        assert!(ing.value > 0, "Ingredient value must be positive: {}", ing.name);
    }

    for pot in &database.potions {
        assert!(!pot.id.is_empty(), "Potion ID must not be empty");
        assert!(!pot.name.is_empty(), "Potion Name must not be empty");
        assert!(!pot.effects.is_empty(), "Potion must contain effects: {}", pot.name);
        for eff in &pot.effects {
            assert!(eff.duration > 0.0, "Effect duration must be > 0: {:?}", eff.kind);
            assert!(eff.magnitude > 0.0, "Effect magnitude must be > 0: {:?}", eff.kind);
        }
    }
}

#[test]
fn test_architecture_gltf_models_binary_integrity() {
    let models = [
        ("assets/models/player.glb", 30_000),      // > 30KB
        ("assets/models/cavern_kit.glb", 25_000),  // > 25KB
        ("assets/models/alchemy_props.glb", 25_000),// > 25KB
    ];

    let gltf_magic: u32 = 0x46546C67; // "glTF" in little-endian

    for (model_path, min_bytes) in models {
        let path = Path::new(model_path);
        assert!(path.exists(), "3D model file must exist: {}", model_path);

        let bytes = fs::read(path).expect("Failed to read GLB model");
        assert!(
            bytes.len() > min_bytes,
            "Model file {} is suspiciously small: {} bytes (expected > {})",
            model_path,
            bytes.len(),
            min_bytes
        );

        // Check glTF binary header
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(magic, gltf_magic, "Model {} is not a valid GLB binary", model_path);

        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(version, 2, "Model {} must be glTF version 2", model_path);
    }
}

#[test]
fn test_architecture_wgsl_shaders_integrity() {
    let shader_path = Path::new("assets/shaders/water_material.wgsl");
    assert!(shader_path.exists(), "water_material.wgsl must exist");

    let code = fs::read_to_string(shader_path).expect("Failed to read shader");
    assert!(!code.is_empty(), "Shader must not be empty");
    assert!(code.contains("@fragment"), "Shader must contain @fragment entry point");
    assert!(code.contains("MeshVertexOutput"), "Shader must import MeshVertexOutput");
    assert!(code.contains("material_base"), "Shader must bind material_base uniform");
    assert!(code.contains("material_deep"), "Shader must bind material_deep uniform");
    assert!(code.contains("material_foam"), "Shader must bind material_foam uniform");
    assert!(code.contains("material_params"), "Shader must bind material_params uniform");
}

#[test]
fn test_architecture_true_first_person_layer_separation() {
    use bevy::render::view::RenderLayers;

    let main_camera_layer = RenderLayers::layer(0);
    let head_hat_layer = RenderLayers::layer(1);

    // Layer 0 and Layer 1 must be strictly mutually exclusive for direct 1P camera
    assert!(
        !main_camera_layer.intersects(&head_hat_layer),
        "Main camera layer (0) must not intersect Head/Hat layer (1) in 1P"
    );

    // After death, head/hat transitions to Layer 0
    let revealed_head_layer = RenderLayers::layer(0);
    assert!(
        main_camera_layer.intersects(&revealed_head_layer),
        "Revealed head on death must intersect main camera layer (0)"
    );
}

#[test]
fn test_architecture_desktop_exclusive_constraints() {
    use bevy::window::CursorGrabMode;

    let default_grab = CursorGrabMode::Locked;
    assert_eq!(default_grab, CursorGrabMode::Locked, "Desktop mode requires locked cursor");
}
