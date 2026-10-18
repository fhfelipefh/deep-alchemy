#![allow(dead_code, unused)]

use bevy::{
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
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

#[derive(Component, Clone, Copy, ExtractComponent, ShaderType, Reflect, Debug)]
#[reflect(Component)]
#[allow(unused)]
pub struct OutlineSettings {
    pub outline_threshold: f32,
    pub outline_thickness: f32,
    pub normal_threshold: f32,
    pub depth_threshold: f32,
}

impl Default for OutlineSettings {
    fn default() -> Self {
        Self {
            outline_threshold: 0.28,
            outline_thickness: 2.0,
            normal_threshold: 0.25,
            depth_threshold: 0.05,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct OutlineLabel;

#[derive(Resource)]
pub struct OutlinePipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub shader: Handle<Shader>,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct OutlinePipelineKey {
    pub texture_format: TextureFormat,
}

impl FromWorld for OutlinePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();

        let layout = render_device.create_bind_group_layout(
            Some("outline_bind_group_layout"),
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<OutlineSettings>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        });

        let shader = asset_server.load("shaders/outline_post_process.wgsl");

        Self {
            layout,
            sampler,
            shader,
        }
    }
}

impl SpecializedRenderPipeline for OutlinePipeline {
    type Key = OutlinePipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("outline_post_process_pipeline".into()),
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
pub struct OutlinePipelineId(pub CachedRenderPipelineId);

#[derive(Default)]
pub struct OutlineNode;

impl ViewNode for OutlineNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static OutlinePipelineId,
        &'static DynamicUniformIndex<OutlineSettings>,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_target, pipeline_id, uniform_index): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let outline_pipeline = world.resource::<OutlinePipeline>();
        let settings_uniforms = world.resource::<ComponentUniforms<OutlineSettings>>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id.0) else {
            return Ok(());
        };

        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            Some("outline_bind_group"),
            &outline_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &outline_pipeline.sampler,
                settings_binding,
            )),
        );

        let pass_descriptor = RenderPassDescriptor {
            label: Some("outline_post_process_pass"),
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

pub fn prepare_outline_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<OutlinePipeline>>,
    outline_pipeline: Res<OutlinePipeline>,
    views: Query<(Entity, &ExtractedView), With<OutlineSettings>>,
) {
    for (entity, view) in views.iter() {
        let format = if view.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        let pipeline_id = pipelines.specialize(
            &pipeline_cache,
            &outline_pipeline,
            OutlinePipelineKey {
                texture_format: format,
            },
        );

        commands
            .entity(entity)
            .insert(OutlinePipelineId(pipeline_id));
    }
}

pub struct OutlinePostProcessPlugin;

impl Plugin for OutlinePostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OutlineSettings>().add_plugins((
            ExtractComponentPlugin::<OutlineSettings>::default(),
            UniformComponentPlugin::<OutlineSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedRenderPipelines<OutlinePipeline>>()
            .add_systems(Render, prepare_outline_pipelines.in_set(RenderSet::Prepare))
            .add_render_graph_node::<ViewNodeRunner<OutlineNode>>(Core3d, OutlineLabel)
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    OutlineLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<OutlinePipeline>();
    }
}
