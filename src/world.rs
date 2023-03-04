use bevy::{
    prelude::{Entity, Resource},
    utils::HashMap,
};
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, Perlin,
};

use crate::{
    chunk::{ChunkData, ChunkPos},
    voxel::{Voxel, VoxelPos},
};

#[derive(Resource)]
pub struct World {
    chunks: HashMap<ChunkPos, Entity>,
    generator: NoiseMap,
}

impl World {
    pub fn new() -> Self {
        let fbm = Fbm::<Perlin>::default();
        let noise = PlaneMapBuilder::<_, 2>::new(fbm)
            .set_is_seamless(true)
            .set_size(1000, 1000)
            .set_x_bounds(-5.0, 5.0)
            .set_y_bounds(-5.0, 5.0)
            .build();
        Self {
            chunks: HashMap::new(),
            generator: noise,
        }
    }

    pub fn set(&mut self, pos: ChunkPos, entity: Entity) {
        self.chunks.insert(pos, entity);
    }

    pub fn load(&mut self, pos: ChunkPos) -> ChunkData {
        let mut chunk = ChunkData::default();

        for z in 0..ChunkData::edge() {
            for y in 0..ChunkData::edge() {
                for x in 0..ChunkData::edge() {
                    let voxel_pos = VoxelPos::from_chunk_coords(pos, x, y, z);
                    let voxel = if (voxel_pos.y as f64)
                        < (40.0
                            + self
                                .generator
                                .get_value(voxel_pos.x as usize, voxel_pos.z as usize)
                                * 10.0)
                    {
                        Voxel::Opaque(1)
                    } else {
                        Voxel::Empty
                    };

                    chunk.set(x, y, z, voxel);
                }
            }
        }

        chunk
    }

    pub fn load_inside_range(
        &mut self,
        pos: ChunkPos,
        distance: u32,
    ) -> Vec<(ChunkPos, ChunkData)> {
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
}
