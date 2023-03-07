use bevy::prelude::Component;
use ndshape::{ConstShape, ConstShape3usize};

use crate::voxel::Voxel;

use storage::Storage;

mod position;
mod storage;

pub use position::ChunkPos;

pub const CHUNK_EDGE: usize = 16;
type ChunkShape = ConstShape3usize<CHUNK_EDGE, CHUNK_EDGE, CHUNK_EDGE>;

#[derive(Component, Clone)]
pub struct ChunkData {
    voxels: Storage,
    change_count: u16,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            voxels: Storage::new(ChunkShape::USIZE),
            change_count: 0,
        }
    }
}

#[allow(dead_code)]
impl ChunkData {
    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels.get(Self::linearize(x, y, z))
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        self.voxels.set(Self::linearize(x, y, z), voxel);
        self.change_count += 1;

        if self.change_count > 500 {
            self.voxels.trim();
            self.change_count = 0;
        }
    }

    pub fn is_uniform(&self) -> bool {
        match self.voxels {
            Storage::Single(_) => true,
            Storage::Multi(_) => false,
        }
    }

    pub fn trim(&mut self) {
        self.voxels.trim();
    }

    pub const fn size() -> usize {
        ChunkShape::USIZE
    }

    pub const fn edge() -> usize {
        CHUNK_EDGE
    }

    pub fn linearize(x: usize, y: usize, z: usize) -> usize {
        ChunkShape::linearize([x, y, z])
    }

    pub fn delinearize(idx: usize) -> (usize, usize, usize) {
        let res = ChunkShape::delinearize(idx);
        (res[0], res[1], res[2])
    }
}
