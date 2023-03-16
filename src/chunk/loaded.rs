use bevy::{
    prelude::{Component, Entity, Resource},
    utils::{HashMap, HashSet},
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::chunk::ChunkPos;

#[derive(Resource)]
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
}

impl Database {
    pub fn new() -> Self {
        let manager = SqliteConnectionManager::file("worlds/world.db3");
        let pool = Pool::builder()
            .max_size(30)
            .test_on_check_out(false)
            .build(manager)
            .unwrap();
        pool.get()
            .unwrap()
            .execute(
                "create table if not exists blocks (
                    posx integer not null,
                    posy integer not null,
                    posz integer not null,
                    data blob,
                 PRIMARY KEY (posx, posy, posz)
                )",
                [],
            )
            .unwrap();

        Self { pool }
    }

    pub fn get_connection_pool(&self) -> Pool<SqliteConnectionManager> {
        self.pool.clone()
    }
}

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

    pub fn load_inside_range(&mut self, pos_lit: &[(ChunkPos, usize, usize)]) -> Vec<ChunkPos> {
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

        to_load.into_iter().collect()
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

    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Entity> {
        self.chunks.get(&pos)
    }

    pub fn get_loaded_chunk_neighbors(&self, pos: ChunkPos) -> Vec<Entity> {
        pos.neighbors()
            .iter()
            .filter_map(|pos| self.chunks.get(pos).copied())
            .collect()
    }

    pub fn get_unique_loaded_chunks_and_neighbors(&self, pos_list: &[ChunkPos]) -> Vec<Entity> {
        let mut set: HashSet<Entity> = pos_list
            .iter()
            .filter_map(|pos| self.chunks.get(pos).copied())
            .collect();
        pos_list
            .iter()
            .flat_map(|pos| pos.neighbors())
            .filter_map(|pos| self.chunks.get(&pos).copied())
            .for_each(|entity| {
                set.insert(entity);
            });

        set.into_iter().collect()
    }
}
