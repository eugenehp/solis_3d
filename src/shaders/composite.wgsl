#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import solis_3d::common::{GiConfig3d, debug_gi_only, debug_normals}

@group(0) @binding(0) var main_tex: texture_2d<f32>;
@group(0) @binding(1) var gi_tex: texture_2d<f32>;
@group(0) @binding(2) var normal_tex: texture_2d<f32>;
@group(0) @binding(3) var linear_sampler: sampler;
@group(0) @binding(4) var point_sampler: sampler;
@group(0) @binding(5) var<uniform> cfg: GiConfig3d;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(main_tex, point_sampler, in.uv);

    // bilinear upsample from half-res GI: rgb = indirect light, a = visibility (AO)
    let gi_sample = textureSampleLevel(gi_tex, linear_sampler, in.uv, 0.0);
    let indirect = gi_sample.rgb;
    let ao = gi_sample.a;

    // apply: multiply scene by AO, add indirect light
    var out = vec4(scene.rgb * ao + indirect * cfg.gi_intensity, scene.a);
    out *= cfg.modulate;

    out = mix(out, vec4(indirect, 1.0), debug_gi_only(cfg));
    out = mix(out, vec4(textureSample(normal_tex, linear_sampler, in.uv).rgb, 1.0), debug_normals(cfg));

    return max(out, vec4(0.0));
}
