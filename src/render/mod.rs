pub mod cel_moss_material;
pub mod heat_haze;
pub mod outline;
pub mod particles;
pub mod planar_reflection;
pub mod potion_fluid_material;
pub mod surface_fluid_material;
pub mod water_material;

use bevy::prelude::*;
use cel_moss_material::CelShadedMossMaterial;
use heat_haze::{sync_heat_sources_system, update_heat_haze_uniforms, HeatHazePlugin};
use outline::OutlinePostProcessPlugin;
use particles::{spawn_particles, update_particles, ParticleAssets};
use planar_reflection::PlanarReflectionPlugin;
use potion_fluid_material::{update_potion_sloshing, PotionFluidMaterial};
use surface_fluid_material::SurfaceFluidMaterial;
use water_material::{update_water_time, WaterMaterial};

pub struct RenderPipelinePlugin;

impl Plugin for RenderPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<WaterMaterial>::default(),
            MaterialPlugin::<CelShadedMossMaterial>::default(),
            MaterialPlugin::<PotionFluidMaterial>::default(),
            MaterialPlugin::<SurfaceFluidMaterial>::default(),
            HeatHazePlugin,
            OutlinePostProcessPlugin,
            PlanarReflectionPlugin,
        ))
        .init_resource::<ParticleAssets>()
        .add_systems(
            Update,
            (
                update_water_time,
                sync_heat_sources_system,
                update_heat_haze_uniforms,
                spawn_particles,
                update_particles,
            ),
        )
        .add_systems(
            PostUpdate,
            update_potion_sloshing.after(bevy::transform::TransformSystem::TransformPropagate),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_pipeline_plugin_build() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::transform::TransformPlugin,
        ));
        app.init_asset::<bevy::render::render_resource::Shader>();
        app.add_plugins(RenderPipelinePlugin);
        assert!(app.is_plugin_added::<RenderPipelinePlugin>());
    }
}
