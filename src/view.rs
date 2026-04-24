use crate::{
    config::{GiGpuConfig3d, RadianceCascade3d},
    pipeline::CASCADE_FORMAT,
};
use bevy::{
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureUsages, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{CachedTexture, TextureCache},
        view::{ExtractedView, ViewTarget},
    },
};

#[derive(Component, Default)]
pub struct GiBuffers {
    pub config_buffer: UniformBuffer<GiGpuConfig3d>,
}

#[derive(Component)]
pub struct GiTargets {
    pub gi_output: CachedTexture,
}

#[derive(Resource, Default)]
pub struct GiFrameCounter(pub u32);

pub(crate) fn tick_frame_counter(mut counter: ResMut<GiFrameCounter>) {
    counter.0 = counter.0.wrapping_add(1);
}

pub(crate) fn prepare_config(
    views: Query<(Entity, &ViewTarget, &RadianceCascade3d, &ExtractedView)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    frame_counter: Res<GiFrameCounter>,
    mut cmd: Commands,
) {
    for (entity, view_target, cfg, extracted_view) in views.iter() {
        let target_size = view_target.main_texture().size();
        let native = Vec2::new(target_size.width as f32, target_size.height as f32);
        let scaled = native / cfg.scale_factor;

        let mut config_buffer = UniformBuffer::<GiGpuConfig3d>::default();
        let gpu_cfg = config_buffer.get_mut();
        gpu_cfg.screen_size = native.as_uvec2();
        gpu_cfg.scaled_size = scaled.as_uvec2();
        gpu_cfg.cascade_count = cfg.cascade_count;
        gpu_cfg.probe_base = cfg.probe_base;
        gpu_cfg.interval = cfg.interval;
        gpu_cfg.scale = cfg.scale_factor;
        gpu_cfg.gi_intensity = cfg.gi_intensity;
        gpu_cfg.thickness = cfg.thickness;
        gpu_cfg.flags = cfg.flags.bits();
        gpu_cfg.frame = frame_counter.0;
        gpu_cfg.modulate = cfg.modulate;

        let proj = extracted_view.clip_from_view;
        gpu_cfg.proj = proj;
        gpu_cfg.inv_proj = proj.inverse();

        let view_mat = extracted_view.world_from_view.compute_matrix().inverse();
        gpu_cfg.view_mat = view_mat;
        gpu_cfg.inv_view = view_mat.inverse();

        let max_dim = target_size.width.max(target_size.height) as f32;
        gpu_cfg.max_mip = (max_dim.log2().floor() as u32).max(1);

        config_buffer.write_buffer(&render_device, &render_queue);

        cmd.entity(entity).insert(GiBuffers { config_buffer });
    }
}

pub(crate) fn prepare_textures(
    views: Query<(Entity, &ViewTarget, &RadianceCascade3d)>,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    mut cmd: Commands,
) {
    for (entity, view_target, cfg) in views.iter() {
        let full_size = view_target.main_texture().size();
        let gi_size = Extent3d {
            width: ((full_size.width as f32 / cfg.scale_factor) as u32).max(1),
            height: ((full_size.height as f32 / cfg.scale_factor) as u32).max(1),
            depth_or_array_layers: 1,
        };

        let gi_output = texture_cache.get(
            &render_device,
            TextureDescriptor {
                label: Some("solis3d_gi_output"),
                size: gi_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: CASCADE_FORMAT,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            },
        );

        cmd.entity(entity).insert(GiTargets { gi_output });
    }
}
