#![allow(dead_code, unused)]

use bevy::{
    core_pipeline::{
        core_3d::graph::Core3d,
        fullscreen_vertex_shader,
    },
    ecs::query::QueryItem,
    image::BevyDefault,
    prelude::*,
    render::{
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, ViewTarget},
        Render, RenderApp, RenderSet,
    },
};

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct HeatHazeSource {
    pub radius: f32,
    pub intensity: f32,
    pub speed: f32,
    pub is_active: bool,
}

impl Default for HeatHazeSource {
    fn default() -> Self {
        Self {
            radius: 2.0,
            intensity: 0.02,
            speed: 1.0,
            is_active: true,
        }
    }
}

#[derive(ShaderType, Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct HeatSourceGpu {
    pub screen_pos: Vec2,
    pub radius: f32,
    pub intensity: f32,
}

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType, Reflect, Debug)]
#[reflect(Component)]
pub struct HeatHazeSettings {
    pub time: f32,
    pub count: u32,
    pub distortion_scale: f32,
    pub flow_speed: f32,
    pub sources: [HeatSourceGpu; 16],
}

impl Default for HeatHazeSettings {
    fn default() -> Self {
        Self {
            time: 0.0,
            count: 0,
            distortion_scale: 1.0,
            flow_speed: 1.0,
            sources: [HeatSourceGpu::default(); 16],
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct HeatHazeLabel;

#[derive(Resource)]
pub struct HeatHazePipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub shader: Handle<Shader>,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct HeatHazePipelineKey {
    pub texture_format: TextureFormat,
}

impl FromWorld for HeatHazePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();

        let layout = render_device.create_bind_group_layout(
            Some("heat_haze_bind_group_layout"),
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<HeatHazeSettings>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        });

        let shader = asset_server.load("shaders/heat_haze.wgsl");

        Self {
            layout,
            sampler,
            shader,
        }
    }
}

impl SpecializedRenderPipeline for HeatHazePipeline {
    type Key = HeatHazePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("heat_haze_post_process_pipeline".into()),
            layout: vec![self.layout.clone()],
            vertex: fullscreen_vertex_shader::fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: key.texture_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: default(),
            depth_stencil: None,
            multisample: default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}

#[derive(Component)]
pub struct HeatHazePipelineId(pub CachedRenderPipelineId);

#[derive(Default)]
pub struct HeatHazeNode;

