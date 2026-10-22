use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn estimate_vision_tokens(width: u32, height: u32, tile_size: u32) -> (u32, u32) {
    let tiles_x = (width + tile_size - 1) / tile_size;
    let tiles_y = (height + tile_size - 1) / tile_size;
    let total_tiles = tiles_x * tiles_y;
    let approx_tokens = total_tiles * 258 + 85;
    (total_tiles, approx_tokens)
}

pub fn squeeze_screenshot(
    input_path: &Path,
    output_path: Option<&Path>,
    max_width: u32,
) -> Result<PathBuf, String> {
    if !input_path.exists() {
        return Err(format!("File not found: {}", input_path.display()));
    }

    let bytes = std::fs::read(input_path)
        .map_err(|e| format!("Failed to read {}: {}", input_path.display(), e))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let (orig_w, orig_h) = (img.width(), img.height());
    if orig_w == 0 || orig_h == 0 {
        return Err("Invalid image dimensions 0".to_string());
    }

    let aspect = orig_h as f64 / orig_w as f64;
    let mut new_w = orig_w.min(max_width);
    new_w = (new_w / 16) * 16;
    if new_w == 0 {
        new_w = 16;
    }
    let mut new_h = (new_w as f64 * aspect) as u32;
    new_h = (new_h / 16) * 16;
    if new_h == 0 {
        new_h = 16;
    }

    let rgb = img.to_rgb8();
    let squeezed = image::imageops::resize(&rgb, new_w, new_h, image::imageops::FilterType::Lanczos3);

    let out_path = if let Some(p) = output_path {
        p.to_path_buf()
    } else {
        let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
        let stem = input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("frame");
        parent.join(format!("{}_squeezed.png", stem))
    };

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
    }

    squeezed
        .save_with_format(&out_path, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to save image {}: {}", out_path.display(), e))?;

    let (_orig_tiles, orig_tokens) = estimate_vision_tokens(orig_w, orig_h, 512);
    let (_new_tiles, new_tokens) = estimate_vision_tokens(new_w, new_h, 512);
    let savings_pct = (1.0 - (new_tokens as f64 / orig_tokens as f64)) * 100.0;

    println!("\n=================================================================");
    println!("🔍 [VISION TOKEN SQUEEZER MIDDLEWARE REPORT]");
    println!("📁 Imagem Original : {} ({}x{} px) ~ {} tokens", input_path.display(), orig_w, orig_h, orig_tokens);
    println!("⚡ Imagem Otimizada: {} ({}x{} px) ~ {} tokens", out_path.display(), new_w, new_h, new_tokens);
    println!("💰 Economia de Tokens de Visão: {:.1}% de redução!", savings_pct);
    println!("=================================================================\n");

    Ok(out_path)
}

pub fn generate_roi_crops(
    input_path: &Path,
    output_dir: &Path,
) -> Result<HashMap<String, (PathBuf, u32, u32, u32)>, String> {
    if !input_path.exists() {
        return Err(format!("File not found: {}", input_path.display()));
    }

    let bytes = std::fs::read(input_path)
        .map_err(|e| format!("Failed to read {}: {}", input_path.display(), e))?;

    let mut img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let (w, h) = (img.width(), img.height());
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create directory {}: {}", output_dir.display(), e))?;

    let crop_defs = [
        ("ceiling_arch_stalactites", 0, 0, w, ((h as f64) * 0.40) as u32),
        (
            "central_waterfall_lagoon",
            ((w as f64) * 0.25) as u32,
            ((h as f64) * 0.20) as u32,
            ((w as f64) * 0.75) as u32,
            ((h as f64) * 0.85) as u32,
        ),
        ("foreground_dirt_and_bench", 0, ((h as f64) * 0.60) as u32, w, h),
        (
            "hotbar_potions",
            ((w as f64) * 0.32) as u32,
            ((h as f64) * 0.86) as u32,
            ((w as f64) * 0.68) as u32,
            h,
        ),
        (
            "left_mushrooms",
            0,
            ((h as f64) * 0.30) as u32,
            ((w as f64) * 0.35) as u32,
            ((h as f64) * 0.85) as u32,
        ),
        (
            "right_mushrooms",
            ((w as f64) * 0.65) as u32,
            ((h as f64) * 0.30) as u32,
            w,
            ((h as f64) * 0.85) as u32,
        ),
    ];

    let mut generated = HashMap::new();

    for (name, x0, y0, x1, y1) in crop_defs {
        let cw = x1.saturating_sub(x0).max(1);
        let ch = y1.saturating_sub(y0).max(1);

        let sub_img = image::imageops::crop(&mut img, x0, y0, cw, ch).to_image();
        let aspect = ch as f64 / cw as f64;
        let nw = 512u32;
        let nh = ((nw as f64) * aspect) as u32;

        let crop_squeezed = image::imageops::resize(&sub_img, nw, nh, image::imageops::FilterType::Lanczos3);
        let path = output_dir.join(format!("{}.png", name));
        crop_squeezed
            .save_with_format(&path, image::ImageFormat::Png)
            .map_err(|e| format!("Failed to save crop {}: {}", path.display(), e))?;

        let (_tiles, tokens) = estimate_vision_tokens(nw, nh, 512);
        println!("   • Crop Especializado: {:<26} -> {}x{} px (~{} tokens)", name, nw, nh, tokens);
        generated.insert(name.to_string(), (path, nw, nh, tokens));
    }

    Ok(generated)
}

pub fn main_entry() {
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 {
        args[1].clone()
    } else {
        "debug_output/frame_check.png".to_string()
    };

    let with_crops = args.iter().any(|a| a == "--crops");

    match squeeze_screenshot(Path::new(&target), None, 640) {
        Ok(_) => {
            if with_crops {
                if let Err(e) = generate_roi_crops(Path::new(&target), Path::new("debug_output/crops")) {
                    eprintln!("Error generating crops: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
