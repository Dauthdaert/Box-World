use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::{camera::CameraProjection, primitives::Frustum},
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, SolverGroups, Vect};

use crate::{
    chunk::{ChunkPos, LoadedChunks, CHUNK_EDGE},
    voxel::VOXEL_SIZE,
    HORIZONTAL_VIEW_DISTANCE,
};

use super::{Player, PlayerBundle};

#[derive(Component)]
pub struct FPSCamera {
    pub phi: f32,
    pub theta: f32,
    pub velocity: Vect,
}

impl Default for FPSCamera {
    fn default() -> Self {
        Self {
            phi: 0.0,
            theta: FRAC_PI_2,
            velocity: Vect::ZERO,
        }
    }
}

pub fn spawn(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    cameras
        .iter()
        .for_each(|entity| commands.entity(entity).despawn_recursive());

    let mut window = windows.get_single_mut().unwrap();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;

    let camera = {
        let perspective_projection = PerspectiveProjection {
            fov: std::f32::consts::PI / 1.8,
            near: 0.001,
            far: 1000.0,
            aspect_ratio: 1.0,
        };
        let view_projection = perspective_projection.get_projection_matrix();
        let frustum = Frustum::from_view_projection_custom_far(
            &view_projection,
            &Vec3::ZERO,
            &Vec3::Z,
            perspective_projection.far(),
        );
        Camera3dBundle {
            projection: Projection::Perspective(perspective_projection),
            frustum,
            ..default()
        }
    };
    commands.spawn(PlayerBundle::default()).with_children(|c| {
        c.spawn((
            GlobalTransform::default(),
            Transform::from_xyz(0.0, 2.0, 0.0),
            Collider::cylinder(1.6, 0.4),
            SolverGroups::new(Group::GROUP_1, Group::GROUP_2),
            CollisionGroups::new(Group::GROUP_1, Group::GROUP_2),
        ));
        c.spawn((
            FPSCamera::default(),
            camera,
            FogSettings {
                color: Color::rgba(0.5, 0.5, 0.5, 1.0),
                falloff: FogFalloff::Linear {
                    start: ((HORIZONTAL_VIEW_DISTANCE - 4) * CHUNK_EDGE) as f32 * VOXEL_SIZE,
                    end: ((HORIZONTAL_VIEW_DISTANCE - 2) * CHUNK_EDGE) as f32 * VOXEL_SIZE,
                },
                ..default()
            },
        ));
    });
}

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

#[allow(clippy::too_many_arguments)]
pub(super) fn movement_input_system(
    mut player: Query<&mut FPSCamera>,
    player_position: Query<&Transform, With<Player>>,
    camera_transform: Query<&Transform, With<Camera>>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_sensitivity: Res<MouseSensitivity>,
    key_events: Res<Input<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut stationary_frames: Local<i32>,
    current_chunks: Res<LoadedChunks>,
) {
    let mut window = windows.get_single_mut().unwrap();
    if key_events.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = match window.cursor.grab_mode {
            CursorGrabMode::None => CursorGrabMode::Locked,
            CursorGrabMode::Confined | CursorGrabMode::Locked => CursorGrabMode::None,
        };
        window.cursor.visible = !window.cursor.visible;
    }

    if let Ok(translation) = player_position.get_single() {
        let translation = translation.translation;
        if current_chunks
            .get_chunk(ChunkPos::from_global_coords(
                translation.x,
                translation.y,
                translation.z,
            ))
            .is_none()
        {
            return;
        }

        let mut movement = Vec3::default();
        if let Ok(mut fps_camera) = player.get_single_mut() {
            let transform = camera_transform.single();

            if window.cursor.grab_mode == CursorGrabMode::Locked {
                for MouseMotion { delta } in mouse_events.iter() {
                    fps_camera.phi += delta.x * mouse_sensitivity.0 * 0.003;
                    fps_camera.theta = (fps_camera.theta + delta.y * mouse_sensitivity.0 * 0.003)
                        .clamp(0.00005, PI - 0.00005);
                }

                if key_events.pressed(KeyCode::W) {
                    let mut fwd = transform.forward();
                    fwd.y = 0.0;
                    let fwd = fwd.normalize();
                    movement += fwd;
                }
                if key_events.pressed(KeyCode::A) {
                    movement += transform.left()
                }
                if key_events.pressed(KeyCode::D) {
                    movement += transform.right()
                }
                if key_events.pressed(KeyCode::S) {
                    let mut back = transform.back();
                    back.y = 0.0;
                    let back = back.normalize();
                    movement += back;
                }

                if key_events.pressed(KeyCode::Space) && *stationary_frames > 2 {
                    *stationary_frames = 0;
                    fps_camera.velocity.y = 16.0;
                }
            }

            movement = movement.normalize_or_zero();

            if fps_camera.velocity.y.abs() < 0.001 && *stationary_frames < 10 {
                *stationary_frames += 4;
            } else if *stationary_frames >= 0 {
                *stationary_frames -= 1;
            }

            let y = fps_camera.velocity.y;
            fps_camera.velocity.y = 0.0;
            fps_camera.velocity = movement;
            if key_events.pressed(KeyCode::LShift) {
                fps_camera.velocity *= 20.0;
            } else {
                fps_camera.velocity *= 10.0;
            }
            fps_camera.velocity.y = y;
            let chunk_pos =
                ChunkPos::from_global_coords(translation.x, translation.y, translation.z);

            if current_chunks.get_chunk(chunk_pos).is_none() {
                return;
            }

            fps_camera.velocity.y -= 35.0 * time.delta().as_secs_f32().clamp(0.0, 0.1);
        }
    }
}