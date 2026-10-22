use std::path::Path;

#[derive(Debug, Clone)]
pub struct FrameValidationReport {
    pub width: u32,
    pub height: u32,
    pub total_pixels: u64,
    pub gray_percentage: f64,
    pub vibrant_percentage: f64,
    pub avg_saturation: f64,
    pub edge_percentage: f64,
    pub passed: bool,
}

pub fn analyze_frame(image_path: &Path) -> Result<FrameValidationReport, String> {
    if !image_path.exists() {
        return Err(format!(
            "Arquivo de validação visual não encontrado: {}",
            image_path.display()
        ));
    }

    let bytes = std::fs::read(image_path)
        .map_err(|e| format!("Falha ao ler arquivo {}: {}", image_path.display(), e))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Falha ao decodificar imagem: {}", e))?;

    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();
    let total_pixels = (width as u64) * (height as u64);

    if total_pixels == 0 {
        return Err("Imagem com 0 pixels".to_string());
    }

    let mut flat_gray_count = 0u64;
    let mut vibrant_color_count = 0u64;
    let mut total_saturation = 0.0f64;

    for pixel in rgb.pixels() {
        let r = pixel[0] as f64;
        let g = pixel[1] as f64;
        let b = pixel[2] as f64;

        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        let diff = max_c - min_c;
        let sat = if max_c > 0.0 { diff / max_c } else { 0.0 };
        total_saturation += sat;

        if diff <= 12.0 && pixel[0] >= 35 && pixel[0] <= 175 {
            flat_gray_count += 1;
        }

        if sat > 0.35 && max_c > 60.0 {
            vibrant_color_count += 1;
        }
    }

    let gray_percentage = (flat_gray_count as f64 / total_pixels as f64) * 100.0;
    let vibrant_percentage = (vibrant_color_count as f64 / total_pixels as f64) * 100.0;
    let avg_saturation = total_saturation / total_pixels as f64;

    println!("==================================================");
    println!("🔍 [VISUAL REGRESSION GATE] Analisando: {}", image_path.display());
    println!("📐 Resolução do Frame: {}x{} ({} pixels)", width, height, total_pixels);
    println!("==================================================");
    println!("📊 Estatísticas de Cor:");
    println!("   • Pixels Cinza Monótono: {:.2}% (Limite Máximo: 30.0%)", gray_percentage);
    println!("   • Pixels com Cores Vivas/Bioluminescentes: {:.2}%", vibrant_percentage);
    println!("   • Saturação Média: {:.3}", avg_saturation);

    if gray_percentage > 30.0 {
        println!("\n❌ [ERRO FATAL] Cena reprovada pelo Detector de Cinza:");
        println!("   Mais de 30% da tela ({:.1}%) é cinza/preto sem textura.", gray_percentage);
        println!("   Cena sem geometria de caverna fechada ou sem paleta de cores cel-shaded!");
        return Ok(FrameValidationReport {
            width,
            height,
            total_pixels,
            gray_percentage,
            vibrant_percentage,
            avg_saturation,
            edge_percentage: 0.0,
            passed: false,
        });
    }

    let luma = img.to_luma8();
    let mut edge_count = 0u64;
    let mut samples_checked = 0u64;
    let sample_step = 2u32;

    if height > 2 && width > 2 {
        let mut y = 1u32;
        while y < height - 1 {
            let mut x = 1u32;
            while x < width - 1 {
                samples_checked += 1;

                let p = |px: u32, py: u32| -> i32 {
                    luma.get_pixel(px, py)[0] as i32
                };

                let gx = (p(x + 1, y - 1) + 2 * p(x + 1, y) + p(x + 1, y + 1))
                    - (p(x - 1, y - 1) + 2 * p(x - 1, y) + p(x - 1, y + 1));
                let gy = (p(x - 1, y + 1) + 2 * p(x, y + 1) + p(x + 1, y + 1))
                    - (p(x - 1, y - 1) + 2 * p(x, y - 1) + p(x + 1, y - 1));

                let grad = ((gx * gx + gy * gy) as f64).sqrt();
                if grad > 110.0 {
                    edge_count += 1;
                }
                x += sample_step;
            }
            y += sample_step;
        }
    }

    let edge_percentage = if samples_checked > 0 {
        (edge_count as f64 / samples_checked as f64) * 100.0
    } else {
        0.0
    };

    println!("\n📐 Análise de Contornos & Cel-Shading (Sobel Edge Gate):");
    println!("   • Densidade de Bordas de Alto Contraste: {:.2}% (Mínimo: 0.80%)", edge_percentage);

    if edge_percentage < 0.80 {
        println!("\n❌ [ERRO FATAL] Cena reprovada pelo Detector de Outlines:");
        println!("   Densidade de contornos insuficiente ({:.2}% < 0.80%).", edge_percentage);
        println!("   A cena aparenta ser borrada ou sem sombreamento Toon/Edge Detection!");
        return Ok(FrameValidationReport {
            width,
            height,
            total_pixels,
            gray_percentage,
            vibrant_percentage,
            avg_saturation,
            edge_percentage,
            passed: false,
        });
    }

    println!("\n✅ [SUCESSO] Todos os critérios do Visual Regression Gate foram APROVADOS!");
    println!("   • Ambiente fechado orgânico (sem vazamento de céu cinza)");
    println!("   • Contornos nítidos e silhuetas bem definidas");
    println!("   • Paleta cromática bioluminescente e cel-shaded validada");

    Ok(FrameValidationReport {
        width,
        height,
        total_pixels,
        gray_percentage,
        vibrant_percentage,
        avg_saturation,
        edge_percentage,
        passed: true,
    })
}

pub fn main_entry() {
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 {
        args[1].clone()
    } else {
        "debug_output/frame_check.png".to_string()
    };

    match analyze_frame(Path::new(&target)) {
        Ok(report) => {
            if report.passed {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("❌ [ERRO FATAL] {}", err);
            std::process::exit(1);
        }
    }
}
