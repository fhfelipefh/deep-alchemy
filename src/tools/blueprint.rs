use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DepthRegionStats {
    pub avg_luma: f64,
    pub biolum_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct DominantColor {
    pub hex: String,
    pub percentage: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct BlueprintReport {
    pub width: u32,
    pub height: u32,
    pub foreground: DepthRegionStats,
    pub midground: DepthRegionStats,
    pub background: DepthRegionStats,
    pub dominant_colors: Vec<DominantColor>,
    pub sobel_density: f64,
}

pub fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;

    let cmax = rf.max(gf).max(bf);
    let cmin = rf.min(gf).min(bf);
    let delta = cmax - cmin;

    let h = if delta == 0.0 {
        0.0
    } else if cmax == rf {
        60.0 * (((gf - bf) / delta) % 6.0)
    } else if cmax == gf {
        60.0 * (((bf - rf) / delta) + 2.0)
    } else {
        60.0 * (((rf - gf) / delta) + 4.0)
    };

    let h_normalized = if h < 0.0 { h + 360.0 } else { h };
    let s = if cmax == 0.0 { 0.0 } else { delta / cmax };
    let v = cmax;

    (h_normalized, s, v)
}

pub fn compute_sobel_density(img: &image::DynamicImage) -> f64 {
    let gray = img.to_luma8();
    let (w, h) = gray.dimensions();
    if w < 3 || h < 3 {
        return 0.0;
    }

    let mut edge_pixels = 0u64;
    let threshold = 38.0f64;

    let mut y = 1u32;
    while y < h - 1 {
        let mut x = 1u32;
        while x < w - 1 {
            let p = |px: u32, py: u32| -> i32 {
                gray.get_pixel(px, py)[0] as i32
            };

            let p00 = p(x - 1, y - 1);
            let p10 = p(x, y - 1);
            let p20 = p(x + 1, y - 1);
            let p01 = p(x - 1, y);
            let p21 = p(x + 1, y);
            let p02 = p(x - 1, y + 1);
            let p12 = p(x, y + 1);
            let p22 = p(x + 1, y + 1);

            let gx = (p20 + 2 * p21 + p22) - (p00 + 2 * p01 + p02);
            let gy = (p02 + 2 * p12 + p22) - (p00 + 2 * p10 + p20);
            let mag = ((gx * gx + gy * gy) as f64).sqrt();
            if mag > threshold {
                edge_pixels += 1;
            }

            x += 2;
        }
        y += 2;
    }

    let sampled_pixels = (((w - 2) / 2) as u64) * (((h - 2) / 2) as u64);
    if sampled_pixels > 0 {
        (edge_pixels as f64 / sampled_pixels as f64) * 100.0
    } else {
        0.0
    }
}

pub fn generate_blueprint_report(img_path: &Path) -> Result<BlueprintReport, String> {
    if !img_path.exists() {
        return Err(format!("Imagem não encontrada: {}", img_path.display()));
    }

    let bytes = std::fs::read(img_path)
        .map_err(|e| format!("Falha ao ler {}: {}", img_path.display(), e))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Falha ao decodificar imagem: {}", e))?;

    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let total_pixels = (w as u64) * (h as u64);
    if total_pixels == 0 {
        return Err("Imagem com 0 pixels".to_string());
    }

    let mut bg_luma = 0.0f64;
    let mut bg_biolum = 0u64;
    let mut bg_pixels = 0u64;

    let mut mg_luma = 0.0f64;
    let mut mg_biolum = 0u64;
    let mut mg_pixels = 0u64;

    let mut fg_luma = 0.0f64;
    let mut fg_biolum = 0u64;
    let mut fg_pixels = 0u64;

    let mut color_bins: HashMap<(u8, u8, u8), u64> = HashMap::new();

    for y in 0..h {
        let is_bg = y < h / 3;
        let is_mg = !is_bg && y < 2 * h / 3;

        for x in 0..w {
            let px = rgb.get_pixel(x, y);
            let (r, g, b) = (px[0], px[1], px[2]);
            let luma = 0.299 * (r as f64) + 0.587 * (g as f64) + 0.114 * (b as f64);

            let qr = ((r / 24) * 24).saturating_add(12);
            let qg = ((g / 24) * 24).saturating_add(12);
            let qb = ((b / 24) * 24).saturating_add(12);
            *color_bins.entry((qr, qg, qb)).or_insert(0) += 1;

            let (h_val, s_val, v_val) = rgb_to_hsv(r, g, b);
            let is_biolum = s_val > 0.40
                && v_val > 0.50
                && ((h_val >= 160.0 && h_val <= 210.0) || (h_val >= 90.0 && h_val <= 150.0));

            if is_bg {
                bg_luma += luma;
                bg_pixels += 1;
                if is_biolum {
                    bg_biolum += 1;
                }
            } else if is_mg {
                mg_luma += luma;
                mg_pixels += 1;
                if is_biolum {
                    mg_biolum += 1;
                }
            } else {
                fg_luma += luma;
                fg_pixels += 1;
                if is_biolum {
                    fg_biolum += 1;
                }
            }
        }
    }

    let bg_stats = DepthRegionStats {
        avg_luma: if bg_pixels > 0 { bg_luma / bg_pixels as f64 } else { 0.0 },
        biolum_ratio: if bg_pixels > 0 { bg_biolum as f64 / bg_pixels as f64 } else { 0.0 },
    };

    let mg_stats = DepthRegionStats {
        avg_luma: if mg_pixels > 0 { mg_luma / mg_pixels as f64 } else { 0.0 },
        biolum_ratio: if mg_pixels > 0 { mg_biolum as f64 / mg_pixels as f64 } else { 0.0 },
    };

    let fg_stats = DepthRegionStats {
        avg_luma: if fg_pixels > 0 { fg_luma / fg_pixels as f64 } else { 0.0 },
        biolum_ratio: if fg_pixels > 0 { fg_biolum as f64 / fg_pixels as f64 } else { 0.0 },
    };

    let mut sorted_bins: Vec<((u8, u8, u8), u64)> = color_bins.into_iter().collect();
    sorted_bins.sort_by(|a, b| b.1.cmp(&a.1));
    sorted_bins.truncate(8);

    let mut dominant_colors = Vec::new();
    for ((qr, qg, qb), count) in sorted_bins {
        let pct = (count as f64 / total_pixels as f64) * 100.0;
        let hex = rgb_to_hex(qr, qg, qb);

        let category = if (qb as f64) > (qr as f64) * 1.3 && (qg as f64 + qb as f64) > (qr as f64) * 2.0 {
            "Água / Cascata / Bioluminescência Ciano"
        } else if (qg as f64) > (qr as f64) * 1.15 && qg > qb {
            "Grama / Musgo Aveludado das Rochas"
        } else if (qr as f64) > (qg as f64) * 1.1 && (qr as f64) > (qb as f64) * 1.3 {
            "Terra Batida Marrom / Madeira do Banco"
        } else if qr.abs_diff(qg).max(qr.abs_diff(qb)).max(qg.abs_diff(qb)) < 22 {
            "Rochas Cinzas da Caverna / Teto"
        } else if qr > 180 && qg > 180 && qb > 180 {
            "Espuma Aerada / Cristas da Queda d'Água"
        } else {
            "Sombra Profunda / Penumbra da Gruta"
        };

        dominant_colors.push(DominantColor {
            hex,
            percentage: pct,
            r: qr,
            g: qg,
            b: qb,
            category: category.to_string(),
        });
    }

    let sobel_density = compute_sobel_density(&img);

    println!("\n======================================================================");
    println!("🛠️  [TECHNICAL ARTIST BLUEPRINT & VISUAL DIFF REPORT]");
    println!("📁 Arquivo Analisado: {} ({}x{} px)", img_path.display(), w, h);
    println!("======================================================================");

    println!("\n🎨 1. PALETA CROMÁTICA DOMINANTE EXTRAÍDA (Top 8):");
    println!("{:<10} | {:<10} | {}", "Hex", "Cobertura", "Interpretação do Technical Artist");
    println!("-----------------------------------------------------------------");
    for color in &dominant_colors {
        println!("{:<10} | {:>8.2}% | {}", color.hex, color.percentage, color.category);
    }

    println!("\n📐 2. DECOMPOSIÇÃO EM PLANOS DE PROFUNDIDADE (Z-DEPTH):");
    println!("   • Foreground  : Luminosidade Média = {:>5.1}/255 | Bioluminescência = {:>5.2}%", fg_stats.avg_luma, fg_stats.biolum_ratio * 100.0);
    println!("   • Midground   : Luminosidade Média = {:>5.1}/255 | Bioluminescência = {:>5.2}%", mg_stats.avg_luma, mg_stats.biolum_ratio * 100.0);
    println!("   • Background  : Luminosidade Média = {:>5.1}/255 | Bioluminescência = {:>5.2}%", bg_stats.avg_luma, bg_stats.biolum_ratio * 100.0);

    println!("\n🖋️ 3. DENSIDADE DE CONTORNOS CEL-SHADED (Sobel Edge Gate):");
    println!("   • Linhas de Nanquim Detectadas: {:.2}% (Meta recomendada: 5.0% - 12.0%)", sobel_density);

    println!("\n📋 4. BLUEPRINT DE COORDENADAS RECOMENDADAS PARA CENA 3D:");
    println!("   • Foreground (Solo e Banco):");
    println!("     - Caminho de Terra Marrom: X central entre -2.5 e +2.5, Z de -1.0 a -4.5");
    println!("     - Banco Rústico com Musgo: X = +2.65, Z = -2.1, Rotação Y = -0.50 rad");
    println!("   • Midground (Lagoa e Queda d'Água):");
    println!("     - Lagoa Circular: Centro (0.0, -8.5), Raio = 5.2m, Borda Z = 0.08m");
    println!("     - Cascata Central: Eixo X = 0.0, Z = -8.5 a -10.8, Altura Y = 0.0 a 18.0m");
    println!("   • Background (Paredes e Teto):");
    println!("     - Rochas Facetadas com Musgo Superior: X laterais < -5.0 e > +5.0");
    println!("     - Teto Abobadado: Cúpula fechando até Y = 23.5m com estalactites pontiagudas laterais");
    println!("======================================================================\n");

    Ok(BlueprintReport {
        width: w,
        height: h,
        foreground: fg_stats,
        midground: mg_stats,
        background: bg_stats,
        dominant_colors,
        sobel_density,
    })
}

pub fn main_entry() {
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 {
        args[1].clone()
    } else {
        "debug_output/frame_check.png".to_string()
    };

    if let Err(e) = generate_blueprint_report(Path::new(&target)) {
        eprintln!("❌ Erro: {}", e);
        std::process::exit(1);
    }
}
