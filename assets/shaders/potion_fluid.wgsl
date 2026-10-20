#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

@group(2) @binding(0) var<uniform> base_color: vec4<f32>;
@group(2) @binding(1) var<uniform> surface_color: vec4<f32>;
@group(2) @binding(2) var<uniform> slosh_normal: vec4<f32>;
@group(2) @binding(3) var<uniform> fluid_params: vec4<f32>;

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let time = fluid_params.x;
    let wave_amp = fluid_params.y;
    let meniscus_w = fluid_params.z;

    let phi = (in.uv.y - 0.5) * 3.14159265;
    let theta = in.uv.x * 6.2831853;
    let local_y = -sin(phi);
    let local_x = cos(phi) * sin(theta);
    let local_z = cos(phi) * cos(theta);
    let local_norm = vec3<f32>(local_x, local_y, local_z);

    let normal = normalize(slosh_normal.xyz);
    let fill_level = slosh_normal.w;

    let ripple1 = sin(local_x * 12.0 + time * 6.0) * cos(local_z * 12.0 + time * 5.0) * wave_amp;
    let ripple2 = sin((local_x + local_z) * 18.0 - time * 8.0) * (wave_amp * 0.4);
    let total_ripple = ripple1 + ripple2;

    let plane_dist = dot(local_norm, normal) - (fill_level + total_ripple);

    if (plane_dist > 0.0) {
        let vapor_fade = clamp(1.0 - plane_dist * 5.0, 0.0, 1.0);
        let vapor_alpha = clamp(0.06 * vapor_fade, 0.0, 0.12);
        return vec4<f32>(base_color.rgb * 0.45, vapor_alpha);
    }

    let depth = clamp(-plane_dist * 2.2, 0.0, 1.0);
    let meniscus_dist = abs(plane_dist);
    let meniscus = smoothstep(meniscus_w, 0.0, meniscus_dist);

    let core_color = mix(surface_color.rgb * 0.85, base_color.rgb, depth);

    let V = normalize(view.world_position.xyz - in.world_position.xyz);
    let N = normalize(in.world_normal);
    let NdotV = max(dot(N, V), 0.0);
    let fresnel = pow(1.0 - NdotV, 3.0);

    let emissive = core_color * 1.5 + surface_color.rgb * (meniscus * 2.8 + fresnel * 0.9);
    let alpha = clamp(base_color.a * 0.85 + depth * 0.25 + meniscus * 0.35 + fresnel * 0.15, 0.2, 0.98);

    return vec4<f32>(emissive, alpha);
}
