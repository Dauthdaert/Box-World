use bevy::prelude::{Component, Vec3};

use crate::voxel::{Voxel, VoxelPos};

use super::ChunkData;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl ChunkPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn from_global_coords(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: (x / (Voxel::size() * ChunkData::edge() as f32)).floor() as usize,
            y: (y / (Voxel::size() * ChunkData::edge() as f32)).floor() as usize,
            z: (z / (Voxel::size() * ChunkData::edge() as f32)).floor() as usize,
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

    pub fn neighbors(&self) -> Vec<ChunkPos> {
        vec![
            ChunkPos::new(
                self.x.wrapping_sub(1),
                self.y.wrapping_sub(1),
                self.z.wrapping_sub(1),
            ),
            ChunkPos::new(self.x.wrapping_sub(1), self.y.wrapping_sub(1), self.z),
            ChunkPos::new(self.x.wrapping_sub(1), self.y.wrapping_sub(1), self.z + 1),
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z),
            ChunkPos::new(self.x.wrapping_sub(1), self.y, self.z + 1),
            ChunkPos::new(self.x.wrapping_sub(1), self.y + 1, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x.wrapping_sub(1), self.y + 1, self.z),
            ChunkPos::new(self.x.wrapping_sub(1), self.y + 1, self.z + 1),
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z.wrapping_sub(1)),
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z),
            ChunkPos::new(self.x, self.y.wrapping_sub(1), self.z + 1),
            ChunkPos::new(self.x, self.y, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x, self.y, self.z + 1),
            ChunkPos::new(self.x, self.y + 1, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x, self.y + 1, self.z),
            ChunkPos::new(self.x, self.y + 1, self.z + 1),
            ChunkPos::new(self.x + 1, self.y.wrapping_sub(1), self.z.wrapping_sub(1)),
            ChunkPos::new(self.x + 1, self.y.wrapping_sub(1), self.z),
            ChunkPos::new(self.x + 1, self.y.wrapping_sub(1), self.z + 1),
            ChunkPos::new(self.x + 1, self.y, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x + 1, self.y, self.z),
            ChunkPos::new(self.x + 1, self.y, self.z + 1),
            ChunkPos::new(self.x + 1, self.y + 1, self.z.wrapping_sub(1)),
            ChunkPos::new(self.x + 1, self.y + 1, self.z),
            ChunkPos::new(self.x + 1, self.y + 1, self.z + 1),
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
