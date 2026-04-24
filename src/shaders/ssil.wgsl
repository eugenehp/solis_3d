#import solis_3d::common::{
    GiConfig3d, reconstruct_view_pos,
    PI, TAU, EPSILON
}

// Screen Space Indirect Lighting with Visibility Bitmask
// Based on Therrien et al. 2023 (arxiv:2301.11376)

@group(0) @binding(0) var scene_color: texture_2d<f32>;
@group(0) @binding(1) var depth_tex: texture_depth_2d;
@group(0) @binding(2) var normal_tex: texture_2d<f32>;
@group(0) @binding(3) var<uniform> config: GiConfig3d;
@group(0) @binding(4) var output_tex: texture_storage_2d<rgba16float, write>;

const HALF_PI: f32 = PI * 0.5;
const SECTOR_COUNT: u32 = 32u;
const SLICE_COUNT: u32 = 8u;
const SAMPLE_COUNT: u32 = 8u;
const SAMPLE_RADIUS: f32 = 8.0;

fn update_sectors(min_horizon: f32, max_horizon: f32) -> u32 {
    let start_bit = u32(min_horizon * f32(SECTOR_COUNT));
    let horizon_angle = u32(ceil((max_horizon - min_horizon) * f32(SECTOR_COUNT)));
    let angle_bit = select(0u, 0xFFFFFFFFu >> (SECTOR_COUNT - horizon_angle), horizon_angle > 0u);
    return angle_bit << start_bit;
}

// interleaved gradient noise — better than simple hash, no banding
fn ign(coord: vec2<f32>, frame: u32) -> f32 {
    let wrapped = coord + vec2<f32>(f32(frame % 64u) * 5.588238, f32(frame % 64u) * 3.225);
    return fract(52.9829189 * fract(0.06711056 * wrapped.x + 0.00583715 * wrapped.y));
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let output_size = vec2<u32>(textureDimensions(output_tex));
    if any(gid.xy >= output_size) { return; }

    let uv = (vec2<f32>(gid.xy) + 0.5) / vec2<f32>(output_size);
    let screen_coord = vec2<i32>(uv * vec2<f32>(config.screen_size));

    let depth = textureLoad(depth_tex, screen_coord, 0);
    if depth < EPSILON {
        textureStore(output_tex, vec2<i32>(gid.xy), vec4(0.0, 0.0, 0.0, 1.0));
        return;
    }

    let position = reconstruct_view_pos(uv, depth, config.inv_proj);
    let normal_raw = textureLoad(normal_tex, screen_coord, 0);
    let world_normal = normalize(normal_raw.xyz * 2.0 - 1.0);
    let view_normal = normalize((config.view_mat * vec4(world_normal, 0.0)).xyz);
    let camera_dir = normalize(-position);

    // project sample radius to screen space
    let sample_scale = (-SAMPLE_RADIUS * config.proj[0][0]) / position.z;
    let aspect = vec2<f32>(config.screen_size.yx) / f32(config.screen_size.x);

    let jitter = ign(vec2<f32>(gid.xy), config.frame) - 0.5;

    let slice_rotation = TAU / f32(SLICE_COUNT);
    var total_visibility = 0.0;
    var total_lighting = vec3(0.0);

    for (var slice = 0u; slice < SLICE_COUNT; slice++) {
        var occlusion_mask = 0u;

        let phi = slice_rotation * (f32(slice) + jitter) + PI;
        let omega = vec2(cos(phi), sin(phi));
        let direction = vec3(omega, 0.0);
        let ortho_dir = direction - dot(direction, camera_dir) * camera_dir;
        let axis = cross(direction, camera_dir);

        let proj_normal = view_normal - axis * dot(view_normal, axis);
        let proj_length = length(proj_normal);
        let sign_n = sign(dot(ortho_dir, proj_normal));
        let cos_n = clamp(dot(proj_normal, camera_dir) / max(proj_length, 0.001), -1.0, 1.0);
        let n_angle = sign_n * acos(cos_n);

        for (var s = 0u; s < SAMPLE_COUNT; s++) {
            // exponential distribution: more samples close, fewer far
            let t_linear = (f32(s) + 0.5 + jitter * 0.4) / f32(SAMPLE_COUNT);
            let sample_step = t_linear * t_linear + 0.01;
            let sample_uv = uv - sample_step * sample_scale * omega * aspect;

            if any(sample_uv < vec2(0.0)) || any(sample_uv > vec2(1.0)) { continue; }

            let sample_sc = vec2<i32>(sample_uv * vec2<f32>(config.screen_size));
            let sample_depth = textureLoad(depth_tex, sample_sc, 0);
            if sample_depth < EPSILON { continue; }

            let sample_pos = reconstruct_view_pos(sample_uv, sample_depth, config.inv_proj);
            let sample_normal_raw = textureLoad(normal_tex, sample_sc, 0);
            let sample_world_normal = normalize(sample_normal_raw.xyz * 2.0 - 1.0);
            let sample_view_normal = normalize((config.view_mat * vec4(sample_world_normal, 0.0)).xyz);
            let sample_light = min(textureLoad(scene_color, sample_sc, 0).rgb, vec3(4.0));

            let sample_distance = sample_pos - position;
            let sample_length = length(sample_distance);
            if sample_length < 0.01 { continue; }
            let sample_horizon = sample_distance / sample_length;

            // front and back horizon angles with thickness
            var front_back = vec2(0.0);
            front_back.x = dot(sample_horizon, camera_dir);
            front_back.y = dot(normalize(sample_distance - camera_dir * config.thickness), camera_dir);

            front_back = acos(clamp(front_back, vec2(-1.0), vec2(1.0)));
            front_back = clamp((front_back + n_angle + HALF_PI) / PI, vec2(0.0), vec2(1.0));

            let min_h = min(front_back.x, front_back.y);
            let max_h = max(front_back.x, front_back.y);

            let sample_mask = update_sectors(min_h, max_h);

            // GI contribution from newly occluded sectors
            let new_bits = countOneBits(sample_mask & ~occlusion_mask);
            let gi_weight = f32(new_bits) / f32(SECTOR_COUNT);

            // cosine-weighted form factors
            let cos_receiver = max(dot(view_normal, sample_horizon), 0.0);
            let cos_emitter = max(dot(sample_view_normal, -sample_horizon), 0.0);

            // distance falloff — closer surfaces contribute more
            let falloff = 1.0 / (1.0 + sample_length * sample_length * 0.1);

            total_lighting += gi_weight * sample_light * cos_receiver * cos_emitter * falloff;
            occlusion_mask |= sample_mask;
        }

        total_visibility += 1.0 - f32(countOneBits(occlusion_mask)) / f32(SECTOR_COUNT);
    }

    total_visibility /= f32(SLICE_COUNT);
    total_lighting /= f32(SLICE_COUNT);

    // boost indirect to make color bleeding more visible
    total_lighting *= TAU;

    textureStore(output_tex, vec2<i32>(gid.xy), vec4(total_lighting, total_visibility));
}
