#define_import_path solis_3d::common

const PI: f32 = 3.14159265;
const TAU: f32 = PI * 2.0;
const EPSILON: f32 = 4.88e-04;

struct GiConfig3d {
    screen_size: vec2<u32>,
    scaled_size: vec2<u32>,
    cascade_count: u32,
    probe_base: u32,
    interval: f32,
    scale: f32,
    gi_intensity: f32,
    thickness: f32,
    max_mip: u32,
    flags: u32,
    frame: u32,
    proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    view_mat: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    modulate: vec4<f32>,
}

struct ProbeParams {
    cascade_index: u32,
}

// --- hemisphere sampling ---

const GOLDEN_RATIO: f32 = 1.6180339887;

/// Fibonacci hemisphere with per-frame rotation for temporal accumulation.
/// Each frame rotates all directions by a golden-ratio angle, so over many
/// frames the accumulated samples densely cover the hemisphere.
fn fibonacci_hemisphere(idx: u32, count: u32, frame: u32) -> vec3<f32> {
    let i = f32(idx) + 0.5;
    let n = f32(count);
    let cos_theta = 1.0 - i / n;
    let sin_theta = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));
    // rotate phi by golden_ratio * frame for temporal variation
    let phi = TAU * (i + f32(frame) * GOLDEN_RATIO) / GOLDEN_RATIO;
    return vec3(
        sin_theta * cos(phi),
        sin_theta * sin(phi),
        cos_theta,
    );
}

/// Map a 2D angular tile index to a linear sample index for Fibonacci sampling.
fn angular_tile_to_index(tile: vec2<u32>, tile_sub: vec2<u32>, sqr_angular: u32, sub_count: u32) -> u32 {
    let angular_2d = sqr_angular * sub_count;
    let ax = tile.x * sub_count + tile_sub.x;
    let ay = tile.y * sub_count + tile_sub.y;
    return ay * angular_2d + ax;
}

/// Total direction count for a given cascade level.
fn total_directions(sqr_angular: u32, sub_count: u32) -> u32 {
    let s = sqr_angular * sub_count;
    return s * s;
}

// --- TBN construction ---

fn build_tbn(normal: vec3<f32>) -> mat3x3<f32> {
    let up = select(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), abs(normal.y) < 0.999);
    let tangent = normalize(cross(up, normal));
    let bitangent = cross(normal, tangent);
    return mat3x3(tangent, bitangent, normal);
}

fn align_to_normal(local_dir: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    return build_tbn(normal) * local_dir;
}

// --- view reconstruction ---

fn reconstruct_view_pos(uv: vec2<f32>, depth: f32, inv_proj: mat4x4<f32>) -> vec3<f32> {
    let ndc = vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
    let view_pos = inv_proj * ndc;
    return view_pos.xyz / view_pos.w;
}

fn project_view_to_uv(view_pos: vec3<f32>, proj: mat4x4<f32>) -> vec3<f32> {
    let clip = proj * vec4(view_pos, 1.0);
    let ndc = clip.xyz / clip.w;
    return vec3(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, ndc.z);
}

// --- debug flag helpers ---

fn debug_cascade(cfg: GiConfig3d) -> f32 {
    return select(0.0, 1.0, (cfg.flags & 0x1u) != 0u);
}

fn debug_normals(cfg: GiConfig3d) -> f32 {
    return select(0.0, 1.0, (cfg.flags >> 1u & 0x1u) != 0u);
}

fn debug_depth(cfg: GiConfig3d) -> f32 {
    return select(0.0, 1.0, (cfg.flags >> 2u & 0x1u) != 0u);
}

fn debug_gi_only(cfg: GiConfig3d) -> f32 {
    return select(0.0, 1.0, (cfg.flags >> 3u & 0x1u) != 0u);
}
