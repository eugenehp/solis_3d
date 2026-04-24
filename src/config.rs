use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponent,
        render_resource::ShaderType,
    },
};

/// Screen-space indirect lighting configuration.
///
/// Add this to a `Camera3d` entity alongside [`DepthPrepass`] and [`NormalPrepass`]
/// to enable SSIL (indirect lighting + ambient occlusion). MSAA must be disabled.
///
/// ```rust,no_run
/// # use bevy::prelude::*;
/// # use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
/// # use solis_3d::prelude::*;
/// commands.spawn((
///     Camera3d::default(),
///     Camera { hdr: true, ..default() },
///     Msaa::Off,
///     DepthPrepass,
///     NormalPrepass,
///     RadianceCascade3d::default(),
/// ));
/// ```
///
/// [`DepthPrepass`]: bevy::core_pipeline::prepass::DepthPrepass
/// [`NormalPrepass`]: bevy::core_pipeline::prepass::NormalPrepass
#[derive(Component, ExtractComponent, Clone)]
pub struct RadianceCascade3d {
    /// Resolution scale factor (1.0 = full res, 2.0 = half res — faster but softer)
    pub scale_factor: f32,
    /// GI intensity multiplier
    pub gi_intensity: f32,
    /// Depth thickness for occluder back-face detection (view-space units)
    pub thickness: f32,
    /// Final output color modulation
    pub modulate: LinearRgba,
    /// Debug/feature flags
    pub flags: Gi3dFlags,
    // kept for GPU config struct compatibility (unused by SSIL)
    #[doc(hidden)]
    pub interval: f32,
    #[doc(hidden)]
    pub cascade_count: u32,
    #[doc(hidden)]
    pub probe_base: u32,
}

impl Default for RadianceCascade3d {
    fn default() -> Self {
        Self {
            interval: 4.0,
            scale_factor: 1.0,
            cascade_count: 5,
            probe_base: 1,
            gi_intensity: 0.6,
            thickness: 0.5,
            modulate: LinearRgba::WHITE,
            flags: Gi3dFlags::DEFAULT,
        }
    }
}

/// Disable GI on this camera
#[derive(Component, Default, Clone, ExtractComponent)]
pub struct DisableGi3d;

// ---- render world types ----

#[derive(ShaderType, Clone, Default)]
#[allow(dead_code)]
pub struct GiGpuConfig3d {
    pub screen_size: UVec2,
    pub scaled_size: UVec2,
    pub cascade_count: u32,
    pub probe_base: u32,
    pub interval: f32,
    pub scale: f32,
    pub gi_intensity: f32,
    pub thickness: f32,
    pub max_mip: u32,
    pub flags: u32,
    pub frame: u32,
    pub proj: Mat4,
    pub inv_proj: Mat4,
    pub view_mat: Mat4,
    pub inv_view: Mat4,
    pub modulate: LinearRgba,
}

bitflags::bitflags! {
    #[derive(Clone, Default, Copy, Debug, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct Gi3dFlags: u32 {
        const DEFAULT           = 0;
        const DEBUG_CASCADE     = 0x1 << 0;
        const DEBUG_NORMALS     = 0x1 << 1;
        const DEBUG_DEPTH       = 0x1 << 2;
        const DEBUG_GI_ONLY     = 0x1 << 3;
    }
}
