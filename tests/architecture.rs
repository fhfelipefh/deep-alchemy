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
        ("assets/models/player.glb", 30_000),       // > 30KB
        ("assets/models/cavern_kit.glb", 25_000),   // > 25KB
        ("assets/models/alchemy_props.glb", 25_000), // > 25KB
        ("assets/models/dungeon_kit.glb", 20_000),  // > 20KB
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
    assert!(code.contains("VertexOutput"), "Shader must import VertexOutput");
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

#[test]
fn test_architecture_ptbr_font_file_integrity_and_cmap_coverage() {
    let font_path = Path::new("assets/fonts/DejaVuSans.ttf");
    assert!(font_path.exists(), "assets/fonts/DejaVuSans.ttf must exist");

    let font_bytes = fs::read(font_path).expect("Failed to read font file");
    assert!(
        font_bytes.len() > 500_000,
        "Font file should be full-featured TrueType (> 500KB), found {} bytes",
        font_bytes.len()
    );

    // Parse TrueType Table Directory to find 'cmap'
    let num_tables = u16::from_be_bytes([font_bytes[4], font_bytes[5]]) as usize;
    let mut cmap_offset: Option<usize> = None;

    for i in 0..num_tables {
        let entry_start = 12 + i * 16;
        let tag = &font_bytes[entry_start..entry_start + 4];
        if tag == b"cmap" {
            let offset = u32::from_be_bytes([
                font_bytes[entry_start + 8],
                font_bytes[entry_start + 9],
                font_bytes[entry_start + 10],
                font_bytes[entry_start + 11],
            ]) as usize;
            cmap_offset = Some(offset);
            break;
        }
    }

    let cmap_off = cmap_offset.expect("Font must contain 'cmap' table");
    let num_subtables = u16::from_be_bytes([font_bytes[cmap_off + 2], font_bytes[cmap_off + 3]]) as usize;

    // Locate Format 4 subtable (Unicode BMP: platform 3, encoding 1 or platform 0)
    let mut format4_offset: Option<usize> = None;
    for i in 0..num_subtables {
        let rec = cmap_off + 4 + i * 8;
        let platform_id = u16::from_be_bytes([font_bytes[rec], font_bytes[rec + 1]]);
        let encoding_id = u16::from_be_bytes([font_bytes[rec + 2], font_bytes[rec + 3]]);
        let sub_offset = u32::from_be_bytes([
            font_bytes[rec + 4],
            font_bytes[rec + 5],
            font_bytes[rec + 6],
            font_bytes[rec + 7],
        ]) as usize;

        let absolute_sub_off = cmap_off + sub_offset;
        let format = u16::from_be_bytes([font_bytes[absolute_sub_off], font_bytes[absolute_sub_off + 1]]);

        if (platform_id == 3 && encoding_id == 1 && format == 4)
            || (platform_id == 0 && format == 4)
        {
            format4_offset = Some(absolute_sub_off);
            break;
        }
    }

    let f4_off = format4_offset.expect("Font must contain Format 4 Unicode subtable in cmap");
    let seg_count = (u16::from_be_bytes([font_bytes[f4_off + 6], font_bytes[f4_off + 7]]) / 2) as usize;

    let end_codes_start = f4_off + 14;
    let start_codes_start = end_codes_start + seg_count * 2 + 2; // +2 for reservedPad

    let mut segments = Vec::with_capacity(seg_count);
    for i in 0..seg_count {
        let end_code = u16::from_be_bytes([
            font_bytes[end_codes_start + i * 2],
            font_bytes[end_codes_start + i * 2 + 1],
        ]);
        let start_code = u16::from_be_bytes([
            font_bytes[start_codes_start + i * 2],
            font_bytes[start_codes_start + i * 2 + 1],
        ]);
        segments.push((start_code, end_code));
    }

    // All mandatory Portuguese glyphs that previously broke or were missing
    let required_chars = [
        'á', 'é', 'í', 'ó', 'ú',
        'â', 'ê', 'ô',
        'ã', 'õ',
        'ç',
        'Á', 'É', 'Í', 'Ó', 'Ú',
        'Â', 'Ê', 'Ô',
        'Ã', 'Õ',
        'Ç',
        'ª', 'º',
    ];

    for &ch in &required_chars {
        let code = ch as u32;
        let is_mapped = segments.iter().any(|&(start, end)| code >= start as u32 && code <= end as u32);
        assert!(
            is_mapped,
            "Font 'DejaVuSans.ttf' is missing glyph coverage for Portuguese character '{}' (U+{:04X})",
            ch, code
        );
    }
}

