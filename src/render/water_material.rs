use bevy::{
    pbr::Material,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaterMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
    #[uniform(1)]
    pub deep_color: LinearRgba,
    #[uniform(2)]
    pub foam_color: LinearRgba,
    #[uniform(3)]
    pub params: Vec4, // x: time, y: speed, z: foam_threshold, w: clarity
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::new(0.08, 0.65, 0.78, 0.82),
            deep_color: LinearRgba::new(0.02, 0.22, 0.35, 0.95),
            foam_color: LinearRgba::new(0.92, 0.98, 1.0, 0.95),
            params: Vec4::new(0.0, 1.0, 0.15, 0.8),
        }
    }
}

impl Material for WaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/water_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

pub fn update_water_time(time: Res<Time>, mut materials: ResMut<Assets<WaterMaterial>>) {
    let t = time.elapsed_secs();
    for (_, mat) in materials.iter_mut() {
        mat.params.x = t;
    }
}
