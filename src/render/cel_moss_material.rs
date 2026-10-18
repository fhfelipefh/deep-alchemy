use bevy::{
    pbr::Material,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

/// Material implementing World-Space Normal top-moss blending (Ny > 0.6)
/// and stepped toon shading (2-3 discrete light bands).
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CelShadedMossMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[uniform(1)]
    pub moss_color: LinearRgba,
    #[uniform(2)]
    pub light_dir: Vec4,
    #[uniform(3)]
    pub moss_params: Vec4, // x: ny_threshold (0.60), y: noise_scale, z: noise_strength, w: toon_steps
}

impl Default for CelShadedMossMaterial {
    fn default() -> Self {
        Self {
            // Dark cavern rock (#26333a)
            base_color: LinearRgba::new(0.149, 0.200, 0.227, 1.0),
            // Lush velvet green moss (#246e34)
            moss_color: LinearRgba::new(0.141, 0.431, 0.204, 1.0),
            // Directional key light
            light_dir: Vec4::new(0.25, 0.85, -0.45, 0.0),
            // Normal threshold = 0.60, noise scale = 3.5, noise strength = 0.22, toon bands = 3.0
            moss_params: Vec4::new(0.60, 3.50, 0.22, 3.0),
        }
    }
}

impl Material for CelShadedMossMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/cel_shaded_world_moss.wgsl".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cel_shaded_moss_material_default_and_shader() {
        let mat = CelShadedMossMaterial::default();
        // Ny threshold must be exactly 0.6 per Technical Artist rules
        assert_eq!(mat.moss_params.x, 0.60);
        assert_eq!(mat.moss_params.w, 3.0); // 3 toon steps

        match CelShadedMossMaterial::fragment_shader() {
            ShaderRef::Path(path) => {
                assert_eq!(
                    path.path(),
                    std::path::Path::new("shaders/cel_shaded_world_moss.wgsl")
                );
            }
            _ => panic!("Expected ShaderRef::Path for cel_shaded_world_moss.wgsl"),
        }
    }
}
