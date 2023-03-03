use bevy::prelude::{Component, Vec3};

use crate::chunk::ChunkPos;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelPos {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[allow(dead_code)]
impl VoxelPos {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub fn from_chunk_coords(
        chunk_pos: ChunkPos,
        chunk_local_x: u32,
        chunk_local_y: u32,
        chunk_local_z: u32,
    ) -> Self {
        let chunk_voxel_pos = chunk_pos.to_voxel_coords();
        Self::new(
            chunk_voxel_pos.x + chunk_local_x,
            chunk_voxel_pos.y + chunk_local_y,
            chunk_voxel_pos.z + chunk_local_z,
        )
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
