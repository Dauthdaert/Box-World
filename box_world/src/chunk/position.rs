use std::ops::Mul;

use bevy::prelude::{Component, Deref, DerefMut, IVec3, Vec3};

use super::ChunkData;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct ChunkPos(IVec3);

impl Mul<IVec3> for ChunkPos {
    type Output = ChunkPos;

    fn mul(self, rhs: IVec3) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ChunkPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        ChunkPos(IVec3::new(x, y, z))
    }

    pub fn from_global_coords(pos: Vec3) -> Self {
        ChunkPos((pos / ChunkData::edge() as f32).floor().as_ivec3())
    }

    pub fn to_global_coords(self) -> Vec3 {
        (self.0 * ChunkData::edge() as i32).as_vec3()
    }

    pub fn neighbors(&self) -> Vec<ChunkPos> {
        vec![
            ChunkPos::new(self.x - 1, self.y - 1, self.z - 1),
            ChunkPos::new(self.x - 1, self.y - 1, self.z),
            ChunkPos::new(self.x - 1, self.y - 1, self.z + 1),
            ChunkPos::new(self.x - 1, self.y, self.z - 1),
            ChunkPos::new(self.x - 1, self.y, self.z),
            ChunkPos::new(self.x - 1, self.y, self.z + 1),
            ChunkPos::new(self.x - 1, self.y + 1, self.z - 1),
            ChunkPos::new(self.x - 1, self.y + 1, self.z),
            ChunkPos::new(self.x - 1, self.y + 1, self.z + 1),
            ChunkPos::new(self.x, self.y - 1, self.z - 1),
            ChunkPos::new(self.x, self.y - 1, self.z),
            ChunkPos::new(self.x, self.y - 1, self.z + 1),
            ChunkPos::new(self.x, self.y, self.z - 1),
            ChunkPos::new(self.x, self.y, self.z + 1),
            ChunkPos::new(self.x, self.y + 1, self.z - 1),
            ChunkPos::new(self.x, self.y + 1, self.z),
            ChunkPos::new(self.x, self.y + 1, self.z + 1),
            ChunkPos::new(self.x + 1, self.y - 1, self.z - 1),
            ChunkPos::new(self.x + 1, self.y - 1, self.z),
            ChunkPos::new(self.x + 1, self.y - 1, self.z + 1),
            ChunkPos::new(self.x + 1, self.y, self.z - 1),
            ChunkPos::new(self.x + 1, self.y, self.z),
            ChunkPos::new(self.x + 1, self.y, self.z + 1),
            ChunkPos::new(self.x + 1, self.y + 1, self.z - 1),
            ChunkPos::new(self.x + 1, self.y + 1, self.z),
            ChunkPos::new(self.x + 1, self.y + 1, self.z + 1),
        ]
    }

    #[allow(dead_code)]
    pub fn distance(&self, other: &ChunkPos) -> f32 {
        self.as_vec3().distance(other.as_vec3())
    }

    pub fn distance_squared(&self, other: &ChunkPos) -> f32 {
        self.as_vec3().distance_squared(other.as_vec3())
    }
}
