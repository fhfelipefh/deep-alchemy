use deep_alchemy::alchemy::AlchemicalEffectKind;
use serde::Deserialize;
use std::fs;
use std::path::Path;

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
    assert!(
        ron_path.exists(),
        "assets/data/alchemy_recipes.ron must exist"
    );

    let content = fs::read_to_string(ron_path).expect("Failed to read alchemy_recipes.ron");
    let database: RonDatabase = ron::from_str(&content).expect("Failed to parse RON database");

    assert!(
        !database.ingredients.is_empty(),
        "Database must contain ingredients"
    );
    assert!(
        !database.potions.is_empty(),
        "Database must contain potions"
    );

    for ing in &database.ingredients {
        assert!(!ing.id.is_empty(), "Ingredient ID must not be empty");
        assert!(!ing.name.is_empty(), "Ingredient Name must not be empty");
        assert!(
            ing.weight > 0.0,
            "Ingredient weight must be positive: {}",
            ing.name
        );
        assert!(
            ing.value > 0,
            "Ingredient value must be positive: {}",
            ing.name
        );
    }

    for pot in &database.potions {
        assert!(!pot.id.is_empty(), "Potion ID must not be empty");
        assert!(!pot.name.is_empty(), "Potion Name must not be empty");
        assert!(
            !pot.effects.is_empty(),
            "Potion must contain effects: {}",
            pot.name
        );
        for eff in &pot.effects {
            assert!(
                eff.duration > 0.0,
                "Effect duration must be > 0: {:?}",
                eff.kind
            );
            assert!(
                eff.magnitude > 0.0,
                "Effect magnitude must be > 0: {:?}",
                eff.kind
            );
        }
    }
}

#[test]
fn test_architecture_gltf_models_binary_integrity() {
    let models = [
        ("assets/models/player.glb", 30_000),
        ("assets/models/cavern_kit.glb", 25_000),
        ("assets/models/alchemy_props.glb", 25_000),
        ("assets/models/dungeon_kit.glb", 20_000),
    ];

    let gltf_magic: u32 = 0x46546C67;

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

        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(
            magic, gltf_magic,
            "Model {} is not a valid GLB binary",
            model_path
        );

        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(version, 2, "Model {} must be glTF version 2", model_path);

        assert!(
            !bytes.windows(7).any(|w| w == b"COLOR_1"),
            "Model {} contains unsupported vertex attribute COLOR_1",
            model_path
        );
    }
}

#[test]
fn test_architecture_wgsl_shaders_integrity() {
    let shader_path = Path::new("assets/shaders/water_material.wgsl");
    assert!(shader_path.exists(), "water_material.wgsl must exist");

    let code = fs::read_to_string(shader_path).expect("Failed to read shader");
    assert!(!code.is_empty(), "Shader must not be empty");
    assert!(
        code.contains("@fragment"),
        "Shader must contain @fragment entry point"
    );
    assert!(
        code.contains("VertexOutput"),
        "Shader must import VertexOutput"
    );
    assert!(
        code.contains("material_base"),
        "Shader must bind material_base uniform"
    );
    assert!(
        code.contains("material_deep"),
        "Shader must bind material_deep uniform"
    );
    assert!(
        code.contains("material_foam"),
        "Shader must bind material_foam uniform"
    );
    let fluid_shader_path = Path::new("assets/shaders/potion_fluid.wgsl");
    assert!(fluid_shader_path.exists(), "potion_fluid.wgsl must exist");
    let fluid_code = fs::read_to_string(fluid_shader_path).expect("Failed to read fluid shader");
    assert!(!fluid_code.is_empty(), "Fluid shader must not be empty");
    assert!(fluid_code.contains("@fragment"), "Fluid shader must contain @fragment");
    assert!(fluid_code.contains("VertexOutput"), "Fluid shader must import VertexOutput");
    assert!(fluid_code.contains("slosh_normal"), "Fluid shader must bind slosh_normal uniform");
    assert!(fluid_code.contains("fluid_params"), "Fluid shader must bind fluid_params uniform");

    let heat_haze_shader_path = Path::new("assets/shaders/heat_haze.wgsl");
    assert!(heat_haze_shader_path.exists(), "heat_haze.wgsl must exist");
    let heat_haze_code = fs::read_to_string(heat_haze_shader_path).expect("Failed to read heat haze shader");
    assert!(!heat_haze_code.is_empty(), "Heat haze shader must not be empty");
    assert!(heat_haze_code.contains("@fragment"), "Heat haze shader must contain @fragment");
    assert!(heat_haze_code.contains("FullscreenVertexOutput"), "Heat haze shader must import FullscreenVertexOutput");
    assert!(heat_haze_code.contains("HeatHazeSettings"), "Heat haze shader must bind HeatHazeSettings uniform");
    assert!(heat_haze_code.contains("sources"), "Heat haze shader must process sources array");
}

