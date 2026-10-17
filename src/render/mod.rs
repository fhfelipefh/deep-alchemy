pub mod particles;
pub mod water_material;

use bevy::prelude::*;
use particles::{spawn_particles, update_particles};
use water_material::{update_water_time, WaterMaterial};

pub struct RenderPipelinePlugin;

impl Plugin for RenderPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WaterMaterial>::default())
            .add_systems(Update, (update_water_time, spawn_particles, update_particles));
    }
}