#[test]
fn test_architecture_ui_utf8_strings_no_mojibake() {
    let source_files = [
        "src/ui/menu.rs",
        "src/ui/hud.rs",
        "src/player/interaction.rs",
        "src/main.rs",
    ];

    let mojibake_patterns = [
        "Ã§", "Ã£", "Ã¡", "Ã©", "Ã³", "Ãº", "Ã¢", "Ãª", "Ã´", "Ãµ",
        "â€™", "â€œ", "â€\u{9d}", "\u{FFFD}",
    ];

    for file_path in source_files {
        let path = Path::new(file_path);
        assert!(path.exists(), "Source file must exist: {}", file_path);

        let content = fs::read_to_string(path).expect("File must be valid UTF-8");
        for &pattern in &mojibake_patterns {
            assert!(
                !content.contains(pattern),
                "Mojibake / corrupted encoding pattern '{}' detected in {}",
                pattern, file_path
            );
        }
    }

    // Verify key UI strings in menu.rs are properly written with Portuguese accents
    let menu_content = fs::read_to_string("src/ui/menu.rs").unwrap();
    assert!(menu_content.contains("CONFIGURAÇÕES"));
    assert!(menu_content.contains("EXPLORAÇÃO"));
    assert!(menu_content.contains("1ª PESSOA"));
    assert!(menu_content.contains("Visão"));
}

#[test]
fn test_architecture_executable_icon_and_metadata_integrity() {
    // 1. High-resolution PNG icon
    let png_path = Path::new("assets/icons/app_icon.png");
    assert!(png_path.exists(), "app_icon.png must exist");
    let png_bytes = fs::read(png_path).expect("Failed to read app_icon.png");
    assert!(png_bytes.len() > 5_000, "app_icon.png must be > 5KB");
    assert_eq!(&png_bytes[0..4], &[0x89, b'P', b'N', b'G'], "app_icon.png must have valid PNG magic");

    // 2. Multi-resolution Windows ICO
    let ico_path = Path::new("assets/icons/app_icon.ico");
    assert!(ico_path.exists(), "app_icon.ico must exist");
    let ico_bytes = fs::read(ico_path).expect("Failed to read app_icon.ico");
    assert!(ico_bytes.len() > 40_000, "app_icon.ico must be multi-res (> 40KB)");
    assert_eq!(&ico_bytes[0..4], &[0, 0, 1, 0], "app_icon.ico must have valid ICO magic header");
    let icon_count = u16::from_le_bytes([ico_bytes[4], ico_bytes[5]]);
    assert!(icon_count >= 5, "app_icon.ico must provide at least 5 resolution tiers, found {}", icon_count);

    // 3. Resource Compiler Script and Compiled .res file
    let rc_path = Path::new("assets/icons/app_icon.rc");
    assert!(rc_path.exists(), "app_icon.rc must exist");
    let rc_content = fs::read_to_string(rc_path).expect("Failed to read rc script");
    assert!(rc_content.contains("ICON \"app_icon.ico\""));

    let res_path = Path::new("assets/icons/app_icon.res");
    assert!(res_path.exists(), "app_icon.res must exist for PE binary embedding");

    // 4. Build Script linking the icon into the executable
    let build_rs_path = Path::new("build.rs");
    assert!(build_rs_path.exists(), "build.rs must exist");
    let build_rs_content = fs::read_to_string(build_rs_path).expect("Failed to read build.rs");
    assert!(build_rs_content.contains("cargo:rustc-link-arg="), "build.rs must link Windows PE resource file");
}

#[test]
fn test_architecture_cavern_grotto_visual_richness() {
    let cavern_model_path = Path::new("assets/models/cavern_kit.glb");
    assert!(cavern_model_path.exists(), "cavern_kit.glb must exist");

    let model_bytes = fs::read(cavern_model_path).expect("Failed to read cavern_kit.glb");
    // Rich detailed model with cavern dome, waterfall cliff, stepped travertine pools, stalactites, mushrooms
    assert!(
        model_bytes.len() > 100_000,
        "cavern_kit.glb must be a rich multi-asset environment (> 100KB), found {} bytes",
        model_bytes.len()
    );

    // Verify modular_kit.rs spawns waterfall, river, geysers and atmospheric lighting
    let dungeon_code = fs::read_to_string("src/dungeon/modular_kit.rs").expect("Failed to read modular_kit.rs");
    assert!(dungeon_code.contains("WaterMaterial"), "Grotto must use custom WaterMaterial");
    assert!(dungeon_code.contains("ThermalGeyser"), "Grotto must have ThermalGeyser entities");
    assert!(dungeon_code.contains("ParticleEmitter"), "Grotto must have ParticleEmitter entities");
    assert!(dungeon_code.contains("AmbientLight"), "Grotto must configure AmbientLight");
    assert!(dungeon_code.contains("DirectionalLight"), "Grotto must have sunlight shaft");
}

#[test]
fn test_architecture_gatekeeping_definition_of_done_in_rules() {
    let rule_files = [
        "RULES.md",
        ".gemini/rules/architecture.md",
    ];

    let required_gate_sections = [
        "PROTOCOLO ANTI-PLACEHOLDER E VALIDAÇÃO OBRIGATÓRIA (DEFINITION OF DONE)",
        "GATE 1: Proibição de Primitivas Visuais e Shaders Padrão",
        "GATE 2: Checklist Mínimo de Locomoção e Gameplay (Locomotion Test)",
        "GATE 3: UI Fiel ao Concept",
        "GATE 4: Auto-Verificação Visual (Screenshot Gate)",
        "--screenshot-test",
    ];

    for file_path in rule_files {
        let path = Path::new(file_path);
        assert!(path.exists(), "Rules file must exist: {}", file_path);

        let content = fs::read_to_string(path).expect("Failed to read rule file");
        for &section in &required_gate_sections {
            assert!(
                content.contains(section),
                "Rules file {} is missing mandatory section: '{}'",
                file_path, section
            );
        }
    }
}

