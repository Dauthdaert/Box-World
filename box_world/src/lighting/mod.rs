use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashSet};

use crate::{
    chunk::{ChunkData, ChunkPos, LoadedChunks, VoxelAddedEvent, VoxelRemovedEvent},
    mesher::NeedsMesh,
    states::GameStates,
};

mod torch_added;
mod torch_removed;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
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

pub fn propagate_lighting(
    mut commands: Commands,
    mut chunks: Query<(&ChunkPos, &mut ChunkData)>,
    loaded_chunks: Res<LoadedChunks>,
    mut voxel_add_event: EventReader<VoxelAddedEvent>,
    mut voxel_rem_event: EventReader<VoxelRemovedEvent>,
) {
    let mut added_queue = VecDeque::new();
    let mut rem_queue = VecDeque::new();
    let mut changed = HashSet::new();

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