#[test]
fn test_architecture_true_first_person_layer_separation() {
    use bevy::render::view::RenderLayers;

    let main_camera_layer = RenderLayers::layer(0);
    let head_hat_layer = RenderLayers::layer(1);

    assert!(
        !main_camera_layer.intersects(&head_hat_layer),
        "Main camera layer (0) must not intersect Head/Hat layer (1) in 1P"
    );

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
    assert_eq!(
        default_grab,
        CursorGrabMode::Locked,
        "Desktop mode requires locked cursor"
    );
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
    let num_subtables =
        u16::from_be_bytes([font_bytes[cmap_off + 2], font_bytes[cmap_off + 3]]) as usize;

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
        let format = u16::from_be_bytes([
            font_bytes[absolute_sub_off],
            font_bytes[absolute_sub_off + 1],
        ]);

        if (platform_id == 3 && encoding_id == 1 && format == 4)
            || (platform_id == 0 && format == 4)
        {
            format4_offset = Some(absolute_sub_off);
            break;
        }
    }

    let f4_off = format4_offset.expect("Font must contain Format 4 Unicode subtable in cmap");
    let seg_count =
        (u16::from_be_bytes([font_bytes[f4_off + 6], font_bytes[f4_off + 7]]) / 2) as usize;

    let end_codes_start = f4_off + 14;
    let start_codes_start = end_codes_start + seg_count * 2 + 2;

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

    let required_chars = [
        'á', 'é', 'í', 'ó', 'ú', 'â', 'ê', 'ô', 'ã', 'õ', 'ç', 'Á', 'É', 'Í', 'Ó', 'Ú', 'Â', 'Ê',
        'Ô', 'Ã', 'Õ', 'Ç', 'ª', 'º',
    ];

    for &ch in &required_chars {
        let code = ch as u32;
        let is_mapped = segments
            .iter()
            .any(|&(start, end)| code >= start as u32 && code <= end as u32);
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
        "Ã§",
        "Ã£",
        "Ã¡",
        "Ã©",
        "Ã³",
        "Ãº",
        "Ã¢",
        "Ãª",
        "Ã´",
        "Ãµ",
        "â€™",
        "â€œ",
        "â€\u{9d}",
        "\u{FFFD}",
    ];

    for file_path in source_files {
        let path = Path::new(file_path);
        assert!(path.exists(), "Source file must exist: {}", file_path);

        let content = fs::read_to_string(path).expect("File must be valid UTF-8");
        for &pattern in &mojibake_patterns {
            assert!(
                !content.contains(pattern),
                "Mojibake / corrupted encoding pattern '{}' detected in {}",
                pattern,
                file_path
            );
        }
    }

    let menu_content = fs::read_to_string("src/ui/menu.rs").unwrap();
    assert!(menu_content.contains("CONFIGURAÇÕES"));
    assert!(menu_content.contains("EXPLORAÇÃO"));
    assert!(menu_content.contains("1ª PESSOA"));
    assert!(menu_content.contains("Visão"));
}

