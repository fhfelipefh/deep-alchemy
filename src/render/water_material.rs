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
    pub params: Vec4,
    #[texture(4)]
    #[sampler(5)]
    pub reflection_texture: Option<Handle<Image>>,
    #[uniform(6)]
    pub reflection_params: Vec4,
}

impl Default for WaterMaterial {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::new(0.08, 0.65, 0.78, 0.82),
            deep_color: LinearRgba::new(0.02, 0.22, 0.35, 0.95),
            foam_color: LinearRgba::new(0.92, 0.98, 1.0, 0.95),
            params: Vec4::new(0.0, 1.0, 0.15, 0.8),
            reflection_texture: None,
            reflection_params: Vec4::new(0.18, 0.045, 0.85, 0.70),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_water_material_alpha_and_shader() {
        let mat = WaterMaterial::default();
        assert_eq!(mat.alpha_mode(), AlphaMode::Blend);
        match WaterMaterial::fragment_shader() {
            ShaderRef::Path(path) => assert_eq!(
                path.path(),
                std::path::Path::new("shaders/water_material.wgsl")
            ),
            _ => panic!("Expected ShaderRef::Path"),
        }
    }

    #[test]
    fn test_update_water_time_system() {
        let mut app = App::new();
        app.init_resource::<Assets<WaterMaterial>>();
        app.add_systems(Update, update_water_time);

        let handle = {
            let mut mats = app.world_mut().resource_mut::<Assets<WaterMaterial>>();
            mats.add(WaterMaterial::default())
        };

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(500));
        app.insert_resource(time);

        app.update();

        let mats = app.world().resource::<Assets<WaterMaterial>>();
        let mat = mats.get(&handle).unwrap();
        assert!(mat.params.x > 0.0, "Water time param must have updated");
    }
}
