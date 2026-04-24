# solis_3d

Screen-space indirect lighting and ambient occlusion for [Bevy](https://bevyengine.org/).

Implements the [Visibility Bitmask SSIL](https://arxiv.org/abs/2301.11376) algorithm (Therrien et al. 2023) as a Bevy render plugin. Produces noise-free indirect lighting and ambient occlusion in a single compute pass, with no ray tracing hardware required.

## Features

- **Indirect lighting** -- color bleeding from nearby surfaces (e.g. a red wall tints the floor)
- **Ambient occlusion** -- darkens corners, contact shadows, crevices
- **Noise-free** -- deterministic visibility bitmask, no stochastic sampling
- **Single pass** -- one compute dispatch + one fullscreen composite
- **No preprocessing** -- fully dynamic, works with any Bevy PBR scene

## Usage

Add the dependency:

```toml
[dependencies]
solis_3d = "0.1"
```

Add the plugin and configure your camera:

```rust
use bevy::prelude::*;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use solis_3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Solis3dPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera { hdr: true, ..default() },
        Msaa::Off,
        DepthPrepass,
        NormalPrepass,
        RadianceCascade3d::default(),
    ));
}
```

`DepthPrepass`, `NormalPrepass`, and `Msaa::Off` are required. The plugin reads the depth and normal buffers to compute screen-space occlusion and indirect light. MSAA is not supported because the depth prepass does not produce a resolved single-sample texture.

## Configuration

| Field | Default | Description |
|---|---|---|
| `scale_factor` | `1.0` | Resolution scale. `2.0` = half-res (faster, slightly softer) |
| `gi_intensity` | `0.6` | Indirect lighting strength multiplier |
| `thickness` | `0.5` | Occluder thickness in view-space units |
| `modulate` | `WHITE` | Final color multiplier |
| `flags` | `DEFAULT` | Debug flags (see below) |

### Debug flags

```rust
cfg.flags |= Gi3dFlags::DEBUG_GI_ONLY;   // show only indirect light
cfg.flags |= Gi3dFlags::DEBUG_NORMALS;    // show normal buffer
```

## How it works

The plugin inserts a render node after `Node3d::EndMainPass` in Bevy's 3D render graph:

1. **SSIL compute pass** -- for each pixel, samples depth along 8 azimuthal slices with 8 samples each. A 32-bit visibility bitmask tracks which hemisphere sectors are occluded. Newly occluded sectors contribute indirect light weighted by Lambert's cosine law. The result is RGB indirect light + alpha AO visibility.

2. **Composite fragment pass** -- blends the SSIL result onto the scene: `scene * ao + indirect * gi_intensity`.

No emitter components needed -- emissive materials and Bevy's PBR lights naturally appear as indirect light sources via the scene color buffer.

## Algorithm

Based on "Screen Space Indirect Lighting with Visibility Bitmask" (Therrien, Levesque, Gilet, 2023). Key properties:

- Replaces horizon angles with a 32-bit bitmask for finer occlusion tracking
- Handles surface thickness correctly (light passes behind thin objects)
- Produces both AO and indirect lighting from a single set of depth samples
- Deterministic -- no temporal accumulation needed for a clean result

## Examples

```sh
cargo run --example basic    # Cornell box with orbit camera (LMB/RMB/scroll)
cargo run --example msaa     # demonstrates MSAA graceful degradation
```

### Controls (basic example)

| Key | Action |
|---|---|
| LMB drag | Orbit camera |
| RMB drag | Pan |
| Scroll | Zoom |
| G | Toggle GI-only view |
| N | Toggle normals view |
| +/- | Adjust GI intensity |

## Compatibility

| solis_3d | Bevy |
|---|---|
| 0.1 | 0.15 |

## License

MIT OR Apache-2.0