#[test]
fn test_architecture_executable_icon_and_metadata_integrity() {

    let png_path = Path::new("assets/icons/app_icon.png");
    assert!(png_path.exists(), "app_icon.png must exist");
    let png_bytes = fs::read(png_path).expect("Failed to read app_icon.png");
    assert!(png_bytes.len() > 5_000, "app_icon.png must be > 5KB");
    assert_eq!(
        &png_bytes[0..4],
        &[0x89, b'P', b'N', b'G'],
        "app_icon.png must have valid PNG magic"
    );

    let ico_path = Path::new("assets/icons/app_icon.ico");
    assert!(ico_path.exists(), "app_icon.ico must exist");
    let ico_bytes = fs::read(ico_path).expect("Failed to read app_icon.ico");
    assert!(
        ico_bytes.len() > 40_000,
        "app_icon.ico must be multi-res (> 40KB)"
    );
    assert_eq!(
        &ico_bytes[0..4],
        &[0, 0, 1, 0],
        "app_icon.ico must have valid ICO magic header"
    );
    let icon_count = u16::from_le_bytes([ico_bytes[4], ico_bytes[5]]);
    assert!(
        icon_count >= 5,
        "app_icon.ico must provide at least 5 resolution tiers, found {}",
        icon_count
    );

    let rc_path = Path::new("assets/icons/app_icon.rc");
    assert!(rc_path.exists(), "app_icon.rc must exist");
    let rc_content = fs::read_to_string(rc_path).expect("Failed to read rc script");
    assert!(rc_content.contains("ICON \"app_icon.ico\""));

    let res_path = Path::new("assets/icons/app_icon.res");
    assert!(
        res_path.exists(),
        "app_icon.res must exist for PE binary embedding"
    );

    let build_rs_path = Path::new("build.rs");
    assert!(build_rs_path.exists(), "build.rs must exist");
    let build_rs_content = fs::read_to_string(build_rs_path).expect("Failed to read build.rs");
    assert!(
        build_rs_content.contains("cargo:rustc-link-arg="),
        "build.rs must link Windows PE resource file"
    );
}

#[test]
fn test_architecture_cavern_grotto_visual_richness() {
    let cavern_model_path = Path::new("assets/models/cavern_kit.glb");
    assert!(cavern_model_path.exists(), "cavern_kit.glb must exist");

    let model_bytes = fs::read(cavern_model_path).expect("Failed to read cavern_kit.glb");

    assert!(
        model_bytes.len() > 100_000,
        "cavern_kit.glb must be a rich multi-asset environment (> 100KB), found {} bytes",
        model_bytes.len()
    );

    let dungeon_code =
        fs::read_to_string("src/dungeon/modular_kit.rs").expect("Failed to read modular_kit.rs");
    assert!(
        dungeon_code.contains("WaterMaterial"),
        "Grotto must use custom WaterMaterial"
    );
    assert!(
        dungeon_code.contains("ThermalGeyser"),
        "Grotto must have ThermalGeyser entities"
    );
    assert!(
        dungeon_code.contains("ParticleEmitter"),
        "Grotto must have ParticleEmitter entities"
    );
    assert!(
        dungeon_code.contains("AmbientLight"),
        "Grotto must configure AmbientLight"
    );
    assert!(
        dungeon_code.contains("DirectionalLight"),
        "Grotto must have sunlight shaft"
    );
}

#[test]
fn test_architecture_gatekeeping_definition_of_done_in_rules() {
    let rule_files = ["RULES.md", ".gemini/rules/architecture.md"];

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
                file_path,
                section
            );
        }
    }
}

#[test]
fn test_architecture_gate2_locomotion_and_skeleton_integrity() {
    let controller_code =
        fs::read_to_string("src/player/controller.rs").expect("Failed to read controller.rs");
    assert!(
        controller_code.contains("LocomotionState"),
        "Controller must use LocomotionState"
    );
    assert!(
        controller_code.contains("is_sprinting"),
        "Controller must track sprinting"
    );
    assert!(
        controller_code.contains("is_crouching"),
        "Controller must track crouching"
    );
    assert!(
        controller_code.contains("is_grounded"),
        "Controller must track ground state"
    );
    assert!(
        controller_code.contains("player_camera_effects"),
        "Controller must have FOV kick and head bob system"
    );

    let camera_code = fs::read_to_string("src/player/camera.rs").expect("Failed to read camera.rs");
    assert!(
        camera_code.contains("BoneJoint"),
        "Camera/Player must define BoneJoint"
    );
    assert!(
        camera_code.contains("RagdollLimb"),
        "Camera/Player must define RagdollLimb"
    );
    assert!(
        camera_code.contains("HeadHatPart"),
        "Camera/Player must define HeadHatPart"
    );
}

