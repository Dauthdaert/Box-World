use bevy::{
    prelude::{Component, Entity, Resource},
    utils::{HashMap, HashSet},
};

use crate::chunk::{ChunkData, ChunkPos};

#[derive(Component, Default)]
pub struct LoadPoint {
    pub horizontal: usize,
    pub vertical: usize,
}

#[derive(Resource)]
pub struct LoadedChunks {
    chunks: HashMap<ChunkPos, Entity>,
}

impl LoadedChunks {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn set(&mut self, pos: ChunkPos, entity: Entity) {
        self.chunks.insert(pos, entity);
    }

    fn load(&mut self, _pos: ChunkPos) -> Option<ChunkData> {
        // TODO: Load existing chunks from persistent storage
        None
    }

    pub fn load_inside_range(
        &mut self,
        pos_lit: &[(ChunkPos, usize, usize)],
    ) -> Vec<(ChunkPos, Option<ChunkData>)> {
        let mut to_load = HashSet::new();
        for (pos, horizontal_distance, vertical_distance) in pos_lit.iter().copied() {
            for z in 0..=horizontal_distance * 2 {
                for y in 0..=vertical_distance * 2 {
                    for x in 0..=horizontal_distance * 2 {
                        if pos.x + x < horizontal_distance
                            || pos.y + y < vertical_distance
                            || pos.z + z < horizontal_distance
                        {
                            continue;
                        }

                        let other_pos = ChunkPos::new(
                            pos.x + x - horizontal_distance,
                            pos.y + y - vertical_distance,
                            pos.z + z - horizontal_distance,
                        );

                        let chunk_distance = pos.distance(&other_pos);
                        if chunk_distance < horizontal_distance as f32
                            && !self.chunks.contains_key(&other_pos)
                        {
                            to_load.insert(other_pos);
                        }
                    }
                }
            }
        }

        to_load
            .into_iter()
            .map(|pos| (pos, self.load(pos)))
            .collect()
    }

    fn unload(&mut self, pos: ChunkPos) -> Entity {
        // TODO: Save unloaded chunks to persistent storage
        self.chunks
            .remove(&pos)
            .expect("Chunk should exist at ChunkPos for unloading")
    }

    pub fn unload_outside_range(&mut self, pos_list: &[(ChunkPos, usize, usize)]) -> Vec<Entity> {
        let mut to_remove = Vec::new();
        self.chunks.keys().for_each(|other_pos| {
            if pos_list
                .iter()
                .all(|(pos, horizontal, _vertical)| pos.distance(other_pos) > *horizontal as f32)
            {
                to_remove.push(*other_pos);
            }
        });

        to_remove.into_iter().map(|pos| self.unload(pos)).collect()
    }

    #[allow(dead_code)]
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Entity> {
        self.chunks.get(&pos)
    }

    pub fn get_loaded_chunk_neighbors(&self, pos: ChunkPos) -> Vec<Entity> {
        pos.neighbors()
            .iter()
            .filter_map(|pos| self.chunks.get(pos).copied())
            .collect()
    }

    pub fn get_unique_loaded_chunk_neighbors(&self, pos_list: &[ChunkPos]) -> Vec<Entity> {
        let set: HashSet<Entity> = pos_list
            .iter()
            .flat_map(|pos| pos.neighbors())
            .filter_map(|pos| self.chunks.get(&pos).copied())
            .collect();
        set.into_iter().collect()
    }
}
