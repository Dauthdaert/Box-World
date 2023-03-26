use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashSet};

use crate::chunk::{ChunkData, ChunkPos, LoadedChunks};

use super::LightAddNode;

pub(super) fn handle_added(
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
        let down_level = if source_level == 15 { 15 } else { new_level };

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
                down_level,
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
                down_level,
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
                down_level,
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
