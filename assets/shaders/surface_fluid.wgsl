#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

@group(2) @binding(0) var<uniform> fluid_color: vec4<f32>;
@group(2) @binding(1) var<uniform> flow_data: vec4<f32>;
@group(2) @binding(2) var<uniform> film_data: vec4<f32>;

fn hash2(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let speed = length(flow_data.xy);
    let flow = flow_data.xy * (film_data.w * (1.2 + speed * 3.0));
    let grain = hash2(floor((in.world_position.xz + flow) * 42.0));
    let rivulet = smoothstep(0.48, 0.82, grain + film_data.x * 0.75);
    let normal = normalize(in.world_normal);
    let view_dir = normalize(view.world_position.xyz - in.world_position.xyz);
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    let glow = fluid_color.rgb * (0.16 + film_data.z * 0.62 + fresnel * 0.30);
    let alpha = clamp(fluid_color.a * (0.34 + film_data.x * 0.75 + rivulet * 0.18), 0.08, 0.94);
    return vec4<f32>(mix(fluid_color.rgb * 0.58, glow, 0.65 + rivulet * 0.35), alpha);
}
