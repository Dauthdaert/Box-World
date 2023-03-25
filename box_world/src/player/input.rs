use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    chunk::{ChunkData, LoadedChunks},
    mesher::NeedsMesh,
    voxel::{GlobalVoxelPos, Voxel, VoxelRegistry},
};

use super::{highlight::HighlightCube, Player};

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(super) fn interact(
    mut commands: Commands,
    voxel_registry: Res<VoxelRegistry>,
    loaded_chunks: Res<LoadedChunks>,
    mouse_input: Res<Input<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    player_position: Query<&Transform, With<Player>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut highlight_cube: Query<
        (&mut Transform, &mut Visibility),
        (With<HighlightCube>, Without<Player>),
    >,
    mut chunks: Query<&mut ChunkData>,
) {
    let window = window.single();

    // Do nothing if player insn't focused
    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let player_equipped_block = voxel_registry.get_voxel("stone");

    let player_translation = player_position.single().translation;
    let player_head_pos = GlobalVoxelPos::from_global_coords(player_translation);
    let player_feet_pos =
        GlobalVoxelPos::new(player_head_pos.x, player_head_pos.y - 1, player_head_pos.z);

    let (mut cube_position, mut cube_visibility) = highlight_cube.single_mut();
    *cube_visibility = Visibility::Hidden;

    let cursor_position = Vec2::new(window.width() / 2., window.height() / 2.);

    const RAY_STEP: f32 = 0.5;
    for (camera, camera_transform) in camera.iter() {
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { return; };

        for i in 1..10 {
            let ray_pos = ray.get_point(i as f32 * RAY_STEP);
            let voxel_pos = GlobalVoxelPos::from_global_coords(ray_pos);
            let (mut chunk_pos, mut local_pos) = voxel_pos.to_chunk_local();

            let Some(chunk_entity) = loaded_chunks.get_chunk(chunk_pos) else { continue; };
            let Ok(mut chunk_data) = chunks.get_mut(*chunk_entity) else { continue; };

            if !chunk_data
                .get(local_pos.x, local_pos.y, local_pos.z)
                .is_empty()
            {
                // Highlight selected block
                cube_position.translation = voxel_pos.to_global_coords() + 0.5;
                *cube_visibility = Visibility::Inherited;

                // Interact with selected block
                let changed = if mouse_input.just_pressed(MouseButton::Left) {
                    chunk_data.set(local_pos.x, local_pos.y, local_pos.z, Voxel::default());
                    true
                } else if mouse_input.just_pressed(MouseButton::Right) {
                    // Place in previous spot
                    let mut prev_voxel_pos = voxel_pos;

                    // Rewind ray backwards by one voxel
                    for j in 1..i {
                        let prev_ray_pos = ray.get_point((i - j) as f32 * RAY_STEP);
                        prev_voxel_pos = GlobalVoxelPos::from_global_coords(prev_ray_pos);

                        if prev_voxel_pos != voxel_pos {
                            break;
                        }
                    }

                    // Can't place on top of the player
                    if prev_voxel_pos == player_head_pos || prev_voxel_pos == player_feet_pos {
                        return;
                    }

                    let (prev_chunk_pos, prev_local_pos) = prev_voxel_pos.to_chunk_local();

                    let mut prev_chunk_data = if chunk_pos == prev_chunk_pos {
                        chunk_data
                    } else {
                        let Some(chunk_entity) = loaded_chunks.get_chunk(prev_chunk_pos) else { continue; };
                        let Ok(chunk_data) = chunks.get_mut(*chunk_entity) else { continue; };
                        chunk_data
                    };
                    prev_chunk_data.set(
                        prev_local_pos.x,
                        prev_local_pos.y,
                        prev_local_pos.z,
                        player_equipped_block,
                    );

                    // Propagate change of voxel position
                    chunk_pos = prev_chunk_pos;
                    local_pos = prev_local_pos;

                    true
                } else {
                    false
                };

                if changed {
                    commands.entity(*chunk_entity).insert(NeedsMesh);

                    // If change is on a border, update neighbors
                    if local_pos.x == 0
                        || local_pos.x == ChunkData::edge() - 1
                        || local_pos.y == 0
                        || local_pos.y == ChunkData::edge() - 1
                        || local_pos.z == 0
                        || local_pos.z == ChunkData::edge() - 1
                    {
                        for neighbor in loaded_chunks.get_loaded_chunk_neighbors(chunk_pos) {
                            commands.entity(neighbor).insert(NeedsMesh);
                        }
                    }
                }

                // Stop ray casting
                break;
            }
        }
    }
}
