pub mod cel_moss_material;
pub mod outline;
pub mod particles;
pub mod planar_reflection;
pub mod potion_fluid_material;
pub mod water_material;

use bevy::prelude::*;
use cel_moss_material::CelShadedMossMaterial;
use outline::OutlinePostProcessPlugin;
use particles::{spawn_particles, update_particles};
use planar_reflection::PlanarReflectionPlugin;
use potion_fluid_material::{update_potion_sloshing, PotionFluidMaterial};
use water_material::{update_water_time, WaterMaterial};

pub struct RenderPipelinePlugin;

impl Plugin for RenderPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<WaterMaterial>::default(),
            MaterialPlugin::<CelShadedMossMaterial>::default(),
            MaterialPlugin::<PotionFluidMaterial>::default(),
            OutlinePostProcessPlugin,
            PlanarReflectionPlugin,
        ))
        .add_systems(
            Update,
            (
                update_water_time,
                update_potion_sloshing,
                spawn_particles,
                update_particles,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_pipeline_plugin_build() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<bevy::render::render_resource::Shader>();
        app.add_plugins(RenderPipelinePlugin);
        assert!(app.is_plugin_added::<RenderPipelinePlugin>());
    }
}
