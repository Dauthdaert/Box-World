use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashSet};

use crate::chunk::{ChunkData, ChunkPos, LoadedChunks};

use super::{LightAddNode, LightRemNode};

pub(super) fn handle_removed(
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
            let Ok((pos, _chunk_data)) = chunks.get(node.chunk) else {
                continue;
            };
            *pos
        };
        let source_level = node.val;

        changed.insert(pos);

        const MAX: u32 = ChunkData::edge() - 1;

        if x > 0 && x < MAX {
            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else {
                continue;
            };

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

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else {
                continue;
            };
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

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else {
                continue;
            };
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
            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else {
                continue;
            };

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

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else {
                continue;
            };
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

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else {
                continue;
            };
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
            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else {
                continue;
            };

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

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else {
                continue;
            };
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

            let Ok((_pos, mut chunk_data)) = chunks.get_mut(node.chunk) else {
                continue;
            };
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
        let Some(chunk_entity) = loaded_chunks.get_chunk(pos) else {
            return;
        };
        let Ok((_pos, chunk_data)) = chunks.get_mut(*chunk_entity) else {
            return;
        };
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
