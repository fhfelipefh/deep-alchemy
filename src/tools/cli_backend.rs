use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::alchemy::AlchemicalEffectKind;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub workspace_root: String,
    pub game_binary: Option<String>,
    pub binary_compiled: bool,
    pub assets_models: bool,
    pub recipes_ron: bool,
    pub scene_spec: bool,
    pub active_dungeon_seed: u64,
    pub active_hotbar_slot: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RonIngredient {
    pub id: String,
    pub name: String,
    pub description: String,
    pub weight: f32,
    pub value: u32,
    pub primary_aspect: AlchemicalEffectKind,
    pub secondary_aspect: AlchemicalEffectKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RonPotionEffect {
    pub kind: AlchemicalEffectKind,
    pub duration: f32,
    pub magnitude: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RonPotion {
    pub id: String,
    pub name: String,
    pub liquid_color: (f32, f32, f32, f32),
    pub effects: Vec<RonPotionEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RonDatabase {
    pub ingredients: Vec<RonIngredient>,
    pub potions: Vec<RonPotion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrewResult {
    pub success: bool,
    pub brewed_potion: Option<String>,
    pub liquid_color: Option<Vec<f32>>,
    pub applied_effects: Vec<RonPotionEffect>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerStats {
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub active_slot: usize,
    pub hotbar_slots: Vec<String>,
}

pub fn find_workspace_root() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    while !current.join("Cargo.toml").exists() {
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            return PathBuf::from(".");
        }
    }
    current
}

pub fn find_game_binary(root: &Path) -> Option<PathBuf> {
    let debug_bin = root.join("target").join("debug").join("deep_alchemy.exe");
    if debug_bin.exists() {
        return Some(debug_bin);
    }
    let linux_bin = root.join("target").join("debug").join("deep_alchemy");
    if linux_bin.exists() {
        return Some(linux_bin);
    }
    None
}

pub fn get_system_status() -> SystemStatus {
    let root = find_workspace_root();
    let binary = find_game_binary(&root);
    let binary_str = binary.as_ref().map(|p| p.display().to_string());
    let models_exist = root.join("assets").join("models").join("cavern_kit.glb").exists();
    let ron_exists = root.join("assets").join("data").join("alchemy_recipes.ron").exists();
    let spec_exists = root.join("assets").join("data").join("scene_spec.json").exists();

    SystemStatus {
        workspace_root: root.display().to_string(),
        game_binary: binary_str,
        binary_compiled: binary.is_some(),
        assets_models: models_exist,
        recipes_ron: ron_exists,
        scene_spec: spec_exists,
        active_dungeon_seed: 42,
        active_hotbar_slot: 1,
    }
}

pub fn load_ron_database() -> Result<RonDatabase, String> {
    let root = find_workspace_root();
    let ron_path = root.join("assets").join("data").join("alchemy_recipes.ron");
    if !ron_path.exists() {
        return Err(format!("File not found: {}", ron_path.display()));
    }
    let content = std::fs::read_to_string(&ron_path)
        .map_err(|e| format!("Failed to read {}: {}", ron_path.display(), e))?;
    ron::from_str(&content)
        .map_err(|e| format!("Failed to parse RON: {}", e))
}

pub fn brew(ingredients: &[String]) -> BrewResult {
    let db = match load_ron_database() {
        Ok(db) => db,
        Err(e) => {
            return BrewResult {
                success: false,
                brewed_potion: None,
                liquid_color: None,
                applied_effects: Vec::new(),
                error: Some(e),
            }
        }
    };

    let mut unknown = Vec::new();
    let mut selected_aspects = Vec::new();

    for ing_id in ingredients {
        if let Some(ing) = db.ingredients.iter().find(|i| &i.id == ing_id || &i.name == ing_id) {
            selected_aspects.push(ing.primary_aspect.clone());
            selected_aspects.push(ing.secondary_aspect.clone());
        } else {
            unknown.push(ing_id.clone());
        }
    }

    if !unknown.is_empty() {
        return BrewResult {
            success: false,
            brewed_potion: None,
            liquid_color: None,
            applied_effects: Vec::new(),
            error: Some(format!("Unknown ingredients: {}", unknown.join(", "))),
        };
    }

    let mut effects = Vec::new();
    for aspect in selected_aspects {
        effects.push(RonPotionEffect {
            kind: aspect,
            duration: 25.0,
            magnitude: 1.0,
        });
    }

    BrewResult {
        success: true,
        brewed_potion: Some("Custom Cavern Brew".to_string()),
        liquid_color: Some(vec![0.15, 0.85, 0.95, 0.85]),
        applied_effects: effects,
        error: None,
    }
}

pub fn get_player_stats() -> PlayerStats {
    PlayerStats {
        health: 100.0,
        max_health: 100.0,
        stamina: 100.0,
        max_stamina: 100.0,
        active_slot: 1,
        hotbar_slots: vec![
            "Fire Potion".to_string(),
            "Frost Potion".to_string(),
            "Bioluminescent Elixir".to_string(),
            "Vitality Draught".to_string(),
            "Swift Potion".to_string(),
        ],
    }
}

pub fn main_entry() {
    let args: Vec<String> = std::env::args().collect();
    let as_json = args.iter().any(|a| a == "--json");
    let cmd = if args.len() > 1 && !args[1].starts_with("--") {
        args[1].as_str()
    } else {
        "status"
    };

    match cmd {
        "status" => {
            let status = get_system_status();
            if as_json {
                println!("{}", serde_json::to_string_pretty(&status).unwrap());
            } else {
                println!("=== Deep Alchemy System Status ===");
                println!("Workspace Root  : {}", status.workspace_root);
                println!("Game Binary     : {:?}", status.game_binary);
                println!("Binary Compiled : {}", status.binary_compiled);
                println!("Models GLB      : {}", status.assets_models);
                println!("Recipes RON     : {}", status.recipes_ron);
                println!("Scene Spec JSON : {}", status.scene_spec);
                println!("Active Seed     : {}", status.active_dungeon_seed);
                println!("Active Slot     : {}", status.active_hotbar_slot);
            }
        }
        "recipes" => {
            match load_ron_database() {
                Ok(db) => {
                    if as_json {
                        println!("{}", serde_json::to_string_pretty(&db).unwrap());
                    } else {
                        println!("=== Alchemy Database (RON) ===");
                        println!("Ingredients ({}):", db.ingredients.len());
                        for ing in &db.ingredients {
                            println!("  • {} [{}] - {:?} & {:?}", ing.name, ing.id, ing.primary_aspect, ing.secondary_aspect);
                        }
                        println!("\nPotions ({}):", db.potions.len());
                        for pot in &db.potions {
                            println!("  • {} [{}]", pot.name, pot.id);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "brew" => {
            let ingredients: Vec<String> = args.iter().skip(2).filter(|a| !a.starts_with("--")).cloned().collect();
            if ingredients.is_empty() {
                eprintln!("Usage: deep_alchemy_cli brew <ingredient_id1> [ingredient_id2...]");
                std::process::exit(1);
            }
            let res = brew(&ingredients);
            if as_json {
                println!("{}", serde_json::to_string_pretty(&res).unwrap());
            } else if res.success {
                println!("=== Brewing Success ===");
                println!("Potion: {:?}", res.brewed_potion);
                println!("Effects: {} combined effects", res.applied_effects.len());
            } else {
                eprintln!("Brewing failed: {:?}", res.error);
                std::process::exit(1);
            }
        }
        "player" => {
            let stats = get_player_stats();
            if as_json {
                println!("{}", serde_json::to_string_pretty(&stats).unwrap());
            } else {
                println!("=== Player State ===");
                println!("Health  : {}/{}", stats.health, stats.max_health);
                println!("Stamina : {}/{}", stats.stamina, stats.max_stamina);
                println!("Slot    : {}", stats.active_slot);
            }
        }
        "validate" => {
            let target = args.iter().skip(2).find(|a| !a.starts_with("--")).cloned().unwrap_or_else(|| "debug_output/frame_check.png".to_string());
            match super::validator::analyze_frame(Path::new(&target)) {
                Ok(rep) => {
                    if !rep.passed {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "squeeze" => {
            let target = args.iter().skip(2).find(|a| !a.starts_with("--")).cloned().unwrap_or_else(|| "debug_output/frame_check.png".to_string());
            if let Err(e) = super::vision_squeezer::squeeze_screenshot(Path::new(&target), None, 640) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "blueprint" => {
            let target = args.iter().skip(2).find(|a| !a.starts_with("--")).cloned().unwrap_or_else(|| "debug_output/frame_check.png".to_string());
            if let Err(e) = super::blueprint::generate_blueprint_report(Path::new(&target)) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            println!("Deep Alchemy Rust CLI Harness");
            println!("Commands: status, recipes, brew, player, validate, squeeze, blueprint");
        }
    }
}
