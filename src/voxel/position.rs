use bevy::prelude::{Component, Vec3};

use crate::chunk::{ChunkData, ChunkPos};

use super::Voxel;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

#[allow(dead_code)]
impl VoxelPos {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }

    pub fn from_chunk_coords(
        chunk_pos: ChunkPos,
        chunk_local_x: usize,
        chunk_local_y: usize,
        chunk_local_z: usize,
    ) -> Self {
        let chunk_voxel_pos = chunk_pos.to_voxel_coords();
        Self::new(
            chunk_voxel_pos.x + chunk_local_x,
            chunk_voxel_pos.y + chunk_local_y,
            chunk_voxel_pos.z + chunk_local_z,
        )
    }

    pub fn to_chunk_coords(self) -> (ChunkPos, usize, usize, usize) {
        (
            ChunkPos::new(
                self.x / ChunkData::edge(),
                self.y / ChunkData::edge(),
                self.z / ChunkData::edge(),
            ),
            self.x.rem_euclid(ChunkData::edge()),
            self.y.rem_euclid(ChunkData::edge()),
            self.z.rem_euclid(ChunkData::edge()),
        )
    }

    pub fn from_global_coords(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: (x / Voxel::size()).floor() as usize,
            y: (y / Voxel::size()).floor() as usize,
            z: (z / Voxel::size()).floor() as usize,
        }
    }

    pub fn neighbors(&self) -> [VoxelPos; 6] {
        [
            VoxelPos::new(self.x + 1, self.y, self.z),
            VoxelPos::new(self.x.wrapping_sub(1), self.y, self.z),
            VoxelPos::new(self.x, self.y + 1, self.z),
            VoxelPos::new(self.x, self.y.wrapping_sub(1), self.z),
            VoxelPos::new(self.x, self.y, self.z + 1),
            VoxelPos::new(self.x, self.y, self.z.wrapping_sub(1)),
        ]
    }

    pub fn distance(&self, other: &VoxelPos) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.y as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
    }
}
