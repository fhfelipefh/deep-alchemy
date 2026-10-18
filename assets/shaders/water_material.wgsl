#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

@group(2) @binding(0) var<uniform> material_base: vec4<f32>;
@group(2) @binding(1) var<uniform> material_deep: vec4<f32>;
@group(2) @binding(2) var<uniform> material_foam: vec4<f32>;
@group(2) @binding(3) var<uniform> material_params: vec4<f32>;

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
        // =========================================================================
        // 1. VOLUMETRIC CEL-SHADED WATERFALL CURTAIN
        // =========================================================================
        // Vertically scrolling UVs (scrolling downward along fall)
        let flow_uv1 = vec2<f32>(in.uv.x * 8.0, in.uv.y * 14.0 - t * 3.6);
        let flow_uv2 = vec2<f32>(in.uv.x * 12.0 + sin(t * 1.2) * 0.35, in.uv.y * 22.0 - t * 5.4);

        // Dual-scale Voronoi Noise
        let v1 = 1.0 - voronoi(flow_uv1);
        let v2 = 1.0 - voronoi(flow_uv2);
        let foam_noise = max(v1, v2);

        // Longitudinal fluted stream ribbons
        let stream_wave = sin(in.uv.x * 24.0 + sin(in.uv.y * 8.0 - t * 3.0) * 1.5);
        let stream_ribbon = smoothstep(0.35, 0.85, stream_wave);

        // Strict Toon Water Ramp: Solid White Foam (#ffffff) over Deep Cyan (#08486b to #28e0ff)
        // Strict step() eliminates smooth realistic gradients!
        let foam_threshold = 0.60;
        let combined_foam_metric = foam_noise * 0.65 + stream_ribbon * 0.35;
        let main_foam_band = step(foam_threshold, combined_foam_metric);

        // Lateral edge churning foam (at the sides where water hits rock flanks)
        let flank_dist = abs(in.uv.x - 0.5) * 2.0;
        let flank_foam = step(0.75, flank_dist) * step(0.48, foam_noise);

        // Bottom plunge churning foam (localized strictly to the point of impact with the lake)
        let plunge_zone = smoothstep(0.91, 0.99, in.uv.y);
        let plunge_foam = plunge_zone * step(0.42, foam_noise);

        let total_foam = clamp(main_foam_band + flank_foam + plunge_foam, 0.0, 1.0);

        // Cel-Shaded Colors
        let deep_cyan = vec3<f32>(0.02, 0.22, 0.36);   // #05385c
        let bright_cyan = vec3<f32>(0.15, 0.86, 0.98); // #26dcfb
        let pure_white = vec3<f32>(1.0, 1.0, 1.0);     // #ffffff

        // Stylized toon ramp for water body: 2 crisp steps
        let water_tone = step(0.42, stream_ribbon);
        let water_base = mix(deep_cyan, bright_cyan, water_tone);

        // Composite solid foam bands
        let col_rgb = mix(water_base, pure_white, total_foam);

        // HDR Emission > 2.0 to trigger camera Bloom (calibrated to preserve contrast)
        let foam_emission = pure_white * (bloom_boost * 0.72);
        let water_emission = bright_cyan * (bloom_boost * 0.30);
        let hdr_emission = mix(water_emission, foam_emission, total_foam);

        // Smooth dissolve as the waterfall plunges below the lake surface (Y = 0.18)
        let entry_alpha = smoothstep(0.04, 0.32, in.world_position.y);
        let final_alpha = clamp(0.98 * entry_alpha, 0.0, 1.0);

        return vec4<f32>(col_rgb + hdr_emission, final_alpha);
    } else {
        // =========================================================================
        // 2. CEL-SHADED LAKE POOL WITH CONCENTRIC RIPPLES & SHORELINE FOAM
        // =========================================================================
        // Lake center in world space is (0.0, -8.5)
        let dist_to_impact = length(in.world_position.xz - vec2<f32>(0.0, -8.5));

        // 1. Concentric ripples radiating outward from waterfall plunge
        let ripple_phase = dist_to_impact * 3.8 - t * 3.5;
        let ripple_sine = sin(ripple_phase);
        let ripple_crest = step(0.78, ripple_sine) * smoothstep(5.6, 0.8, dist_to_impact);

        // 2. Shoreline foam and depth fade (translucent emerald at shore, deep turquoise at center)
        let uv_shore = in.uv * vec2<f32>(14.0, 14.0) + vec2<f32>(cos(t * 0.8) * 0.4, sin(t * 0.8) * 0.4);
        let shore_noise = 1.0 - voronoi(uv_shore);
        let shore_zone = smoothstep(4.7, 5.7, dist_to_impact);
        let shoreline_foam = step(0.52, shore_noise) * shore_zone;

        // 3. Boiling plunge pool foam directly under waterfall
        let uv_plunge = in.uv * vec2<f32>(16.0, 20.0) + vec2<f32>(0.0, -t * 2.2);
        let plunge_noise = 1.0 - voronoi(uv_plunge);
        let plunge_zone = smoothstep(2.2, 0.4, dist_to_impact);
        let plunge_pool_foam = step(0.46, plunge_noise) * plunge_zone;

        let lake_foam = clamp(ripple_crest * 0.55 + shoreline_foam + plunge_pool_foam, 0.0, 1.0);

        // Cel-Shaded Depth Color Ramp:
        // Center: Deep Turquoise (#022232)
        // Mid: Radiant Cyan (#18d2ea)
        // Shore: Translucent Emerald (#18c096)
        let depth_metric = smoothstep(0.4, 5.5, dist_to_impact);
        let deep_color = vec3<f32>(0.02, 0.22, 0.32);
        let mid_color = vec3<f32>(0.10, 0.82, 0.92);
        let shore_emerald = vec3<f32>(0.09, 0.75, 0.58);

        // Crisp toon stepping between deep, cyan, and emerald
        var lake_water = deep_color;
        if (depth_metric > 0.65) {
            lake_water = shore_emerald;
        } else if (depth_metric > 0.25) {
            lake_water = mid_color;
        }

        let lake_white_foam = vec3<f32>(1.0, 1.0, 1.0);
        let col_rgb = mix(lake_water, lake_white_foam, lake_foam);

        // HDR Bloom Emission on foam and ripple crests
        let foam_emission = vec3<f32>(1.0, 1.1, 1.15) * (lake_foam * bloom_boost);
        let ambient_glow = mid_color * (bloom_boost * 0.25);

        // Translucent at shore, opaque in center, fully opaque foam
        let alpha_base = mix(0.96, 0.78, depth_metric);
        let final_alpha = clamp(alpha_base + lake_foam * 0.3, 0.0, 1.0);

        return vec4<f32>(col_rgb + foam_emission + ambient_glow, final_alpha);
    }
}
