#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

@group(2) @binding(0) var<uniform> base_color: vec4<f32>;
@group(2) @binding(1) var<uniform> surface_color: vec4<f32>;
@group(2) @binding(2) var<uniform> slosh_normal: vec4<f32>;
@group(2) @binding(3) var<uniform> fluid_params: vec4<f32>;
@group(2) @binding(4) var<uniform> bulb_center: vec4<f32>;

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let time = fluid_params.x;
    let wave_amp = fluid_params.y;
    let meniscus_w = fluid_params.z;

    let center = bulb_center.xyz;
    let sphere_radius = bulb_center.w;

    let v_world = in.world_position.xyz - center;

    let normal = normalize(slosh_normal.xyz);
    let fill_level = slosh_normal.w;

    let ripple1 = sin(v_world.x * 52.0 + time * 4.5) * cos(v_world.z * 52.0 + time * 3.8) * (wave_amp * 0.65);
    let ripple2 = sin((v_world.x + v_world.z) * 78.0 - time * 5.2) * (wave_amp * 0.35);
    let total_ripple = ripple1 + ripple2;

    let plane_dist = dot(v_world, normal) - (fill_level + total_ripple);

    let fill_factor = smoothstep(0.002, -0.002, plane_dist);

    let depth = clamp(-plane_dist * 28.0, 0.0, 1.0);
    let meniscus_dist = abs(plane_dist);
    let meniscus = smoothstep(meniscus_w, 0.0, meniscus_dist);

    let core_color = mix(surface_color.rgb * 0.88, base_color.rgb, depth);

    let V = normalize(view.world_position.xyz - in.world_position.xyz);
    let N = normalize(in.world_normal);
    let NdotV = max(abs(dot(N, V)), 0.0);
    let fresnel = pow(1.0 - NdotV, 2.2);

    let liquid_emissive = core_color * 0.82 + surface_color.rgb * (meniscus * 1.15 + fresnel * 0.35);

    let liquid_alpha = clamp(base_color.a * 0.44 + depth * 0.18 + meniscus * 0.20 + fresnel * 0.08, 0.20, 0.72);

    let vapor_dist = max(plane_dist, 0.0);
    let vapor_fade = clamp(1.0 - vapor_dist * 35.0, 0.0, 1.0);
    let vapor_color = base_color.rgb * 0.16 + surface_color.rgb * (fresnel * 0.22);
    let vapor_alpha = clamp(0.025 * vapor_fade + fresnel * 0.04, 0.0, 0.07);

    let final_rgb = mix(vapor_color, liquid_emissive, fill_factor);
    let final_alpha = mix(vapor_alpha, liquid_alpha, fill_factor);

    return vec4<f32>(final_rgb, final_alpha);
}
