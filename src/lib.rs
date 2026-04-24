#![doc = include_str!("../README.md")]
#![allow(clippy::too_many_arguments)]

use bevy::{
    asset::{embedded_asset, load_internal_asset},
    core_pipeline::core_3d::graph::{Core3d, Node3d},
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin,
        render_graph::{RenderGraphApp, ViewNodeRunner},
        Render, RenderApp, RenderSet,
    },
};

#[allow(dead_code)]
mod config;
mod node;
#[allow(dead_code)]
mod pipeline;
mod view;

pub mod prelude {
    pub use super::config::{DisableGi3d, Gi3dFlags, RadianceCascade3d};
    pub use super::Solis3dPlugin;
}

const COMMON_SHADER: Handle<Shader> = Handle::weak_from_u128(92318475610293847561029384);

#[derive(Default)]
pub struct Solis3dPlugin;

impl Plugin for Solis3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<config::RadianceCascade3d>::default(),
            ExtractComponentPlugin::<config::DisableGi3d>::default(),
        ));

        load_internal_asset!(
            app,
            COMMON_SHADER,
            "shaders/common.wgsl",
            Shader::from_wgsl
        );

        embedded_asset!(app, "shaders/ssil.wgsl");
        embedded_asset!(app, "shaders/composite.wgsl");

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<view::GiFrameCounter>()
            .add_systems(
                Render,
                (view::prepare_config, view::prepare_textures, view::tick_frame_counter)
                    .in_set(RenderSet::Prepare),
            )
            .add_render_graph_node::<ViewNodeRunner<node::GiNode>>(Core3d, node::GiNodeLabel)
            .add_render_graph_edge(Core3d, Node3d::EndMainPass, node::GiNodeLabel);
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<pipeline::GiPipelines>();
    }
}
