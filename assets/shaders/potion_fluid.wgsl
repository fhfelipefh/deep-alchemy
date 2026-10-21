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

    let ripple1 = sin(v_world.x * 220.0 + time * 7.5) * cos(v_world.z * 220.0 + time * 6.5) * wave_amp;
    let ripple2 = sin((v_world.x + v_world.z) * 320.0 - time * 9.0) * (wave_amp * 0.4);
    let total_ripple = ripple1 + ripple2;

    let plane_dist = dot(v_world, normal) - (fill_level + total_ripple);

    if (plane_dist > 0.0) {
        let vapor_fade = clamp(1.0 - plane_dist * 50.0, 0.0, 1.0);
        let vapor_alpha = clamp(0.04 * vapor_fade, 0.0, 0.08);
        return vec4<f32>(base_color.rgb * 0.35, vapor_alpha);
    }

    let depth = clamp(-plane_dist * 35.0, 0.0, 1.0);
    let meniscus_dist = abs(plane_dist);
    let meniscus = smoothstep(meniscus_w, 0.0, meniscus_dist);

    let core_color = mix(surface_color.rgb * 0.9, base_color.rgb, depth);

    let V = normalize(view.world_position.xyz - in.world_position.xyz);
    let N = normalize(in.world_normal);
    let NdotV = max(abs(dot(N, V)), 0.0);
    let fresnel = pow(1.0 - NdotV, 2.5);

    let emissive = core_color * 1.5 + surface_color.rgb * (meniscus * 2.6 + fresnel * 0.7);
    let alpha = clamp(base_color.a * 0.75 + depth * 0.22 + meniscus * 0.25 + fresnel * 0.12, 0.35, 0.98);

    return vec4<f32>(emissive, alpha);
}
