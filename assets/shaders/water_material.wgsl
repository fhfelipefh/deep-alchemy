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
        // 1. REALISTIC FLUID WATERFALL WITH NATURAL AERATION & HYDRODYNAMICS
        // =========================================================================
        
        // Gravitational acceleration: velocity increases with vertical descent (v^2 = 2gy)
        // This stretches the water stream vertically into natural hydrodynamic filaments.
        let y_accel = in.uv.y * 1.1 + in.uv.y * in.uv.y * 1.6;
        
        // Multi-octave cascading flow coordinates (elongated vertically by gravitational descent)
        let uv_sheet = vec2<f32>(in.uv.x * 4.0, y_accel * 32.0 - t * 5.2);
        let uv_filaments = vec2<f32>(in.uv.x * 8.0 + sin(y_accel * 6.0 - t * 2.2) * 0.15, y_accel * 56.0 - t * 9.6);
        let uv_spray = vec2<f32>(in.uv.x * 16.0, y_accel * 72.0 - t * 14.0);

        // Hydrodynamic Voronoi noise for turbulent water fracturing
        let v_macro = 1.0 - voronoi(uv_sheet);
        let v_micro = 1.0 - voronoi(uv_filaments);
        let spray_noise = voronoi(uv_spray);

        // Longitudinal water filaments & gravity-stretched cascading tendrils
        // Water naturally shears into fine vertical ribbons rather than solid walls.
        let rib1 = sin(in.uv.x * 18.0 + sin(y_accel * 5.0 - t * 3.2) * 1.4);
        let rib2 = cos(in.uv.x * 30.0 + sin(y_accel * 8.0 - t * 4.6) * 1.0);
        let filament_metric = clamp((rib1 * 0.5 + 0.5) * 0.58 + (rib2 * 0.5 + 0.5) * 0.42, 0.0, 1.0);
        
        // Aeration intensity increases as water falls and mixes with air
        let aeration_depth = smoothstep(0.06, 0.94, in.uv.y);
        
        // Delicate, soft aeration tendrils (elongated sleek ribbons)
        let aerated_foam = smoothstep(0.58, 0.88, (v_micro * 0.45 + filament_metric * 0.55) * (0.60 + aeration_depth * 0.48));
        
        // Flank shearing: natural spray where water boundaries meet air
        let flank_dist = abs(in.uv.x - 0.5) * 2.0;
        let flank_spray = smoothstep(0.70, 0.98, flank_dist) * smoothstep(0.42, 0.80, v_macro);

        // Plunge impact zone: turbulent, aerated frothy churning at the base
        let plunge_dist = smoothstep(0.85, 0.98, in.uv.y);
        let plunge_froth = plunge_dist * smoothstep(0.36, 0.74, v_micro * 0.7 + spray_noise * 0.3);

        // Combined organic aeration & foam factor
        let total_froth = clamp(aerated_foam * 0.58 + flank_spray * 0.42 + plunge_froth * 0.70, 0.0, 1.0);

        // Realistic water colors: Deep Crystalline Azure & Radiant Turquoise Cyan
        let deep_water = vec3<f32>(0.03, 0.26, 0.42);       // Rich aquatic deep azure
        let shallow_cyan = vec3<f32>(0.12, 0.78, 0.90);     // Vibrant crystalline turquoise
        let aerated_white = vec3<f32>(0.92, 0.97, 1.0);     // Soft cool aerated froth

        // Smooth fluid gradient across the falling sheet
        let fluid_body = mix(deep_water, shallow_cyan, smoothstep(0.22, 0.78, filament_metric * 0.65 + v_macro * 0.35));
        
        // Composite froth softly into the water body
        let surface_color = mix(fluid_body, aerated_white, total_froth * 0.82);

        // Natural Fresnel Specular Glisten & Shimmer
        let V = normalize(view.world_position.xyz - in.world_position.xyz);
        let N = normalize(in.world_normal);
        let NdotV = max(dot(N, V), 0.0);
        let fresnel = pow(1.0 - NdotV, 3.0); // Glancing specular glisten
        
        // Subtle water glisten highlight on high-speed filaments
        let glisten = pow(filament_metric * (1.0 - spray_noise * 0.5), 3.0) * 0.40;
        let specular_highlight = vec3<f32>(0.85, 0.96, 1.0) * (fresnel * 0.38 + glisten);

        // Gentle, calibrated Bloom emission on froth crests (subtle sheen, NO nuclear blowout!)
        let ambient_froth_glow = aerated_white * (total_froth * bloom_boost * 0.28);

        // Translucency & Realistic Alpha:
        // Real falling water is translucent (alpha ~ 0.55 to 0.78), showing the cavern wall behind it.
        // Froth and aeration increase opacity locally.
        let base_alpha = mix(0.55, 0.78, aeration_depth * 0.6 + fresnel * 0.4);
        let entry_dissolve = smoothstep(0.02, 0.25, in.world_position.y);
        let final_alpha = clamp((base_alpha + total_froth * 0.28) * entry_dissolve, 0.0, 1.0);

        return vec4<f32>(surface_color + specular_highlight + ambient_froth_glow, final_alpha);
    } else {
        // =========================================================================
        // 2. REALISTIC CALIBRATED LAKE POOL WITH ORGANIC RIPPLES & CLEAR WATER
        // =========================================================================
        // Lake center in world space is (0.0, -8.5)
        let dist_to_impact = length(in.world_position.xz - vec2<f32>(0.0, -8.5));

        // Concentric ripples radiating outward from waterfall plunge (smooth wave crests)
        let ripple_phase = dist_to_impact * 3.6 - t * 3.2;
        let ripple_sine = sin(ripple_phase);
        // Soft smoothstep ripple crests rather than hard binary steps
        let ripple_crest = smoothstep(0.40, 0.88, ripple_sine) * smoothstep(5.6, 0.6, dist_to_impact);

        // Caustic and surface turbulence
        let uv_surface = in.uv * vec2<f32>(12.0, 12.0) + vec2<f32>(cos(t * 0.7) * 0.3, sin(t * 0.7) * 0.3);
        let surface_caustic = 1.0 - voronoi(uv_surface);

        // Shoreline foam gently lapping against the travertine rim
        let shore_zone = smoothstep(4.8, 5.75, dist_to_impact);
        let shoreline_foam = smoothstep(0.45, 0.78, surface_caustic) * shore_zone;

        // Boiling plunge pool aeration directly under waterfall
        let uv_plunge = in.uv * vec2<f32>(14.0, 18.0) + vec2<f32>(0.0, -t * 2.0);
        let plunge_noise = 1.0 - voronoi(uv_plunge);
        let plunge_zone = smoothstep(2.4, 0.3, dist_to_impact);
        let plunge_froth = smoothstep(0.40, 0.75, plunge_noise) * plunge_zone;

        let lake_froth = clamp(ripple_crest * 0.35 + shoreline_foam * 0.65 + plunge_froth * 0.75, 0.0, 1.0);

        // Deep mineral spring colors:
        let lake_abyss = vec3<f32>(0.02, 0.16, 0.26);       // Deep abyss turquoise
        let lake_mid = vec3<f32>(0.08, 0.62, 0.74);         // Crystalline body cyan
        let lake_shore = vec3<f32>(0.10, 0.72, 0.60);       // Translucent emerald at shallows
        let froth_white = vec3<f32>(0.92, 0.98, 1.0);

        let depth_metric = smoothstep(0.5, 5.5, dist_to_impact);
        let lake_water = mix(lake_abyss, mix(lake_mid, lake_shore, smoothstep(0.4, 0.9, depth_metric)), depth_metric);

        // Composite froth into water
        let col_rgb = mix(lake_water, froth_white, lake_froth * 0.85);

        // Fresnel reflection on lake surface
        let V = normalize(view.world_position.xyz - in.world_position.xyz);
        let N = normalize(in.world_normal);
        let lake_fresnel = pow(1.0 - max(dot(N, V), 0.0), 3.0);
        let lake_specular = vec3<f32>(0.85, 0.96, 1.0) * (lake_fresnel * 0.35);

        // Soft calibrated glow on froth and ripple crests
        let lake_glow = froth_white * (lake_froth * bloom_boost * 0.22);

        // Translucent lake body (depth is readable through water)
        let lake_alpha = clamp(mix(0.85, 0.70, depth_metric) + lake_froth * 0.25, 0.0, 1.0);

        return vec4<f32>(col_rgb + lake_specular + lake_glow, lake_alpha);
    }
}
