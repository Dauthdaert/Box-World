use bevy::prelude::Component;
use ndshape::{ConstShape, ConstShape3usize};
use serde::{Deserialize, Serialize};

use crate::voxel::Voxel;

use super::storage::Storage;

pub const CHUNK_EDGE: usize = 16;
type ChunkShape = ConstShape3usize<CHUNK_EDGE, CHUNK_EDGE, CHUNK_EDGE>;

#[derive(Serialize, Deserialize)]
pub struct RawChunk {
    voxels: Storage,
}

#[derive(Component, Clone, Debug)]
pub struct ChunkData {
    voxels: Storage,
    change_count: u16,
    dirty: bool,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            voxels: Storage::new(ChunkShape::USIZE),
            change_count: 0,
            dirty: true,
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
        self.set_dirty(true);

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

    pub fn is_empty(&self) -> bool {
        self.is_uniform() && self.get(0, 0, 0) == Voxel::Empty
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
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

    pub fn from_raw(raw_chunk: RawChunk) -> Self {
        Self {
            voxels: raw_chunk.voxels,
            change_count: 0,
            dirty: false,
        }
    }

    pub fn to_raw(&self) -> RawChunk {
        RawChunk {
            voxels: self.voxels.clone(),
        }
    }
}