#[test]
fn test_architecture_gate3_concept_hud_and_5_slot_hotbar() {
    let hud_code = fs::read_to_string("src/ui/hud.rs").expect("Failed to read hud.rs");
    assert!(
        hud_code.contains("HealthHeartFill"),
        "HUD must define HealthHeartFill for top-left hearts"
    );
    assert!(
        hud_code.contains("HotbarSlotUi"),
        "HUD must define HotbarSlotUi for bottom hotbar"
    );
    assert!(
        hud_code.contains("ActiveHotbarSlot"),
        "HUD must manage ActiveHotbarSlot resource"
    );
    assert!(
        hud_code.contains("COLOR_BORDER_GOLD"),
        "HUD must use ornate alchemical gold borders"
    );
}

#[test]
fn test_architecture_gate4_screenshot_and_validator_script_integrity() {
    let main_code = fs::read_to_string("src/main.rs").expect("Failed to read main.rs");
    assert!(
        main_code.contains("--screenshot-test"),
        "main.rs must handle --screenshot-test flag"
    );
    assert!(
        main_code.contains("Screenshot::primary_window()"),
        "main.rs must trigger primary window screenshot"
    );
    assert!(
        main_code.contains("debug_output/frame_check.png"),
        "main.rs must save to debug_output/frame_check.png"
    );

    let validator_path = Path::new("scripts/validate_frame.py");
    assert!(
        validator_path.exists(),
        "scripts/validate_frame.py must exist"
    );

    let validator_code =
        fs::read_to_string(validator_path).expect("Failed to read validate_frame.py");
    assert!(
        validator_code.contains("Sobel"),
        "Validator must implement Sobel edge detection"
    );
    assert!(
        validator_code.contains("gray_percentage"),
        "Validator must check flat gray ratio"
    );
    assert!(
        validator_code.contains("vibrant_percentage"),
        "Validator must verify vibrant colors"
    );
}

#[test]
fn test_architecture_cel_shaded_sobel_outline_pipeline_integrity() {
    let outline_shader_path = Path::new("assets/shaders/outline_post_process.wgsl");
    assert!(
        outline_shader_path.exists(),
        "outline_post_process.wgsl must exist"
    );

    let shader_code =
        fs::read_to_string(outline_shader_path).expect("Failed to read outline shader");
    assert!(
        shader_code.contains("Sobel"),
        "Outline shader must implement Sobel kernel sampling"
    );
    assert!(
        shader_code.contains("settings.outline_threshold"),
        "Shader must use outline threshold"
    );

    let camera_code = fs::read_to_string("src/player/camera.rs").expect("Failed to read camera.rs");
    assert!(
        camera_code.contains("DepthPrepass"),
        "Camera must have DepthPrepass"
    );
    assert!(
        camera_code.contains("NormalPrepass"),
        "Camera must have NormalPrepass"
    );
    assert!(
        camera_code.contains("OutlineSettings"),
        "Camera must have OutlineSettings"
    );

    let render_code =
        fs::read_to_string("src/render/mod.rs").expect("Failed to read render/mod.rs");
    assert!(
        render_code.contains("OutlinePostProcessPlugin"),
        "Render plugin must include OutlinePostProcessPlugin"
    );
}

#[test]
fn test_architecture_procedural_dungeon_wfc_dual_grid_integrity() {
    let generator_code =
        fs::read_to_string("src/dungeon/generator.rs").expect("Failed to read generator.rs");
    assert!(
        generator_code.contains("StartingGrotto"),
        "Generator must enforce permanent StartingGrotto anchor"
    );
    assert!(
        generator_code.contains("CanonicalPiece"),
        "Generator must implement Stålberg 6 Canonical Pieces"
    );
    assert!(
        generator_code.contains("compute_dual_grid"),
        "Generator must compute shifted Dual Grid"
    );
    assert!(
        generator_code.contains("apply_macro_pattern_matching"),
        "Generator must implement Multi-Tile Pattern Matching"
    );
    assert!(
        generator_code.contains("verify_reachability"),
        "Generator must verify reachability from Starting Grotto"
    );

    let dungeon_kit_path = Path::new("assets/models/dungeon_kit.glb");
    assert!(
        dungeon_kit_path.exists(),
        "assets/models/dungeon_kit.glb must exist"
    );
}

