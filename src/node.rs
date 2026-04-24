use crate::{
    config::{DisableGi3d, RadianceCascade3d},
    pipeline::GiPipelines,
    view::{GiBuffers, GiTargets},
};
use bevy::{
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::*,
    render::{
        render_graph::{self, NodeRunError, RenderGraphContext, RenderLabel},
        render_resource::{
            BindGroupEntries, ComputePassDescriptor, Operations, PipelineCache,
            RenderPassColorAttachment, RenderPassDescriptor,
        },
        renderer::RenderContext,
        view::ViewTarget,
    },
};
use bevy::core_pipeline::prepass::ViewPrepassTextures;

#[derive(Hash, PartialEq, Eq, Clone, Copy, RenderLabel, Debug)]
pub struct GiNodeLabel;

#[derive(Default)]
pub struct GiNode;

impl render_graph::ViewNode for GiNode {
    type ViewQuery = (
        Read<ViewTarget>,
        Read<GiBuffers>,
        Read<GiTargets>,
        Read<RadianceCascade3d>,
        Read<ViewPrepassTextures>,
        Has<DisableGi3d>,
    );

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (view_target, gi_buffers, gi_targets, _config, prepass_textures, disabled): QueryItem<
            'w,
            Self::ViewQuery,
        >,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        if disabled {
            return Ok(());
        }

        let pipelines = world.resource::<GiPipelines>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(ssil_pipeline) = pipeline_cache.get_compute_pipeline(pipelines.ssil_id) else {
            return Ok(());
        };
        let Some(composite_pipeline) =
            pipeline_cache.get_render_pipeline(pipelines.composite_id)
        else {
            return Ok(());
        };

        let Some(config_binding) = gi_buffers.config_buffer.binding() else {
            return Ok(());
        };

        let (Some(depth_attachment), Some(normal_attachment)) = (
            prepass_textures.depth.as_ref(),
            prepass_textures.normal.as_ref(),
        ) else {
            warn_once!("solis_3d: DepthPrepass and NormalPrepass are required on the camera");
            return Ok(());
        };

        let depth_tex = match depth_attachment.resolve_target.as_ref() {
            Some(resolved) => resolved,
            None => {
                if depth_attachment.texture.texture.sample_count() > 1 {
                    warn_once!("solis_3d: MSAA depth has no resolve target — use Msaa::Off");
                    return Ok(());
                }
                &depth_attachment.texture
            }
        };
        let normal_tex = match normal_attachment.resolve_target.as_ref() {
            Some(resolved) => resolved,
            None => &normal_attachment.texture,
        };

        let post_process = view_target.post_process_write();

        // ---------------------------------------------------------------
        // SSIL compute pass — single pass, noise-free visibility bitmask
        let gi_w = gi_targets.gi_output.texture.size().width;
        let gi_h = gi_targets.gi_output.texture.size().height;

        let ssil_bind_group = render_context.render_device().create_bind_group(
            Some("solis3d_ssil_bg"),
            &pipelines.ssil_layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &depth_tex.default_view,
                &normal_tex.default_view,
                config_binding.clone(),
                &gi_targets.gi_output.default_view,
            )),
        );

        {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("solis3d_ssil_pass"),
                    timestamp_writes: None,
                });
            pass.set_pipeline(ssil_pipeline);
            pass.set_bind_group(0, &ssil_bind_group, &[]);
            pass.dispatch_workgroups((gi_w + 7) / 8, (gi_h + 7) / 8, 1);
        }

        // ---------------------------------------------------------------
        // composite — blend indirect light + AO onto scene
        let composite_bind_group = render_context.render_device().create_bind_group(
            Some("solis3d_composite_bg"),
            &pipelines.composite_layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &gi_targets.gi_output.default_view,
                &normal_tex.default_view,
                &pipelines.linear_sampler,
                &pipelines.point_sampler,
                config_binding,
            )),
        );

        {
            let mut render_pass =
                render_context.begin_tracked_render_pass(RenderPassDescriptor {
                    label: Some("solis3d_composite_pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: post_process.destination,
                        resolve_target: None,
                        ops: Operations::default(),
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            render_pass.set_render_pipeline(composite_pipeline);
            render_pass.set_bind_group(0, &composite_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}
