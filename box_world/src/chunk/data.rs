use bevy::prelude::Component;
use ndshape::{ConstShape, ConstShape3usize};
use serde::{Deserialize, Serialize};

use crate::voxel::Voxel;

use super::{lighting::LightStorage, storage::Storage};

const CHUNK_EDGE: usize = 16;
type ChunkShape = ConstShape3usize<CHUNK_EDGE, CHUNK_EDGE, CHUNK_EDGE>;

#[derive(Serialize, Deserialize)]
pub struct RawChunk {
    voxels: Storage,
    lights: LightStorage,
}

#[derive(Component, Clone, Debug)]
pub struct ChunkData {
    voxels: Storage,
    lights: LightStorage,
    change_count: u16,
    dirty: bool,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            voxels: Storage::new(ChunkShape::USIZE),
            lights: LightStorage::new(),
            change_count: 0,
            dirty: true,
        }
    }
}

#[allow(dead_code)]
impl ChunkData {
    pub fn get(&self, x: u32, y: u32, z: u32) -> Voxel {
        self.voxels.get(Self::linearize(x, y, z))
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, voxel: Voxel) {
        self.voxels.set(Self::linearize(x, y, z), voxel);
        self.change_count += 1;
        self.set_dirty(true);

        if self.change_count > 500 {
            self.voxels.trim();
            self.change_count = 0;
        }
    }

    /// Output contains both lights
    pub fn get_light(&self, x: u32, y: u32, z: u32) -> u8 {
        self.lights.get_light(Self::linearize(x, y, z))
    }

    /// Output is bounded between 0 and 15
    pub fn get_torchlight(&self, x: u32, y: u32, z: u32) -> u8 {
        self.lights.get_torchlight(Self::linearize(x, y, z))
    }

    /// Input is bounded between 0 and 15
    pub fn set_torchlight(&mut self, x: u32, y: u32, z: u32, value: u8) {
        self.lights.set_torchlight(Self::linearize(x, y, z), value);
    }

    /// Output is bounded between 0 and 15
    pub fn get_sunlight(&self, x: u32, y: u32, z: u32) -> u8 {
        self.lights.get_sunlight(Self::linearize(x, y, z))
    }

    /// Input is bounded between 0 and 15
    pub fn set_sunlight(&mut self, x: u32, y: u32, z: u32, value: u8) {
        self.lights.set_sunlight(Self::linearize(x, y, z), value);
    }

    pub fn is_uniform(&self) -> bool {
        match self.voxels {
            Storage::Single(_) => true,
            Storage::Multi(_) => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.is_uniform() && self.get(0, 0, 0).is_empty()
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

    pub const fn size() -> u32 {
        ChunkShape::USIZE as u32
    }

    pub const fn usize() -> usize {
        ChunkShape::USIZE
    }

    pub const fn edge() -> u32 {
        CHUNK_EDGE as u32
    }

    #[inline]
    pub fn linearize(x: u32, y: u32, z: u32) -> usize {
        ChunkShape::linearize([x as usize, y as usize, z as usize])
    }

    #[inline]
    pub fn delinearize(idx: usize) -> (u32, u32, u32) {
        let res = ChunkShape::delinearize(idx);
        (res[0] as u32, res[1] as u32, res[2] as u32)
    }

    pub fn from_raw(raw_chunk: RawChunk) -> Self {
        Self {
            voxels: raw_chunk.voxels,
            lights: raw_chunk.lights,
            change_count: 0,
            dirty: false,
        }
    }

    pub fn to_raw(&self) -> RawChunk {
        RawChunk {
            voxels: self.voxels.clone(),
            lights: self.lights.clone(),
        }
    }
}
