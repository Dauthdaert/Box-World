use bevy::{
    prelude::{Entity, Plugin, Query, Resource},
    utils::{HashMap, HashSet},
};
use rand::seq::IteratorRandom;

use crate::chunk::{ChunkData, ChunkPos};

mod generator;
pub use generator::NeedsChunkData;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(World::new());

        app.add_system(periodic_chunk_trim);

        app.add_plugin(generator::GeneratorPlugin);
    }
}

fn periodic_chunk_trim(mut chunks: Query<&mut ChunkData>) {
    let mut rng = rand::thread_rng();
    for mut data in chunks
        .iter_mut()
        .filter(|data| !data.is_uniform())
        .choose_multiple(&mut rng, 2)
    {
        data.trim();
    }
}

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

    fn load(&mut self, _pos: ChunkPos) -> Option<ChunkData> {
        // TODO: Load existing chunks from persistent storage
        None
    }

    pub fn load_inside_range(
        &mut self,
        pos: ChunkPos,
        horizontal_distance: u32,
        vertical_distance: u32,
    ) -> Vec<(ChunkPos, Option<ChunkData>)> {
        let mut to_load = Vec::new();
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

    fn unload(&mut self, pos: ChunkPos) -> Entity {
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
