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
            .execute_batch(
                "create table if not exists blocks (
                    posx integer not null,
                    posy integer not null,
                    posz integer not null,
                    data blob,
                 PRIMARY KEY (posx, posy, posz)
                );
                PRAGMA journal_mode=WAL;
                PRAGMA synchronous=NORMAL;
            ",
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
    pub horizontal: u32,
    pub vertical: u32,
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

    pub fn load_inside_range(&mut self, pos_lit: &[(ChunkPos, u32, u32)]) -> Vec<ChunkPos> {
        let tentative_max_chunks = (pos_lit[0].1 * pos_lit[0].1) as usize;
        let mut to_load = Vec::with_capacity(tentative_max_chunks);

        for (pos, horizontal_distance, vertical_distance) in pos_lit.iter().copied() {
            let horizontal_distance = horizontal_distance as i32;
            let horizontal_distance_squared = horizontal_distance * horizontal_distance;
            let vertical_distance = vertical_distance as i32;

            for z in -horizontal_distance..=horizontal_distance {
                for y in -vertical_distance..=vertical_distance {
                    for x in -horizontal_distance..=horizontal_distance {
                        let other_pos = ChunkPos::new(pos.x + x, pos.y + y, pos.z + z);

                        let chunk_distance = pos.distance_squared(&other_pos);
                        if chunk_distance <= horizontal_distance_squared as f32 {
                            to_load.push(other_pos);
                        }
                    }
                }
            }
        }

        to_load
    }

    fn unload(&mut self, pos: ChunkPos) -> Entity {
        self.chunks
            .remove(&pos)
            .expect("Chunk should exist at ChunkPos for unloading")
    }

    pub fn unload_outside_range(&mut self, pos_list: &[(ChunkPos, u32, u32)]) -> Vec<Entity> {
        let mut to_remove = Vec::new();
        self.chunks.keys().for_each(|other_pos| {
            if pos_list.iter().all(|(pos, horizontal, _vertical)| {
                pos.distance_squared(other_pos) > (horizontal * horizontal) as f32
            }) {
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
