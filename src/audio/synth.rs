use std::f32::consts::PI;
use std::fs;
use std::path::Path;

pub fn generate_wav_bytes(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len() as u32;
    let byte_rate = sample_rate * 2;
    let block_align: u16 = 2;
    let subchunk2_size = num_samples * 2;
    let chunk_size = 36 + subchunk2_size;

    let mut wav = Vec::with_capacity(44 + subchunk2_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&subchunk2_size.to_le_bytes());
    for &sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

fn pseudo_noise(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    ((*seed >> 16) as f32 / 32767.5) - 1.0
}

pub fn synthesize_potion_throw(sample_rate: u32) -> Vec<i16> {
    let duration = 0.16;
    let total_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);
    let mut seed = 12345;

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = t / duration;
        let freq = 520.0 - progress * 340.0;
        let env = (progress * PI).sin().powf(1.4);
        let tone = (2.0 * PI * freq * t).sin();
        let noise = pseudo_noise(&mut seed) * 0.45;
        let mixed = (tone * 0.65 + noise * 0.35) * env * 0.85;
        let sample = (mixed.clamp(-1.0, 1.0) * 32767.0) as i16;
        samples.push(sample);
    }
    samples
}

pub fn synthesize_potion_shatter(sample_rate: u32) -> Vec<i16> {
    let duration = 0.28;
    let total_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);
    let mut seed = 54321;

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let _progress = t / duration;
        let decay = (-t * 18.0).exp();
        let ping1 = (2.0 * PI * 2800.0 * t).sin() * 0.35;
        let ping2 = (2.0 * PI * 3650.0 * t).sin() * 0.25;
        let ping3 = (2.0 * PI * 4820.0 * t).sin() * 0.20;
        let thud = (2.0 * PI * 130.0 * t).sin() * (-t * 30.0).exp() * 0.50;
        let noise = pseudo_noise(&mut seed) * (-t * 24.0).exp() * 0.40;
        let mixed = (ping1 + ping2 + ping3 + thud + noise) * decay * 0.95;
        let sample = (mixed.clamp(-1.0, 1.0) * 32767.0) as i16;
        samples.push(sample);
    }
    samples
}

pub fn synthesize_water_splash(sample_rate: u32) -> Vec<i16> {
    let duration = 0.35;
    let total_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);
    let mut seed = 99887;

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = t / duration;
        let decay = (-t * 11.0).exp();
        let freq = 380.0 - progress * 240.0;
        let bubble = (2.0 * PI * 35.0 * t).sin() * 40.0;
        let tone = (2.0 * PI * (freq + bubble) * t).sin() * 0.60;
        let noise = pseudo_noise(&mut seed) * (-t * 14.0).exp() * 0.40;
        let mixed = (tone + noise) * decay * 0.88;
        let sample = (mixed.clamp(-1.0, 1.0) * 32767.0) as i16;
        samples.push(sample);
    }
    samples
}

pub fn synthesize_torch_ignite(sample_rate: u32) -> Vec<i16> {
    let duration = 0.40;
    let total_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);
    let mut seed = 77113;

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = t / duration;
        let freq = 140.0 + progress * 240.0;
        let attack_decay = (progress * PI).sin().powf(0.8);
        let tone = (2.0 * PI * freq * t).sin() * 0.45;
        let noise_val = pseudo_noise(&mut seed);
        let crackle = if noise_val > 0.88 { 0.65 } else { 0.0 };
        let whoosh = noise_val * 0.30;
        let mixed = (tone + whoosh + crackle) * attack_decay * 0.80;
        let sample = (mixed.clamp(-1.0, 1.0) * 32767.0) as i16;
        samples.push(sample);
    }
    samples
}

pub fn synthesize_ice_freeze(sample_rate: u32) -> Vec<i16> {
    let duration = 0.45;
    let total_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let decay = (-t * 7.5).exp();
        let vibrato = (2.0 * PI * 6.0 * t).sin() * 12.0;
        let c1 = (2.0 * PI * (1046.5 + vibrato) * t).sin() * 0.45;
        let c2 = (2.0 * PI * (1567.9 + vibrato) * t).sin() * 0.30;
        let c3 = (2.0 * PI * (2093.0 + vibrato) * t).sin() * 0.20;
        let mixed = (c1 + c2 + c3) * decay * 0.82;
        let sample = (mixed.clamp(-1.0, 1.0) * 32767.0) as i16;
        samples.push(sample);
    }
    samples
}

pub fn synthesize_creature_chirp(sample_rate: u32) -> Vec<i16> {
    let duration = 0.18;
    let total_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = t / duration;
        let freq = 580.0 + (progress * PI * 0.5).sin() * 380.0;
        let env = (progress * PI).sin().powf(1.2);
        let tone = (2.0 * PI * freq * t).sin();
        let overtone = (2.0 * PI * (freq * 2.0) * t).sin() * 0.25;
        let mixed = (tone + overtone) * env * 0.75;
        let sample = (mixed.clamp(-1.0, 1.0) * 32767.0) as i16;
        samples.push(sample);
    }
    samples
}

pub fn synthesize_ui_click(sample_rate: u32) -> Vec<i16> {
    let duration = 0.035;
    let total_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let decay = (-t * 90.0).exp();
        let tone = (2.0 * PI * 1400.0 * t).sin();
        let mixed = tone * decay * 0.70;
        let sample = (mixed.clamp(-1.0, 1.0) * 32767.0) as i16;
        samples.push(sample);
    }
    samples
}

pub fn synthesize_water_drop(sample_rate: u32) -> Vec<i16> {
    let duration = 0.22;
    let total_samples = (duration * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let progress = t / duration;
        let freq = 1150.0 + progress * 450.0;
        let decay = (-t * 22.0).exp();
        let tone = (2.0 * PI * freq * t).sin();
        let mixed = tone * decay * 0.65;
        let sample = (mixed.clamp(-1.0, 1.0) * 32767.0) as i16;
        samples.push(sample);
    }
    samples
}

pub fn ensure_procedural_audio_assets() {
    let audio_dir = Path::new("assets/audio");
    if !audio_dir.exists() {
        fs::create_dir_all(audio_dir).ok();
    }

    let sample_rate = 44100;

    let wav_assets: [(&str, fn(u32) -> Vec<i16>); 8] = [
        ("assets/audio/potion_throw.wav", synthesize_potion_throw),
        ("assets/audio/potion_shatter.wav", synthesize_potion_shatter),
        ("assets/audio/water_splash.wav", synthesize_water_splash),
        ("assets/audio/torch_ignite.wav", synthesize_torch_ignite),
        ("assets/audio/ice_freeze.wav", synthesize_ice_freeze),
        ("assets/audio/creature_chirp.wav", synthesize_creature_chirp),
        ("assets/audio/ui_click.wav", synthesize_ui_click),
        ("assets/audio/water_drop.wav", synthesize_water_drop),
    ];

    for (file_path, synth_fn) in wav_assets {
        let path = Path::new(file_path);
        if !path.exists() {
            let pcm = synth_fn(sample_rate);
            let wav_bytes = generate_wav_bytes(&pcm, sample_rate);
            fs::write(path, wav_bytes).ok();
        }
    }
}
