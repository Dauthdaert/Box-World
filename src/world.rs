use bevy::{prelude::Resource, utils::HashMap};
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, Perlin,
};

use crate::{
    chunk::{Chunk, ChunkPos},
    mesher::ChunkBoundary,
    voxel::{Voxel, VoxelPos},
};

#[derive(Resource)]
pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
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

    pub fn load(&mut self, pos: ChunkPos) {
        let mut chunk = Chunk::default();

        for z in 0..Chunk::edge() {
            for y in 0..Chunk::edge() {
                for x in 0..Chunk::edge() {
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

        self.chunks.insert(pos, chunk);
    }

    pub fn load_inside_range(&mut self, pos: ChunkPos, distance: u32) -> Vec<ChunkPos> {
        let mut to_load = Vec::new();
        for x in 0..=distance * 2 {
            for y in 0..=distance * 2 {
                for z in 0..=distance * 2 {
                    if pos.x + x < distance || pos.y + y < distance || pos.z + z < distance {
                        continue;
                    }

                    let other_pos = ChunkPos::new(
                        pos.x + x - distance,
                        pos.y + y - distance,
                        pos.z + z - distance,
                    );

                    if pos.distance(&other_pos) > distance as f32 {
                        continue;
                    }

                    if !self.chunks.contains_key(&other_pos) {
                        to_load.push(other_pos);
                    }
                }
            }
        }

        to_load.iter().for_each(|pos| self.load(*pos));
        to_load
    }

    pub fn unload(&mut self, pos: ChunkPos) {
        self.chunks.remove(&pos);
    }

    pub fn unload_outside_range(&mut self, pos: ChunkPos, distance: u32) -> Vec<ChunkPos> {
        let mut to_remove = Vec::new();
        self.chunks.keys().for_each(|other_pos| {
            if pos.distance(other_pos) > distance as f32 {
                to_remove.push(*other_pos);
            }
        });

        to_remove.iter().for_each(|pos| self.unload(*pos));
        to_remove
    }

    #[allow(dead_code)]
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_boundary(&self, pos: ChunkPos) -> Option<ChunkBoundary> {
        self.chunks.get(&pos).map(|center| {
            ChunkBoundary::new(
                center.clone(),
                pos.neighbors()
                    .map(|pos| self.chunks.get(&pos).cloned().unwrap_or_default()),
            )
        })
    }
}
