use bevy::{
    prelude::{Component, Resource, Vec3},
    utils::HashMap,
};

use crate::{
    chunk::{Chunk, CHUNK_EDGE},
    mesher::ChunkBoundary,
    voxel::{Voxel, VOXEL_SIZE},
};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl ChunkPos {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub fn from_world(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: (x / (VOXEL_SIZE * CHUNK_EDGE as f32)) as u32,
            y: (y / (VOXEL_SIZE * CHUNK_EDGE as f32)) as u32,
            z: (z / (VOXEL_SIZE * CHUNK_EDGE as f32)) as u32,
        }
    }

    pub fn to_world(self) -> (f32, f32, f32) {
        (
            (self.x * CHUNK_EDGE) as f32 * VOXEL_SIZE,
            (self.y * CHUNK_EDGE) as f32 * VOXEL_SIZE,
            (self.z * CHUNK_EDGE) as f32 * VOXEL_SIZE,
        )
    }

    pub fn neighbors(&self) -> [ChunkPos; 6] {
        [
            ChunkPos::new(self.x + 1, self.y, self.z),
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z),
            ChunkPos::new(self.x, self.y + 1, self.z),
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z),
            ChunkPos::new(self.x, self.y, self.z + 1),
            ChunkPos::new(self.x, self.y, self.z.wrapping_sub(1)),
        ]
    }

    pub fn distance(&self, other: &ChunkPos) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.y as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
    }
}

#[derive(Resource)]
pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn load(&mut self, pos: ChunkPos) {
        let mut chunk = Chunk::default();

        for i in 0..Chunk::size() {
            let (x, y, z) = Chunk::delinearize(i);

            let voxel = if ((y * x) as f32).sqrt() < 1.0 {
                Voxel::Opaque(1)
            } else {
                Voxel::Empty
            };

            chunk.set(x, y, z, voxel);
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
