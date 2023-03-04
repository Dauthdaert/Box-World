use bevy::prelude::{Component, Vec3};

use crate::voxel::{Voxel, VoxelPos};

use super::ChunkData;

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

    pub fn from_global_coords(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: (x / (Voxel::size() * ChunkData::edge() as f32)) as u32,
            y: (y / (Voxel::size() * ChunkData::edge() as f32)) as u32,
            z: (z / (Voxel::size() * ChunkData::edge() as f32)) as u32,
        }
    }

    pub fn to_global_coords(self) -> (f32, f32, f32) {
        (
            (self.x * ChunkData::edge()) as f32 * Voxel::size(),
            (self.y * ChunkData::edge()) as f32 * Voxel::size(),
            (self.z * ChunkData::edge()) as f32 * Voxel::size(),
        )
    }

    pub fn to_voxel_coords(self) -> VoxelPos {
        VoxelPos::new(
            self.x * ChunkData::edge(),
            self.y * ChunkData::edge(),
            self.z * ChunkData::edge(),
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
        Vec3::new(self.x as f32, self.y as f32, self.z as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
    }
}
