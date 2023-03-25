use bevy::prelude::{Component, Deref, DerefMut, IVec3, UVec3, Vec3};

use crate::chunk::{ChunkData, ChunkPos};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct GlobalVoxelPos(IVec3);

#[allow(dead_code)]
impl GlobalVoxelPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn from_chunk_local(chunk_pos: ChunkPos, voxel_pos: ChunkLocalVoxelPos) -> Self {
        let chunk_voxel_pos = chunk_pos * IVec3::splat(ChunkData::edge() as i32);
        Self::new(
            chunk_voxel_pos.x + voxel_pos.x as i32,
            chunk_voxel_pos.y + voxel_pos.y as i32,
            chunk_voxel_pos.z + voxel_pos.z as i32,
        )
    }

    pub fn to_chunk_local(self) -> (ChunkPos, ChunkLocalVoxelPos) {
        (
            ChunkPos::from_global_coords(self.0.as_vec3()),
            ChunkLocalVoxelPos::new(
                self.x.rem_euclid(ChunkData::edge() as i32) as u32,
                self.y.rem_euclid(ChunkData::edge() as i32) as u32,
                self.z.rem_euclid(ChunkData::edge() as i32) as u32,
            ),
        )
    }

    pub fn from_global_coords(pos: Vec3) -> Self {
        Self(pos.floor().as_ivec3())
    }

    pub fn to_global_coords(self) -> Vec3 {
        self.0.as_vec3()
    }

    pub fn neighbors(&self) -> [GlobalVoxelPos; 6] {
        [
            GlobalVoxelPos::new(self.x + 1, self.y, self.z),
            GlobalVoxelPos::new(self.x - 1, self.y, self.z),
            GlobalVoxelPos::new(self.x, self.y + 1, self.z),
            GlobalVoxelPos::new(self.x, self.y - 1, self.z),
            GlobalVoxelPos::new(self.x, self.y, self.z + 1),
            GlobalVoxelPos::new(self.x, self.y, self.z - 1),
        ]
    }

    pub fn distance(&self, other: &GlobalVoxelPos) -> f32 {
        Vec3::new(self.x as f32, self.y as f32, self.y as f32).distance(Vec3::new(
            other.x as f32,
            other.y as f32,
            other.z as f32,
        ))
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct ChunkLocalVoxelPos(UVec3);

impl ChunkLocalVoxelPos {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self(UVec3::new(x, y, z))
    }
}
