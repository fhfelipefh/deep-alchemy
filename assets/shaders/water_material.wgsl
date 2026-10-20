#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

@group(2) @binding(0) var<uniform> material_base: vec4<f32>;
@group(2) @binding(1) var<uniform> material_deep: vec4<f32>;
@group(2) @binding(2) var<uniform> material_foam: vec4<f32>;
@group(2) @binding(3) var<uniform> material_params: vec4<f32>;
@group(2) @binding(4) var reflection_texture: texture_2d<f32>;
@group(2) @binding(5) var reflection_sampler: sampler;
@group(2) @binding(6) var<uniform> reflection_params: vec4<f32>;

fn hash2(p: vec2<f32>) -> vec2<f32> {
    let p2 = vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)), dot(p, vec2<f32>(269.5, 183.3)));
    return fract(sin(p2) * 43758.5453);
}

fn voronoi(p: vec2<f32>) -> f32 {
    let n = floor(p);
    let f = fract(p);
    var md = 8.0;
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let g = vec2<f32>(f32(i), f32(j));
            let o = hash2(n + g);
            let r = g + o - f;
            let d = dot(r, r);
            if (d < md) {
                md = d;
            }
        }
    }
    return sqrt(md);
}

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let time = material_params.x;
    let speed = material_params.y;
    let is_waterfall = material_params.z > 0.5;
    let bloom_boost = material_params.w;
    let t = time * speed;

    if (is_waterfall) {
        let y_accel = in.uv.y * 1.1 + in.uv.y * in.uv.y * 1.6;

        let uv_sheet = vec2<f32>(in.uv.x * 4.0, y_accel * 32.0 - t * 5.2);
        let uv_filaments = vec2<f32>(in.uv.x * 8.0 + sin(y_accel * 6.0 - t * 2.2) * 0.15, y_accel * 56.0 - t * 9.6);
        let uv_spray = vec2<f32>(in.uv.x * 16.0, y_accel * 72.0 - t * 14.0);

        let v_macro = 1.0 - voronoi(uv_sheet);
        let v_micro = 1.0 - voronoi(uv_filaments);
        let spray_noise = voronoi(uv_spray);

        let rib1 = sin(in.uv.x * 18.0 + sin(y_accel * 5.0 - t * 3.2) * 1.4);
        let rib2 = cos(in.uv.x * 30.0 + sin(y_accel * 8.0 - t * 4.6) * 1.0);
        let filament_metric = clamp((rib1 * 0.5 + 0.5) * 0.58 + (rib2 * 0.5 + 0.5) * 0.42, 0.0, 1.0);

        let aeration_depth = smoothstep(0.06, 0.94, in.uv.y);
        let aerated_foam = smoothstep(0.58, 0.88, (v_micro * 0.45 + filament_metric * 0.55) * (0.60 + aeration_depth * 0.48));

        let flank_dist = abs(in.uv.x - 0.5) * 2.0;
        let flank_spray = smoothstep(0.70, 0.98, flank_dist) * smoothstep(0.42, 0.80, v_macro);

        let plunge_dist = smoothstep(0.85, 0.98, in.uv.y);
        let plunge_froth = plunge_dist * smoothstep(0.36, 0.74, v_micro * 0.7 + spray_noise * 0.3);

        let total_froth = clamp(aerated_foam * 0.58 + flank_spray * 0.42 + plunge_froth * 0.70, 0.0, 1.0);

        let deep_water = vec3<f32>(0.03, 0.26, 0.42);
        let shallow_cyan = vec3<f32>(0.12, 0.78, 0.90);
        let aerated_white = vec3<f32>(0.92, 0.97, 1.0);

        let fluid_body = mix(deep_water, shallow_cyan, smoothstep(0.22, 0.78, filament_metric * 0.65 + v_macro * 0.35));
        let surface_color = mix(fluid_body, aerated_white, total_froth * 0.65);

        let V = normalize(view.world_position.xyz - in.world_position.xyz);
        let N = normalize(in.world_normal);
        let NdotV = max(dot(N, V), 0.0);
        let fresnel = pow(1.0 - NdotV, 3.0);

        let glisten = pow(filament_metric * (1.0 - spray_noise * 0.5), 3.0) * 0.28;
        let specular_highlight = vec3<f32>(0.85, 0.96, 1.0) * (fresnel * 0.25 + glisten);

        let ambient_froth_glow = aerated_white * (total_froth * bloom_boost * 0.12);

        let base_alpha = mix(0.18, 0.32, aeration_depth * 0.4 + fresnel * 0.20);
        let entry_dissolve = smoothstep(0.02, 0.25, in.world_position.y);
        let final_alpha = clamp((base_alpha + total_froth * 0.25) * entry_dissolve, 0.08, 0.58);

        return vec4<f32>(surface_color + specular_highlight + ambient_froth_glow, final_alpha);
    } else {
        let dist_to_impact = length(in.world_position.xz - vec2<f32>(0.0, -8.5));
        let lake_max_radius = 5.85;
        let depth_factor = clamp(1.0 - (dist_to_impact / lake_max_radius), 0.0, 1.0);
        let depth_curve = pow(depth_factor, 1.65);

        let shallow_transparent_cyan = material_base.rgb;
        let deep_abyssal_navy = material_deep.rgb;

        let lake_water = mix(shallow_transparent_cyan, deep_abyssal_navy, depth_curve);
        let clear_water_alpha = mix(material_base.a, material_deep.a, depth_curve);

        let wave_p1 = (in.world_position.x * 0.707 + in.world_position.z * 0.707) * 1.8 - t * 2.0;
        let wave_p2 = (in.world_position.x * -0.45 + in.world_position.z * 0.89) * 3.2 - t * 2.8;

        let g_dx = 0.707 * 1.8 * 0.028 * cos(wave_p1) - 0.45 * 3.2 * 0.016 * cos(wave_p2);
        let g_dz = 0.707 * 1.8 * 0.028 * cos(wave_p1) + 0.89 * 3.2 * 0.016 * cos(wave_p2);

        let ripple_speed = 2.8;
        let ripple_freq_1 = 4.2;
        let ripple_freq_2 = 7.0;

        let ripple_p1 = dist_to_impact * ripple_freq_1 - t * ripple_speed;
        let ripple_p2 = dist_to_impact * ripple_freq_2 - t * (ripple_speed * 1.35);

        let ripple_sin1 = sin(ripple_p1);
        let ripple_sin2 = sin(ripple_p2);

        let ripple_envelope = smoothstep(0.35, 1.3, dist_to_impact) * smoothstep(5.6, 2.0, dist_to_impact);
        let ripple_crests = (smoothstep(0.70, 0.96, ripple_sin1) * 0.70 + smoothstep(0.78, 0.98, ripple_sin2) * 0.30) * ripple_envelope;

        let r_dx = (in.world_position.x / max(dist_to_impact, 0.01)) * ripple_freq_1 * 0.025 * cos(ripple_p1) * ripple_envelope;
        let r_dz = ((in.world_position.z + 8.5) / max(dist_to_impact, 0.01)) * ripple_freq_1 * 0.025 * cos(ripple_p1) * ripple_envelope;

        let wave_normal = normalize(vec3<f32>(-(g_dx + r_dx), 1.0, -(g_dz + r_dz)));

        let screen_uv = in.position.xy / view.viewport.zw;
        let distortion_strength = reflection_params.y;
        let refl_offset = vec2<f32>(wave_normal.x, wave_normal.z) * distortion_strength;
        let refl_uv = clamp(screen_uv + refl_offset, vec2<f32>(0.002, 0.002), vec2<f32>(0.998, 0.998));
        let refl_sample = textureSample(reflection_texture, reflection_sampler, refl_uv);

        let V = normalize(view.world_position.xyz - in.world_position.xyz);
        let NdotV = max(dot(wave_normal, V), 0.0);
        let lake_fresnel = pow(1.0 - NdotV, 2.8);

        let refl_blend_factor = reflection_params.w;
        let cel_refl = smoothstep(0.12, 0.65, lake_fresnel) * refl_blend_factor;
        let reflected_lake = mix(lake_water, refl_sample.rgb, clamp(cel_refl + 0.15, 0.0, 0.88));

        let uv_macro = in.uv * vec2<f32>(22.0, 22.0) + vec2<f32>(cos(t * 0.5) * 0.2, sin(t * 0.5) * 0.2);
        let uv_micro = in.uv * vec2<f32>(44.0, 44.0) + vec2<f32>(0.0, -t * 1.1);
        let v_edge1 = 1.0 - voronoi(uv_macro);
        let v_edge2 = 1.0 - voronoi(uv_micro);
        let froth_noise = v_edge1 * 0.65 + v_edge2 * 0.35;

        let shore_zone = smoothstep(4.65, 5.82, dist_to_impact);
        let edge_foam_threshold = reflection_params.z;
        let shoreline_foam = smoothstep(0.36, 0.72, froth_noise * (0.8 + edge_foam_threshold * 0.2)) * shore_zone;

        let uv_plunge = in.uv * vec2<f32>(14.0, 18.0) + vec2<f32>(0.0, -t * 2.2);
        let plunge_noise = 1.0 - voronoi(uv_plunge);
        let plunge_zone = smoothstep(2.2, 0.25, dist_to_impact);
        let plunge_froth = smoothstep(0.35, 0.75, plunge_noise) * plunge_zone;

        let total_foam = clamp(shoreline_foam * 1.25 + ripple_crests * 0.62 + plunge_froth * 0.85, 0.0, 1.0);

        let foam_cel_tint = mix(material_foam.rgb, vec3<f32>(0.84, 0.98, 1.0), 0.45);
        let col_rgb = mix(reflected_lake, foam_cel_tint, total_foam * 0.94);

        let lake_specular = vec3<f32>(0.88, 0.97, 1.0) * (pow(lake_fresnel, 2.0) * 0.32 + pow(ripple_crests, 2.0) * 0.18);
        let lake_glow = material_foam.rgb * (total_foam * bloom_boost * 0.16);
        let lake_alpha = clamp(clear_water_alpha + total_foam * 0.42 + lake_fresnel * 0.22, 0.10, 0.98);

        return vec4<f32>(col_rgb + lake_specular + lake_glow, lake_alpha);
    }
}
