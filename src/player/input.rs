use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::Vect;

use crate::{
    chunk::{ChunkData, ChunkPos, LoadedChunks},
    mesher::NeedsMesh,
    voxel::{Voxel, VoxelPos, VOXEL_SIZE, VOXEL_STONE},
};

use super::Player;

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

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

#[allow(clippy::too_many_arguments)]
pub(super) fn movement_input(
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

pub(super) fn interact(
    mut commands: Commands,
    loaded_chunks: Res<LoadedChunks>,
    mouse_input: Res<Input<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut chunks: Query<&mut ChunkData>,
) {
    let window = window.single();

    // Do nothing if player insn't focused
    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let cursor_position = Vec2::new(window.width() / 2., window.height() / 2.);

    for (camera, camera_transform) in camera.iter() {
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { return; };

        for i in 1..5 {
            let ray_pos = ray.get_point(i as f32 * VOXEL_SIZE);
            let voxel_pos = VoxelPos::from_global_coords(ray_pos.x, ray_pos.y, ray_pos.z);
            let (mut chunk_pos, mut local_x, mut local_y, mut local_z) =
                voxel_pos.to_chunk_coords();

            let Some(chunk_entity) = loaded_chunks.get_chunk(chunk_pos) else { continue; };
            let Ok(mut chunk_data) = chunks.get_mut(*chunk_entity) else { continue; };

            let changed = if mouse_input.just_pressed(MouseButton::Left) {
                if chunk_data.get(local_x, local_y, local_z) != Voxel::Empty {
                    chunk_data.set(local_x, local_y, local_z, Voxel::Empty);
                    true
                } else {
                    false
                }
            } else if mouse_input.just_pressed(MouseButton::Right) {
                if chunk_data.get(local_x, local_y, local_z) != Voxel::Empty {
                    // Place in previous spot
                    let prev_ray_pos = ray.get_point((i - 1) as f32 * VOXEL_SIZE);
                    let prev_voxel_pos = VoxelPos::from_global_coords(
                        prev_ray_pos.x,
                        prev_ray_pos.y,
                        prev_ray_pos.z,
                    );
                    let (prev_chunk_pos, prev_local_x, prev_local_y, prev_local_z) =
                        prev_voxel_pos.to_chunk_coords();

                    let mut prev_chunk_data = if chunk_pos == prev_chunk_pos {
                        chunk_data
                    } else {
                        let Some(chunk_entity) = loaded_chunks.get_chunk(prev_chunk_pos) else { continue; };
                        let Ok(chunk_data) = chunks.get_mut(*chunk_entity) else { continue; };
                        chunk_data
                    };
                    prev_chunk_data.set(prev_local_x, prev_local_y, prev_local_z, VOXEL_STONE);

                    // Propagate change of voxel position
                    chunk_pos = prev_chunk_pos;
                    local_x = prev_local_x;
                    local_y = prev_local_y;
                    local_z = prev_local_z;

                    true
                } else {
                    false
                }
            } else {
                false
            };

            if changed {
                commands.entity(*chunk_entity).insert(NeedsMesh);

                // If change is on a border, update neighbors
                if local_x == 0
                    || local_x == ChunkData::edge() - 1
                    || local_y == 0
                    || local_y == ChunkData::edge() - 1
                    || local_z == 0
                    || local_z == ChunkData::edge() - 1
                {
                    for neighbor in loaded_chunks.get_loaded_chunk_neighbors(chunk_pos) {
                        commands.entity(neighbor).insert(NeedsMesh);
                    }
                }

                // Stop ray casting
                break;
            }
        }
    }
}
