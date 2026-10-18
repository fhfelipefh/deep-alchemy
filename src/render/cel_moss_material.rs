use bevy::{
    pbr::Material,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CelShadedMossMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[uniform(1)]
    pub moss_color: LinearRgba,
    #[uniform(2)]
    pub light_dir: Vec4,
    #[uniform(3)]
    pub moss_params: Vec4,
}

impl Default for CelShadedMossMaterial {
    fn default() -> Self {
        Self {

            base_color: LinearRgba::new(0.149, 0.200, 0.227, 1.0),

            moss_color: LinearRgba::new(0.141, 0.431, 0.204, 1.0),

            light_dir: Vec4::new(0.25, 0.85, -0.45, 0.0),

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

        assert_eq!(mat.moss_params.x, 0.60);
        assert_eq!(mat.moss_params.w, 3.0);

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
