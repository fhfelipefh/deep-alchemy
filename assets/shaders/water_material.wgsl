#import bevy_pbr::mesh_vertex_output::MeshVertexOutput
#import bevy_pbr::mesh_view_bindings::view

@group(2) @binding(0) var<uniform> material_base: vec4<f32>;
@group(2) @binding(1) var<uniform> material_deep: vec4<f32>;
@group(2) @binding(2) var<uniform> material_foam: vec4<f32>;
@group(2) @binding(3) var<uniform> material_params: vec4<f32>;

@fragment
fn fragment(
    in: MeshVertexOutput,
    @builtin(position) frag_coord: vec4<f32>,
) -> @location(0) vec4<f32> {
    let time = material_params.x;
    let speed = material_params.y;
    let t = time * speed;
    
    // Stylized dual-wave interference pattern
    let uv = in.uv * 16.0;
    let wave1 = sin(uv.x * 1.5 + t * 2.2 + uv.y * 1.2) * 0.5 + 0.5;
    let wave2 = cos(uv.x * 2.1 - t * 1.8 + uv.y * 1.7) * 0.5 + 0.5;
    let wave_combined = (wave1 + wave2) * 0.5;
    
    // Blend deep turquoise into sparkling surface
    var water_col = mix(material_deep, material_base, 0.4 + wave_combined * 0.6);
    
    // Edge foam / wave crest foam
    let crest = smoothstep(0.65, 0.85, wave_combined);
    water_col = mix(water_col, material_foam, crest * 0.7);
    
    // Fresnel reflection
    let view_dir = normalize(view.world_position - in.world_position.xyz);
    let normal_pert = vec3<f32>((wave1 - 0.5) * 0.15, 0.0, (wave2 - 0.5) * 0.15);
    let n = normalize(in.world_normal + normal_pert);
    let fresnel = pow(1.0 - max(dot(view_dir, n), 0.0), 3.0);
    
    water_col = mix(water_col, vec4<f32>(0.9, 0.98, 1.0, 1.0), fresnel * 0.55);
    water_col.a = clamp(material_base.a + fresnel * 0.25, 0.0, 1.0);
    
    return water_col;
}
