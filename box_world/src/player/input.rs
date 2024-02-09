use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    chunk::{ChunkData, LoadedChunks, VoxelAddedEvent, VoxelRemovedEvent},
    mesher::NeedsMesh,
    voxel::{GlobalVoxelPos, Voxel, VoxelRegistry},
};

use super::Player;

#[derive(Debug, Clone, Resource, Default)]
pub struct CurrentBlock(Option<Voxel>);

pub(super) fn change_current_block(
    mut current_block: ResMut<CurrentBlock>,
    keyboard_input: Res<Input<KeyCode>>,
    voxel_registry: Res<VoxelRegistry>,
) {
    if keyboard_input.just_pressed(KeyCode::Key1) {
        current_block.0 = None;
    } else if keyboard_input.just_pressed(KeyCode::Key2) {
        current_block.0 = Some(voxel_registry.get_voxel("stone"));
    } else if keyboard_input.just_pressed(KeyCode::Key3) {
        current_block.0 = Some(voxel_registry.get_voxel("glass"));
    } else if keyboard_input.just_pressed(KeyCode::Key4) {
        current_block.0 = Some(voxel_registry.get_voxel("torch"));
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(super) fn interact(
    mut commands: Commands,
    loaded_chunks: Res<LoadedChunks>,
    mouse_input: Res<Input<MouseButton>>,
    current_block: Res<CurrentBlock>,
    window: Query<&Window, With<PrimaryWindow>>,
    player_position: Query<&Transform, With<Player>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut highlight: Gizmos,
    mut chunks: Query<&mut ChunkData>,
    mut added_events: EventWriter<VoxelAddedEvent>,
    mut removed_events: EventWriter<VoxelRemovedEvent>,
) {
    let window = window.single();

    // Do nothing if player insn't focused
    if window.cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let player_translation = player_position.single().translation;
    let player_head_pos = GlobalVoxelPos::from_global_coords(player_translation);
    let player_feet_pos =
        GlobalVoxelPos::new(player_head_pos.x, player_head_pos.y - 1, player_head_pos.z);

    let cursor_position = Vec2::new(window.width() / 2., window.height() / 2.);

    const RAY_STEP: f32 = 0.5;
    for (camera, camera_transform) in camera.iter() {
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

        for i in 1..10 {
            let ray_pos = ray.get_point(i as f32 * RAY_STEP);
            let voxel_pos = GlobalVoxelPos::from_global_coords(ray_pos);
            let (mut chunk_pos, mut local_pos) = voxel_pos.to_chunk_local();

            let Some(chunk_entity) = loaded_chunks.get_chunk(chunk_pos) else {
                continue;
            };
            let Ok(mut chunk_data) = chunks.get_mut(*chunk_entity) else {
                continue;
            };

            if !chunk_data
                .get(local_pos.x, local_pos.y, local_pos.z)
                .is_empty()
            {
                // Highlight selected block
                highlight.cuboid(
                    Transform::from_translation(voxel_pos.to_global_coords() + 0.5)
                        .with_scale(Vec3::splat(1.02)),
                    Color::BLACK,
                );

                // Interact with selected block
                let changed = if mouse_input.just_pressed(MouseButton::Left) {
                    chunk_data.set(local_pos.x, local_pos.y, local_pos.z, Voxel::default());
                    removed_events.send(VoxelRemovedEvent::new(GlobalVoxelPos::from_chunk_local(
                        chunk_pos, local_pos,
                    )));
                    true
                } else if mouse_input.just_pressed(MouseButton::Right) {
                    // Stop if player has no block in hand
                    let Some(player_equipped_block) = current_block.0 else {
                        return;
                    };

                    // Place in previous spot
                    // FIXME: Place on selected face, not just previous ray voxel
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
                        let Some(chunk_entity) = loaded_chunks.get_chunk(prev_chunk_pos) else {
                            continue;
                        };
                        let Ok(chunk_data) = chunks.get_mut(*chunk_entity) else {
                            continue;
                        };
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

                    added_events.send(VoxelAddedEvent::new(
                        GlobalVoxelPos::from_chunk_local(chunk_pos, local_pos),
                        player_equipped_block,
                    ));

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
