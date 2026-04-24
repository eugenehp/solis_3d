use bevy::prelude::*;
use bevy::core_pipeline::prepass::{DepthPrepass, NormalPrepass};
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use solis_3d::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Solis3dPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, multi_angle_screenshots)
        .insert_resource(ShotState { frame: 0, angle: 0 })
        .run();
}

#[derive(Resource)]
struct ShotState {
    frame: u32,
    angle: u32,
}

const ANGLES: &[(Vec3, Vec3, &str)] = &[
    (Vec3::new(0.0, 2.5, 6.0), Vec3::new(0.0, 1.5, 0.0), "/Users/Shared/solis_3d/screenshots/front.png"),
    (Vec3::new(-5.0, 3.0, 0.0), Vec3::new(0.0, 1.5, 0.0), "/Users/Shared/solis_3d/screenshots/left.png"),
    (Vec3::new(4.0, 1.0, 4.0), Vec3::new(0.0, 1.5, -1.0), "/Users/Shared/solis_3d/screenshots/corner.png"),
];

fn multi_angle_screenshots(
    mut state: ResMut<ShotState>,
    mut commands: Commands,
    mut camera: Query<&mut Transform, With<RadianceCascade3d>>,
    mut exit: EventWriter<AppExit>,
) {
    state.frame += 1;

    let angle_idx = state.angle as usize;
    if angle_idx >= ANGLES.len() {
        if state.frame > (ANGLES.len() as u32) * 20 + 10 {
            exit.send(AppExit::Success);
        }
        return;
    }

    // set camera, wait 15 frames for pipelines + render, screenshot at frame 15
    let base_frame = angle_idx as u32 * 20;

    if state.frame == base_frame + 1 {
        if let Ok(mut tf) = camera.get_single_mut() {
            let (pos, target, _) = ANGLES[angle_idx];
            *tf = Transform::from_translation(pos).looking_at(target, Vec3::Y);
        }
    }

    if state.frame == base_frame + 15 {
        let (_, _, path) = ANGLES[angle_idx];
        commands.spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        state.angle += 1;
    }
}

fn setup(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmd.spawn((
        Camera3d::default(),
        Camera { hdr: true, ..default() },
        Msaa::Off,
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

    // emissive ceiling panel
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
            intensity: 30000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 5.5, 0.0),
    ));
}
