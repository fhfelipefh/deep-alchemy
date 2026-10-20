#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct HeatSourceGpu {
    screen_pos: vec2<f32>,
    radius: f32,
    intensity: f32,
};

struct HeatHazeSettings {
    time: f32,
    count: u32,
    distortion_scale: f32,
    flow_speed: f32,
    sources: array<HeatSourceGpu, 16>,
};

@group(0) @binding(2) var<uniform> settings: HeatHazeSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    if (settings.count == 0u) {
        return textureSample(screen_texture, texture_sampler, in.uv);
    }

    var total_offset = vec2<f32>(0.0, 0.0);
    var max_heat_factor = 0.0;

    for (var i = 0u; i < settings.count; i = i + 1u) {
        let source = settings.sources[i];
        let diff = in.uv - source.screen_pos;
        let dist = length(diff);

        if (dist < source.radius && source.radius > 0.001) {
            let norm_dist = dist / source.radius;
            let falloff = (1.0 - norm_dist) * (1.0 - norm_dist);

            let flow_t = settings.time * settings.flow_speed;
            let uv_scaled = in.uv * 50.0;

            let wave_x = sin(uv_scaled.y * 1.5 + flow_t * 6.0 + diff.x * 20.0) 
                       * cos(uv_scaled.x * 0.8 + flow_t * 4.0);
            let wave_y = cos(uv_scaled.x * 1.4 - flow_t * 5.0 + diff.y * 15.0) 
                       * sin(uv_scaled.y * 1.1 + flow_t * 7.0);

            let drift_up = -abs(sin(flow_t * 3.0 + uv_scaled.x * 2.0)) * 0.4;

            let distortion = vec2<f32>(wave_x, wave_y + drift_up) 
                           * source.intensity 
                           * settings.distortion_scale 
                           * falloff;

            total_offset = total_offset + distortion;
            max_heat_factor = max(max_heat_factor, falloff * source.intensity);
        }
    }

    if (length(total_offset) < 0.0001) {
        return textureSample(screen_texture, texture_sampler, in.uv);
    }

    let clamped_offset = clamp(total_offset, vec2<f32>(-0.08, -0.08), vec2<f32>(0.08, 0.08));

    let uv_r = clamp(in.uv + clamped_offset * 1.04, vec2<f32>(0.001, 0.001), vec2<f32>(0.999, 0.999));
    let uv_g = clamp(in.uv + clamped_offset, vec2<f32>(0.001, 0.001), vec2<f32>(0.999, 0.999));
    let uv_b = clamp(in.uv + clamped_offset * 0.96, vec2<f32>(0.001, 0.001), vec2<f32>(0.999, 0.999));

    let col_r = textureSample(screen_texture, texture_sampler, uv_r).r;
    let col_g = textureSample(screen_texture, texture_sampler, uv_g).g;
    let col_b = textureSample(screen_texture, texture_sampler, uv_b).b;
    let base_a = textureSample(screen_texture, texture_sampler, uv_g).a;

    let heat_tint = vec3<f32>(0.12, 0.04, 0.01) * min(max_heat_factor * 25.0, 0.18);
    let final_rgb = vec3<f32>(col_r, col_g, col_b) + heat_tint;

    return vec4<f32>(final_rgb, base_a);
}
