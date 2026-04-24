use crate::config::GiGpuConfig3d;
use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        render_resource::{
            binding_types::{
                sampler, texture_2d, texture_depth_2d, texture_storage_2d, uniform_buffer,
            },
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, ComputePipelineDescriptor,
            FilterMode, FragmentState, MultisampleState, PipelineCache, PrimitiveState,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            StorageTextureAccess, TextureFormat,
        },
        renderer::RenderDevice,
    },
};

pub const CASCADE_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

#[derive(Resource)]
pub struct GiPipelines {
    pub ssil_layout: BindGroupLayout,
    pub ssil_id: CachedComputePipelineId,
    pub composite_layout: BindGroupLayout,
    pub composite_id: CachedRenderPipelineId,
    pub linear_sampler: Sampler,
    pub point_sampler: Sampler,
}

impl FromWorld for GiPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let ssil_layout = create_ssil_layout(render_device);
        let composite_layout = create_composite_layout(render_device);

        let server = world.resource::<AssetServer>();
        let ssil_shader = server.load("embedded://solis_3d/shaders/ssil.wgsl");
        let composite_shader = server.load("embedded://solis_3d/shaders/composite.wgsl");

        let cache = world.resource::<PipelineCache>();

        let ssil_id = cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("solis3d_ssil_pipeline".into()),
            layout: vec![ssil_layout.clone()],
            push_constant_ranges: vec![],
            shader: ssil_shader,
            shader_defs: vec![],
            entry_point: "main".into(),
            zero_initialize_workgroup_memory: false,
        });

        let composite_id = cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("solis3d_composite_pipeline".into()),
            layout: vec![composite_layout.clone()],
            push_constant_ranges: vec![],
            vertex: fullscreen_shader_vertex_state(),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                shader: composite_shader,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: CASCADE_FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            zero_initialize_workgroup_memory: false,
        });

        let linear_sampler = render_device.create_sampler(&SamplerDescriptor {
            label: Some("solis3d_linear_sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..default()
        });

        let point_sampler = render_device.create_sampler(&SamplerDescriptor::default());

        Self {
            ssil_layout,
            ssil_id,
            composite_layout,
            composite_id,
            linear_sampler,
            point_sampler,
        }
    }
}

fn create_ssil_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(
        "solis3d_ssil_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                // 0: scene color
                texture_2d(bevy::render::render_resource::TextureSampleType::Float {
                    filterable: true,
                }),
                // 1: depth prepass
                texture_depth_2d(),
                // 2: normal prepass
                texture_2d(bevy::render::render_resource::TextureSampleType::Float {
                    filterable: true,
                }),
                // 3: config
                uniform_buffer::<GiGpuConfig3d>(false),
                // 4: output (GI + AO)
                texture_storage_2d(TextureFormat::Rgba16Float, StorageTextureAccess::WriteOnly),
            ),
        ),
    )
}

fn create_composite_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(
        "solis3d_composite_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                // 0: main scene
                texture_2d(bevy::render::render_resource::TextureSampleType::Float {
                    filterable: true,
                }),
                // 1: gi result (rgb=indirect, a=ao)
                texture_2d(bevy::render::render_resource::TextureSampleType::Float {
                    filterable: true,
                }),
                // 2: normal prepass
                texture_2d(bevy::render::render_resource::TextureSampleType::Float {
                    filterable: true,
                }),
                // 3: linear sampler
                sampler(SamplerBindingType::Filtering),
                // 4: point sampler
                sampler(SamplerBindingType::NonFiltering),
                // 5: config
                uniform_buffer::<GiGpuConfig3d>(false),
            ),
        ),
    )
}