impl ViewNode for HeatHazeNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static HeatHazePipelineId,
        &'static DynamicUniformIndex<HeatHazeSettings>,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_target, pipeline_id, uniform_index): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let heat_haze_pipeline = world.resource::<HeatHazePipeline>();
        let settings_uniforms = world.resource::<ComponentUniforms<HeatHazeSettings>>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id.0) else {
            return Ok(());
        };

        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            Some("heat_haze_bind_group"),
            &heat_haze_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &heat_haze_pipeline.sampler,
                settings_binding,
            )),
        );

        let pass_descriptor = RenderPassDescriptor {
            label: Some("heat_haze_post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        let mut render_pass = render_context
            .command_encoder()
            .begin_render_pass(&pass_descriptor);

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[uniform_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

pub fn prepare_heat_haze_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<HeatHazePipeline>>,
    heat_haze_pipeline: Res<HeatHazePipeline>,
    views: Query<(Entity, &ExtractedView), With<HeatHazeSettings>>,
) {
    for (entity, view) in views.iter() {
        let format = if view.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &heat_haze_pipeline,
            HeatHazePipelineKey {
                texture_format: format,
            },
        );

        commands
            .entity(entity)
            .insert(HeatHazePipelineId(pipeline_id));
    }
}

pub fn project_world_to_viewport(
    world_pos: Vec3,
    radius_world: f32,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<HeatSourceGpu> {
    let cam_pos = camera_transform.translation();
    let cam_forward = camera_transform.forward();
    let to_source = world_pos - cam_pos;

    let z_depth = cam_forward.dot(to_source);
    if z_depth <= 0.15 {
        return None;
    }

    let (screen_x, screen_y) = if let Some(ndc) = camera
        .world_to_ndc(camera_transform, world_pos)
        .filter(|n| n.is_finite() && n.z >= 0.0 && n.z <= 1.0)
    {
        (ndc.x * 0.5 + 0.5, 1.0 - (ndc.y * 0.5 + 0.5))
    } else {
        let cam_right = camera_transform.right();
        let cam_up = camera_transform.up();
        let ndc_x = (cam_right.dot(to_source) / z_depth).clamp(-2.0, 2.0);
        let ndc_y = (cam_up.dot(to_source) / z_depth).clamp(-2.0, 2.0);
        (ndc_x * 0.5 + 0.5, 1.0 - (ndc_y * 0.5 + 0.5))
    };

    if screen_x < -0.25 || screen_x > 1.25 || screen_y < -0.25 || screen_y > 1.25 {
        return None;
    }

    let dist = to_source.length().max(0.4);
    let screen_radius = (radius_world / dist * 0.12).clamp(0.01, 0.09);

    Some(HeatSourceGpu {
        screen_pos: Vec2::new(screen_x, screen_y),
        radius: screen_radius,
        intensity: 1.0,
    })
}

pub fn sync_heat_sources_system(
    mut commands: Commands,
    geysers: Query<(Entity, &crate::alchemy::ThermalGeyser), Without<HeatHazeSource>>,
    puddles: Query<(Entity, &crate::alchemy::PotionPuddle), Without<HeatHazeSource>>,
    projectiles: Query<(Entity, &crate::alchemy::PotionProjectile), Without<HeatHazeSource>>,
    flammables: Query<(Entity, &crate::alchemy::Flammable), Without<HeatHazeSource>>,
    mut haze_sources: Query<(
        &mut HeatHazeSource,
        Option<&crate::alchemy::ThermalGeyser>,
        Option<&crate::alchemy::Flammable>,
    )>,
) {
    for (entity, geyser) in geysers.iter() {
        let (radius, intensity) = if geyser.is_erupting {
            (3.4, 0.008)
        } else {
            (1.8, 0.003)
        };
        commands.entity(entity).insert(HeatHazeSource {
            radius,
            intensity,
            speed: 1.0,
            is_active: true,
        });
    }
    for (entity, puddle) in puddles.iter() {
        let is_thermal = puddle
            .effects
            .iter()
            .any(|e| e.kind == crate::alchemy::AlchemicalEffectKind::ThermalHeat);
        if is_thermal {
            commands.entity(entity).insert(HeatHazeSource {
                radius: 2.2,
                intensity: 0.004,
                speed: 1.0,
                is_active: true,
            });
        }
    }
    for (entity, proj) in projectiles.iter() {
        let is_fire = proj.slot == 2
            || proj
                .effects
                .iter()
                .any(|e| e.kind == crate::alchemy::AlchemicalEffectKind::ThermalHeat);
        if is_fire {
            commands.entity(entity).insert(HeatHazeSource {
                radius: 1.4,
                intensity: 0.005,
                speed: 1.2,
                is_active: true,
            });
        }
    }
    for (entity, flammable) in flammables.iter() {
        if flammable.is_burning {
            commands.entity(entity).insert(HeatHazeSource {
                radius: 2.0,
                intensity: 0.004,
                speed: 1.0,
                is_active: true,
            });
        }
    }
    for (mut haze, maybe_geyser, maybe_flammable) in haze_sources.iter_mut() {
        if let Some(geyser) = maybe_geyser {
            if geyser.is_erupting {
                haze.radius = 3.4;
                haze.intensity = 0.008;
                haze.speed = 1.2;
            } else {
                haze.radius = 1.8;
                haze.intensity = 0.003;
                haze.speed = 0.8;
            }
        }
        if let Some(flammable) = maybe_flammable {
            haze.is_active = flammable.is_burning;
        }
    }
}

pub fn update_heat_haze_uniforms(
    time: Res<Time>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut HeatHazeSettings)>,
    sources: Query<(&GlobalTransform, &HeatHazeSource)>,
) {
    let dt = time.delta_secs();
    for (camera, cam_transform, mut settings) in cameras.iter_mut() {
        settings.time += dt;
        let mut gpu_sources = [HeatSourceGpu::default(); 16];
        let mut count = 0usize;

        for (source_transform, haze) in sources.iter() {
            if !haze.is_active || count >= 16 {
                continue;
            }
            let source_pos = source_transform.translation() + Vec3::Y * 0.45;
            if let Some(mut gpu_source) =
                project_world_to_viewport(source_pos, haze.radius, camera, cam_transform)
            {
                gpu_source.intensity = haze.intensity;
                gpu_sources[count] = gpu_source;
                count += 1;
            }
        }

        settings.count = count as u32;
        settings.sources = gpu_sources;
    }
}

pub struct HeatHazePlugin;

impl Plugin for HeatHazePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HeatHazeSettings>()
            .register_type::<HeatHazeSource>()
            .add_plugins((
                ExtractComponentPlugin::<HeatHazeSettings>::default(),
                UniformComponentPlugin::<HeatHazeSettings>::default(),
            ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<HeatHazePipeline>>()
            .add_systems(Render, prepare_heat_haze_pipelines.in_set(RenderSet::Prepare))
            .add_render_graph_node::<ViewNodeRunner<HeatHazeNode>>(Core3d, HeatHazeLabel);
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<HeatHazePipeline>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heat_source_gpu_defaults_and_alignment() {
        let source = HeatSourceGpu::default();
        assert_eq!(source.screen_pos, Vec2::ZERO);
        assert_eq!(source.radius, 0.0);
        assert_eq!(source.intensity, 0.0);
    }

    #[test]
    fn test_heat_haze_settings_defaults() {
        let settings = HeatHazeSettings::default();
        assert_eq!(settings.count, 0);
        assert_eq!(settings.time, 0.0);
        assert_eq!(settings.distortion_scale, 1.0);
        assert_eq!(settings.sources.len(), 16);
    }

    #[test]
    fn test_heat_haze_source_defaults() {
        let haze = HeatHazeSource::default();
        assert!(haze.radius > 0.0);
        assert!(haze.intensity > 0.0);
        assert!(haze.is_active);
    }
}
