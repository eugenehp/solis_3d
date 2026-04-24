//! Demonstrates solis_3d behavior with MSAA.
//!
//! MSAA depth textures don't have resolve targets on most platforms,
//! so screen-space GI is automatically disabled when MSAA is active.
//! The scene renders normally with direct lighting but without GI bounce.
//!
//! For GI + anti-aliasing, use `Msaa::Off` with FXAA or SMAA instead.

use bevy::prelude::*;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use solis_3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Solis3dPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera with MSAA 4x (Bevy default)
    cmd.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Msaa::Sample4,
        Transform::from_xyz(0.0, 2.5, 6.0).looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y),
        DepthPrepass,
        NormalPrepass,
        RadianceCascade3d::default(),
    ));

    let box_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    // floor
    cmd.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(3.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            ..default()
        })),
    ));

    // back wall
    cmd.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::new(3.0, 3.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            ..default()
        })),
        Transform::from_xyz(0.0, 3.0, -3.0),
    ));

    // left wall (red)
    cmd.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::X, Vec2::new(3.0, 3.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(-3.0, 3.0, 0.0),
    ));

    // right wall (green)
    cmd.spawn((
        Mesh3d(meshes.add(Plane3d::new(-Vec3::X, Vec2::new(3.0, 3.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2),
            ..default()
        })),
        Transform::from_xyz(3.0, 3.0, 0.0),
    ));

    // ceiling
    cmd.spawn((
        Mesh3d(meshes.add(Plane3d::new(-Vec3::Y, Vec2::splat(3.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            ..default()
        })),
        Transform::from_xyz(0.0, 6.0, 0.0),
    ));

    // emissive light panel on ceiling
    cmd.spawn((
        Mesh3d(meshes.add(Plane3d::new(-Vec3::Y, Vec2::splat(0.8)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(10.0, 9.0, 7.0, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 5.99, 0.0),
    ));

    // tall box
    cmd.spawn((
        Mesh3d(box_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            ..default()
        })),
        Transform::from_xyz(-1.0, 1.0, -0.5)
            .with_rotation(Quat::from_rotation_y(0.3))
            .with_scale(Vec3::new(1.0, 2.0, 1.0)),
    ));

    // short box
    cmd.spawn((
        Mesh3d(box_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.8, 0.8),
            ..default()
        })),
        Transform::from_xyz(1.0, 0.5, 1.0).with_rotation(Quat::from_rotation_y(-0.3)),
    ));

    // point light
    cmd.spawn((
        PointLight {
            color: Color::WHITE,
            intensity: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 5.5, 0.0),
    ));
}
