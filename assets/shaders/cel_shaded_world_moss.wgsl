#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

// Material Bindings for Cel-Shaded World-Space Normal Moss Shader
@group(2) @binding(0) var<uniform> base_color: vec4<f32>;    // Base Rock/Wood color (#26333a or #362215)
@group(2) @binding(1) var<uniform> moss_color: vec4<f32>;    // Velvet Moss color (#246e34)
@group(2) @binding(2) var<uniform> light_dir: vec4<f32>;     // Directional key light vector
@group(2) @binding(3) var<uniform> moss_params: vec4<f32>;   // x: ny_threshold (0.6), y: noise_scale, z: noise_strength, w: toon_steps (3.0)

// 3D Hash and Value Noise for Organic Fringes
fn hash3(p: vec3<f32>) -> vec3<f32> {
    let q = vec3<f32>(
        dot(p, vec3<f32>(127.1, 311.7, 74.7)),
        dot(p, vec3<f32>(269.5, 183.3, 246.1)),
        dot(p, vec3<f32>(113.5, 271.9, 124.6))
    );
    return fract(sin(q) * 43758.5453123);
}

fn noise3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let n000 = dot(hash3(i + vec3<f32>(0.0, 0.0, 0.0)) - vec3<f32>(0.5), f - vec3<f32>(0.0, 0.0, 0.0));
    let n100 = dot(hash3(i + vec3<f32>(1.0, 0.0, 0.0)) - vec3<f32>(0.5), f - vec3<f32>(1.0, 0.0, 0.0));
    let n010 = dot(hash3(i + vec3<f32>(0.0, 1.0, 0.0)) - vec3<f32>(0.5), f - vec3<f32>(0.0, 1.0, 0.0));
    let n110 = dot(hash3(i + vec3<f32>(1.0, 1.0, 0.0)) - vec3<f32>(0.5), f - vec3<f32>(1.0, 1.0, 0.0));
    let n001 = dot(hash3(i + vec3<f32>(0.0, 0.0, 1.0)) - vec3<f32>(0.5), f - vec3<f32>(0.0, 0.0, 1.0));
    let n101 = dot(hash3(i + vec3<f32>(1.0, 0.0, 1.0)) - vec3<f32>(0.5), f - vec3<f32>(1.0, 0.0, 1.0));
    let n011 = dot(hash3(i + vec3<f32>(0.0, 1.0, 1.0)) - vec3<f32>(0.5), f - vec3<f32>(0.0, 1.0, 1.0));
    let n111 = dot(hash3(i + vec3<f32>(1.0, 1.0, 1.0)) - vec3<f32>(0.5), f - vec3<f32>(1.0, 1.0, 1.0));

    let nx00 = mix(n000, n100, u.x);
    let nx10 = mix(n010, n110, u.x);
    let nx01 = mix(n001, n101, u.x);
    let nx11 = mix(n011, n111, u.x);

    let nxy0 = mix(nx00, nx10, u.y);
    let nxy1 = mix(nx01, nx11, u.y);

    return mix(nxy0, nxy1, u.z) * 0.5 + 0.5;
}

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let ny_threshold = moss_params.x; // default 0.60
    let noise_scale = moss_params.y;   // default 3.50
    let noise_strength = moss_params.z; // default 0.22
    let toon_bands = moss_params.w;    // default 3.0

    let normal = normalize(in.world_normal);
    let world_pos = in.world_position.xyz;

    // Organic 3D noise perturbation of the vertical normal component
    let n_val = noise3d(world_pos * noise_scale);
    let perturbed_ny = normal.y + (n_val - 0.5) * noise_strength;

    // World-Space Normal top moss blending (Ny > 0.6)
    // Upward-facing horizontal surfaces blend smoothly into lush velvet moss.
    // Vertical and downward cliff faces maintain dark stone or rustic wood.
    let moss_factor = smoothstep(ny_threshold - 0.08, ny_threshold + 0.08, perturbed_ny);

    // Combine diffuse albedo
    let surface_albedo = mix(base_color.rgb, moss_color.rgb, moss_factor);

    // Directional Key Light vector for Toon Shading
    var l_dir = normalize(light_dir.xyz);
    if (dot(l_dir, l_dir) < 0.1) {
        l_dir = normalize(vec3<f32>(0.25, 0.85, -0.45));
    }

    let n_dot_l = max(dot(normal, l_dir), 0.0);

    // Stepped Toon Shading ramp (2-3 crisp bands)
    let ramp_step = floor(n_dot_l * toon_bands + 0.05) / toon_bands;
    let ambient_term = 0.38;
    let light_intensity = mix(ambient_term, 1.0, clamp(ramp_step, 0.0, 1.0));

    let final_color = surface_albedo * light_intensity;
    return vec4<f32>(final_color, base_color.a);
}
