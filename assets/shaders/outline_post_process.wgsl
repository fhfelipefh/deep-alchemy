#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct OutlineSettings {
    outline_threshold: f32,
    outline_thickness: f32,
    normal_threshold: f32,
    depth_threshold: f32,
};

@group(0) @binding(2) var<uniform> settings: OutlineSettings;

fn rgb2luma(col: vec3<f32>) -> f32 {
    return dot(col, vec3<f32>(0.299, 0.587, 0.114));
}

fn is_water_or_foam(col: vec3<f32>) -> bool {

    if (col.r > 0.92 || col.g > 0.92 || col.b > 0.92) {
        return true;
    }

    if (col.r > 0.62 && col.g > 0.70 && col.b > 0.72) {
        return true;
    }

    if (col.b > 0.16 && (col.b * 1.15) > col.r) {
        return true;
    }

    if (col.g > 0.25 && col.b > 0.20 && (col.g + col.b) > (col.r * 1.35)) {
        return true;
    }
    return false;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(screen_texture));
    let texel = (vec2<f32>(1.0, 1.0) / dims) * settings.outline_thickness;

    let center_col = textureSample(screen_texture, texture_sampler, in.uv);

    if (is_water_or_foam(center_col.rgb)) {
        return center_col;
    }

    let c00 = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(-texel.x, -texel.y)).rgb);
    let c10 = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>( 0.0,     -texel.y)).rgb);
    let c20 = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>( texel.x, -texel.y)).rgb);

    let c01 = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(-texel.x,  0.0)).rgb);
    let c21 = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>( texel.x,  0.0)).rgb);

    let c02 = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(-texel.x,  texel.y)).rgb);
    let c12 = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>( 0.0,      texel.y)).rgb);
    let c22 = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>( texel.x,  texel.y)).rgb);

    let Sobel_gradient_x = (c20 + 2.0 * c21 + c22) - (c00 + 2.0 * c01 + c02);
    let Sobel_gradient_y = (c02 + 2.0 * c12 + c22) - (c00 + 2.0 * c10 + c20);
    let Sobel_edge_mag = sqrt(Sobel_gradient_x * Sobel_gradient_x + Sobel_gradient_y * Sobel_gradient_y);

    let d2x = texel * 1.85;
    let o_r = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>( d2x.x, 0.0)).rgb);
    let o_l = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(-d2x.x, 0.0)).rgb);
    let o_u = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(0.0,  d2x.y)).rgb);
    let o_d = rgb2luma(textureSample(screen_texture, texture_sampler, in.uv + vec2<f32>(0.0, -d2x.y)).rgb);
    let edge_outer = max(abs(o_r - o_l), abs(o_u - o_d));

    let is_edge = max(Sobel_edge_mag, edge_outer * 0.85) > settings.outline_threshold;

    let center_luma = rgb2luma(center_col.rgb);
    let max_neighbor_luma = max(max(max(c00, c10), max(c20, c01)), max(max(c21, c02), max(c12, c22)));

    if (is_edge && center_luma < 0.82 && max_neighbor_luma < 0.94) {

        return vec4<f32>(0.02, 0.03, 0.05, 1.0);
    }

    return center_col;
}
