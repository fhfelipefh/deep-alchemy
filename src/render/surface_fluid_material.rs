use bevy::{
    pbr::Material,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SurfaceFluidMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(1)]
    pub flow_data: Vec4,
    #[uniform(2)]
    pub film_data: Vec4,
}

impl Material for SurfaceFluidMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/surface_fluid.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

impl Default for SurfaceFluidMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::new(0.12, 0.92, 0.42, 0.78),
            flow_data: Vec4::ZERO,
            film_data: Vec4::new(0.2, 0.08, 0.75, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_fluid_material_uses_the_film_shader() {
        match SurfaceFluidMaterial::fragment_shader() {
            ShaderRef::Path(path) => assert_eq!(path.path(), std::path::Path::new("shaders/surface_fluid.wgsl")),
            _ => panic!("Expected ShaderRef::Path"),
        }
    }
}