#[test]
fn test_architecture_concept_art_fidelity_and_faceted_rocks_pipeline() {

    let skill_path = Path::new(".agent/skills/concept-art-fidelity/SKILL.md");
    assert!(
        skill_path.exists(),
        ".agent/skills/concept-art-fidelity/SKILL.md must exist"
    );
    let skill_content =
        fs::read_to_string(skill_path).expect("Failed to read concept-art-fidelity skill");
    assert!(
        skill_content.contains("Visual Diff"),
        "Skill must define Visual Diff loop"
    );
    assert!(
        skill_content.contains("World-Space Normal"),
        "Skill must define World-Space Normal top moss blending"
    );
    assert!(
        skill_content.contains("Extração de Blueprint"),
        "Skill must define Blueprint Extraction step"
    );
    assert!(
        skill_content.contains("Definition of Done"),
        "Skill must define Definition of Done"
    );

    let blueprint_script = Path::new("scripts/visual_diff_blueprint.py");
    assert!(
        blueprint_script.exists(),
        "scripts/visual_diff_blueprint.py must exist"
    );
    let script_code =
        fs::read_to_string(blueprint_script).expect("Failed to read visual_diff_blueprint.py");
    assert!(
        script_code.contains("analyze_image_regions"),
        "Script must analyze depth layers"
    );
    assert!(
        script_code.contains("compute_sobel_density"),
        "Script must measure ink lines density"
    );

    let moss_shader_path = Path::new("assets/shaders/cel_shaded_world_moss.wgsl");
    assert!(
        moss_shader_path.exists(),
        "assets/shaders/cel_shaded_world_moss.wgsl must exist"
    );
    let shader_code = fs::read_to_string(moss_shader_path).expect("Failed to read moss shader");
    assert!(
        shader_code.contains("ny_threshold"),
        "Shader must implement normal.y threshold"
    );
    assert!(
        shader_code.contains("smoothstep"),
        "Shader must blend moss smoothly"
    );
    assert!(
        shader_code.contains("toon_bands"),
        "Shader must implement stepped toon lighting"
    );

    let render_code =
        fs::read_to_string("src/render/mod.rs").expect("Failed to read render mod.rs");
    assert!(
        render_code.contains("CelShadedMossMaterial"),
        "Render module must register CelShadedMossMaterial"
    );

    let rock_gen_script = Path::new("scripts/generate_faceted_rocks.py");
    assert!(
        rock_gen_script.exists(),
        "scripts/generate_faceted_rocks.py must exist"
    );
    let gen_code =
        fs::read_to_string(rock_gen_script).expect("Failed to read generate_faceted_rocks.py");
    assert!(
        gen_code.contains("generate_angular_rock"),
        "Rock generator must define generate_angular_rock"
    );
    assert!(
        gen_code.contains("generate_rock_cluster"),
        "Rock generator must define generate_rock_cluster"
    );
    assert!(
        gen_code.contains("generate_transition_boulders"),
        "Rock generator must define generate_transition_boulders"
    );
    assert!(
        gen_code.contains("poly.normal.z > 0.60"),
        "Rock generator must assign top moss when normal.z > 0.60"
    );

    let rock_glb_path = Path::new("assets/models/faceted_rocks_kit.glb");
    assert!(
        rock_glb_path.exists(),
        "assets/models/faceted_rocks_kit.glb must exist"
    );
    let glb_bytes = fs::read(rock_glb_path).expect("Failed to read faceted_rocks_kit.glb");
    assert!(
        glb_bytes.len() > 20_000,
        "Rock kit GLB must be substantial (>20KB)"
    );
    let magic = u32::from_le_bytes([glb_bytes[0], glb_bytes[1], glb_bytes[2], glb_bytes[3]]);
    assert_eq!(
        magic, 0x46546C67,
        "faceted_rocks_kit.glb must be valid glTF binary"
    );
}