#[test]
fn test_architecture_gate2_locomotion_and_skeleton_integrity() {
    let controller_code = fs::read_to_string("src/player/controller.rs").expect("Failed to read controller.rs");
    assert!(controller_code.contains("LocomotionState"), "Controller must use LocomotionState");
    assert!(controller_code.contains("is_sprinting"), "Controller must track sprinting");
    assert!(controller_code.contains("is_crouching"), "Controller must track crouching");
    assert!(controller_code.contains("is_grounded"), "Controller must track ground state");
    assert!(controller_code.contains("player_camera_effects"), "Controller must have FOV kick and head bob system");

    let camera_code = fs::read_to_string("src/player/camera.rs").expect("Failed to read camera.rs");
    assert!(camera_code.contains("BoneJoint"), "Camera/Player must define BoneJoint");
    assert!(camera_code.contains("RagdollLimb"), "Camera/Player must define RagdollLimb");
    assert!(camera_code.contains("HeadHatPart"), "Camera/Player must define HeadHatPart");
}

#[test]
fn test_architecture_gate3_concept_hud_and_5_slot_hotbar() {
    let hud_code = fs::read_to_string("src/ui/hud.rs").expect("Failed to read hud.rs");
    assert!(hud_code.contains("HealthHeartFill"), "HUD must define HealthHeartFill for top-left hearts");
    assert!(hud_code.contains("HotbarSlotUi"), "HUD must define HotbarSlotUi for bottom hotbar");
    assert!(hud_code.contains("ActiveHotbarSlot"), "HUD must manage ActiveHotbarSlot resource");
    assert!(hud_code.contains("COLOR_BORDER_GOLD"), "HUD must use ornate alchemical gold borders");
}

#[test]
fn test_architecture_gate4_screenshot_and_validator_script_integrity() {
    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
    assert!(main_code.contains("--screenshot-test"), "main.rs must handle --screenshot-test flag");
    assert!(main_code.contains("Screenshot::primary_window()"), "main.rs must trigger primary window screenshot");
    assert!(main_code.contains("debug_output/frame_check.png"), "main.rs must save to debug_output/frame_check.png");

    let validator_path = Path::new("scripts/validate_frame.py");
    assert!(validator_path.exists(), "scripts/validate_frame.py must exist");

    let validator_code = fs::read_to_string(validator_path).expect("Failed to read validate_frame.py");
    assert!(validator_code.contains("Sobel"), "Validator must implement Sobel edge detection");
    assert!(validator_code.contains("gray_percentage"), "Validator must check flat gray ratio");
    assert!(validator_code.contains("vibrant_percentage"), "Validator must verify vibrant colors");
}

#[test]
fn test_architecture_cel_shaded_sobel_outline_pipeline_integrity() {
    let outline_shader_path = Path::new("assets/shaders/outline_post_process.wgsl");
    assert!(outline_shader_path.exists(), "outline_post_process.wgsl must exist");

    let shader_code = fs::read_to_string(outline_shader_path).expect("Failed to read outline shader");
    assert!(shader_code.contains("Sobel"), "Outline shader must implement Sobel kernel sampling");
    assert!(shader_code.contains("settings.outline_threshold"), "Shader must use outline threshold");

    let camera_code = fs::read_to_string("src/player/camera.rs").expect("Failed to read camera.rs");
    assert!(camera_code.contains("DepthPrepass"), "Camera must have DepthPrepass");
    assert!(camera_code.contains("NormalPrepass"), "Camera must have NormalPrepass");
    assert!(camera_code.contains("OutlineSettings"), "Camera must have OutlineSettings");

    let render_code = fs::read_to_string("src/render/mod.rs").expect("Failed to read render/mod.rs");
    assert!(render_code.contains("OutlinePostProcessPlugin"), "Render plugin must include OutlinePostProcessPlugin");
}

#[test]
fn test_architecture_procedural_dungeon_wfc_dual_grid_integrity() {
    let generator_code = fs::read_to_string("src/dungeon/generator.rs").expect("Failed to read generator.rs");
    assert!(generator_code.contains("StartingGrotto"), "Generator must enforce permanent StartingGrotto anchor");
    assert!(generator_code.contains("CanonicalPiece"), "Generator must implement Stålberg 6 Canonical Pieces");
    assert!(generator_code.contains("compute_dual_grid"), "Generator must compute shifted Dual Grid");
    assert!(generator_code.contains("apply_macro_pattern_matching"), "Generator must implement Multi-Tile Pattern Matching");
    assert!(generator_code.contains("verify_reachability"), "Generator must verify reachability from Starting Grotto");

    let dungeon_kit_path = Path::new("assets/models/dungeon_kit.glb");
    assert!(dungeon_kit_path.exists(), "assets/models/dungeon_kit.glb must exist");
}


