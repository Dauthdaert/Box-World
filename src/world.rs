use bevy::{
    prelude::{Entity, Resource},
    utils::{HashMap, HashSet},
};

use crate::chunk::{ChunkData, ChunkPos};

#[derive(Resource)]
pub struct World {
    chunks: HashMap<ChunkPos, Entity>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn set(&mut self, pos: ChunkPos, entity: Entity) {
        self.chunks.insert(pos, entity);
    }

    pub fn load(&mut self, _pos: ChunkPos) -> Option<ChunkData> {
        // TODO: Load existing chunks from persistent storage
        None
    }

    pub fn load_inside_range(
        &mut self,
        pos: ChunkPos,
        distance: u32,
    ) -> Vec<(ChunkPos, Option<ChunkData>)> {
        let mut to_load = Vec::new();
        for z in 0..=distance * 2 {
            for y in 0..=distance * 2 {
                for x in 0..=distance * 2 {
                    if pos.x + x < distance || pos.y + y < distance || pos.z + z < distance {
                        continue;
                    }

                    let other_pos = ChunkPos::new(
                        pos.x + x - distance,
                        pos.y + y - distance,
                        pos.z + z - distance,
                    );

                    let chunk_distance = pos.distance(&other_pos);
                    if chunk_distance < distance as f32 && !self.chunks.contains_key(&other_pos) {
                        to_load.push(other_pos);
                    }
                }
            }
        }

        to_load
            .into_iter()
            .map(|pos| (pos, self.load(pos)))
            .collect()
    }

    pub fn unload(&mut self, pos: ChunkPos) -> Entity {
        self.chunks
            .remove(&pos)
            .expect("Chunk should exist at ChunkPos for unloading")
    }

    pub fn unload_outside_range(&mut self, pos: ChunkPos, distance: u32) -> Vec<Entity> {
        let mut to_remove = Vec::new();
        self.chunks.keys().for_each(|other_pos| {
            if pos.distance(other_pos) > distance as f32 {
                to_remove.push(*other_pos);
            }
        });

        to_remove.into_iter().map(|pos| self.unload(pos)).collect()
    }

    #[allow(dead_code)]
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Entity> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_neighbors(&self, pos: ChunkPos) -> [Option<Entity>; 6] {
        pos.neighbors().map(|pos| self.chunks.get(&pos).copied())
    }

    pub fn get_unique_chunk_neighbors(&self, pos_list: Vec<ChunkPos>) -> Vec<Entity> {
        let set: HashSet<Entity> = pos_list
            .iter()
            .flat_map(|pos| pos.neighbors())
            .filter_map(|pos| {
                let entity = self.chunks.get(&pos).copied();
                if let Some(entity) = entity {
                    return Some(entity);
                }
                None
            })
            .collect();
        set.into_iter().collect()
    }
}