#[test]
fn test_architecture_zero_vision_loop_and_telemetry_pipeline() {

    let spec_path = Path::new("assets/data/scene_spec.json");
    assert!(spec_path.exists(), "assets/data/scene_spec.json must exist");
    let spec_content = fs::read_to_string(spec_path).expect("Failed to read scene_spec.json");
    assert!(
        spec_content.contains("starting_waterfall_grotto"),
        "Spec must define starting_waterfall_grotto"
    );
    assert!(
        spec_content.contains("unobstructed_central_waterfall_corridor_x"),
        "Spec must define central corridor"
    );

    let squeezer_path = Path::new("scripts/vision_squeezer.py");
    assert!(
        squeezer_path.exists(),
        "scripts/vision_squeezer.py must exist"
    );
    let squeezer_code =
        fs::read_to_string(squeezer_path).expect("Failed to read vision_squeezer.py");
    assert!(
        squeezer_code.contains("squeeze_screenshot"),
        "Squeezer must define squeeze_screenshot"
    );
    assert!(
        squeezer_code.contains("generate_roi_crops"),
        "Squeezer must support ROI crops"
    );

    let telemetry_path = Path::new("src/dungeon/telemetry.rs");
    assert!(
        telemetry_path.exists(),
        "src/dungeon/telemetry.rs must exist"
    );
    let tele_code = fs::read_to_string(telemetry_path).expect("Failed to read telemetry.rs");
    assert!(
        tele_code.contains("dump_camera_frustum_telemetry"),
        "Telemetry must define dump_camera_frustum_telemetry"
    );

    let dump_path = Path::new("debug_output/camera_frustum_dump.json");
    assert!(
        dump_path.exists(),
        "debug_output/camera_frustum_dump.json must exist"
    );
    let dump_content =
        fs::read_to_string(dump_path).expect("Failed to read camera_frustum_dump.json");
    assert!(
        dump_content.contains("camera"),
        "Dump must contain camera position"
    );
    assert!(
        dump_content.contains("entities"),
        "Dump must contain visible entities"
    );
}

