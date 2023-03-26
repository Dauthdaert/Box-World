use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashSet};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Bytes};

use crate::mesher::NeedsMesh;

use super::{ChunkData, ChunkPos, LoadedChunks, VoxelAddedEvent, VoxelRemovedEvent};

#[inline]
pub fn to_torchlight(value: u8) -> u8 {
    value & 0xFu8
}

#[inline]
pub fn to_sunlight(value: u8) -> u8 {
    (value >> 4) & 0xFu8
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightStorage {
    #[serde_as(as = "Bytes")]
    lights: Box<[u8; ChunkData::usize()]>,
}

impl LightStorage {
    pub fn new() -> Self {
        Self {
            lights: Box::new([0; ChunkData::usize()]),
        }
    }

    /// Output contains both torch and sun light
    pub fn get_light(&self, idx: usize) -> u8 {
        self.lights[idx]
    }

    /// Output is bounded between 0 and 15
    pub fn get_torchlight(&self, idx: usize) -> u8 {
        to_torchlight(self.lights[idx])
    }

    /// Input is bounded between 0 and 15
    pub fn set_torchlight(&mut self, idx: usize, value: u8) {
        debug_assert!(value < 16);

        self.lights[idx] = (self.lights[idx] & 0xF0u8) | value;
    }

    /// Output is bounded between 0 and 15
    pub fn get_sunlight(&self, idx: usize) -> u8 {
        to_sunlight(self.lights[idx])
    }

    /// Input is bounded between 0 and 15
    pub fn set_sunlight(&mut self, idx: usize, value: u8) {
        debug_assert!(value < 16);

        self.lights[idx] = (self.lights[idx] & 0xFu8) | (value << 4);
    }
}

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

    handle_removed(
        &mut added_queue,
        &mut rem_queue,
        &mut changed,
        &mut chunks,
        &loaded_chunks,
    );
    handle_added(&mut added_queue, &mut changed, &mut chunks, &loaded_chunks);

    let changed: Vec<ChunkPos> = changed.into_iter().collect();
    for chunk_entity in loaded_chunks.get_unique_loaded_chunks_and_neighbors(&changed) {
        commands.entity(chunk_entity).insert(NeedsMesh);
    }
}

