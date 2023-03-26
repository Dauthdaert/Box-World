use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashSet};

use crate::{
    chunk::{ChunkData, ChunkPos, LoadedChunks, VoxelAddedEvent, VoxelRemovedEvent},
    mesher::NeedsMesh,
    states::GameStates,
};

mod sun_added;
mod torch_added;
mod torch_removed;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(initial_light_pass);
        app.add_system(propagate_lighting.run_if(in_state(GameStates::InGame)));
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct NeedsLightPass;

struct LightAddNode {
    idx: usize,
    chunk: Entity,
}

struct LightRemNode {
    idx: usize,
    val: u8,
    chunk: Entity,
}

#[allow(clippy::type_complexity)]
pub fn initial_light_pass(
    mut commands: Commands,
    needs_light: Query<Entity, With<NeedsLightPass>>,
    has_light: Query<Entity, Without<NeedsLightPass>>,
    mut chunks: Query<(&ChunkPos, &mut ChunkData)>,
    loaded_chunks: Res<LoadedChunks>,
) {
    // FIXME: Handle sideways propagation into unloaded chunks
    let mut sunlight_queue = VecDeque::new();
    let mut work_count = 0;
    for chunk_entity in needs_light.iter() {
        // Only do 10 columns at a time
        if work_count > 10 {
            break;
        }

        let Ok((chunk_pos, _chunk_data)) = chunks.get(chunk_entity) else { continue; };
        let top_pos = ChunkPos::new(chunk_pos.x, chunk_pos.y + 1, chunk_pos.z);
        if let Some(top_chunk_entity) = loaded_chunks.get_chunk(top_pos) {
            if has_light.contains(*top_chunk_entity) {
                let Ok((_top_chunk_pos, top_chunk_data)) = chunks.get(*top_chunk_entity) else { continue; };
                let y = 0;
                for z in 0..ChunkData::edge() {
                    for x in 0..ChunkData::edge() {
                        let idx = ChunkData::linearize(x, y, z);
                        if top_chunk_data.get_sunlight(x, y, z) > 0 {
                            sunlight_queue.push_back(LightAddNode {
                                idx,
                                chunk: *top_chunk_entity,
                            });
                        }
                    }
                }

                work_count += 1;
                commands
                    .entity(chunk_entity)
                    .remove::<NeedsLightPass>()
                    .insert(NeedsMesh);

                // Mark the whole column as done
                let mut chunk_pos = ChunkPos::new(chunk_pos.x, chunk_pos.y - 1, chunk_pos.z);
                loop {
                    chunk_pos = ChunkPos::new(chunk_pos.x, chunk_pos.y - 1, chunk_pos.z);
                    let Some(chunk_entity) = loaded_chunks.get_chunk(chunk_pos) else { break; };
                    commands
                        .entity(*chunk_entity)
                        .remove::<NeedsLightPass>()
                        .insert(NeedsMesh);
                }
            }
        } else {
            // Decide if top chunk is in sunlight
            // FIXME: For now, if current chunk is empty we assume sky
            let (chunk_pos, mut chunk_data) = chunks.get_mut(chunk_entity).unwrap();
            if chunk_data.is_empty() {
                for z in 0..ChunkData::edge() {
                    for y in 0..ChunkData::edge() {
                        for x in 0..ChunkData::edge() {
                            let idx = ChunkData::linearize(x, y, z);
                            chunk_data.set_sunlight(x, y, z, 15);
                            if y == 0 {
                                sunlight_queue.push_back(LightAddNode {
                                    idx,
                                    chunk: chunk_entity,
                                });
                            }
                        }
                    }
                }
            }

            work_count += 1;
            commands
                .entity(chunk_entity)
                .remove::<NeedsLightPass>()
                .insert(NeedsMesh);

            // Mark the whole column as done
            let mut chunk_pos = ChunkPos::new(chunk_pos.x, chunk_pos.y - 1, chunk_pos.z);
            loop {
                chunk_pos = ChunkPos::new(chunk_pos.x, chunk_pos.y - 1, chunk_pos.z);
                let Some(chunk_entity) = loaded_chunks.get_chunk(chunk_pos) else { break; };
                commands
                    .entity(*chunk_entity)
                    .remove::<NeedsLightPass>()
                    .insert(NeedsMesh);
            }
        }
    }

    let mut changed = HashSet::new();
    sun_added::handle_added(
        &mut sunlight_queue,
        &mut changed,
        &mut chunks,
        &loaded_chunks,
    );

    let changed: Vec<ChunkPos> = changed.into_iter().collect();
    for chunk_entity in loaded_chunks.get_unique_loaded_chunks_and_neighbors(&changed) {
        commands.entity(chunk_entity).insert(NeedsMesh);
    }
}

pub fn propagate_lighting(
    mut commands: Commands,
    mut chunks: Query<(&ChunkPos, &mut ChunkData)>,
    loaded_chunks: Res<LoadedChunks>,
    mut voxel_add_event: EventReader<VoxelAddedEvent>,
    mut voxel_rem_event: EventReader<VoxelRemovedEvent>,
) {
    let mut added_queue = VecDeque::new();
    let mut rem_queue = VecDeque::new();

    for event in voxel_rem_event.iter() {
        let (chunk_pos, local_pos) = event.pos.to_chunk_local();
        let Some(chunk_entity) = loaded_chunks.get_chunk(chunk_pos) else { continue; };
        let Ok((_pos, mut chunk_data)) = chunks.get_mut(*chunk_entity) else { continue; };

        let source_level = chunk_data.get_torchlight(local_pos.x, local_pos.y, local_pos.z);
        chunk_data.set_torchlight(local_pos.x, local_pos.y, local_pos.z, 0);

        rem_queue.push_back(LightRemNode {
            idx: ChunkData::linearize(local_pos.x, local_pos.y, local_pos.z),
            val: source_level,
            chunk: *chunk_entity,
        });
    }

    for event in voxel_add_event.iter() {
        let (chunk_pos, local_pos) = event.pos.to_chunk_local();
        let Some(chunk_entity) = loaded_chunks.get_chunk(chunk_pos) else { continue; };
        let Ok((_pos, mut chunk_data)) = chunks.get_mut(*chunk_entity) else { continue; };

        if event.value.is_opaque() {
            let source_level = chunk_data.get_torchlight(local_pos.x, local_pos.y, local_pos.z);
            chunk_data.set_torchlight(local_pos.x, local_pos.y, local_pos.z, 0);

            rem_queue.push_back(LightRemNode {
                idx: ChunkData::linearize(local_pos.x, local_pos.y, local_pos.z),
                val: source_level,
                chunk: *chunk_entity,
            });
        } else if event.value.emissiveness() > 0 {
            chunk_data.set_torchlight(
                local_pos.x,
                local_pos.y,
                local_pos.z,
                event.value.emissiveness(),
            );
            added_queue.push_back(LightAddNode {
                idx: ChunkData::linearize(local_pos.x, local_pos.y, local_pos.z),
                chunk: *chunk_entity,
            });
        }
    }

    let mut changed = HashSet::new();
    torch_removed::handle_removed(
        &mut added_queue,
        &mut rem_queue,
        &mut changed,
        &mut chunks,
        &loaded_chunks,
    );
    torch_added::handle_added(&mut added_queue, &mut changed, &mut chunks, &loaded_chunks);

    let changed: Vec<ChunkPos> = changed.into_iter().collect();
    for chunk_entity in loaded_chunks.get_unique_loaded_chunks_and_neighbors(&changed) {
        commands.entity(chunk_entity).insert(NeedsMesh);
    }
}