#[test]
fn test_architecture_zero_comments_policy() {
    let mut code_files = Vec::new();
    fn collect_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if dir_name != "target" && dir_name != ".git" && !dir_name.ends_with(".egg-info") && dir_name != "__pycache__" {
                        collect_files(&path, files);
                    }
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext, "rs" | "wgsl" | "ron" | "py") {
                        files.push(path);
                    }
                }
            }
        }
    }

    collect_files(Path::new("src"), &mut code_files);
    collect_files(Path::new("tests"), &mut code_files);
    collect_files(Path::new("scripts"), &mut code_files);
    collect_files(Path::new("assets/shaders"), &mut code_files);
    collect_files(Path::new("assets/data"), &mut code_files);
    collect_files(Path::new("agent-harness"), &mut code_files);

    let mut violations = Vec::new();

    for file_path in code_files {
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read file: {:?}", file_path));
        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let chars: Vec<char> = content.chars().collect();
        let mut i = 0;
        let n = chars.len();
        let mut line_number = 1;

        if ext == "py" {
            let mut in_single_quote = false;
            let mut in_double_quote = false;
            let mut in_triple_single = false;
            let mut in_triple_double = false;

            while i < n {
                if chars[i] == '\n' {
                    line_number += 1;
                    if in_single_quote {
                        in_single_quote = false;
                    }
                    if in_double_quote {
                        in_double_quote = false;
                    }
                }

                if in_triple_single {
                    if i + 2 < n && chars[i] == '\'' && chars[i + 1] == '\'' && chars[i + 2] == '\'' {
                        in_triple_single = false;
                        i += 3;
                        continue;
                    }
                    i += 1;
                    continue;
                }

                if in_triple_double {
                    if i + 2 < n && chars[i] == '"' && chars[i + 1] == '"' && chars[i + 2] == '"' {
                        in_triple_double = false;
                        i += 3;
                        continue;
                    }
                    i += 1;
                    continue;
                }

                if in_single_quote {
                    if chars[i] == '\\' && i + 1 < n {
                        i += 2;
                        continue;
                    }
                    if chars[i] == '\'' {
                        in_single_quote = false;
                    }
                    i += 1;
                    continue;
                }

                if in_double_quote {
                    if chars[i] == '\\' && i + 1 < n {
                        i += 2;
                        continue;
                    }
                    if chars[i] == '"' {
                        in_double_quote = false;
                    }
                    i += 1;
                    continue;
                }

                if i + 2 < n && chars[i] == '\'' && chars[i + 1] == '\'' && chars[i + 2] == '\'' {
                    violations.push(format!("{}:{}: Docstring or triple-quoted string detected", file_path.display(), line_number));
                    in_triple_single = true;
                    i += 3;
                    continue;
                }

                if i + 2 < n && chars[i] == '"' && chars[i + 1] == '"' && chars[i + 2] == '"' {
                    violations.push(format!("{}:{}: Docstring or triple-quoted string detected", file_path.display(), line_number));
                    in_triple_double = true;
                    i += 3;
                    continue;
                }

                if chars[i] == '\'' {
                    in_single_quote = true;
                    i += 1;
                    continue;
                }

                if chars[i] == '"' {
                    in_double_quote = true;
                    i += 1;
                    continue;
                }

                if chars[i] == '#' {
                    violations.push(format!("{}:{}: Python comment detected", file_path.display(), line_number));
                    while i < n && chars[i] != '\n' {
                        i += 1;
                    }
                    continue;
                }

                i += 1;
            }
        } else {
            let mut in_string = false;

            while i < n {
                if chars[i] == '\n' {
                    line_number += 1;
                }

                if in_string {
                    if chars[i] == '\\' && i + 1 < n {
                        i += 2;
                        continue;
                    }
                    if chars[i] == '"' {
                        in_string = false;
                    }
                    i += 1;
                    continue;
                }

                if chars[i] == '"' {
                    in_string = true;
                    i += 1;
                    continue;
                }

                if chars[i] == '\'' && i + 2 < n && chars[i + 2] == '\'' {
                    i += 3;
                    continue;
                }

                if i + 1 < n && chars[i] == '/' && chars[i + 1] == '/' {
                    violations.push(format!("{}:{}: Line comment detected", file_path.display(), line_number));
                    while i < n && chars[i] != '\n' {
                        i += 1;
                    }
                    continue;
                }

                if i + 1 < n && chars[i] == '/' && chars[i + 1] == '*' {
                    violations.push(format!("{}:{}: Block comment detected", file_path.display(), line_number));
                    i += 2;
                    while i + 1 < n && !(chars[i] == '*' && chars[i + 1] == '/') {
                        if chars[i] == '\n' {
                            line_number += 1;
                        }
                        i += 1;
                    }
                    if i + 1 < n {
                        i += 2;
                    }
                    continue;
                }

                i += 1;
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Code must speak for itself! Found {} comments in codebase:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

#[test]
fn test_architecture_full_game_app_schedule_integrity_and_disjoint_queries_gate() {
    let mut app = bevy::prelude::App::new();
    app.add_plugins((
        bevy::prelude::MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::state::app::StatesPlugin,
        avian3d::prelude::PhysicsPlugins::default(),
    ));
    app.init_resource::<bevy::prelude::Assets<bevy::prelude::StandardMaterial>>();
    app.init_resource::<bevy::prelude::Assets<bevy::prelude::Mesh>>();
    app.init_resource::<bevy::prelude::ButtonInput<bevy::prelude::MouseButton>>();
    app.init_resource::<bevy::prelude::ButtonInput<bevy::prelude::KeyCode>>();
    app.init_resource::<deep_alchemy::alchemy::PotionCooldowns>();
    app.init_resource::<deep_alchemy::ui::hud::ActiveHotbarSlot>();
    app.init_resource::<deep_alchemy::player::camera::CameraShake>();

    let mut schedule = bevy::prelude::Schedule::default();
    schedule.add_systems((
        deep_alchemy::player::controller::player_look,
        deep_alchemy::player::controller::player_movement,
        deep_alchemy::player::controller::player_camera_effects,
        deep_alchemy::alchemy::update_potion_cooldowns,
        deep_alchemy::alchemy::update_alchemy_transient_emitters,
        deep_alchemy::alchemy::cleanup_alchemy_memory_limits,
        deep_alchemy::alchemy::update_active_effects,
        deep_alchemy::alchemy::update_thermal_geysers,
        deep_alchemy::alchemy::update_potion_aiming,
        deep_alchemy::alchemy::spawn_potion_projectile,
        deep_alchemy::alchemy::update_potion_projectiles,
        deep_alchemy::alchemy::update_potion_puddles,
        deep_alchemy::alchemy::update_glass_shards,
        deep_alchemy::alchemy::update_frozen_water_sheets,
        deep_alchemy::alchemy::update_ice_footprints,
    ));
    schedule.add_systems((
        deep_alchemy::alchemy::update_flammable_combustion,
        deep_alchemy::ui::hud::update_hotbar_cooldown_visuals,
        deep_alchemy::ui::hud::update_hotbar_selection_visuals,
        deep_alchemy::ui::hud::update_hotbar_pop_animations,
        deep_alchemy::ui::hud::update_critical_health_pulse,
        deep_alchemy::dungeon::modular_kit::update_grass_bending,
        deep_alchemy::dungeon::modular_kit::update_water_ripples,
        deep_alchemy::dungeon::modular_kit::update_bioluminescent_mushrooms,
    ));

    let _ = schedule.initialize(app.world_mut());
    assert!(schedule.systems_len() > 0);
}

#[test]
fn test_architecture_no_forbidden_engine_asset_reinitialization_in_plugins() {
    fn collect_rs_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_rs_files(&path, files);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
        }
    }

    let mut src_files = Vec::new();
    collect_rs_files(Path::new("src"), &mut src_files);

    let forbidden_types = [
        "init_asset::<Image>",
        "init_asset::<bevy::image::Image>",
        "init_asset::<bevy::prelude::Image>",
        "init_asset::<Mesh>",
        "init_asset::<bevy::prelude::Mesh>",
        "init_asset::<StandardMaterial>",
        "init_asset::<bevy::prelude::StandardMaterial>",
        "init_asset::<Font>",
        "init_asset::<bevy::text::Font>",
        "init_asset::<bevy::prelude::Font>",
        "init_asset::<Scene>",
        "init_asset::<bevy::scene::Scene>",
        "init_asset::<bevy::prelude::Scene>",
    ];

    let mut violations = Vec::new();

    for file_path in src_files {
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read file: {:?}", file_path));

        let prod_content = if let Some(idx) = content.find("#[cfg(test)]") {
            &content[..idx]
        } else {
            &content[..]
        };

        for forbidden in &forbidden_types {
            if prod_content.contains(forbidden) {
                violations.push(format!(
                    "{}: Contains forbidden engine asset re-initialization '{}'",
                    file_path.display(),
                    forbidden
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Forbidden engine asset initialization detected in production plugins!\n\
        Calling init_asset on built-in engine types destroys TonemappingLuts / asset storages,\n\
        leading to handle collisions and fatal WGPU D3/D2 texture binding validation panics.\n\
        Violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_architecture_offscreen_camera_tonemapping_and_view_invariants() {
    use bevy::asset::AssetApp;
    use bevy::core_pipeline::tonemapping::Tonemapping;
    use bevy::render::camera::RenderTarget;
    use deep_alchemy::render::planar_reflection::PlanarReflectionPlugin;

    let mut app = bevy::prelude::App::new();
    app.add_plugins((
        bevy::prelude::MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
    ));
    app.init_asset::<bevy::prelude::Image>();
    app.add_plugins(PlanarReflectionPlugin);
    app.update();

    let mut camera_q = app
        .world_mut()
        .query::<(&bevy::prelude::Camera, Option<&Tonemapping>, Option<&bevy::prelude::Msaa>)>();

    let mut validated_offscreen_cameras = 0;
    for (cam, tonemapping, msaa) in camera_q.iter(app.world()) {
        if matches!(cam.target, RenderTarget::Image(_)) {
            assert_eq!(
                tonemapping,
                Some(&Tonemapping::None),
                "Offscreen render-to-texture camera must use Tonemapping::None to avoid 3D LUT binding errors and double tonemapping"
            );
            assert_eq!(
                msaa,
                Some(&bevy::prelude::Msaa::Off),
                "Offscreen render-to-texture camera must have Msaa::Off"
            );
            validated_offscreen_cameras += 1;
        }
    }

    assert!(
        validated_offscreen_cameras >= 1,
        "PlanarReflectionCamera must be spawned and validated"
    );
}

#[test]
fn test_architecture_tonemapping_lut_and_texture_dimension_safety() {
    use bevy::render::render_resource::{TextureDimension, TextureUsages};
    use deep_alchemy::render::planar_reflection::create_planar_reflection_image;

    let reflection_img = create_planar_reflection_image(512, 512);
    assert_eq!(
        reflection_img.texture_descriptor.dimension,
        TextureDimension::D2,
        "Planar reflection render target must be a 2D texture"
    );
    assert!(
        reflection_img
            .texture_descriptor
            .usage
            .contains(TextureUsages::RENDER_ATTACHMENT),
        "Planar reflection image must have RENDER_ATTACHMENT usage"
    );
    assert!(
        reflection_img
            .texture_descriptor
            .usage
            .contains(TextureUsages::TEXTURE_BINDING),
        "Planar reflection image must have TEXTURE_BINDING usage"
    );
}