fn handle_added(
    added_queue: &mut VecDeque<LightAddNode>,
    changed: &mut HashSet<ChunkPos>,
    chunks: &mut Query<(&ChunkPos, &mut ChunkData)>,
    loaded_chunks: &LoadedChunks,
) {
    while !added_queue.is_empty() {
        let node = added_queue.pop_front().unwrap();

        let (x, y, z) = ChunkData::delinearize(node.idx);
        let (pos, source_level) = {
            let Ok((pos, chunk_data)) = chunks.get(node.chunk) else { continue; };
            (*pos, chunk_data.get_torchlight(x, y, z))
        };
        let new_level = source_level.saturating_sub(1);

        changed.insert(pos);

        const MAX: u32 = ChunkData::edge() - 1;

        if x > 0 && x < MAX {
            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };

            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x - 1,
                y,
                z,
                source_level,
                new_level,
            );

            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x + 1,
                y,
                z,
                source_level,
                new_level,
            );
        } else if x == 0 {
            check_neighbor_complex_add(
                added_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x - 1, pos.y, pos.z),
                MAX,
                y,
                z,
                source_level,
                new_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x + 1,
                y,
                z,
                source_level,
                new_level,
            );
        } else if x == MAX {
            check_neighbor_complex_add(
                added_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x + 1, pos.y, pos.z),
                0,
                y,
                z,
                source_level,
                new_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x - 1,
                y,
                z,
                source_level,
                new_level,
            );
        }

        if y > 0 && y < MAX {
            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };

            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y - 1,
                z,
                source_level,
                new_level,
            );

            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y + 1,
                z,
                source_level,
                new_level,
            );
        } else if y == 0 {
            check_neighbor_complex_add(
                added_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x, pos.y - 1, pos.z),
                x,
                MAX,
                z,
                source_level,
                new_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y + 1,
                z,
                source_level,
                new_level,
            );
        } else if y == MAX {
            check_neighbor_complex_add(
                added_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x, pos.y + 1, pos.z),
                x,
                0,
                z,
                source_level,
                new_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y - 1,
                z,
                source_level,
                new_level,
            );
        }

        if z > 0 && z < MAX {
            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };

            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y,
                z - 1,
                source_level,
                new_level,
            );

            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y,
                z + 1,
                source_level,
                new_level,
            );
        } else if z == 0 {
            check_neighbor_complex_add(
                added_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x, pos.y, pos.z - 1),
                x,
                y,
                MAX,
                source_level,
                new_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y,
                z + 1,
                source_level,
                new_level,
            );
        } else if z == MAX {
            check_neighbor_complex_add(
                added_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x, pos.y, pos.z + 1),
                x,
                y,
                0,
                source_level,
                new_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_add(
                added_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y,
                z - 1,
                source_level,
                new_level,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_neighbor_simple_add(
    add_queue: &mut VecDeque<LightAddNode>,
    chunk_entity: Entity,
    chunk_data: &mut ChunkData,
    x: u32,
    y: u32,
    z: u32,
    source_level: u8,
    new_level: u8,
) {
    if !chunk_data.get(x, y, z).is_opaque()
        && chunk_data.get_torchlight(x, y, z) + 2 <= source_level
    {
        chunk_data.set_torchlight(x, y, z, new_level);
        add_queue.push_back(LightAddNode {
            idx: ChunkData::linearize(x, y, z),
            chunk: chunk_entity,
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn check_neighbor_complex_add(
    add_queue: &mut VecDeque<LightAddNode>,
    loaded_chunks: &LoadedChunks,
    chunks: &mut Query<(&ChunkPos, &mut ChunkData)>,
    pos: ChunkPos,
    x: u32,
    y: u32,
    z: u32,
    source_level: u8,
    new_level: u8,
) {
    let (chunk_entity, mut chunk_data) = {
        let Some(chunk_entity) = loaded_chunks.get_chunk(pos) else { return; };
        let Ok((_pos, chunk_data)) = chunks.get_mut(*chunk_entity) else { return; };
        (chunk_entity, chunk_data)
    };

    check_neighbor_simple_add(
        add_queue,
        *chunk_entity,
        &mut chunk_data,
        x,
        y,
        z,
        source_level,
        new_level,
    );
}

fn handle_removed(
    add_queue: &mut VecDeque<LightAddNode>,
    rem_queue: &mut VecDeque<LightRemNode>,
    changed: &mut HashSet<ChunkPos>,
    chunks: &mut Query<(&ChunkPos, &mut ChunkData)>,
    loaded_chunks: &LoadedChunks,
) {
    while !rem_queue.is_empty() {
        let node = rem_queue.pop_front().unwrap();

        let (x, y, z) = ChunkData::delinearize(node.idx);
        let pos = {
            let Ok((pos, _chunk_data)) = chunks.get(node.chunk) else { continue; };
            *pos
        };
        let source_level = node.val;

        changed.insert(pos);

        const MAX: u32 = ChunkData::edge() - 1;

        if x > 0 && x < MAX {
            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };

            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x - 1,
                y,
                z,
                source_level,
            );

            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x + 1,
                y,
                z,
                source_level,
            );
        } else if x == 0 {
            check_neighbor_complex_rem(
                add_queue,
                rem_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x - 1, pos.y, pos.z),
                MAX,
                y,
                z,
                source_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x + 1,
                y,
                z,
                source_level,
            );
        } else if x == MAX {
            check_neighbor_complex_rem(
                add_queue,
                rem_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x + 1, pos.y, pos.z),
                0,
                y,
                z,
                source_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x - 1,
                y,
                z,
                source_level,
            );
        }

        if y > 0 && y < MAX {
            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };

            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y - 1,
                z,
                source_level,
            );

            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y + 1,
                z,
                source_level,
            );
        } else if y == 0 {
            check_neighbor_complex_rem(
                add_queue,
                rem_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x, pos.y - 1, pos.z),
                x,
                MAX,
                z,
                source_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y + 1,
                z,
                source_level,
            );
        } else if y == MAX {
            check_neighbor_complex_rem(
                add_queue,
                rem_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x, pos.y + 1, pos.z),
                x,
                0,
                z,
                source_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y - 1,
                z,
                source_level,
            );
        }

        if z > 0 && z < MAX {
            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };

            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y,
                z - 1,
                source_level,
            );

            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y,
                z + 1,
                source_level,
            );
        } else if z == 0 {
            check_neighbor_complex_rem(
                add_queue,
                rem_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x, pos.y, pos.z - 1),
                x,
                y,
                MAX,
                source_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y,
                z + 1,
                source_level,
            );
        } else if z == MAX {
            check_neighbor_complex_rem(
                add_queue,
                rem_queue,
                loaded_chunks,
                chunks,
                ChunkPos::new(pos.x, pos.y, pos.z + 1),
                x,
                y,
                0,
                source_level,
            );

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else { continue; };
            check_neighbor_simple_rem(
                add_queue,
                rem_queue,
                node.chunk,
                &mut chunk_data,
                x,
                y,
                z - 1,
                source_level,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_neighbor_simple_rem(
    add_queue: &mut VecDeque<LightAddNode>,
    rem_queue: &mut VecDeque<LightRemNode>,
    chunk_entity: Entity,
    chunk_data: &mut ChunkData,
    x: u32,
    y: u32,
    z: u32,
    source_level: u8,
) {
    let curr_level = chunk_data.get_torchlight(x, y, z);
    if curr_level != 0 && curr_level < source_level {
        chunk_data.set_torchlight(x, y, z, 0);
        rem_queue.push_back(LightRemNode {
            idx: ChunkData::linearize(x, y, z),
            chunk: chunk_entity,
            val: curr_level,
        });
    } else {
        add_queue.push_back(LightAddNode {
            idx: ChunkData::linearize(x, y, z),
            chunk: chunk_entity,
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn check_neighbor_complex_rem(
    add_queue: &mut VecDeque<LightAddNode>,
    rem_queue: &mut VecDeque<LightRemNode>,
    loaded_chunks: &LoadedChunks,
    chunks: &mut Query<(&ChunkPos, &mut ChunkData)>,
    pos: ChunkPos,
    x: u32,
    y: u32,
    z: u32,
    source_level: u8,
) {
    let (chunk_entity, mut chunk_data) = {
        let Some(chunk_entity) = loaded_chunks.get_chunk(pos) else { return; };
        let Ok((_pos, chunk_data)) = chunks.get_mut(*chunk_entity) else { return; };
        (chunk_entity, chunk_data)
    };

    check_neighbor_simple_rem(
        add_queue,
        rem_queue,
        *chunk_entity,
        &mut chunk_data,
        x,
        y,
        z,
        source_level,
    );
}
